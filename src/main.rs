#![allow(dead_code, unused_imports, unused_variables)]
// Super simple status bar written in rust with direct Xlib.
// By: Curtis Jones
// Started on Ausust 06, 2020

use libc;
use std::{ffi::CString, io::stdin, mem, os::raw::*, process, ptr, sync::mpsc, thread, time};
use unibar::*;
use x11_dl::{xft, xlib, xrender::XGlyphInfo};

fn wait(time_ms: u64) {
    let time = time::Duration::from_millis(time_ms);
    thread::sleep(time);
}

unsafe fn get_atom(xlib: &xlib::Xlib, dpy: *mut xlib::Display, name: &str) -> xlib::Atom {
    (xlib.XInternAtom)(
        dpy,
        CString::new(name).unwrap().as_ptr() as *const c_char,
        xlib::False,
    )
}

unsafe fn set_atoms(xlib: &xlib::Xlib, dpy: *mut xlib::Display, window: c_ulong) {
    // Set the WM_NAME.
    let title = CString::new("Unibar-rs").unwrap();
    (xlib.XStoreName)(dpy, window, title.as_ptr() as *mut c_char);

    // Set WM_CLASS
    let class: *mut xlib::XClassHint = (xlib.XAllocClassHint)();
    let cl_names = [
        CString::new("unibar").unwrap(),
        CString::new("Unibar").unwrap(),
    ];
    (*class).res_name = cl_names[0].as_ptr() as *mut c_char;
    (*class).res_class = cl_names[1].as_ptr() as *mut c_char;
    (xlib.XSetClassHint)(dpy, window, class);

    // Set WM_CLIENT_MACHINE
    let hn_size = libc::sysconf(libc::_SC_HOST_NAME_MAX) as libc::size_t;
    let hn_buffer: *mut c_char = vec![0 as c_char; hn_size].as_mut_ptr();
    libc::gethostname(hn_buffer, hn_size);
    let mut hn_list = [hn_buffer];
    let mut hn_text_prop: xlib::XTextProperty = mem::MaybeUninit::uninit().assume_init();
    (xlib.XStringListToTextProperty)(hn_list.as_mut_ptr(), 1, &mut hn_text_prop);
    (xlib.XSetWMClientMachine)(dpy, window, &mut hn_text_prop);

    // Set _NET_WM_PID
    let pid = [process::id()].as_ptr();
    let wm_pid_atom = get_atom(&xlib, dpy, "_NET_WM_PID");
    (xlib.XChangeProperty)(
        dpy,
        window,
        wm_pid_atom,
        xlib::XA_CARDINAL,
        32,
        xlib::PropModeReplace,
        pid as *const c_uchar,
        1,
    );

    // Set _NET_WM_DESKTOP
    let dk_num = [0xFFFFFFFF as c_ulong].as_ptr();
    let wm_dktp_atom = get_atom(&xlib, dpy, "_NET_WM_DESKTOP");
    (xlib.XChangeProperty)(
        dpy,
        window,
        wm_dktp_atom,
        xlib::XA_CARDINAL,
        32,
        xlib::PropModeReplace,
        dk_num as *const c_uchar,
        1,
    );

    // Change _NET_WM_STATE
    let wm_state_atom = get_atom(&xlib, dpy, "_NET_WM_STATE");
    let state_atoms = [
        get_atom(&xlib, dpy, "_NET_WM_STATE_STICKY"),
        get_atom(&xlib, dpy, "_NET_WM_STATE_ABOVE"),
    ];
    (xlib.XChangeProperty)(
        dpy,
        window,
        wm_state_atom,
        xlib::XA_ATOM,
        32,
        xlib::PropModeAppend,
        state_atoms.as_ptr() as *const c_uchar,
        2,
    );

    // Set the _NET_WM_STRUT[_PARTIAL]
    let strut: [c_long; 12] = [0, 0, 32, 0, 0, 0, 0, 0, 0, 1920, 0, 0];
    let strut_atoms = [
        get_atom(&xlib, dpy, "_NET_WM_STRUT"),
        get_atom(&xlib, dpy, "_NET_WM_STRUT_PARTIAL"),
    ];
    (xlib.XChangeProperty)(
        dpy,
        window,
        strut_atoms[0],
        xlib::XA_CARDINAL,
        32,
        xlib::PropModeReplace,
        strut.as_ptr() as *const c_uchar,
        4,
    );
    (xlib.XChangeProperty)(
        dpy,
        window,
        strut_atoms[1],
        xlib::XA_CARDINAL,
        32,
        xlib::PropModeReplace,
        strut.as_ptr() as *const c_uchar,
        12,
    );

    // Set the _NET_WM_WINDOW_TYPE atom
    let win_type_atom = get_atom(&xlib, dpy, "_NET_WM_WINDOW_TYPE");
    let dock_atom = [get_atom(&xlib, dpy, "_NET_WM_WINDOW_TYPE_DOCK")];
    (xlib.XChangeProperty)(
        dpy,
        window,
        win_type_atom,
        xlib::XA_ATOM,
        32,
        xlib::PropModeReplace,
        dock_atom.as_ptr() as *const c_uchar,
        1,
    );
}

unsafe fn poll_events(
    xlib: &xlib::Xlib,
    dpy: *mut xlib::Display,
    window: c_ulong,
    event_mask: c_long,
    return_event: &mut xlib::XEvent,
) -> bool {
    if (xlib.XCheckWindowEvent)(dpy, window, event_mask, return_event) == 1 {
        return true;
    } else {
        return false;
    }
}

unsafe fn get_font(
    xft: &xft::Xft,
    dpy: *mut xlib::Display,
    screen: c_int,
    name: &str,
) -> *mut xft::XftFont {
    let tmp = (xft.XftFontOpenName)(
        dpy,
        screen,
        CString::new(name).unwrap().as_ptr() as *const c_char,
    );
    if tmp.is_null() {
        panic!("Font {} not found!!", name)
    } else {
        tmp
    }
}

unsafe fn get_xft_colour(
    xft: &xft::Xft,
    dpy: *mut xlib::Display,
    cmap: xlib::Colormap,
    visual: *mut xlib::Visual,
    name: &str,
) -> xft::XftColor {
    let mut tmp: xft::XftColor = mem::MaybeUninit::uninit().assume_init();
    (xft.XftColorAllocName)(
        dpy,
        visual,
        cmap,
        CString::new(name).unwrap().as_ptr() as *const c_char,
        &mut tmp,
    );
    tmp
}

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

fn poll_stdin(stdin: std::io::Stdin, send: mpsc::Sender<String>) {
    loop {
        let mut tmp = String::new();
        stdin.read_line(&mut tmp).expect("wont fail.");
        if tmp.is_empty() {
            return;
        } else {
            send.send(tmp.trim().to_owned()).unwrap();
        }
    }
}

unsafe fn default_font_idx(
    xft: &xft::Xft,
    dpy: *mut xlib::Display,
    fonts: &Vec<*mut xft::XftFont>,
    chr: char,
) -> usize {
    match fonts
        .iter()
        .position(|&f| (xft.XftCharExists)(dpy, f, chr as c_uint) > 0)
    {
        Some(i) => i,
        None => 0,
    }
}

fn main() {
    unsafe {
        // Open display connection.
        let xlib = xlib::Xlib::open().unwrap();
        let dpy = (xlib.XOpenDisplay)(ptr::null());
        let xft = xft::Xft::open().unwrap();

        let screen = (xlib.XDefaultScreen)(dpy);
        let root = (xlib.XRootWindow)(dpy, screen);
        let visual = (xlib.XDefaultVisual)(dpy, screen);
        let cmap = (xlib.XDefaultColormap)(dpy, screen);
        // let height = (xlib.XDisplayHeight)(dpy, screen);
        let width = (xlib.XDisplayWidth)(dpy, screen);
        let width = if width > 1920 { 1920 } else { width };

        // let white = (xlib.XWhitePixel)(dpy, screen);
        let backc = get_color(&xlib, dpy, cmap, "#282A36");

        // Manually set the attributes here so we can get more fine grain control.
        let mut attributes: xlib::XSetWindowAttributes = mem::MaybeUninit::uninit().assume_init();
        attributes.background_pixel = backc;
        attributes.colormap = cmap;
        attributes.override_redirect = xlib::False;
        attributes.event_mask = xlib::ExposureMask | xlib::ButtonPressMask;

        // Use the attributes we created to make a window.
        let window = (xlib.XCreateWindow)(
            dpy,                         // Display to use.
            root,                        // Parent window.
            0,                           // X position (from top-left.
            0,                           // Y position (from top-left.
            width as u32,                // Length of the bar in x direction.
            32,                          // Height of the bar in y direction.
            0,                           // Border-width.
            xlib::CopyFromParent,        // Window depth.
            xlib::InputOutput as c_uint, // Window class.
            visual,                      // Visual type to use.
            xlib::CWBackPixel | xlib::CWColormap | xlib::CWOverrideRedirect | xlib::CWEventMask, // Mask for which attributes are set.
            &mut attributes, // Pointer to the attributes to use.
        );

        // Create draw object for the window.
        let draw = (xft.XftDrawCreate)(dpy, window, visual, cmap);

        // Set the EWMH Atoms.
        set_atoms(&xlib, dpy, window);

        // Map this bitch
        (xlib.XMapWindow)(dpy, window);

        // Init variables for event loop.
        let mut event: xlib::XEvent = mem::MaybeUninit::uninit().assume_init();

        // Test vars
        let fonts = vec![
            get_font(&xft, dpy, screen, "Anonymous Pro:size=12:antialias=true"),
            get_font(&xft, dpy, screen, "Unifont Upper:size=14:antialias=true"),
            get_font(&xft, dpy, screen, "Siji:size=8:"),
            get_font(&xft, dpy, screen, "Unifont:size=14:antialias=true"),
        ];
        let colour_palette = ColourPalette {
            background: vec![
                get_xft_colour(&xft, dpy, cmap, visual, "#000000"),
                get_xft_colour(&xft, dpy, cmap, visual, "#787878"),
                get_xft_colour(&xft, dpy, cmap, visual, "#FFFFFF"),
            ],
            highlight: vec![
                get_xft_colour(&xft, dpy, cmap, visual, "#FF0000"),
                get_xft_colour(&xft, dpy, cmap, visual, "#00FF00"),
                get_xft_colour(&xft, dpy, cmap, visual, "#0000FF"),
            ],
            font: vec![
                get_xft_colour(&xft, dpy, cmap, visual, "#FF79C6"),
                get_xft_colour(&xft, dpy, cmap, visual, "#8BE9FD"),
                get_xft_colour(&xft, dpy, cmap, visual, "#0F0F0F"),
            ],
        };

        // Input thread. Has to be seperate to not block xlib events.
        let lock = stdin();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || poll_stdin(lock, tx));

        // Event loop previously mentioned.
        loop {
            // Do we have some input waiting?
            match rx.try_recv() {
                Ok(s) => {
                    // Small kill marker for when I can't click.
                    if s == "QUIT NOW" {
                        break;
                    }
                    let input =
                        ValidString::parse_input_string(&xft, dpy, &fonts, &colour_palette, s);
                    (xlib.XClearWindow)(dpy, window);
                    input.draw(&xft, dpy, draw, &colour_palette, &fonts, 0, 18, 4);
                }
                Err(_) => (),
            }

            // Any events we should care about?
            if poll_events(
                &xlib,
                dpy,
                window,
                xlib::ExposureMask | xlib::ButtonPressMask,
                &mut event,
            ) {
                match event.get_type() {
                    xlib::ButtonPress => break,
                    _ => {}
                }
            }

            // Trying this wait in the middle to see what happens. In the end the text bit has
            // gotta be first.
            wait(250);
        }

        colour_palette.destroy(&xft, dpy, cmap, visual);
        (xft.XftDrawDestroy)(draw);
        (xlib.XFreeColormap)(dpy, cmap);
        (xlib.XDestroyWindow)(dpy, window);
        (xlib.XCloseDisplay)(dpy);
    }
}
