//! Module for config deserialization helper functions for Config struct.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: December 20th, 2020

use super::{util::*, ColourPalette, Monitor};
use serde::{Deserialize, Deserializer};
use std::os::raw::*;
use x11_dl::xft;

pub fn background_colour<'de, D>(inp: D) -> Result<c_ulong, D::Error>
where
    D: Deserializer<'de>,
{
    // first we get our strings.
    let string: String = String::deserialize(inp)?;
    // now we open our xlib connections.
    unsafe { Ok(get_xlib_color(&string)) }
}

// **************************************************************

pub fn monitor<'de, D>(inp: D) -> Result<Monitor, D::Error>
where
    D: Deserializer<'de>,
{
    // Ok(Monitor::deserialize(inp)?)
    todo!()
}

// **************************************************************

pub fn fonts<'de, D>(inp: D) -> Result<Vec<*mut xft::XftFont>, D::Error>
where
    D: Deserializer<'de>,
{
    // first we get our strings.
    let strings: Vec<String> = Vec::deserialize(inp)?;
    // now we open our xlib connections.
    unsafe {
        let (xlib, xft, display, screen) = get_xft_pointers();
        let res = strings
            .iter()
            .map(|fs| get_font(&xft, display, screen, fs))
            .collect();
        (xlib.XCloseDisplay)(display);
        Ok(res)
    }
}

// **************************************************************

#[derive(Debug, Deserialize)]
/// Intermediary type to parse the inital strings from the file.
struct ColourPaletteTemp {
    /// Colours for the background highlight.
    pub background: Vec<String>,
    /// Colours for the underline highlight.
    pub underline: Vec<String>,
    /// Colours for the fonts.
    pub font: Vec<String>,
}

impl ColourPaletteTemp {
    fn convert(&self) -> ColourPalette {
        unsafe {
            let (xlib, xft, display, screen) = get_xft_pointers();
            let visual = (xlib.XDefaultVisual)(display, screen);
            let cmap = (xlib.XDefaultColormap)(display, screen);
            ColourPalette {
                background: self
                    .background
                    .iter()
                    .map(|s| get_xft_colour(&xft, display, visual, cmap, s))
                    .collect(),
                underline: self
                    .underline
                    .iter()
                    .map(|s| get_xft_colour(&xft, display, visual, cmap, s))
                    .collect(),
                font: self
                    .font
                    .iter()
                    .map(|s| get_xft_colour(&xft, display, visual, cmap, s))
                    .collect(),
            }
        }
    }
}

pub fn colours<'de, D>(inp: D) -> Result<ColourPalette, D::Error>
where
    D: Deserializer<'de>,
{
    // first we get our strings.
    let strings: ColourPaletteTemp = ColourPaletteTemp::deserialize(inp)?;
    //return
    Ok(strings.convert())
}
