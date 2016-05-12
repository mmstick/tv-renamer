use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use chrono::*;
use backend::shorten_path;

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
