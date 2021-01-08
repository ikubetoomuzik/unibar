// Struct to load config from file and cli args.
// By: Curtis Jones <mail@curtisjones.ca>
// Started on: September 07, 2020

// gonna start by implementing the loading from file bits.

use super::XLIB;
use serde::Deserialize;
use serde_yaml;
use std::{fs::read_to_string, os::raw::*, ptr};
use x11_dl::xft;

// modules to store deserialization helpers.
mod deser;
// modules to store deserialization helpers.
mod default;
// modules to store deserialization helpers.
mod util;
// module for just the monitor parsing stuff.
mod monitor;

use monitor::Monitor;
use util::{get_font, get_xft_colour, get_xlib_color};

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
    ColoursHighlight,
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

    fn set_a_pixel_height(&mut self, field: ConfigField, new_value: c_int) {
        match field {
            ConfigField::Height => self.height = new_value,
            ConfigField::UlHeight => self.ul_height = new_value,
            ConfigField::FontY => self.font_y = new_value,
            _ => unreachable!(),
        }
    }

    pub fn empty() -> Self {
        Self {
            top: true,
            monitor: Monitor::default(),
            height: 0,
            ul_height: 0,
            font_y: 0,
            fonts: Vec::new(),
            back_colour: 0,
            colours: ColourPalette::empty(),
        }
    }

    pub fn update(&mut self, field: ConfigField, new_value: &str) {
        match field {
            ConfigField::Top => match &new_value.to_lowercase()[..] {
                "top" => self.top = true,
                "bottom" => self.top = false,
                _ => (),
            },
            ConfigField::Monitor => self.monitor = Monitor::from_str(new_value),
            ConfigField::Fonts => unsafe {
                let display = (XLIB.XOpenDisplay)(ptr::null());
                let screen = (XLIB.XDefaultScreen)(display);
                let new_font = get_font(display, screen, new_value);
                (XLIB.XCloseDisplay)(display);
                self.fonts.push(new_font);
            },
            ConfigField::BackgroundColor => unsafe {
                self.back_colour = get_xlib_color(new_value);
            },
            // do the different height options together to avoid replication.
            ConfigField::Height | ConfigField::UlHeight | ConfigField::FontY => {
                if let Ok(int) = new_value.parse::<c_int>() {
                    self.set_a_pixel_height(field, int);
                }
            }
            // same for the colours.
            _ => unsafe {
                let display = (XLIB.XOpenDisplay)(ptr::null());
                let screen = (XLIB.XDefaultScreen)(display);
                let visual = (XLIB.XDefaultVisual)(display, screen);
                let cmap = (XLIB.XDefaultColormap)(display, screen);
                let new_colour = get_xft_colour(display, visual, cmap, new_value);
                (XLIB.XFreeColormap)(display, cmap);
                (XLIB.XCloseDisplay)(display);
                match field {
                    ConfigField::ColoursBackground => self.colours.background.push(new_colour),
                    ConfigField::ColoursFont => self.colours.font.push(new_colour),
                    ConfigField::ColoursHighlight => self.colours.underline.push(new_colour),
                    _ => unreachable!(),
                }
            },
        }
    }
}
