#![allow(
    clippy::missing_safety_doc,
    clippy::len_without_is_empty,
    clippy::not_unsafe_ptr_arg_deref
)]
// this file is basically just to weave together the seperate mod files.
// also the pub use statements bring objects over to our main file.
// By: Curtis Jones <mail@curtisjones.ca>
// Started on: September 07, 2020
//

#[macro_export]
/// Alot of the Xlib & Xft functions require pointers to uninitialized variables.
/// It is very much not in the rust theme but that's the price you pay for using c libraries.
macro_rules! init {
    () => {
        std::mem::MaybeUninit::uninit().assume_init();
    };
}

/// Main meat of the program, where all the direct access to Xlib lives.
pub mod bar;

/// Parsing the config file and adjusting based on command line args provided.
pub mod config;

/// Turning basic random characters into Input struct that the Bar struct can display to the
/// screen.
pub mod input;

/// Module containing optional additions to the bar.
pub mod optional;

/// To be used by the binary crate.
pub use bar::Bar;
pub use config::Config;
