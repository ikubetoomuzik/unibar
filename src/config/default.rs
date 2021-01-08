//! Module to hold default values for the Config struct.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: December 20th, 2020

// imports
use super::{
    super::XLIB,
    monitor::Monitor,
    util::{get_font, get_xlib_color},
};
use std::{
    os::raw::{c_int, c_ulong},
    ptr,
};
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
    Monitor::default()
}
pub const fn fonts() -> Vec<*mut xft::XftFont> {
    unsafe {
        let display = (XLIB.XOpenDisplay)(ptr::null());
        let screen = (XLIB.XDefaultScreen)(display);
        let res = vec![get_font(display, screen, "mono:size=12")];
        (XLIB.XCloseDisplay)(display);
        res
    }
}
pub const fn background_colour() -> c_ulong {
    unsafe { get_xlib_color("#000000") }
}
