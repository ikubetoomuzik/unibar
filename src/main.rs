// Super simple status bar written in rust with direct Xlib.
// By: Curtis Jones
// Started on Ausust 06, 2020

use std::{ffi::CString, mem, os::raw::*, process, ptr};

use x11_dl::{xft, xlib};

use libc;

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
    let mut hn_buffer = vec![0 as c_char; hn_size];
    libc::gethostname(hn_buffer.as_mut_ptr() as *mut c_char, hn_size);
    let hn_list: *mut *mut c_char = [hn_buffer.as_mut_ptr()].as_mut_ptr();
    let mut hn_text_prop: xlib::XTextProperty = mem::MaybeUninit::uninit().assume_init();
    (xlib.XStringListToTextProperty)(hn_list, 1, &mut hn_text_prop);
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
            1920,                        // Length of the bar in x direction.
            32,                          // Height of the bar in y direction.
            0,                           // Border-width.
            xlib::CopyFromParent,        // Window depth.
            xlib::InputOutput as c_uint, // Window class.
            visual,                      // Visual type to use.
            xlib::CWBackPixel | xlib::CWColormap | xlib::CWOverrideRedirect | xlib::CWEventMask, // Mask for which attributes are set.
            &mut attributes, // Pointer to the attributes to use.
        );

        // Set the EWMH Atoms.
        set_atoms(&xlib, dpy, window);

        // Map this bitch
        (xlib.XMapWindow)(dpy, window);

        // Set up Xft
        let xft = xft::Xft::open().unwrap();
        let mut font_colour: xft::XftColor = mem::MaybeUninit::uninit().assume_init();
        (xft.XftColorAllocName)(
            dpy,
            visual,
            cmap,
            CString::new("#8BE9FD").unwrap().as_ptr() as *const c_char,
            &mut font_colour,
        );
        let font = (xft.XftFontOpenName)(
            dpy,
            screen,
            CString::new("Hack:size=12:antialias=true")
                .unwrap()
                .as_ptr() as *const c_char,
        );
        if font.is_null() {
            println!("NO FONT")
        }
        let draw = (xft.XftDrawCreate)(dpy, window, visual, cmap);

        // Init variables for event loop.
        let mut event: xlib::XEvent = mem::MaybeUninit::uninit().assume_init();

        // Event loop previously mentioned.
        loop {
            (xlib.XNextEvent)(dpy, &mut event);

            match event.get_type() {
                xlib::Expose => (xft.XftDrawString8)(
                    draw,                                                             // Draw item to display on.
                    &font_colour, // XftColor to use.
                    font,         // XftFont to use.
                    50,           // X (from top left)
                    22,           // Y (from top left)
                    CString::new("Hello world!").unwrap().as_ptr() as *const c_uchar, // String to print.
                    12, // Length of string.
                ),
                xlib::ButtonPress => break,
                _ => {}
            }
        }

        (xft.XftColorFree)(dpy, visual, cmap, &mut font_colour);
        (xft.XftDrawDestroy)(draw);
        (xlib.XFreeColormap)(dpy, cmap);
        (xlib.XDestroyWindow)(dpy, window);
        (xlib.XCloseDisplay)(dpy);
    }
}
