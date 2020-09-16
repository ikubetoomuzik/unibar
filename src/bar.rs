// Trying to actually organize this a bit.
// By: Curtis Jones <git@curtisjones.ca>
// Started on August 23, 2020

use super::{
    config::{BarPos, Config},
    init,
    input::{ColourPalette, Input},
};
use clap::clap_app;
use dirs::config_dir;
use signal_hook::iterator::Signals;
use std::{ffi::CString, io, mem, os::raw::*, process, ptr, sync::mpsc, thread, time};
use x11_dl::{xft, xlib};

/// The function we dump into a seperate thread to wait for any input.
/// Put in a seperate funtion to make some of the methods cleaner.
///
/// # Arguments
/// * stdin: -> lock on the standard input for the projram.
/// * send:  -> sender part of an across thread message pipe.
fn input_loop(stdin: io::Stdin, send: mpsc::Sender<String>) {
    loop {
        let mut tmp = String::new();
        stdin.read_line(&mut tmp).expect("wont fail.");
        if !tmp.is_empty() {
            send.send(tmp.trim().to_owned()).unwrap();
        }
    }
}

/// Making a full Config by parsing the CLI arguments, parsing the config file, and mashing
/// them together to create whatever.
///
/// # Output
/// Main Config to be used for the Bar.
pub fn gen_config() -> Config {
    // Create an App object for parsing CLI args. Thankfully the library makes the code pretty
    // readable and there is no runtime penalty.
    let matches = clap_app!(Unibar =>
        (version: "0.1.0")
        (author: "Curtis Jones <mail@curtisjones.ca>")
        (about: "Simple Xorg display bar!")
        (@arg CONFIG:         -c --config        +takes_value "Sets a custom config file")
        (@arg NAME:           *                  +takes_value "Sets name and is required")
        (@arg POSITION:       -p --position      +takes_value "overrides config file position option")
        (@arg DEF_BACKGROUND: -b --background    +takes_value "overrides config file default background")
        (@arg HEIGHT:         -h --height        +takes_value "overrides config file bar height option")
        (@arg UNDERLINE:      -u --underline     +takes_value "overrides config file underline height option")
        (@arg FONT_Y:         -y --fonty         +takes_value "overrides config file font y offset option")
        (@arg FONTS:          -f --fonts     ... +takes_value "overrides config file font options")
        (@arg FT_COLOURS:     -F --ftcolours ... +takes_value "overrides config file font colours")
        (@arg BG_COLOURS:     -B --bgcolours ... +takes_value "overrides config file background highlight colours")
        (@arg UL_COLOURS:     -U --ulcolours ... +takes_value "overrides config file underline highlight colours")
        )
        .help_short("H") // We are using the lowercase h to set height.
        .setting(clap::AppSettings::ColoredHelp) // Make it look pretty.
        .get_matches(); // We actually only take the matches because we don't need clap for anything else.

    // Get the name first. It's required.
    let name = matches.value_of("NAME").unwrap();

    // Decide what the default config file will be.
    let default_conf = match config_dir() {
        // We look in XDG_CONFIG_DIR or $HOME/.config for a unibar folder with unibar.conf
        // avaiable.
        Some(mut d) => {
            let config = format!("unibar/{}.conf", name);
            d.push(config);
            String::from(d.to_str().unwrap())
        }
        // If neither of those dirs are a thing, then we just set an empty string.
        None => String::new(),
    };

    // If a explicit config file was set in the CLI args then we use that instead of our
    // default.
    let conf_opt = matches.value_of("CONFIG").unwrap_or(&default_conf);

    // Whatever we chose in the previous step we now try to load that config file.
    let mut tmp = Config::from_file(conf_opt);

    // Set the name first as we got it earlier.
    tmp.change_option("NAME", name);

    // Now we alter the loaded Config object with the CLI args.
    // First we check all of the options that only take one val.
    for opt in &[
        "POSITION",
        "DEF_BACKGROUND",
        "HEIGHT",
        "UNDERLINE",
        "FONT_Y",
    ] {
        if let Some(s) = matches.value_of(opt) {
            tmp.change_option(opt, s);
        }
    }

    // Next we check all of the options that take multiple vals.
    for opt in &["FONTS", "FT_COLOURS", "BG_COLOURS", "UL_COLOURS"] {
        if let Some(strs) = matches.values_of(opt) {
            tmp.replace_opt(opt, strs.map(|s| s.to_string()).collect());
        }
    }

    // Return the final Config to be used.
    tmp
}

pub struct Bar {
    name: String,
    xlib: xlib::Xlib,
    xft: xft::Xft,
    display: *mut xlib::Display,
    screen: c_int,
    position: BarPos,
    monitor: usize,
    x: c_int,
    y: c_int,
    width: c_int,
    height: c_int,
    back_colour: c_ulong,
    cmap: xlib::Colormap,
    visual: *mut xlib::Visual,
    root: c_ulong,
    window_id: c_ulong,
    event: xlib::XEvent,
    draw: *mut xft::XftDraw,
    fonts: Vec<*mut xft::XftFont>,
    font_y: c_int,
    palette: ColourPalette,
    underline_height: c_int,
    left_string: Input,
    center_string: Input,
    right_string: Input,
}

impl Bar {
    pub fn new() -> Bar {
        unsafe {
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
            let root = (xlib.XRootWindow)(display, screen);
            let visual = (xlib.XDefaultVisual)(display, screen);
            let cmap = (xlib.XDefaultColormap)(display, screen);

            Bar {
                name: String::new(),
                xlib,
                xft,
                display,
                screen,
                position: BarPos::Top,
                monitor: 0,
                x: 0,
                y: 0,
                width: 0,
                height: 0,
                back_colour: 0,
                cmap,
                visual,
                root,
                window_id: 0,
                event: init!(),
                draw: init!(),
                fonts: Vec::new(),
                font_y: 0,
                palette: ColourPalette::empty(),
                underline_height: 0,
                left_string: Input::empty(),
                center_string: Input::empty(),
                right_string: Input::empty(),
            }
        }
    }

    pub fn load_config(&mut self, conf: Config) {
        // As per tradition, name first!
        self.name = conf.name;
        unsafe {
            match conf.position {
                BarPos::Top => {
                    self.position = BarPos::Top;
                    self.x = 0;
                    self.y = 0;
                    self.width = (self.xlib.XDisplayWidth)(self.display, self.screen);
                    self.height = conf.height;
                }
                BarPos::Bottom => {
                    self.position = BarPos::Bottom;
                    self.x = 0;
                    self.y = (self.xlib.XDisplayHeight)(self.display, self.screen) - conf.height;
                    self.width = (self.xlib.XDisplayWidth)(self.display, self.screen);
                    self.height = conf.height;
                }
            }

            // xinerama stuff
            // grab the monitor number set in the conf.
            self.monitor = conf.monitor;
            match x11_dl::xinerama::Xlib::open() {
                Ok(xin) => {
                    // Grab another copy of the XDisplay. Because the Xinerama methods change the pointer
                    // and causes the close to seg fault.
                    let dpy = (self.xlib.XOpenDisplay)(ptr::null());
                    // Even if we have connected to the library that doesn't necessarily mean that Xinerama
                    // is active. So we make another check here.
                    match (xin.XineramaIsActive)(dpy) {
                        // Old school c bool where 0 is false and anything else is true.
                        0 => eprintln!(
                            "Xinerama is not currently active -- using full XDisplay width."
                        ),
                        _ => {
                            // Temp var because the query strings funtion needs a pointer to a c_int.
                            let mut num_scr = 0;
                            // Gets a dumb mutable pointer to an array of ScreenInfo objects for each screen.
                            let scrns = (xin.XineramaQueryScreens)(dpy, &mut num_scr);
                            // Using pointer arithmetic and the num_scr variable from the previous function we
                            // fold the range into a Vec of ScreenInfo objects.
                            let scrns = (0..num_scr as usize).fold(Vec::new(), |mut acc, i| {
                                acc.push(*scrns.add(i));
                                acc
                            });
                            if self.monitor >= num_scr as usize {
                                // If the monitor set is not available, use first screen.
                                self.x = scrns[0].x_org as c_int;
                                self.y = scrns[0].y_org as c_int;
                                self.width = scrns[0].width as c_int;
                            } else {
                                // Otherwise grab the monitor from self.
                                self.x = scrns[self.monitor].x_org as c_int;
                                self.y = scrns[self.monitor].y_org as c_int;
                                self.width = scrns[self.monitor].width as c_int;
                            }
                        }
                    }
                    // Close out the temp display we opened.
                    (self.xlib.XCloseDisplay)(dpy);
                }
                Err(e) => eprintln!(
                    "Could not connect to Xinerama lib -- using full XDisplay width.\n{}",
                    e
                ),
            }

            self.underline_height = conf.ul_height;
            self.fonts = conf.fonts.iter().map(|fs| self.get_font(fs)).collect();
            self.font_y = conf.font_y;
            self.back_colour = self.get_xlib_color(&conf.back_color);
            self.palette.font = conf
                .ft_clrs
                .iter()
                .map(|s| self.get_xft_colour(s))
                .collect();
            self.palette.background = conf
                .bg_clrs
                .iter()
                .map(|s| self.get_xft_colour(s))
                .collect();
            self.palette.underline = conf
                .ul_clrs
                .iter()
                .map(|s| self.get_xft_colour(s))
                .collect();
        }
    }

    pub fn init(&mut self) {
        unsafe {
            // Manually set the attributes here so we can get more fine grain control.
            let mut attributes: xlib::XSetWindowAttributes = init!();
            attributes.background_pixel = self.back_colour;
            attributes.colormap = self.cmap;
            attributes.override_redirect = xlib::False;
            attributes.event_mask = xlib::ExposureMask | xlib::ButtonPressMask;

            // Use the attributes we created to make a window.
            self.window_id = (self.xlib.XCreateWindow)(
                self.display,                // Display to use.
                self.root,                   // Parent window.
                self.x,                      // X position (from top-left.
                self.y,                      // Y position (from top-left.
                self.width as c_uint,        // Length of the bar in x direction.
                self.height as c_uint,       // Height of the bar in y direction.
                0,                           // Border-width.
                xlib::CopyFromParent,        // Window depth.
                xlib::InputOutput as c_uint, // Window class.
                self.visual,                 // Visual type to use.
                xlib::CWBackPixel | xlib::CWColormap | xlib::CWOverrideRedirect | xlib::CWEventMask, // Mask for which attributes are set.
                &mut attributes, // Pointer to the attributes to use.
            );
            self.draw =
                (self.xft.XftDrawCreate)(self.display, self.window_id, self.visual, self.cmap);

            self.set_atoms();

            // Map it up.
            (self.xlib.XMapWindow)(self.display, self.window_id);
        }
    }

    pub fn event_loop(&mut self) {
        // Input thread. Has to be seperate to not block xlib events.
        let (tx, rx) = mpsc::channel();
        // If we don't call the stdin function in this thread and pass it then sometimes we lose
        // the connection and the threads both panic.
        thread::spawn(move || input_loop(io::stdin(), tx));

        // Signals that are incoming.
        let signals = Signals::new(&[
            signal_hook::SIGTERM,
            signal_hook::SIGINT,
            signal_hook::SIGQUIT,
            signal_hook::SIGHUP,
        ])
        .unwrap();

        loop {
            // Check signals.
            if signals.pending().count() > 0 {
                self.close(1);
            }

            // Check the input thread.
            if let Ok(s) = rx.try_recv() {
                // Small kill marker for when I can't click.
                if s == "QUIT NOW" {
                    break;
                }
                unsafe {
                    match s.matches("<|>").count() {
                        0 => {
                            self.left_string = Input::parse_string(
                                &self.xft,
                                self.display,
                                &self.fonts,
                                &self.palette,
                                &s,
                            );
                            self.center_string = Input::empty();
                            self.right_string = Input::empty();
                        }
                        1 => {
                            let mut s = s.split("<|>");
                            self.left_string = Input::parse_string(
                                &self.xft,
                                self.display,
                                &self.fonts,
                                &self.palette,
                                s.next().unwrap_or(""),
                            );
                            self.center_string = Input::empty();
                            self.right_string = Input::parse_string(
                                &self.xft,
                                self.display,
                                &self.fonts,
                                &self.palette,
                                s.next().unwrap_or(""),
                            );
                        }
                        _ => {
                            let mut s = s.split("<|>");
                            self.left_string = Input::parse_string(
                                &self.xft,
                                self.display,
                                &self.fonts,
                                &self.palette,
                                s.next().unwrap_or(""),
                            );
                            self.center_string = Input::parse_string(
                                &self.xft,
                                self.display,
                                &self.fonts,
                                &self.palette,
                                s.next().unwrap_or(""),
                            );
                            self.right_string = Input::parse_string(
                                &self.xft,
                                self.display,
                                &self.fonts,
                                &self.palette,
                                s.next().unwrap_or(""),
                            );
                        }
                    }
                    (self.xlib.XClearWindow)(self.display, self.window_id);
                    self.draw_display();
                }
            }

            unsafe {
                // Check events.
                if self.poll_events() {}
            }

            thread::sleep(time::Duration::from_millis(100));
        }
    }

    unsafe fn draw_display(&self) {
        // left string.
        self.left_string.draw(
            &self.xft,
            self.display,
            self.draw,
            &self.palette,
            &self.fonts,
            self.x,
            self.font_y,
            self.height as c_uint,
            self.underline_height as c_uint,
        );

        // center string.
        self.center_string.draw(
            &self.xft,
            self.display,
            self.draw,
            &self.palette,
            &self.fonts,
            (self.x + self.width) / 2
                - (self.right_string.len(&self.xft, self.display, &self.fonts) as c_int / 2),
            self.font_y,
            self.height as c_uint,
            self.underline_height as c_uint,
        );

        // right string.
        self.right_string.draw(
            &self.xft,
            self.display,
            self.draw,
            &self.palette,
            &self.fonts,
            self.x + self.width
                - self.right_string.len(&self.xft, self.display, &self.fonts) as c_int,
            self.font_y,
            self.height as c_uint,
            self.underline_height as c_uint,
        );
    }

    pub fn close(&mut self, code: i32) {
        println!("\nShutting down...");
        unsafe {
            self.palette
                .destroy(&self.xft, self.display, self.cmap, self.visual);
            (self.xft.XftDrawDestroy)(self.draw);
            self.fonts
                .iter()
                .for_each(|&f| (self.xft.XftFontClose)(self.display, f));
            (self.xlib.XFreeColormap)(self.display, self.cmap);
            (self.xlib.XDestroyWindow)(self.display, self.window_id);
            (self.xlib.XCloseDisplay)(self.display);
        }
        process::exit(code);
    }

    unsafe fn get_atom(&self, name: &str) -> xlib::Atom {
        let name = CString::new(name).unwrap();
        (self.xlib.XInternAtom)(self.display, name.as_ptr() as *const c_char, xlib::False)
    }

    unsafe fn get_font(&self, name: &str) -> *mut xft::XftFont {
        let name = CString::new(name).unwrap();
        let tmp =
            (self.xft.XftFontOpenName)(self.display, self.screen, name.as_ptr() as *const c_char);
        if tmp.is_null() {
            panic!("Font {} not found!!", name.to_str().unwrap())
        } else {
            tmp
        }
    }

    unsafe fn get_xft_colour(&self, name: &str) -> xft::XftColor {
        let name = CString::new(name).unwrap();
        let mut tmp: xft::XftColor = init!();
        (self.xft.XftColorAllocName)(
            self.display,
            self.visual,
            self.cmap,
            name.as_ptr() as *const c_char,
            &mut tmp,
        );
        tmp
    }

    unsafe fn get_xlib_color(&self, name: &str) -> c_ulong {
        let name = CString::new(name).unwrap();
        let mut temp: xlib::XColor = init!();
        (self.xlib.XParseColor)(self.display, self.cmap, name.as_ptr(), &mut temp);
        (self.xlib.XAllocColor)(self.display, self.cmap, &mut temp);
        temp.pixel
    }

    unsafe fn poll_events(&mut self) -> bool {
        (self.xlib.XCheckWindowEvent)(
            self.display,
            self.window_id,
            xlib::ButtonPressMask | xlib::ExposureMask,
            &mut self.event,
        ) == 1
    }

    unsafe fn set_atoms(&mut self) {
        // Set the WM_NAME.
        let name = format!("Unibar_{}", self.name);
        let title = CString::new(name).unwrap();
        (self.xlib.XStoreName)(self.display, self.window_id, title.as_ptr() as *mut c_char);
        // Set WM_CLASS
        let class: *mut xlib::XClassHint = (self.xlib.XAllocClassHint)();
        let cl_names = [
            CString::new("unibar").unwrap(),
            CString::new("Unibar").unwrap(),
        ];
        (*class).res_name = cl_names[0].as_ptr() as *mut c_char;
        (*class).res_class = cl_names[1].as_ptr() as *mut c_char;
        (self.xlib.XSetClassHint)(self.display, self.window_id, class);
        // Set WM_CLIENT_MACHINE
        let hn_size = libc::sysconf(libc::_SC_HOST_NAME_MAX) as libc::size_t;
        let hn_buffer: *mut c_char = vec![0 as c_char; hn_size].as_mut_ptr();
        libc::gethostname(hn_buffer, hn_size);
        let mut hn_list = [hn_buffer];
        let mut hn_text_prop: xlib::XTextProperty = init!();
        (self.xlib.XStringListToTextProperty)(hn_list.as_mut_ptr(), 1, &mut hn_text_prop);
        (self.xlib.XSetWMClientMachine)(self.display, self.window_id, &mut hn_text_prop);
        // Set _NET_WM_PID
        let pid = [process::id()].as_ptr();
        let wm_pid_atom = self.get_atom("_NET_WM_PID");
        (self.xlib.XChangeProperty)(
            self.display,
            self.window_id,
            wm_pid_atom,
            xlib::XA_CARDINAL,
            32,
            xlib::PropModeReplace,
            pid as *const c_uchar,
            1,
        );

        // Set _NET_WM_DESKTOP
        let dk_num = [0xFFFFFFFF as c_ulong].as_ptr();
        let wm_dktp_atom = self.get_atom("_NET_WM_DESKTOP");
        (self.xlib.XChangeProperty)(
            self.display,
            self.window_id,
            wm_dktp_atom,
            xlib::XA_CARDINAL,
            32,
            xlib::PropModeReplace,
            dk_num as *const c_uchar,
            1,
        );

        // Change _NET_WM_STATE
        let wm_state_atom = self.get_atom("_NET_WM_STATE");
        let state_atoms = [
            self.get_atom("_NET_WM_STATE_STICKY"),
            self.get_atom("_NET_WM_STATE_ABOVE"),
        ];
        (self.xlib.XChangeProperty)(
            self.display,
            self.window_id,
            wm_state_atom,
            xlib::XA_ATOM,
            32,
            xlib::PropModeAppend,
            state_atoms.as_ptr() as *const c_uchar,
            2,
        );

        // Set the _NET_WM_STRUT[_PARTIAL]
        // TOP    = 2 -> height, 8 -> start x, 9 -> end x
        // BOTTOM = 3 -> height, 10 -> start x, 11 -> end x
        let mut strut: [c_long; 12] = [0; 12];
        match self.position {
            BarPos::Top => {
                strut[2] = self.height as c_long;
                strut[8] = self.x as c_long;
                strut[9] = (self.x + self.width - 1) as c_long;
            }
            BarPos::Bottom => {
                strut[3] = self.height as c_long;
                strut[10] = self.x as c_long;
                strut[11] = (self.x + self.width - 1) as c_long;
            }
        }
        let strut_atoms = [
            self.get_atom("_NET_WM_STRUT_PARTIAL"),
            self.get_atom("_NET_WM_STRUT"),
        ];
        (self.xlib.XChangeProperty)(
            self.display,
            self.window_id,
            strut_atoms[0],
            xlib::XA_CARDINAL,
            32,
            xlib::PropModeReplace,
            strut.as_ptr() as *const c_uchar,
            12,
        );
        (self.xlib.XChangeProperty)(
            self.display,
            self.window_id,
            strut_atoms[1],
            xlib::XA_CARDINAL,
            32,
            xlib::PropModeReplace,
            strut.as_ptr() as *const c_uchar,
            4,
        );

        // Set the _NET_WM_WINDOW_TYPE atom
        let win_type_atom = self.get_atom("_NET_WM_WINDOW_TYPE");
        let dock_atom = [self.get_atom("_NET_WM_WINDOW_TYPE_DOCK")];
        (self.xlib.XChangeProperty)(
            self.display,
            self.window_id,
            win_type_atom,
            xlib::XA_ATOM,
            32,
            xlib::PropModeReplace,
            dock_atom.as_ptr() as *const c_uchar,
            1,
        );
    }
}
