use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

pub mod logging;
pub mod traits;
pub mod tokenizer;
mod mimetypes;

use self::traits::Digits;
use self::tokenizer::TemplateToken as Token;

use tvdb;

#[derive(Clone, Debug)]
pub struct Arguments {
    // Automatically infer the name of a series and season number by the directory structure.
    pub automatic: bool,

    // Print the changes that would have been made without actually making any changes.
    pub dry_run: bool,

    // Log the changes that were made to the disk.
    pub log_changes: bool,

    // Print all changes that are being attempted and performed.
    pub verbose: bool,

    // Contains the base directory of the series to rename.
    pub directory: String,

    // Contains the name of the series to be renamed.
    pub series_name: String,

    // Contains the season number to add to the filename and for use with TVDB lookups.
    pub season_number: usize,

    // The starting episode index count to start renaming from.
    pub episode_count: usize,

    // The number of zeros to use when padding episode numbers.
    pub pad_length: usize,

    // The template used for setting the naming scheme of episodes.
    pub template: Vec<Token>
}

impl Arguments {
    /// Given a source of episodes from a directory, this returns a list of their target paths.
    pub fn get_targets(&self, directory: &str, episodes: &[PathBuf], episode_index: usize) -> Result<Vec<PathBuf>, String> {
        // The API key required by TVDB's API.
        let api = tvdb::Tvdb::new("0629B785CE550C8D");
        // Obtain the TVDB series information if the template contains the TVDB token.
        let series_info = if self.template.contains(&Token::TVDB) {
            match api.search(self.series_name.as_str(), "en") {
                Ok(reply) => Some(reply),
                Err(_)    => { return Err(String::from("unable to get TVDB series information")); }
            }
        } else {
            None
        };

        let mut output: Vec<PathBuf> = Vec::new();
        let mut current_index = episode_index;
        for file in episodes {
            // Obtain the TVDB Title if the template contains the TVDB token.
            let tvdb_title = if self.template.contains(&Token::TVDB) {
                let reply = series_info.clone().unwrap();
                match api.episode(&reply[0], self.season_number as u32, current_index as u32) {
                    Ok(episode) => episode.episode_name,
                    Err(_)      => { return Err(format!("episode '{}' does not exist", file.to_string_lossy())); }
                }
            } else {
                String::new()
            };

            // Get target destination for the current file.
            let new_destination = self.get_destination(Path::new(directory), file, current_index, &tvdb_title);
            output.push(new_destination);
            current_index += 1;
        }
        Ok(output)
    }

    /// Obtain the target path of the file based on the episode count
    pub fn get_destination(&self, directory: &Path, file: &Path, episode: usize, title: &str) -> PathBuf {
        let mut destination = String::from(directory.to_str().unwrap());
        destination.push('/');

        let mut filename = String::new();
        for pattern in self.template.clone() {
            match pattern {
                Token::Character(value) => filename.push(value),
                Token::Series           => filename.push_str(self.series_name.clone().as_str()),
                Token::Season           => filename.push_str(self.season_number.to_string().as_str()),
                Token::Episode          => filename.push_str(episode.to_padded_string('0', self.pad_length).as_str()),
                Token::TVDB             => filename.push_str(title)
            }
        }
        filename = String::from(filename.trim()); // Remove extra spaces
        filename = filename.replace("/", "-");     // Remove characters that are invalid in pathnames

        // Append the extension
        let extension = file.extension().unwrap_or_else(|| OsStr::new("")).to_str().unwrap_or("");
        if !extension.is_empty() {
            filename.push('.');
            filename.push_str(extension);
        }

        // Return the path as a PathBuf
        destination.push_str(&filename);
        PathBuf::from(destination)
    }
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

/// Collects a list of all of the seasons in a given directory.
pub fn get_seasons(directory: &str) -> Result<Vec<PathBuf>, &str> {
    fs::read_dir(directory).ok().map_or(Err("unable to read directory"), |files| {
        let mut seasons = Vec::new();
        for entry in files {
            let status = entry.ok().map_or(Some("tv-renamer: unable to get directory entry"), |entry| {
                entry.metadata().ok().map_or(Some("tv-renamer: unable to get metadata"), |metadata| {
                    if metadata.is_dir() { seasons.push(entry.path()); }
                    None
                })
            });
            if status.is_some() { return Err(status.unwrap()); }
        }
        seasons.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
        Ok(seasons)
    })
}

/// Collects a list of all of the episodes in a given directory. Files that are not videos are ignored.
pub fn get_episodes(directory: &str) -> Result<Vec<PathBuf>, &str> {
    fs::read_dir(directory).ok().map_or(Err("tv-renamer: unable to read file"), |files| {
        let video_extensions = try!(mimetypes::get_video_extensions());
        let mut episodes = Vec::new();
        for entry in files {
            let status = entry.ok().map_or(Some("tv-renamer: unable to get file entry"), |entry| {
                entry.metadata().ok().map_or(Some("tv-renamer: unable to get metadata"), |metadata| {
                    if metadata.is_file() {
                        for extension in &video_extensions {
                            if extension.as_str() == entry.path().extension().unwrap().to_str().unwrap() {
                                episodes.push(entry.path());
                            }
                        }
                    }
                    None
                })
            });
            if status.is_some() { return Err(status.unwrap()); }
        }
        episodes.sort_by(|a, b| a.to_string_lossy().to_lowercase().cmp(&b.to_string_lossy().to_lowercase()));
        Ok(episodes)
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
