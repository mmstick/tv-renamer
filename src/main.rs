extern crate chrono;
extern crate tvdb;
#[cfg(feature = "enable_gtk")]
extern crate gtk;
#[cfg(feature = "enable_gtk")]
extern crate gdk;

pub mod backend;
pub mod frontend {
    #[cfg(not(feature = "enable_gtk"))]
    pub mod cli;
    #[cfg(feature = "enable_gtk")]
    pub mod gtk_interface;
}

#[cfg(not(feature = "enable_gtk"))]
fn main() { frontend::cli::launch() }

#[cfg(feature = "enable_gtk")]
fn main() { frontend::gtk_interface::launch(); }
