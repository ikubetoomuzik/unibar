#![allow(dead_code, unused_variables)]
// Trying to actually organize this a bit.
// By: Curtis Jones <git@curtisjones.ca>
// Started on August 23, 2020

use super::{config::Config, valid_string::ValidString};
use std::os::raw::*;
use x11_dl::{xft, xlib};

pub struct ColourPalette {
    pub background: Vec<xft::XftColor>,
    pub highlight: Vec<xft::XftColor>,
    pub font: Vec<xft::XftColor>,
}

impl ColourPalette {
    pub unsafe fn destroy(
        mut self,
        xft: &xft::Xft,
        dpy: *mut xlib::Display,
        cmap: xlib::Colormap,
        visual: *mut xlib::Visual,
    ) {
        self.background
            .drain(..)
            .for_each(|mut col| (xft.XftColorFree)(dpy, visual, cmap, &mut col));
        self.highlight
            .drain(..)
            .for_each(|mut col| (xft.XftColorFree)(dpy, visual, cmap, &mut col));
        self.font
            .drain(..)
            .for_each(|mut col| (xft.XftColorFree)(dpy, visual, cmap, &mut col));
    }
}

pub struct Bar {
    xlib: xlib::Xlib,
    xft: xft::Xft,
    display: *mut xlib::Display,
    screen: c_int,
    x: c_int,
    y: c_int,
    width: c_int,
    height: c_int,
    back_colour: c_ulong,
    cmap: xlib::Colormap,
    visual: *mut xlib::Visual,
    root: c_long,
    window_id: c_ulong,
    draw: *mut xft::XftDraw,
    fonts: Vec<*mut xft::XftFont>,
    font_y: c_int,
    palette: ColourPalette,
    highlight_height: c_int,
    current_string: ValidString,
    prev_string: ValidString,
}

impl Bar {
    pub fn init(conf: Config) {}
}
