use std::fs;
use std::io::Read;

pub fn get_video_extensions() -> Result<Vec<String>, &'static str> {
    fs::File::open("/etc/mime.types").map(|mut file| {
        let mut contents = String::new();
        file.read_to_string(&mut contents).map(|_| {
            let mut output = Vec::new();
            for item in contents.lines().filter(|x| x.starts_with("video")) {
                for extension in item.split_whitespace().skip(1).map(String::from).collect::<Vec<String>>() {
                    output.push(extension);
                }
            }
            Ok(output)
        }).unwrap_or(Err("tv-renamer: unable to read file to string"))
    }).unwrap_or(Err("tv-renamer: unable to open /etc/mime.types"))
}
