use std::fs;
use std::io::Read;

pub enum MimeError {
    OpenFile,
    ReadFileToString
}

/// Obtains a list of video extensions from the `/etc/mime.types` file on Linux.
pub fn get_video_extensions() -> Result<Vec<String>, MimeError> {
    fs::File::open("/etc/mime.types").ok()
        // Return an error if /etc/mime.types could not be found.
        .map_or(Err(MimeError::OpenFile), |mut file| {
            // Create a buffer with the capacity of the file
            let mut contents = String::with_capacity(file.metadata().map(|x| x.len()).unwrap_or(0) as usize);
            // Store the contents of the file into the `contents variable`.
            file.read_to_string(&mut contents).ok()
                // Return an error if the file could not be read to the string.
                .map_or(Err(MimeError::ReadFileToString), |_| {
                    // Create a vector that will hold the list of video extensions.
                    let mut extension_list = Vec::new();
                    // Collect all the lines that start with "video" because they contain the video extensions.
                    for item in contents.lines().filter(|x| x.starts_with("video")) {
                        // Collect the pairs of extensions associated with that video type.
                        for extension in item.split_whitespace().skip(1).map(String::from).collect::<Vec<String>>() {
                            // Push each of the extensions that are discovered to the list of extensions.
                            extension_list.push(extension);
                        }
                    }
                    // Return the collected list of extensions as an `Ok(Vec<String>)`.
                    Ok(extension_list)
                })
        })
}
