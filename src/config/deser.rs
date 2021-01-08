//! Module for config deserialization helper functions for Config struct.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: December 20th, 2020

use super::{super::XLIB, monitor::Monitor, util::*, ColourPalette};
use serde::{Deserialize, Deserializer};
use std::{os::raw::*, ptr};
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
    let inp: String = String::deserialize(inp)?;
    Ok(Monitor::from_str(&inp))
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
        let display = (XLIB.XOpenDisplay)(ptr::null());
        let screen = (XLIB.XDefaultScreen)(display);
        let res = strings
            .iter()
            .map(|fs| get_font(display, screen, fs))
            .collect();
        (XLIB.XCloseDisplay)(display);
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
            let display = (XLIB.XOpenDisplay)(ptr::null());
            let screen = (XLIB.XDefaultScreen)(display);
            let visual = (XLIB.XDefaultVisual)(display, screen);
            let cmap = (XLIB.XDefaultColormap)(display, screen);
            let val = ColourPalette {
                background: self
                    .background
                    .iter()
                    .map(|s| get_xft_colour(display, visual, cmap, s))
                    .collect(),
                underline: self
                    .underline
                    .iter()
                    .map(|s| get_xft_colour(display, visual, cmap, s))
                    .collect(),
                font: self
                    .font
                    .iter()
                    .map(|s| get_xft_colour(display, visual, cmap, s))
                    .collect(),
            };
            (XLIB.XFreeColormap)(display, cmap);
            (XLIB.XCloseDisplay)(display);
            val
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
