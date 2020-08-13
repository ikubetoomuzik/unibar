// Super simple status bar written in rust with direct Xlib.
// By: Curtis Jones
// Started on Ausust 06, 2020

use std::{ffi::CString, mem, os::raw::*, ptr};

use x11_dl::xlib;

unsafe fn get_color(
    xlib: &xlib::Xlib,
    dpy: *mut xlib::Display,
    cmap: xlib::Colormap,
    name: &str,
) -> c_ulong {
    let name = CString::new(name).unwrap();
    let mut temp: xlib::XColor = mem::MaybeUninit::uninit().assume_init();
    (xlib.XParseColor)(dpy, cmap, name.as_ptr(), &mut temp);
    (xlib.XAllocColor)(dpy, cmap, &mut temp);
    temp.pixel
}

fn main() {
    unsafe {
        // Open display connection.
        let xlib = xlib::Xlib::open().unwrap();
        let dpy = (xlib.XOpenDisplay)(ptr::null());

        let screen = (xlib.XDefaultScreen)(dpy);
        let root = (xlib.XRootWindow)(dpy, screen);
        let visual = (xlib.XDefaultVisual)(dpy, screen);
        let cmap = (xlib.XDefaultColormap)(dpy, screen);

        // let white = (xlib.XWhitePixel)(dpy, screen);
        let backc = get_color(&xlib, dpy, cmap, "#282A36");

        let mut attributes: xlib::XSetWindowAttributes = mem::MaybeUninit::uninit().assume_init();
        attributes.background_pixel = backc;
        attributes.colormap = cmap;
        attributes.override_redirect = xlib::True;
        attributes.event_mask = xlib::ExposureMask | xlib::ButtonPressMask;

        let window = (xlib.XCreateWindow)(
            dpy,
            root,
            0,
            0,
            1920,
            32,
            0,
            xlib::CopyFromParent,
            xlib::InputOutput as c_uint,
            visual,
            xlib::CWBackPixel | xlib::CWColormap | xlib::CWOverrideRedirect | xlib::CWEventMask,
            &mut attributes,
        );

        let title = CString::new("HELLO").unwrap();
        (xlib.XStoreName)(dpy, window, title.as_ptr() as *mut c_char);

        // Map this bitch
        (xlib.XMapWindow)(dpy, window);

        let mut event: xlib::XEvent = mem::MaybeUninit::uninit().assume_init();

        loop {
            (xlib.XNextEvent)(dpy, &mut event);

            match event.get_type() {
                xlib::ButtonPress => break,
                _ => {}
            }
        }

        (xlib.XFreeColormap)(dpy, cmap);
        (xlib.XDestroyWindow)(dpy, window);
        (xlib.XCloseDisplay)(dpy);
    }
}
