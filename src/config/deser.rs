//! Module for config deserialization helper functions for Config struct.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: December 20th, 2020

use super::{util::*, ColourPalette, Monitor};
use serde::{Deserialize, Deserializer};
use std::os::raw::*;
use x11_dl::{xft, xinerama, xlib, xrandr};

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
    if let Ok(mon) = inp.parse::<usize>() {
        unsafe {
            // here is the xinerama stuff.
            let (xlib, xft, display, screen) = get_xft_pointers();
            // xinerama stuff
            // grab the monitor number set in the conf.
            match xinerama::Xlib::open() {
                Ok(xin) => {
                    // Even if we have connected to the library that doesn't necessarily
                    // mean that Xinerama is active. So we make another check here.
                    match (xin.XineramaIsActive)(display) {
                        // Old school c bool where 0 is false and anything else is true.
                        0 => {
                            // Close out the temp display we opened.
                            (xlib.XCloseDisplay)(display);
                            return Ok(Monitor::XDisplay);
                        }
                        _ => {
                            // Temp var because the query strings funtion needs a pointer to a c_int.
                            let mut num_scr = 0;
                            // Gets a dumb mutable pointer to an array of ScreenInfo objects for each screen.
                            let scrns = (xin.XineramaQueryScreens)(display, &mut num_scr);
                            // Using pointer arithmetic and the num_scr variable from the previous function we
                            // Close out the temp display we opened.
                            (xlib.XCloseDisplay)(display);
                            // fold the range into a Vec of ScreenInfo objects.
                            let scrns = (0..num_scr as usize).fold(Vec::new(), |mut acc, i| {
                                acc.push(*scrns.add(i));
                                acc
                            });
                            // If the monitor set is not available, use full display.
                            if mon >= num_scr as usize {
                                return Ok(Monitor::XDisplay);
                            } else {
                                return Ok(Monitor::Xinerama(mon));
                            };
                        }
                    }
                }
                Err(e) => {
                    // Close out the temp display we opened.
                    (xlib.XCloseDisplay)(display);
                    return Ok(Monitor::XDisplay);
                }
            }
        }
    } else if let Ok(xrr) = xrandr::Xrandr::open() {
        unsafe {
            // here is the xrandr stuff.
            // xrandr stuff
            // again we load a seperate pointer to the display, because otherwise we get
            // segfaults. those are hard enough to understand when the language intends that as
            // an error, but rust has a real hard time explaining so we just eat this and try
            // again.
            let (xlib, xft, display, screen) = get_xft_pointers();
            let root = (xlib.XDefaultRootWindow)(display);
            let resources = (xrr.XRRGetScreenResources)(display, root);
            // doesn't matter what we set here, the GetMonitors function overrides with the
            // real val before we read.
            let mut num_mon: c_int = 0;
            // Now we query the library for a list on monitors and it helpfully (kill me now)
            // returns a pointer to the first monitor and a total count in the num_mon var.
            let mons = (xrr.XRRGetMonitors)(display, root, xlib::True, &mut num_mon);
            // translating between weird c structs and pretty rust ones.
            // we create a range iterator as large as the number of monitors and use pointer
            // arithmetic to collect those into a Rust Vec.
            let mons = (0..num_mon as usize).fold(Vec::new(), |mut acc, i| {
                let m = *mons.add(i);
                // The way xrandr organizes information probably makes sense if you wrote the
                // library. or maybe if you can find docs because they either dont exist or
                // suck. Basically every Xrandr Monitor has outputs. Unless you have multiple cords
                // from pc to monitor you only have one output.
                let mut tmp = (0..m.noutput as usize).fold(Vec::new(), |mut ac, i| {
                    let output = *m.outputs.add(i);
                    let info = *(xrr.XRRGetOutputInfo)(display, resources, output);
                    // This library returns strings just like arrays, you get a pointer to the
                    // first char and a count. So we do the same iteration trick to collect
                    // into a string.
                    let name = (0..info.nameLen as usize).fold(Vec::new(), |mut acc, j| {
                        acc.push(*info.name.add(j) as c_uchar);
                        acc
                    });
                    // Inside the CRTC is information that an actual human or basic ass application
                    // like this may need. So we grab what we need there and push a tuple
                    // containing the info into the vec, instead of the full monitor object, to
                    // avoid all this abstraction craziness later.
                    ac.push(String::from_utf8(name).unwrap());
                    ac
                });
                // Append the tmp vec of usable monitor info to the result and finally move
                // onto the next Monitor.
                acc.append(&mut tmp);
                acc
            });
            (xlib.XCloseDisplay)(display);
            if mons.contains(&inp) {
                Ok(Monitor::XRandR(inp))
            } else {
                Ok(Monitor::XDisplay)
            }
        }
    } else {
        Ok(Monitor::XDisplay)
    }
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
            let val = ColourPalette {
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
            };
            (xlib.XFreeColormap)(display, cmap);
            (xlib.XCloseDisplay)(display);
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
