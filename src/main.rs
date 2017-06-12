extern crate tvdb;
extern crate gtk;
extern crate gdk;
#[macro_use] extern crate quick_error;

mod backend;
mod frontend {
    pub mod cli;
    pub mod gtk3;
}

use std::env;

fn main() {
    let mut arguments = env::args();
    if arguments.next().unwrap().ends_with("tv-renamer-gtk") {
        frontend::gtk3::interface();
    } else {
        frontend::cli::interface(arguments);
    }
}
