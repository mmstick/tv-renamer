use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use backend::traits::{Digits};
use chrono::*;
use tvdb;

#[derive(Clone, Debug)]
pub enum TemplateToken {
    Character(char),
    Series,
    Season,
    Episode,
    Title,
}

pub fn default_template() -> Vec<TemplateToken> {
    vec![TemplateToken::Series, TemplateToken::Character(' '), TemplateToken::Season, TemplateToken::Character('x'),
        TemplateToken::Episode, TemplateToken::Character(' '), TemplateToken::Title ]
}

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

    // The template used for setting the naming scheme of episodes.
    pub template:      Vec<TemplateToken>
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
                TemplateToken::Character(value) => filename.push(value),
                TemplateToken::Series  => if !self.no_name { filename.push_str(self.series_name.clone().as_str()); },
                TemplateToken::Season  => filename.push_str(self.season_number.to_string().as_str()),
                TemplateToken::Episode => filename.push_str(episode.to_padded_string('0', self.pad_length).as_str()),
                TemplateToken::Title   => if self.tvdb { filename.push_str(title); }
            }
        }
        filename = String::from(filename.trim()); // Remove extra spaces
        filename = filename.replace("/", "");     // Remove characters that are invalid in pathnames

        // Append the extension
        let extension = file.extension().unwrap_or_else(|| OsStr::new("")).to_str().unwrap_or("");
        if !extension.is_empty() {
            filename.push('.');
            filename.push_str(extension);
        }

        // Return the path as a PathBuf
        destination.push_str(&filename);
        println!("{}", destination);
        PathBuf::from(destination)
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

/// Given a directory path, derive the number of the season and assign it.
pub fn derive_season_number(season: &Path) -> Option<usize> {
    let mut directory_name = season.file_name().unwrap().to_str().unwrap().to_lowercase();
    match directory_name.as_str() {
        "season0" | "season 0" | "specials" => Some(0),
        _ => {
            directory_name = directory_name.replace("season", "");
            directory_name = directory_name.replace(" ", "");
            if let Ok(season_number) = directory_name.parse::<usize>() {
                Some(season_number)
            } else {
                None
            }
        }
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

/// Returns a handle to the log file if it could be opened.
fn open_log() -> Result<fs::File, String> {
    match env::home_dir() {
        Some(mut path) => {
            path.push("tv-renamer.log");
            match fs::OpenOptions::new().create(true).append(true).open(path) {
                Ok(log) => Ok(log),
                Err(error) => Err(format!("unable to open log file: {:?}", error))
            }
        },
        None => Err(String::from("unable to get home directory")),
    }

}

/// Appends the current time to the log file.
pub fn log_append_time() {
    let local_time = Local::now().to_rfc2822();
    if let Ok(mut log) = open_log() {
        let _ = log.write(b"\n");
        let _ = log.write_all(local_time.as_bytes());
        let _ = log.write(b"\n");
        let _ = log.flush();
    }
}

// Log the file renaming modification to the log file.
pub fn log_append_change(source: &Path, target: &Path) {
    if let Ok(mut log) = open_log() {
        let _ = log.write(shorten_path(source).to_string_lossy().as_bytes());
        let _ = log.write(b" -> ");
        let _ = log.write(shorten_path(target).to_string_lossy().as_bytes());
        let _ = log.write(b"\n");
        let _ = log.flush();
    }
}
