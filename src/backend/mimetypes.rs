use std::fs::{File, read_dir};
use std::io::{Error, Read};

/// Obtains a list of video extensions from the `/etc/mime.types` file on Linux.
pub fn get_extensions(kind: &'static str) -> Result<Vec<String>, Error> {
    let mut extension_list = Vec::new();
    let mut buffer = String::new();

    for extension_file in read_dir(&["/usr/share/mime/", kind].concat())? {
        let extension_file = extension_file?;
        let mut file = File::open(extension_file.path())?;
        let capacity = file.metadata().map(|x| x.len()).unwrap_or(0);
        buffer.reserve(capacity as usize);
        file.read_to_string(&mut buffer)?;

        for line in buffer.split('\n') {
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
