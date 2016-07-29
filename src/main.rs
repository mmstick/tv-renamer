extern crate tvdb;
extern crate gtk;
extern crate gdk;
mod backend;
mod frontend {
    pub mod cli;
    pub mod gtk3;
}

use std::env;

fn main() {
    let arguments = env::args().skip(1).collect::<Vec<String>>();
    match arguments.get(0) {
        Some(command) => {
            match command.as_str() {
                "cli" => frontend::cli::interface(&arguments[1..]),
                "gtk" => frontend::gtk3::interface(),
                _     => frontend::cli::interface(&arguments)
            }
        },
        None => frontend::cli::interface(&arguments)
    }
}
