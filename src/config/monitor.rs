//! Module to hold all of the monitor parsing code.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: January 8th, 2020

use super::super::{XINERAMA, XLIB, XRANDR};
use serde::Deserialize;
use std::os::raw::*;
use std::ptr;
use x11_dl::xlib;

#[derive(Debug, Deserialize)]
pub enum Monitor {
    XDisplay(MonitorInfo),
    Xinerama(usize, MonitorInfo),
    XRandR(String, MonitorInfo),
}

impl Monitor {
    pub fn default() -> Self {
        let dpy = (XLIB.XOpenDisplay)(ptr::null());
        let info = MonitorInfo::default(dpy);
        (XLIB.XCloseDisplay)(dpy);
        Monitor::XDisplay(info)
    }
    pub fn from_str(input: &str) -> Self {
        unsafe {
            let display = (XLIB.XOpenDisplay)(ptr::null());
            let result: Self = if XINERAMA.is_some()
                && (XINERAMA.unwrap().XineramaIsActive)(display) > 0
                && input.parse::<usize>().is_ok()
            {
                let xin = XINERAMA.unwrap();
                let monitor = input.parse::<usize>().unwrap();
                // Temp var because the query strings funtion needs a pointer to a c_int.
                let mut num_scr = 0;
                // Gets a dumb mutable pointer to an array of ScreenInfo objects for each screen.
                let scrns = (xin.XineramaQueryScreens)(display, &mut num_scr);
                // Using pointer arithmetic and the num_scr variable from the previous function we
                // fold the range into a Vec of ScreenInfo objects.
                let scrns = (0..num_scr as usize).fold(Vec::new(), |mut acc, i| {
                    acc.push(*scrns.add(i));
                    acc
                });
                // If the monitor set is not available, use first screen.
                let screen = if monitor >= num_scr as usize {
                    return Monitor::XDisplay(MonitorInfo::default(display));
                } else {
                    scrns[monitor]
                };
                Monitor::Xinerama(
                    monitor,
                    MonitorInfo {
                        x: screen.x_org as c_int,
                        y: screen.y_org as c_int,
                        width: screen.width as c_int,
                        height: screen.height as c_int,
                    },
                )
            } else if let Some(xrr) = XRANDR {
                let screen = (XLIB.XDefaultScreen)(display);
                let root = (XLIB.XRootWindow)(display, screen);
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
                        // Inside the output object we have another object called CRTC.
                        let crtc = *(xrr.XRRGetCrtcInfo)(display, resources, info.crtc);
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
                        ac.push((
                            String::from_utf8(name).unwrap(),
                            MonitorInfo {
                                x: crtc.x,
                                y: crtc.y,
                                width: crtc.width as c_int,
                                height: crtc.height as c_int,
                            },
                        ));
                        ac
                    });
                    // Append the tmp vec of usable monitor info to the result and finally move
                    // onto the next Monitor.
                    acc.append(&mut tmp);
                    acc
                });
                match mons.iter().find(|m| input == m.0) {
                    Some(monitor) => Monitor::XRandR(monitor.0, monitor.1),
                    None => Monitor::XDisplay(MonitorInfo::default(display)),
                }
            } else {
                Monitor::XDisplay(MonitorInfo::default(display))
            };
            (XLIB.XCloseDisplay)(display);
            result
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct MonitorInfo {
    x: c_int,
    y: c_int,
    width: c_int,
    height: c_int,
}

impl MonitorInfo {
    /// basically the monitor info for a full xdisplay.
    fn default(display: *mut xlib::Display) -> Self {
        let width: c_int;
        let height: c_int;
        unsafe {
            let screen = (XLIB.XDefaultScreen)(display);
            width = (XLIB.XDisplayWidth)(display, screen);
            height = (XLIB.XDisplayHeight)(display, screen);
        }
        Self {
            x: 0,
            y: 0,
            width,
            height,
        }
    }
}
