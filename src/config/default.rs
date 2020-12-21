//! Module to hold default values for the Config struct.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: December 20th, 2020

// imports
use super::{
    util::{get_font, get_xft_pointers, get_xlib_color},
    Monitor,
};
use std::os::raw::{c_int, c_ulong};
use x11_dl::xft;

// export constants
pub const fn top() -> bool {
    true
}
pub const fn height() -> c_int {
    32
}
pub const fn ul_height() -> c_int {
    4
}
pub const fn font_y() -> c_int {
    20
}
pub const fn monitor() -> Monitor {
    Monitor::XDisplay
}
pub const fn fonts() -> Vec<*mut xft::XftFont> {
    unsafe {
        let (xlib, xft, display, screen) = get_xft_pointers();
        let res = vec![get_font(&xft, display, screen, "mono:size=12")];
        (xlib.XCloseDisplay)(display);
        res
    }
}
pub const fn background_colour() -> c_ulong {
    unsafe { get_xlib_color("#000000") }
}
