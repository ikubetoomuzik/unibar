#![allow(dead_code, unused_variables)]
// Super simple status bar written in rust with direct Xlib.
// By: Curtis Jones
// Started on Ausust 06, 2020

use std::{ffi::CString, io::stdin, mem, os::raw::*, process, ptr, sync::mpsc, thread, time};

use x11_dl::{xft, xlib, xrender::XGlyphInfo};

use libc;

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
        CString::new("Hack:size=12:antialias=true")
            .unwrap()
            .as_ptr() as *const c_char,
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

unsafe fn string_to_fonts_vec(
    xft: &xft::Xft,
    dpy: *mut xlib::Display,
    fonts: [Option<*mut xft::XftFont>; 4],
    string: &str,
) -> Vec<Option<*mut xft::XftFont>> {
    string.chars().fold(Vec::new(), |mut acc, ch| {
        acc.push(
            *fonts
                .iter()
                .find(|f| match f {
                    Some(f) => (xft.XftCharExists)(dpy, *f, ch as c_uint) == 1,
                    None => false,
                })
                .unwrap(),
        );
        acc
    })
}

fn _fonts_vec_to_write_pairs(
    fonts: [Option<*mut xft::XftFont>; 4],
    fonts_vec: Vec<Option<*mut xft::XftFont>>,
) -> Vec<(usize, usize)> {
    let mut tmp = Vec::new();
    let mut tmp_pair: (usize, usize) = (0, 0);
    fonts_vec.iter().for_each(|fo| {
        if *fo == None {
            if tmp_pair.0 == 0 {
                tmp_pair.1 += 1;
            } else {
                tmp.push(tmp_pair);
                tmp_pair = (0, 1);
            }
        } else {
            let font_idx = fonts.iter().enumerate().find(|f| *(f.1) == *fo).unwrap().0;
            if tmp_pair.0 == font_idx {
                tmp_pair.1 += 1;
            } else {
                tmp.push(tmp_pair);
                tmp_pair = (font_idx, 1);
            }
        }
    });
    tmp.push(tmp_pair);
    tmp
}

unsafe fn _string_pixel_width(
    xft: &xft::Xft,
    dpy: *mut xlib::Display,
    font: *mut xft::XftFont,
    string: &str,
) -> u32 {
    let mut extents: XGlyphInfo = mem::MaybeUninit::uninit().assume_init();
    (xft.XftTextExtentsUtf8)(
        dpy,
        font,
        string.as_bytes().as_ptr() as *mut c_uchar,
        string.as_bytes().len() as c_int,
        &mut extents,
    );
    extents.width as u32
}

unsafe fn _string_draw(
    xft: &xft::Xft,
    dpy: *mut xlib::Display,
    draw: *mut xft::XftDraw,
    fonts: [Option<*mut xft::XftFont>; 4],
    colors: &Vec<xft::XftColor>,
    string: &str,
) {
    let write_pairs =
        _fonts_vec_to_write_pairs(fonts, string_to_fonts_vec(xft, dpy, fonts, string));
    let mut x_offset = 0;
    let mut char_offset = 0;
    println!("{:#?}", write_pairs);
    for w_p in write_pairs.iter() {
        let chunk: String = string.chars().skip(char_offset).take(w_p.1).collect();
        (xft.XftDrawStringUtf8)(
            draw,                                        // Draw item to display on.
            &colors[w_p.0],                              // XftColor to use.
            fonts[w_p.0].unwrap(),                       // XftFont to use.
            50 + x_offset as i32,                        // X (from top left)
            20,                                          // Y (from top left)
            chunk.as_bytes().as_ptr() as *const c_uchar, // String to print.
            chunk.as_bytes().len() as c_int,             // Length of string.
        );
        x_offset += _string_pixel_width(xft, dpy, fonts[w_p.0].unwrap(), &chunk);
        char_offset += w_p.1;
    }
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

type Pair = (usize, usize);

struct ColourPalette {
    background: Vec<xft::XftColor>,
    highlight: Vec<xft::XftColor>,
    font: Vec<xft::XftColor>,
}

enum IndexType {
    BackgroundColour,
    HighlightColour,
    FontColour,
    FontFace,
}

struct ValidString {
    text: String,
    background_pair: Vec<Pair>,
    underline_pair: Vec<Pair>,
    font_pair: Vec<Pair>,
    font_colour_pair: Vec<Pair>,
}

impl ValidString {
    fn _empty() -> ValidString {
        ValidString {
            text: String::new(),
            background_pair: Vec::new(),
            underline_pair: Vec::new(),
            font_pair: Vec::new(),
            font_colour_pair: Vec::new(),
        }
    }

    fn make_background_pairs(
        fonts: &Vec<*mut xft::XftFont>,
        colours: &ColourPalette,
        input: String,
    ) {
        // Loop vars.
        let mut in_format_block = false;
        let mut next_is_index = false;
        let mut closing_block = false;
        let mut index_type = IndexType::FontColour;

        // Temp vars.
        let mut printed_chars = String::new();
        let mut background_vec: Vec<Pair> = Vec::new();
        let mut background_pair: Pair = (usize::MAX, 0);
        let mut highlight_vec: Vec<Pair> = Vec::new();
        let mut highlight_pair: Pair = (usize::MAX, 0);
        let mut font_colour_vec: Vec<Pair> = Vec::new();
        let mut font_colour_pair: Pair = (0, 0);
        let mut font_face_vec: Vec<Pair> = Vec::new();
        let mut font_face_pair: Pair = (usize::MAX, 0);

        input.chars().for_each(|ch| {
            if in_format_block {
                if closing_block {
                    match ch {
                        'B' => {
                            background_vec.push(background_pair);
                            background_pair = (usize::MAX, 0);
                        }
                        'H' => {
                            highlight_vec.push(highlight_pair);
                            highlight_pair = (usize::MAX, 0);
                        }
                        'F' => {
                            font_colour_vec.push(font_colour_pair);
                            font_colour_pair = (0, 0);
                        }
                        'f' => {
                            font_face_vec.push(font_face_pair);
                            font_face_pair = (usize::MAX, 0);
                        }
                        '}' => {
                            in_format_block = false;
                            closing_block = false;
                        }
                        _ => (),
                    }
                } else {
                    if next_is_index {
                        if let Some(d) = ch.to_digit(10) {
                            match index_type {
                                IndexType::BackgroundColour => {
                                    if d > (colours.background.len() - 1) as u32 {
                                        println!("Invalid background colour index -- TOO LARGE.");
                                    } else {
                                        background_vec.push(background_pair);
                                        background_pair = (d as usize, 0);
                                    }
                                }
                                IndexType::HighlightColour => {
                                    if d > (colours.highlight.len() - 1) as u32 {
                                        println!("Invalid highlight colour index -- TOO LARGE.");
                                    } else {
                                        highlight_vec.push(highlight_pair);
                                        highlight_pair = (d as usize, 0);
                                    }
                                }
                                IndexType::FontColour => {
                                    if d > (colours.font.len() - 1) as u32 {
                                        println!("Invalid font colour index -- TOO LARGE.");
                                    } else {
                                        font_colour_vec.push(font_colour_pair);
                                        font_colour_pair = (d as usize, 0);
                                    }
                                }
                                IndexType::FontFace => {
                                    if d > (fonts.len() - 1) as u32 {
                                        println!("Invalid font face index -- TOO LARGE.");
                                    } else {
                                        font_face_vec.push(font_face_pair);
                                        font_face_pair = (d as usize, 0);
                                    }
                                }
                            }
                        }
                        next_is_index = false;
                    } else {
                        match ch {
                            '/' => closing_block = true,
                            'B' => {
                                next_is_index = true;
                                index_type = IndexType::BackgroundColour;
                            }
                            'H' => {
                                next_is_index = true;
                                index_type = IndexType::HighlightColour;
                            }
                            'F' => {
                                next_is_index = true;
                                index_type = IndexType::FontColour;
                            }
                            'f' => {
                                next_is_index = true;
                                index_type = IndexType::FontFace;
                            }
                            '}' => in_format_block = false,
                            _ => (),
                        }
                    }
                }
            } else {
                match ch {
                    '{' => in_format_block = true,
                    _ => {
                        background_pair.1 += 1;
                        highlight_pair.1 += 1;
                        font_colour_pair.1 += 1;
                        font_face_pair.1 += 1;
                        printed_chars.push(ch);
                    }
                }
            }
        });

        // If our temp vars are 0 then we don't need this last push.
        if background_pair.1 != 0 {
            background_vec.push(background_pair)
        }
        if highlight_pair.1 != 0 {
            highlight_vec.push(highlight_pair);
        }
        if font_colour_pair.1 != 0 {
            font_colour_vec.push(font_colour_pair);
        }
        if font_face_pair.1 != 0 {
            font_face_vec.push(font_face_pair);
        }

        println!("Backgrond Colour Pairs:\n{:#?}\nHighlight Colour Pairs:\n{:#?}\nFont Colour Pairs:\n{:#?}\nFont Face Pairs:\n{:#?}\nActual String:\n{}", background_vec, highlight_vec, font_colour_vec, font_face_vec, printed_chars);
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
            get_font(&xft, dpy, screen, "Unifont Upper:size=16:antialias=true"),
            get_font(&xft, dpy, screen, "Hack:size=12:antialias=true"),
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
                    println!("{}", s);
                    ValidString::make_background_pairs(&fonts, &colour_palette, s);
                    (xlib.XClearWindow)(dpy, window);
                    // string_draw(&xft, dpy, draw, tmp_fonts, &tmp_font_colors, &s);
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
                    // We can draw unicode symbols woooooo. This was totally worth all this
                    // work... right?
                    // xlib::Expose => string_draw(
                    //     &xft,
                    //     dpy,
                    //     draw,
                    //     tmp_fonts,
                    //     &tmp_font_colors,
                    //     "HelloWorld! 🔉🔉 My name is Curtis🔉 and I can change fonts whenever.",
                    // ),
                    xlib::ButtonPress => break,
                    _ => {}
                }
            }

            // Trying this wait in the middle to see what happens. In the end the text bit has
            // gotta be first.
            wait(250);
        }

        // (xft.XftColorFree)(dpy, visual, cmap, &mut font_colour);
        (xft.XftDrawDestroy)(draw);
        (xlib.XFreeColormap)(dpy, cmap);
        (xlib.XDestroyWindow)(dpy, window);
        (xlib.XCloseDisplay)(dpy);
    }
}
