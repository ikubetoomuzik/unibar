//! Util functions duh.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: December 20th, 2020

use super::super::{XFT, XINERAMA, XLIB};
use std::{ffi::CString, os::raw::*, ptr};
use x11_dl::{xft, xlib};

pub unsafe fn get_xlib_color(name: &str) -> c_ulong {
    let display = (XLIB.XOpenDisplay)(ptr::null());
    let cmap = (XLIB.XDefaultColormap)(display, (XLIB.XDefaultScreen)(display));
    let name = CString::new(name).unwrap();
    let mut temp: xlib::XColor = super::super::init!();
    (XLIB.XParseColor)(display, cmap, name.as_ptr(), &mut temp);
    (XLIB.XAllocColor)(display, cmap, &mut temp);
    (XLIB.XFreeColormap)(display, cmap);
    (XLIB.XCloseDisplay)(display);
    temp.pixel
}

pub unsafe fn get_xft_colour(
    display: *mut xlib::Display,
    visual: *mut xlib::Visual,
    cmap: xlib::Colormap,
    name: &str,
) -> xft::XftColor {
    let name = CString::new(name).unwrap();
    let mut tmp: xft::XftColor = init!();
    (XFT.XftColorAllocName)(
        display,
        visual,
        cmap,
        name.as_ptr() as *const c_char,
        &mut tmp,
    );
    tmp
}

pub unsafe fn get_font(display: *mut xlib::Display, screen: i32, name: &str) -> *mut xft::XftFont {
    let name = CString::new(name).unwrap();
    let tmp = (XFT.XftFontOpenName)(display, screen, name.as_ptr() as *const c_char);
    if tmp.is_null() {
        panic!("Font {} not found!!", name.to_str().unwrap())
    } else {
        tmp
    }
}

pub unsafe fn is_valid_xinerama_mon(display: *mut xlib::Display, monitor: usize) -> bool {
    match XINERAMA {
        Some(xin) => {
            // variable to store the total number of screens.
            let mut num_scr = 0;
            // Gets a dumb mutable pointer to an array of ScreenInfo objects for each screen.
            let scrns = (xin.XineramaQueryScreens)(display, &mut num_scr);
            // test if the monitor is valid
            monitor < num_scr as usize
        }
        None => false,
    }
}
