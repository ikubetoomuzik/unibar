// this file is basically just to weave together the seperate mod files.
// also the pub use statements bring objects over to our main file.
// By: Curtis Jones <mail@curtisjones.ca>
// Started on: September 07, 2020
//

// imports
use std::ptr;
use x11_dl::{xft, xinerama, xlib, xrandr};

// required statics
static XLIB: xlib::Xlib = xlib::Xlib::open().expect("Unable to make connection to Xlib library.");
static XFT: xft::Xft = xft::Xft::open().expect("Unable to make connection to the Xft library.");
// optional statics
static XINERAMA: Option<xinerama::Xlib> = match xinerama::Xlib::open() {
    Ok(xin) => {
        let dpy = (XLIB.XOpenDisplay)(ptr::null());
        // convert from c bool to rust bool by comparing to 0
        let active = (xin.XineramaIsActive)(dpy) > 0;
        // close our local display pointer
        (XLIB.XCloseDisplay)(dpy);
        if active {
            Some(xin)
        } else {
            None
        }
    }
    // don't care why, just care it didn't connect.
    Err(_) => None,
};
// basically we don't care why xrandr doesn't connect,
// we just gotta know if it does or doesn't.
static XRANDR: Option<xrandr::Xrandr> = xrandr::Xrandr::open().ok();

/// static library connections.
#[macro_export]
/// Alot of the Xlib & Xft functions require pointers to uninitialized variables.
/// It is very much not in the rust theme but that's the price you pay for using c libraries.
macro_rules! init {
    () => {
        std::mem::MaybeUninit::uninit().assume_init();
    };
}

/// Main meat of the program, where all the direct access to Xlib lives.
mod bar;

/// Parsing the config file and adjusting based on command line args provided.
mod config;

/// Turning basic random characters into Input struct that the Bar struct can display to the
/// screen.
mod input;

/// To be used by the binary crate.
pub use bar::{gen_config, Bar};
