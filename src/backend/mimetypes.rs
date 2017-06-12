use std::fs::{File, read_dir};
use std::io::Read;

pub enum MimeError {
    OpenDir,
    OpenFile,
    OpenEntry,
    ReadFileToString
}

/// Obtains a list of video extensions from the `/etc/mime.types` file on Linux.
pub fn get_video_extensions() -> Result<Vec<String>, MimeError> {
    let mut extension_list = Vec::new();
    let mut buffer = String::new();

    for extension_file in read_dir("/usr/share/mime/video").map_err(|_| MimeError::OpenDir)? {
        let extension_file = extension_file.map_err(|_| MimeError::OpenEntry)?;
        let mut file = File::open(extension_file.path()).map_err(|_| MimeError::OpenFile)?;
        let capacity = file.metadata().map(|x| x.len()).unwrap_or(0);
        buffer.reserve(capacity as usize);
        file.read_to_string(&mut buffer).map_err(|_| MimeError::ReadFileToString)?;

        for line in buffer.lines() {
            let line = line.trim();
            if line.starts_with("<glob pattern=\"") {
                let extension = line.split('"').nth(1)
                    .and_then(|extension| extension.split('.').last());
                if let Some(extension) = extension {
                    extension_list.push(extension.to_owned());
                }
            }
        }
    }

    Ok(extension_list)
}
