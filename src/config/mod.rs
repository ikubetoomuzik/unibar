// Struct to load config from file and cli args.
// By: Curtis Jones <mail@curtisjones.ca>
// Started on: September 07, 2020

// gonna start by implementing the loading from file bits.

use serde::Deserialize;
use serde_yaml;
use std::fs::read_to_string;
use std::os::raw::*;
use x11_dl::xft;

// modules to store deserialization helpers.
mod deser;
// modules to store deserialization helpers.
mod default;
// modules to store deserialization helpers.
mod util;

#[derive(Debug, Deserialize)]
enum Monitor {
    XDisplay,
    Xinerama(usize),
    XRandR(String),
}

#[derive(Debug)]
/// Private struct to contain colour information for the status bar.
/// Simpler than storing seperate fields as individual Vecs.
pub struct ColourPalette {
    /// Colours for the background highlight.
    pub background: Vec<xft::XftColor>,
    /// Colours for the underline highlight.
    pub underline: Vec<xft::XftColor>,
    /// Colours for the fonts.
    pub font: Vec<xft::XftColor>,
}

impl ColourPalette {
    fn empty() -> Self {
        Self {
            background: Vec::new(),
            underline: Vec::new(),
            font: Vec::new(),
        }
    }
}

enum ConfigField {
    Top,
    Monitor,
    Height,
    UlHeight,
    Fonts,
    FontY,
    BackgroundColor,
    ColoursBackground,
    ColoursHightlight,
    ColoursFont,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default::top")]
    // top or bottom
    pub top: bool,

    #[serde(default = "default::monitor", deserialize_with = "deser::monitor")]
    // xinerama montior list index for monitor
    pub monitor: Monitor,

    #[serde(default = "default::height")]
    // width or height of bar depending on pos.
    pub height: c_int,

    #[serde(default = "default::ul_height")]
    // width or height of bar depending on pos.
    pub ul_height: c_int,

    #[serde(default = "default::fonts", deserialize_with = "deser::fonts")]
    // Vec of strings listing the fonts in FcLookup form.
    pub fonts: Vec<*mut xft::XftFont>,

    #[serde(default = "default::font_y")]
    // pixel offset from the top of bar to bottom font.
    pub font_y: c_int,

    #[serde(
        default = "default::background_colour",
        deserialize_with = "deser::background_colour"
    )]
    // String of the hex color.
    pub back_colour: c_ulong,

    #[serde(deserialize_with = "deser::colours")]
    // user colours
    pub colours: ColourPalette,
}

impl Config {
    pub fn from_file(file: &str) -> Self {
        // Read the config file to a string.
        let conf_file = match read_to_string(file) {
            Ok(s) => s,
            Err(e) => {
                // If we can't read it, let the user know but continue on with the default.
                eprintln!("Could not read config file!\nFile: {}\nError: {}", file, e);
                // just empty string and it will use defaults.
                String::new()
            }
        };
        // Return our temp variable.
        serde_yaml::from_str(&conf_file).unwrap()
    }

    pub fn empty() -> Self {
        Self {
            top: true,
            monitor: Monitor::XDisplay,
            height: 0,
            ul_height: 0,
            font_y: 0,
            fonts: Vec::new(),
            back_colour: 0,
            colours: ColourPalette::empty(),
        }
    }

    pub fn update(&mut self, field: ConfigField, new_value: &str) {}
}
