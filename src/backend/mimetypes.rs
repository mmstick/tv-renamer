use std::fs;
use std::io::Read;

pub fn get_video_extensions() -> Result<Vec<String>, &'static str> {
    match fs::File::open("/etc/mime.types") {
        Ok(mut file) => {
            let mut contents = String::new();
            match file.read_to_string(&mut contents) {
                Ok(_) => {
                    let mut output = Vec::new();
                    for item in contents.lines().filter(|x| x.starts_with("video")) {
                        for extension in item.split_whitespace().skip(1).map(String::from).collect::<Vec<String>>() {
                            output.push(extension);
                        }
                    }
                    Ok(output)
                },
                Err(_) => Err("unable to read file to string")
            }
        },
        Err(_) => Err("unable to open /etc/mime.types")
    }
}
