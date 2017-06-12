pub mod traits;
pub mod tokenizer;
mod mimetypes;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use tvdb;

use self::mimetypes::MimeError;
use self::tokenizer::TemplateToken as Token;
use self::traits::Digits;

pub const DRY_RUN: u8 = 1;
pub const VERBOSE: u8 = 2;

pub struct Arguments {
    pub flags:          u8,
    pub season_index:   u8,
    pub pad_length:     u8,
    pub episode_index:  u16,
    pub base_directory: String,
    pub series_name:    String,
    pub template:       Vec<Token>
}

#[derive(Debug)]
pub struct Season {
    pub season_no: u8,
    pub episodes:  Vec<PathBuf>
}

/// Takes a pathname and shortens it for readability.
pub fn shorten_path(path: &Path) -> PathBuf {
    // Attempt to strip the current working directory from the path.
    path.strip_prefix(&env::current_dir().unwrap())
        // If the home directory was split, return a new `PathBuf` with "." as the replacement.
        .map(|value| PathBuf::from(".").join(value))
        // If the current working directory could not be found, attempt to strip the home directory from the path.
        .unwrap_or(path.strip_prefix(&env::home_dir().unwrap()).ok()
            // Return the input path if the home directory was not found, otherwise prepend "~" to the path.
            .map_or_else(|| path.to_path_buf(), |value| PathBuf::from("~").join(value)))
}

pub enum ScanDir {
    Episodes(Season),
    Seasons(Vec<Season>)
}

pub enum ReadDirError {
    UnableToReadDir,
    InvalidDirEntry,
    MimeDirErr,
    MimeFileErr,
    MimeStringErr
}

impl From<MimeError> for ReadDirError {
    fn from(err: MimeError) -> ReadDirError {
        match err {
            MimeError::OpenEntry        => ReadDirError::InvalidDirEntry,
            MimeError::OpenDir          => ReadDirError::MimeDirErr,
            MimeError::OpenFile         => ReadDirError::MimeFileErr,
            MimeError::ReadFileToString => ReadDirError::MimeStringErr
        }
    }
}

/// Scans a given directory to determine whether the directory contains seasons or episodes, and returns a result
/// that matches the situation.
pub fn scan_directory<P: AsRef<Path>>(directory: P, season_no: u8) -> Result<ScanDir, ReadDirError> {
    // Attempt to read a list of files in a given directory
    for entry in fs::read_dir(directory.as_ref()).map_err(|_| ReadDirError::UnableToReadDir)? {
        // Check if the current entry is valid and return an error if not.
        let entry = entry.map(|entry| entry.path()).map_err(|_| ReadDirError::InvalidDirEntry)?;

        // If the entry is a directory and the directory contains `season`, return a list of seasons
        if entry.is_dir() && entry.to_str().unwrap().to_lowercase().contains("season") {
            return get_seasons(directory).map(ScanDir::Seasons);
        }
    }

    // If the directory does not contain season directories, return a list of episodes.
    get_episodes(directory, season_no).map(ScanDir::Episodes)
}

pub enum TargetErr {
    EpisodeDoesNotExist
}

/// Target requires source path, template tokens, episode number, and name of TV series
pub fn collect_target(source: &Path, season_no: u8, episode_no: u16, arguments: &Arguments,
    tvdb_api: &tvdb::Tvdb, tvdb_series_id: u32)-> Result<PathBuf, TargetErr>
{
    let episode = tvdb_api.episode(tvdb_series_id, season_no as u32, episode_no as u32)
        .map_err(|_| TargetErr::EpisodeDoesNotExist)?;

    let mut filename = String::with_capacity(64);
    for pattern in &arguments.template {
        match *pattern {
            Token::Character(value) => filename.push(value),
            Token::Series           => filename.push_str(&arguments.series_name),
            Token::Season           => filename.push_str(&season_no.to_string()),
            Token::Episode          => filename.push_str(&episode_no.to_padded_string('0', arguments.pad_length as usize)),
            Token::TvdbTitle        => filename.push_str(&episode.episode_name),
            Token::TvdbFirstAired   => if let Some(date) = episode.first_aired.clone() {
                filename.push_str(&date.year.to_string());
                filename.push('-');
                filename.push_str(&date.month.to_padded_string('0', 2));
                filename.push('-');
                filename.push_str(&date.day.to_padded_string('0', 2));
            }
        }
    }

    filename = filename.trim().replace("/", "-") + "." + source.extension().unwrap().to_str().unwrap();
    Ok(PathBuf::from(source.parent().unwrap()).join(filename))
}

/// Collects a list of all episodes belonging to each season within a given directory.
fn get_seasons<P: AsRef<Path>>(directory: P) -> Result<Vec<Season>, ReadDirError> {
    let mut output: Vec<Season> = Vec::new();

    for entry in fs::read_dir(directory).unwrap() {
        let entry = entry.map_err(|_| ReadDirError::InvalidDirEntry)?;
        let season = entry.path();
        if season.is_dir() {
            if let Some(number) = derive_season_number(&season) {
                output.push(get_episodes(&season, number)?);
            }
        }
    }

    output.sort_by(|a, b| a.season_no.cmp(&b.season_no));

    Ok(output)
}

/// Collects a list of all of the episodes in a given directory. Files that are not videos are ignored.
fn get_episodes<P: AsRef<Path>>(directory: P, season_no: u8) -> Result<Season, ReadDirError> {
    // Collect a list of video extensions
    let video_extensions = mimetypes::get_video_extensions()?;

    // It is more likely that all files will have the same extension, so it
    // will be useful to check for the last-matched extension with each
    // file once the first match has been completed.
    let mut last_matched_extension: Option<String> = None;

    // Collect a list of episodes in the directory
    let mut episodes = Vec::with_capacity(32);
    let entries = fs::read_dir(directory).unwrap();
    for entry in entries {
        let entry = entry.map_err(|_| ReadDirError::InvalidDirEntry)?;
        let path = entry.path();
        let mut pushed = false;
        if path.is_file() {
            // Only collect videos from a list of known supported video extensions.
            if let Some(ref extension) = last_matched_extension {
                // Only collect files that contain extensions
                path.extension().map(|entry| {
                    // If the video extension matches the current file, append it to the list of episodes.
                    if extension.as_str() == entry.to_str().unwrap() {
                        episodes.push(path.clone());
                        pushed = true;
                    }
                });

                if pushed { continue }
            }

            for extension in &video_extensions {
                // Only collect files that contain extensions
                path.extension().map(|entry| {
                    // If the video extension matches the current file, append it to the list of episodes.
                    if extension.as_str() == entry.to_str().unwrap() {
                        episodes.push(path.clone());
                        last_matched_extension = Some(extension.clone());
                        pushed = true;
                    }
                });

                if pushed { break }
            }
        }
    }

    episodes.sort_by(|a, b| a.to_string_lossy().to_lowercase().cmp(&b.to_string_lossy().to_lowercase()));

    // Return the list of episodes as a `Season` with the accompanying season number.
    Ok(Season { season_no: season_no, episodes: episodes })
}

/// Given a directory path, derive the number of the season and assign it.
pub fn derive_season_number(season: &Path) -> Option<u8> {
    season.file_name().and_then(|season| season.to_str())
        .map(|season| season.to_lowercase())
        .and_then(|mut directory_name| match directory_name.as_str() {
            "season0" | "season 0" | "specials" => Some(0),
            _ => {
                directory_name = directory_name.replace("season", "");
                directory_name = directory_name.replace(" ", "");
                directory_name.parse::<u8>().ok()
            }
        })
}

#[test]
fn test_derive_season_number() {
    assert_eq!(derive_season_number(&Path::new("Specials")), Some(0));
    assert_eq!(derive_season_number(&Path::new("Season 0")), Some(0));
    assert_eq!(derive_season_number(&Path::new("Season 1")), Some(1));
    assert_eq!(derive_season_number(&Path::new("season9")), Some(9));
    assert_eq!(derive_season_number(&Path::new("Extras")), None);
}
