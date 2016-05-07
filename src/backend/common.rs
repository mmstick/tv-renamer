use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use backend::traits::{Digits};
use tvdb;

#[derive(Clone, Debug)]
pub struct Arguments {
    // Automatically infer the name of a series and season number by the directory structure.
    pub automatic:     bool,

    // Print the changes that would have been made without actually making any changes.
    pub dry_run:       bool,

    // Log the changes that were made to the disk.
    pub log_changes:   bool,

    // Do not write the name of the series in each episode's file name.
    pub no_name:       bool,

    // Find the episode title of each episode from TVDB and place it in the file names.
    pub tvdb:          bool,

    // Print all changes that are being attempted and performed.
    pub verbose:       bool,

    // Contains the base directory of the series to rename.
    pub directory:     String,

    // Contains the name of the series to be renamed.
    pub series_name:   String,

    // Contains the season number to add to the filename and for use with TVDB lookups.
    pub season_number: usize,

    // The starting episode index count to start renaming from.
    pub episode_count: usize,

    // The number of zeros to use when padding episode numbers.
    pub pad_length:    usize,
}

impl Arguments {
    /// Given a source of episodes from a directory, this returns a list of their target paths.
    pub fn get_targets(&self, directory: &str, episodes: &[PathBuf], episode_index: usize) -> Result<Vec<PathBuf>, String> {
        let api = tvdb::Tvdb::new("0629B785CE550C8D");
        let series_info = if self.tvdb {
            match api.search(self.series_name.as_str(), "en") {
                Ok(reply) => Some(reply),
                Err(_) => { return Err(String::from("unable to get TVDB series information")); }
            }
        } else {
            None
        };

        let mut output: Vec<PathBuf> = Vec::new();
        let mut current_index = episode_index;
        for file in episodes {
            // TVDB Titles
            let tvdb_title = if self.tvdb {
                let reply = series_info.clone().unwrap();
                match api.episode(&reply[0], self.season_number as u32, current_index as u32) {
                    Ok(episode) => episode.episode_name,
                    Err(_) => { return Err(format!("media-rename: episode '{}' does not exist", file.to_string_lossy())); }
                }
            } else {
                String::new()
            };

            // Get target destination for the current file.
            let new_destination = self.get_destination(Path::new(directory), &file, current_index, &tvdb_title);
            output.push(new_destination);
            current_index += 1;
        }
        Ok(output)
    }

    /// Obtain the target path of the file based on the episode count
    pub fn get_destination(&self, directory: &Path, file: &Path, episode: usize, title: &str) -> PathBuf {
        let mut destination = PathBuf::from(&directory);
        let extension = file.extension().unwrap_or(&OsStr::new("")).to_str().unwrap_or("");

        // Do not write the series name if no-name is enabled
        let mut filename = if self.no_name {
            String::new()
        } else {
            let mut filename = self.series_name.clone();
            filename.push(' ');
            filename
        };

        // Append the season number, episode number and extension if available.
        filename.push_str(&self.season_number.to_string());
        filename.push('x');
        filename.push_str(episode.to_padded_string('0', self.pad_length).as_str());

        // Add the episode title to the filename if TVDB is enabled.
        if self.tvdb {
            filename.push(' ');
            filename.push_str(title);
        }

        if !extension.is_empty() { filename.push('.'); }
        filename.push_str(extension);
        destination.push(filename);

        destination
    }
}

/// Takes a pathname and shortens it for readability.
pub fn shorten_path(path: &Path) -> PathBuf {
    match path.strip_prefix(&env::current_dir().unwrap()) {
        Ok(value) => {
            let mut path = PathBuf::from(".");
            path.push(value);
            path
        },
        Err(_) => match path.strip_prefix(&env::home_dir().unwrap()) {
            Ok(value) => {
                let mut path = PathBuf::from("~");
                path.push(value);
                path
            },
            Err(_) => path.to_path_buf()
        },
    }
}

/// Collects a list of all of the seasons in a given directory.
pub fn get_seasons(directory: &str) -> Result<Vec<PathBuf>, &str> {
    if let Ok(files) = fs::read_dir(directory) {
        let mut seasons = Vec::new();
        for entry in files {
            if let Ok(entry) = entry {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_dir() {
                        seasons.push(entry.path());
                    }
                } else {
                    return Err("unable to get metadata");
                }
            } else {
                return Err("unable to get directory entry");
            }
        }
        seasons.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
        Ok(seasons)
    } else {
        Err("unable to read directory")
    }
}

/// Collects a list of all of the episodes in a given directory.
pub fn get_episodes(directory: &str) -> Result<Vec<PathBuf>, &str> {
    if let Ok(files) = fs::read_dir(directory) {
        let mut episodes = Vec::new();
        for entry in files {
            if let Ok(entry) = entry {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() { episodes.push(entry.path()); }
                } else {
                    return Err("unable to get metadata");
                }
            } else {
                return Err("unable to get file entry");
            }
        }
        episodes.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
        Ok(episodes)
    } else {
        Err("unable to read file")
    }
}
