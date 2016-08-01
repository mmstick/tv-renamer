pub mod traits;
pub mod tokenizer;
mod mimetypes;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use tvdb;

use self::tokenizer::TemplateToken as Token;
use self::traits::Digits;

pub struct Arguments {
    pub dry_run: bool,
    pub verbose: bool,
    // pub overview: bool,
    pub base_directory: String,
    pub series_name: String,
    pub season_index: usize,
    pub episode_index: usize,
    pub pad_length: usize,
    pub template: Vec<Token>
}

pub struct Season {
    pub season_no: usize,
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
    MimeFileErr,
    MimeStringErr,
}

/// Scans a given directory to determine whether the directory contains seasons or episodes, and returns a result
/// that matches the situation.
pub fn scan_directory<P: AsRef<Path> + Copy>(directory: P, season_no: usize) -> Result<ScanDir, ReadDirError> {
    // Attempt to determine whether the input directory contains seasons, episodes, or has invalid file entries
    let status = fs::read_dir(directory).ok().map_or(Err(ReadDirError::UnableToReadDir), |entries| {
        for entry in entries {
            match entry.map(|entry| entry.path()).ok() {
                None => return Err(ReadDirError::InvalidDirEntry),
                Some(directory) => {
                    if directory.is_dir() && directory.to_str().unwrap().to_lowercase().contains("season") {
                        return Ok(true)
                    }
                }
            }
        }
        Ok(false)
    });

    match status {
        // If the directory contains season folders, return a list of seasons.
        Ok(true)  => get_seasons(directory).map(ScanDir::Seasons),
        // If the directory does not contain season folders, return a list of episodes
        Ok(false) => get_episodes(directory, season_no).map(ScanDir::Episodes),
        // If an error occurred, return the error
        Err(why)  => Err(why)
    }
}

pub enum TargetErr {
    EpisodeDoesNotExist
}

/// Target requires source path, template tokens, episode number, and name of TV series
pub fn collect_target(source: &Path, season_no: usize, episode_no: usize, arguments: &Arguments,
    tvdb_api: &tvdb::Tvdb, tvdb_series_id: u32)-> Result<PathBuf, TargetErr>
{
    let episode = match tvdb_api.episode(tvdb_series_id, season_no as u32, episode_no as u32) {
        Ok(episode) => episode,
        Err(_)      => return Err(TargetErr::EpisodeDoesNotExist)
    };

    let mut filename = String::with_capacity(64);
    for pattern in &arguments.template {
        match *pattern {
            Token::Character(value) => filename.push(value),
            Token::Series           => filename.push_str(&arguments.series_name),
            Token::Season           => filename.push_str(&season_no.to_string()),
            Token::Episode          => filename.push_str(&episode_no.to_padded_string('0', arguments.pad_length)),
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
    // First, collect a list of season paths
    let entries = fs::read_dir(directory).unwrap();
    let mut seasons = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry) => if entry.path().is_dir() { seasons.push(entry.path()); },
            Err(_)    => return Err(ReadDirError::InvalidDirEntry)
        }
    }
    seasons.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));

    // Then, collect all of the episodes that belong to each season, numbering them accordingly to their name.
    let mut output: Vec<Season> = Vec::new();
    for season in seasons {
        if let Some(number) = derive_season_number(&season) {
            match get_episodes(&season, number) {
                Err(why)   => return Err(why),
                Ok(season) => output.push(season)
            }
        }
    }
    Ok(output)
}


/// Collects a list of all of the episodes in a given directory. Files that are not videos are ignored.
fn get_episodes<P: AsRef<Path>>(directory: P, season_no: usize) -> Result<Season, ReadDirError> {
    // Collect a list of video extensions
    let video_extensions = match mimetypes::get_video_extensions() {
        Err(mimetypes::MimeError::OpenFile)         => return Err(ReadDirError::MimeFileErr),
        Err(mimetypes::MimeError::ReadFileToString) => return Err(ReadDirError::MimeStringErr),
        Ok(extensions)                              => extensions
    };

    // Collect a list of episodes in the directory
    let mut episodes = Vec::with_capacity(32);
    let entries = fs::read_dir(directory).unwrap();
    for entry in entries {
        let status = entry.ok().map_or(Some(ReadDirError::InvalidDirEntry), |entry| {
            let path = entry.path();
            if path.is_file() {
                // Only collect videos from a list of known supported video extensions.
                for extension in &video_extensions {
                    // Only collect files that contain extensions
                    path.extension().map(|entry| {
                        // If the video extension matches the current file, append it to the list of episodes.
                        if extension.as_str() == entry.to_str().unwrap() { episodes.push(path.clone()); }
                    });
                }
            }
            None
        });
        if status.is_some() { return Err(status.unwrap()); }
    }
    episodes.sort_by(|a, b| a.to_string_lossy().to_lowercase().cmp(&b.to_string_lossy().to_lowercase()));

    // Return the list of episodes as a `Season` with the accompanying season number.
    Ok(Season { season_no: season_no, episodes: episodes })
}

/// Given a directory path, derive the number of the season and assign it.
pub fn derive_season_number(season: &Path) -> Option<usize> {
    let mut directory_name = season.file_name().unwrap().to_str().unwrap().to_lowercase();
    match directory_name.as_str() {
        "season0" | "season 0" | "specials" => Some(0),
        _ => {
            directory_name = directory_name.replace("season", "");
            directory_name = directory_name.replace(" ", "");
            directory_name.parse::<usize>().ok()
        }
    }
}

#[test]
fn test_derive_season_number() {
    assert_eq!(derive_season_number(&Path::new("Specials")), Some(0));
    assert_eq!(derive_season_number(&Path::new("Season 0")), Some(0));
    assert_eq!(derive_season_number(&Path::new("Season 1")), Some(1));
    assert_eq!(derive_season_number(&Path::new("season9")), Some(9));
    assert_eq!(derive_season_number(&Path::new("Extras")), None);
}
