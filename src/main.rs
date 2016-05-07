extern crate chrono;
extern crate tvdb;
#[cfg(feature = "enable_gtk")] extern crate gtk;
#[cfg(feature = "enable_gtk")] extern crate gdk;

pub mod backend {
    pub mod common;
    pub mod man;
    pub mod traits;
}
pub mod frontend {
    pub mod cli;
    #[cfg(feature = "enable_gtk")] pub mod gtk_interface;
}

fn main() {
    if std::env::args().any(|x| x == "--gui") { launch_gui(); } else { frontend::cli::launch() }
}

#[cfg(not(feature = "enable_gtk"))]
fn launch_gui() { println!("tv-renamer: GUI support was disabled for this build."); }

#[cfg(feature = "enable_gtk")]
fn launch_gui() { frontend::gtk_interface::launch(); }
