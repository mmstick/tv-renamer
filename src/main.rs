extern crate chrono;
extern crate tvdb;
#[cfg(feature = "enable_gtk")]
extern crate gtk;
#[cfg(feature = "enable_gtk")]
extern crate gdk;

pub mod backend;
pub mod frontend {
    pub mod cli;
    #[cfg(feature = "enable_gtk")]
    pub mod gtk3;
}

use std::env;
use std::io;
use std::process::exit;

fn main() {
    let stderr = &mut io::stderr();

    let arguments = env::args().skip(1).collect::<Vec<String>>();

    let command = arguments.get(0).unwrap_or_else(|| {
        frontend::cli::launch(&arguments, stderr);
        exit(0);
    });

    match command.as_str() {
        "cli" => frontend::cli::launch(&arguments[1..], stderr),
        "gtk" => launch_gtk_interface(),
        _     => frontend::cli::launch(&arguments, stderr)
    }
}

#[cfg(not(feature = "enable_gtk"))]
fn launch_gtk_interface() {
    println!("tv-renamer: tv-renamer was not built with GTK3 support.");
}

#[cfg(feature = "enable_gtk")]
fn launch_gtk_interface() {
    frontend::gtk3::launch();
}
