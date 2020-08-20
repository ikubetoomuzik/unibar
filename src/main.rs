#![allow(dead_code, unused_variables)]
// Super simple status bar written in rust with direct Xlib.
// By: Curtis Jones
// Started on Ausust 06, 2020

use libc;
use std::{ffi::CString, io::stdin, mem, os::raw::*, process, ptr, sync::mpsc, thread, time};
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

unsafe fn string_pixel_width(
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

unsafe fn string_pixel_dist_vec(
    xft: &xft::Xft,
    dpy: *mut xlib::Display,
    fonts: &Vec<*mut xft::XftFont>,
    ft_dpy_ojs: &Vec<FontDisplay>,
    string: &str,
) -> Vec<u32> {
    let mut res = Vec::new();
    for i in 0..string.chars().count() {
        let ft_idx = ft_dpy_ojs
            .iter()
            .find(|ft_oj| i >= ft_oj.start || i < ft_oj.end)
            .unwrap()
            .face_idx;
        let tmp = string_pixel_width(
            xft,
            dpy,
            fonts[ft_idx],
            &string
                .chars()
                .skip(i)
                .next()
                .unwrap()
                .encode_utf8(&mut [0u8; 4]),
        );
        res.push(if tmp == 0 { 10 } else { tmp });
    }
    res
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

#[derive(Debug)]
struct BackDisplay {
    idx: usize,
    start: usize,
    end: usize,
}

impl BackDisplay {
    fn generate_list(
        bkgrnd_objs: &Vec<DisplayType>,
        char_pixel_widths: &Vec<u32>,
    ) -> Vec<BackDisplay> {
        bkgrnd_objs.iter().fold(Vec::new(), |mut acc, back_oj| {
            let idx = back_oj.idx;
            let start: u32 = char_pixel_widths.iter().take(back_oj.start).sum();
            let end: u32 = char_pixel_widths
                .iter()
                .skip(back_oj.start)
                .take(back_oj.end - back_oj.start)
                .sum();
            let start = start as usize;
            let end = end as usize;
            acc.push(BackDisplay { idx, start, end });
            acc
        })
    }
}

#[derive(Debug)]
struct HighDisplay {
    idx: usize,
    start: usize,
    end: usize,
}

impl HighDisplay {
    fn generate_list(
        highlight_objs: &Vec<DisplayType>,
        char_pixel_widths: &Vec<u32>,
    ) -> Vec<HighDisplay> {
        highlight_objs.iter().fold(Vec::new(), |mut acc, high_oj| {
            let idx = high_oj.idx;
            let start: u32 = char_pixel_widths.iter().take(high_oj.start).sum();
            let end: u32 = char_pixel_widths
                .iter()
                .skip(high_oj.start)
                .take(high_oj.end - high_oj.start)
                .sum();
            let start = start as usize;
            let end = end as usize;
            acc.push(HighDisplay { idx, start, end });
            acc
        })
    }
}

#[derive(Debug)]
struct FontDisplay {
    face_idx: usize,
    col_idx: usize,
    start: usize,
    end: usize,
}

impl FontDisplay {
    fn generate_list(
        col_objs: &Vec<DisplayType>,
        face_objs: &Vec<DisplayType>,
    ) -> Vec<FontDisplay> {
        let mut strt_idx = 0;
        col_objs.iter().fold(Vec::new(), |mut acc, cl_oj| {
            let col_idx = cl_oj.idx;
            let mut tmp: Vec<FontDisplay> = Vec::new();
            for i in strt_idx..face_objs.len() {
                let fc_oj = &face_objs[i];
                let face_idx = fc_oj.idx;
                let start = if fc_oj.start <= cl_oj.start {
                    cl_oj.start
                } else {
                    fc_oj.start
                };
                let end = if fc_oj.end >= cl_oj.end {
                    cl_oj.end
                } else {
                    fc_oj.end
                };

                if start != end {
                    tmp.push(FontDisplay {
                        face_idx,
                        col_idx,
                        start,
                        end,
                    });
                }

                if fc_oj.end >= cl_oj.end {
                    strt_idx = i;
                    break;
                }
            }
            acc.append(&mut tmp);
            acc
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct DisplayType {
    idx: usize,
    start: usize,
    end: usize,
}

impl DisplayType {
    fn from(inp: (usize, usize, usize)) -> DisplayType {
        let (idx, start, end) = inp;
        DisplayType { idx, start, end }
    }

    fn merge_font_faces(mut default: Vec<usize>, explicit: Vec<DisplayType>) -> Vec<DisplayType> {
        explicit.iter().for_each(|dt| {
            if dt.idx != usize::MAX {
                for i in dt.start..dt.end {
                    default[i] = dt.idx;
                }
            }
        });
        let mut tmp: (usize, usize, usize) = (0, 0, 0);
        let mut count = 0;
        let mut result = default.iter().fold(Vec::new(), |mut acc, &i| {
            if i != tmp.0 {
                if count > 0 {
                    tmp.2 = count;
                    acc.push(DisplayType::from(tmp));
                }
                tmp = (i, count, 0);
            }
            count += 1;
            acc
        });
        if count - tmp.1 > 0 {
            tmp.2 = count;
            result.push(DisplayType::from(tmp));
        }
        result
    }
}

#[derive(Debug)]
struct ValidString {
    text: String,
    text_display: Vec<FontDisplay>,
    backgrounds: Vec<BackDisplay>,
    highlights: Vec<HighDisplay>,
}

impl ValidString {
    fn empty() -> ValidString {
        ValidString {
            text: String::new(),
            text_display: Vec::new(),
            backgrounds: Vec::new(),
            highlights: Vec::new(),
        }
    }

    unsafe fn parse_input_string(
        xft: &xft::Xft,
        dpy: *mut xlib::Display,
        fonts: &Vec<*mut xft::XftFont>,
        colours: &ColourPalette,
        input: String,
    ) -> ValidString {
        // Loop vars.
        let mut in_format_block = false;
        let mut next_is_index = false;
        let mut closing_block = false;
        let mut index_type = IndexType::FontColour;

        // Result vars.
        let mut text = String::new();
        let mut background_vec: Vec<DisplayType> = Vec::new();
        let mut highlight_vec: Vec<DisplayType> = Vec::new();
        let mut font_colour_vec: Vec<DisplayType> = Vec::new();
        let mut font_face_vec: Vec<DisplayType> = Vec::new();
        let mut default_font_faces: Vec<usize> = Vec::new();

        // Temp vars.
        let mut count: usize = 0;
        let mut bckgrnd_tmp: (usize, usize, usize) = (usize::MAX, 0, 0);
        let mut highlht_tmp: (usize, usize, usize) = (usize::MAX, 0, 0);
        let mut fcol_tmp: (usize, usize, usize) = (0, 0, 0);
        let mut fface_tmp: (usize, usize, usize) = (usize::MAX, 0, 0);

        input.chars().for_each(|ch| {
            if in_format_block {
                if closing_block {
                    match ch {
                        'B' => {
                            bckgrnd_tmp.2 = count;
                            background_vec.push(DisplayType::from(bckgrnd_tmp));
                            bckgrnd_tmp = (usize::MAX, count, 0);
                        }
                        'H' => {
                            highlight_vec.push(DisplayType::from(highlht_tmp));
                            highlht_tmp = (usize::MAX, count, 0);
                        }
                        'F' => {
                            fcol_tmp.2 = count;
                            font_colour_vec.push(DisplayType::from(fcol_tmp));
                            fcol_tmp = (0, count, 0);
                        }
                        'f' => {
                            fface_tmp.2 = count;
                            font_face_vec.push(DisplayType::from(fface_tmp));
                            fface_tmp = (usize::MAX, count, 0);
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
                                        bckgrnd_tmp.2 = count;
                                        background_vec.push(DisplayType::from(bckgrnd_tmp));
                                        bckgrnd_tmp = (d as usize, count, 0);
                                    }
                                }
                                IndexType::HighlightColour => {
                                    if d > (colours.highlight.len() - 1) as u32 {
                                        println!("Invalid highlight colour index -- TOO LARGE.");
                                    } else {
                                        highlht_tmp.2 = count;
                                        highlight_vec.push(DisplayType::from(highlht_tmp));
                                        highlht_tmp = (d as usize, count, 0);
                                    }
                                }
                                IndexType::FontColour => {
                                    if d > (colours.font.len() - 1) as u32 {
                                        println!("Invalid font colour index -- TOO LARGE.");
                                    } else {
                                        fcol_tmp.2 = count;
                                        font_colour_vec.push(DisplayType::from(fcol_tmp));
                                        fcol_tmp = (d as usize, count, 0);
                                    }
                                }
                                IndexType::FontFace => {
                                    if d > (fonts.len() - 1) as u32 {
                                        println!("Invalid font face index -- TOO LARGE.");
                                    } else {
                                        fface_tmp.2 = count;
                                        font_face_vec.push(DisplayType::from(fface_tmp));
                                        fface_tmp = (d as usize, count, 0);
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
                        count += 1;
                        default_font_faces.push(default_font_idx(&xft, dpy, fonts, ch));
                        text.push(ch);
                    }
                }
            }
        });

        bckgrnd_tmp.2 = count;
        highlht_tmp.2 = count;
        fcol_tmp.2 = count;
        fface_tmp.2 = count;

        // If our temp vars are 0 then we don't need this last push.
        if bckgrnd_tmp.2 - bckgrnd_tmp.1 != 0 {
            background_vec.push(DisplayType::from(bckgrnd_tmp))
        }
        if highlht_tmp.2 - highlht_tmp.1 != 0 {
            highlight_vec.push(DisplayType::from(highlht_tmp));
        }
        if fcol_tmp.2 - fcol_tmp.1 != 0 {
            font_colour_vec.push(DisplayType::from(fcol_tmp));
        }
        if fface_tmp.2 - fface_tmp.1 != 0 {
            font_face_vec.push(DisplayType::from(fface_tmp));
        }

        let background_vec: Vec<DisplayType> = background_vec
            .iter()
            .filter_map(|&bk_oj| {
                if bk_oj.idx != usize::MAX {
                    Some(bk_oj)
                } else {
                    None
                }
            })
            .collect();

        let highlight_vec: Vec<DisplayType> = highlight_vec
            .iter()
            .filter_map(|&hi_oj| {
                if hi_oj.idx != usize::MAX {
                    Some(hi_oj)
                } else {
                    None
                }
            })
            .collect();

        let merged_faces = DisplayType::merge_font_faces(default_font_faces, font_face_vec);
        let text_display = FontDisplay::generate_list(&font_colour_vec, &merged_faces);
        let char_pixel_widths = string_pixel_dist_vec(&xft, dpy, fonts, &text_display, &text);
        let highlights = HighDisplay::generate_list(&highlight_vec, &char_pixel_widths);
        let backgrounds = BackDisplay::generate_list(&background_vec, &char_pixel_widths);

        println!("Font Objects?: \n{:#?}", text_display);
        println!("Background Objects?: \n{:#?}", backgrounds);
        println!("Highlight Objects?: \n{:#?}", highlights);

        ValidString {
            text,
            text_display,
            backgrounds,
            highlights,
        }
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
            get_font(&xft, dpy, screen, "Hack:size=12:antialias=true"),
            get_font(&xft, dpy, screen, "Unifont Upper:size=16:antialias=true"),
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
                    let input =
                        ValidString::parse_input_string(&xft, dpy, &fonts, &colour_palette, s);
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
