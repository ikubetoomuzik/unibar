//! Util functions duh.
//! By: Curtis Jones <mail@curtisjones.ca>
//! Started on: December 20th, 2020

use std::{ffi::CString, os::raw::*, ptr};
use x11_dl::{xft, xlib};

pub unsafe fn get_xlib_color(name: &str) -> c_ulong {
    let xlib = match xlib::Xlib::open() {
        Ok(xlib) => xlib,
        Err(e) => {
            eprintln!("Could not connect to xlib library!\nError: {}", e);
            std::process::exit(1);
        }
    };
    let display = (xlib.XOpenDisplay)(ptr::null());
    if display.is_null() {
        eprintln!("Could not connect to display!");
        std::process::exit(1);
    }
    let cmap = (xlib.XDefaultColormap)(display, (xlib.XDefaultScreen)(display));
    let name = CString::new(name).unwrap();
    let mut temp: xlib::XColor = super::super::init!();
    (xlib.XParseColor)(display, cmap, name.as_ptr(), &mut temp);
    (xlib.XAllocColor)(display, cmap, &mut temp);
    (xlib.XFreeColormap)(display, cmap);
    (xlib.XCloseDisplay)(display);
    temp.pixel
}

pub unsafe fn get_xft_colour(
    xft: &xft::Xft,
    display: *mut xlib::Display,
    visual: *mut xlib::Visual,
    cmap: xlib::Colormap,
    name: &str,
) -> xft::XftColor {
    let name = CString::new(name).unwrap();
    let mut tmp: xft::XftColor = init!();
    (xft.XftColorAllocName)(
        display,
        visual,
        cmap,
        name.as_ptr() as *const c_char,
        &mut tmp,
    );
    tmp
}

pub unsafe fn get_font(
    xft: &xft::Xft,
    display: *mut xlib::Display,
    screen: i32,
    name: &str,
) -> *mut xft::XftFont {
    let name = CString::new(name).unwrap();
    let tmp = (xft.XftFontOpenName)(display, screen, name.as_ptr() as *const c_char);
    if tmp.is_null() {
        panic!("Font {} not found!!", name.to_str().unwrap())
    } else {
        tmp
    }
}

pub unsafe fn get_xft_pointers() -> (xlib::Xlib, xft::Xft, *mut xlib::Display, i32) {
    let xlib = match xlib::Xlib::open() {
        Ok(xlib) => xlib,
        Err(e) => {
            eprintln!("Could not connect to xlib library!\nError: {}", e);
            std::process::exit(1);
        }
    };
    let xft = match xft::Xft::open() {
        Ok(xft) => xft,
        Err(e) => {
            eprintln!("Could not connect to xft library!\nError: {}", e);
            std::process::exit(1);
        }
    };
    let display = (xlib.XOpenDisplay)(ptr::null());
    if display.is_null() {
        eprintln!("Could not connect to display!");
        std::process::exit(1);
    }
    let screen = (xlib.XDefaultScreen)(display);
    (xlib, xft, display, screen)
}
