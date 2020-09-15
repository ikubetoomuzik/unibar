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
        mem::MaybeUninit::uninit().assume_init();
    };
}

mod bar;
mod config;
mod input;

pub use bar::Bar;
pub use config::Config;
