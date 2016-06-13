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

#[cfg(not(feature = "enable_gtk"))]
fn launch_gtk_interface() {
    println!("tv-renamer: tv-renamer was not built with GTK3 support.");
}

#[cfg(feature = "enable_gtk")]
fn launch_gtk_interface() {
    frontend::gtk3::launch();
}

fn main() {
    let mut iter = std::env::args().skip(1);
    let command = iter.next().unwrap_or(String::from("cli"));
    let arguments = iter.collect::<Vec<String>>();
    match command.as_str() {
        "cli" => frontend::cli::launch(&arguments),
        "gtk" => launch_gtk_interface(),
        _     => panic!("tv-renamer: expected argument `cli` or `gtk`")
    }
}
