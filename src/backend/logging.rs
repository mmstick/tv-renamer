use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use chrono::*;
use backend::shorten_path;

/// Returns a handle to the log file if it could be opened.
fn open_log() -> Result<fs::File, String> {
    env::home_dir()
        // Return an error if the home directory could not be obtained.
        .map_or_else(|| Err(String::from("tv-renamer: unable to get home directory")), |mut path| {
            // Add the log file to the path
            path.push("tv-renamer.log");
            // Attempt to return the file with append mode enabled, and create the file if it does not exist.
            fs::OpenOptions::new().create(true).append(true).open(path)
                // If an error occurs, return the error as a formatted `String`.
                .map_err(|err| format!("tv-renamer: unable to open log file: {}", err.to_string()))
        })
}

/// Appends the current time to the log file.
pub fn append_time() {
    let local_time = Local::now().to_rfc2822();
    if let Ok(mut log) = open_log() {
        let _ = log.write(b"\n");
        let _ = log.write_all(local_time.as_bytes());
        let _ = log.write(b"\n");
        let _ = log.flush();
    }
}

// Log the file renaming modification to the log file.
pub fn append_change(source: &Path, target: &Path) {
    if let Ok(mut log) = open_log() {
        let _ = log.write(shorten_path(source).to_string_lossy().as_bytes());
        let _ = log.write(b" -> ");
        let _ = log.write(shorten_path(target).to_string_lossy().as_bytes());
        let _ = log.write(b"\n");
        let _ = log.flush();
    }
}
