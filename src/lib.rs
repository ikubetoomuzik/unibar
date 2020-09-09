// this file is basically just to weave together the seperate mod files.
// also the pub use statements bring objects over to our main file.
// By: Curtis Jones <mail@curtisjones.ca>
// Started on: September 07, 2020
//

mod bar;
mod config;
mod valid_string;

pub use bar::*;
pub use config::*;
pub use valid_string::*;
