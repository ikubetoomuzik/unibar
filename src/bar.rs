// Trying to actually organize this a bit.
// By: Curtis Jones <git@curtisjones.ca>
// Started on August 23, 2020

use super::{
    config::Config,
    input::{ColourPalette, Input},
    optional::kill_me::KillMeModule,
};
use anyhow::Result;
use signal_hook::iterator::Signals;
use std::{
    collections::HashMap, ffi::CString, io, mem::MaybeUninit, process, ptr, sync::mpsc, thread,
    time,
};
use thiserror::Error;
use x11_dl::{xft, xinerama, xlib, xrandr};

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
            send.send(tmp.trim().to_owned())
                .expect("If this fails then the bar is already gone.");
        }
    }
}

#[derive(Debug, Error)]
enum Error {
    #[error("Failed to open a connection to the default XDisplay")]
    DisplayOpenError,
}

/// Main struct of the whole program.
pub struct Bar {
    name: String,
    xlib: xlib::Xlib,
    xft: xft::Xft,
    display: *mut xlib::Display,
    screen: i32,
    top: bool,
    monitor: String,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    back_colour: u64,
    cmap: xlib::Colormap,
    visual: *mut xlib::Visual,
    root: u64,
    window_id: u64,
    event: MaybeUninit<xlib::XEvent>,
    draw: *mut xft::XftDraw,
    font_map: HashMap<char, usize>,
    fonts: Vec<*mut xft::XftFont>,
    font_y: i32,
    palette: ColourPalette,
    underline_height: i32,
    left_string: Input,
    center_string: Input,
    right_string: Input,
    kill_me: Option<KillMeModule>,
}

impl Bar {
    /// Basic function to generate and empty bar object. Main focus is starting the essential
    /// library connections.
    ///
    /// Output:
    /// An unitialized Bar object, still need to load a config before it is useful.
    pub fn new() -> Result<Self> {
        // big ugly unsafe block here.
        unsafe {
            let xlib = xlib::Xlib::open()?;
            let xft = xft::Xft::open()?;
            let display = (xlib.XOpenDisplay)(ptr::null());
            if display.is_null() {
                return Err(Error::DisplayOpenError.into());
            }
            let screen = (xlib.XDefaultScreen)(display);
            let root = (xlib.XRootWindow)(display, screen);
            let visual = (xlib.XDefaultVisual)(display, screen);
            let cmap = (xlib.XDefaultColormap)(display, screen);

            Ok(Self {
                name: String::new(),
                xlib,
                xft,
                display,
                screen,
                top: true,
                monitor: String::new(),
                x: 0,
                y: 0,
                width: 0,
                height: 0,
                back_colour: 0,
                cmap,
                visual,
                root,
                window_id: 0,
                event: MaybeUninit::uninit(),
                draw: ptr::null_mut(),
                font_map: HashMap::new(),
                fonts: Vec::new(),
                font_y: 0,
                palette: ColourPalette::empty(),
                underline_height: 0,
                left_string: Input::empty(),
                center_string: Input::empty(),
                right_string: Input::empty(),
                kill_me: None,
            })
        }
    }

    /// Main initial load for the program. Parsing the config object defined previously and
    /// creating a usable bar.
    ///
    /// Arguments:
    /// * Config object made using the gen_config function.
    ///
    /// Output:
    /// None, method alters the bar object itself, loading real values into the placeholders
    /// generated in Bar::new().
    pub fn load_config(&mut self, conf: Config) -> Result<()> {
        // As per tradition, name first!
        self.name = conf.name;
        // We are setting x to 0 for now but we check for other monitors later.
        self.x = 0;
        // Bar height is configurable.
        self.height = conf.height;
        // Duh..
        self.top = conf.top;
        // load kill_me module settings
        self.kill_me = conf.kill_me_cmd.map(KillMeModule::new);
        // Now we do all the yucky C library stuff in a big unsafe block.
        unsafe {
            // Width for now is the full XDisplay width.
            self.width = (self.xlib.XDisplayWidth)(self.display, self.screen);
            // If its the top then 0, otherwise subtract bar height from monitor height.
            self.y = if conf.top {
                0
            } else {
                (self.xlib.XDisplayHeight)(self.display, self.screen) - conf.height
            };

            // Setting the monitor using Xinerama or Xrandr depending on value provided.
            // Integer means Xinerama and any non-integer will be used to lookup in Xrandr.
            self.monitor = conf.monitor;
            if self.monitor.is_empty() {
                eprintln!("No monitor provided, using full XDisplay!");
            } else if let Ok(mon) = self.monitor.parse::<usize>() {
                // If the monitor provided is a valid usize number. Then we are using Xinerama to
                // detect monitors.

                // xinerama stuff
                // grab the monitor number set in the conf.
                match xinerama::Xlib::open() {
                    Ok(xin) => {
                        // Grab another copy of the XDisplay. Because the Xinerama methods change the pointer
                        // and causes the close to seg fault.
                        let dpy = (self.xlib.XOpenDisplay)(ptr::null());
                        // Even if we have connected to the library that doesn't necessarily mean that Xinerama
                        // is active. So we make another check here.
                        // Old school c bool where 0 is false and anything else is true.
                        if let 0 = (xin.XineramaIsActive)(dpy) {
                            eprintln!(
                                "Xinerama is not currently active -- using full XDisplay width."
                            );
                        } else {
                            // Temp var because the query strings funtion needs a pointer to a i32.
                            let mut num_scr = 0;
                            // Gets a dumb mutable pointer to an array of ScreenInfo objects for each screen.
                            let scrns = (xin.XineramaQueryScreens)(dpy, &mut num_scr);
                            // Using pointer arithmetic and the num_scr variable from the previous function we
                            // fold the range into a Vec of ScreenInfo objects.
                            let scrns = (0..num_scr as usize).fold(Vec::new(), |mut acc, i| {
                                acc.push(*scrns.add(i));
                                acc
                            });
                            // If the monitor set is not available, use first screen.
                            let scrn = if mon >= num_scr as usize {
                                eprintln!(
                                    "Monitor index: {} is too large! Using first screen.",
                                    mon
                                );
                                scrns[0]
                            } else {
                                scrns[mon]
                            };
                            self.x = scrn.x_org as i32;
                            self.y = if self.top {
                                scrn.y_org as i32
                            } else {
                                (scrn.y_org + scrn.height) as i32 - self.height
                            };
                            self.width = scrn.width as i32;
                        }
                        // Close out the temp display we opened.
                        (self.xlib.XCloseDisplay)(dpy);
                    }
                    Err(e) => eprintln!(
                        "Could not connect to Xinerama lib -- using full XDisplay width.\n{}",
                        e
                    ),
                }
            } else if let Ok(xrr) = xrandr::Xrandr::open() {
                // xrandr stuff
                // again we load a seperate pointer to the display, because otherwise we get
                // segfaults. those are hard enough to understand when the language intends that as
                // an error, but rust has a real hard time explaining so we just eat this and try
                // again.
                let dpy = (self.xlib.XOpenDisplay)(ptr::null());
                let resources = (xrr.XRRGetScreenResources)(dpy, self.root);
                // doesn't matter what we set here, the GetMonitors function overrides with the
                // real val before we read.
                let mut num_mon: i32 = 0;
                // Now we query the library for a list on monitors and it helpfully (kill me now)
                // returns a pointer to the first monitor and a total count in the num_mon var.
                let mons = (xrr.XRRGetMonitors)(dpy, self.root, xlib::True, &mut num_mon);
                // translating between weird c structs and pretty rust ones.
                // we create a range iterator as large as the number of monitors and use pointer
                // arithmetic to collect those into a Rust Vec.
                // plus a quick type alias to make the code signatures easier to read.
                type MonitorInfoList = Vec<(String, i32, i32, i32, i32)>;
                let mons = (0..num_mon as usize).try_fold(
                    Vec::new(),
                    |mut acc, i| -> Result<MonitorInfoList> {
                        let m = *mons.add(i);
                        // The way xrandr organizes information probably makes sense if you wrote the
                        // library. or maybe if you can find docs because they either dont exist or
                        // suck. Basically every Xrandr Monitor has outputs. Unless you have multiple cords
                        // from pc to monitor you only have one output.
                        let mut tmp = (0..m.noutput as usize).try_fold(
                            Vec::new(),
                            |mut ac, j| -> Result<MonitorInfoList> {
                                let output = *m.outputs.add(j);
                                let info = *(xrr.XRRGetOutputInfo)(dpy, resources, output);
                                // Inside the output object we have another object called CRTC.
                                let crtc = *(xrr.XRRGetCrtcInfo)(dpy, resources, info.crtc);
                                // This library returns strings just like arrays, you get a pointer to the
                                // first char and a count. So we do the same iteration trick to collect
                                // into a string.
                                let name =
                                    (0..info.nameLen as usize).fold(Vec::new(), |mut acc, k| {
                                        acc.push(*info.name.add(k) as u8);
                                        acc
                                    });
                                // Inside the CRTC is information that an actual human or basic ass application
                                // like this may need. So we grab what we need there and push a tuple
                                // containing the info into the vec, instead of the full monitor object, to
                                // avoid all this abstraction craziness later.
                                ac.push((
                                    String::from_utf8(name)?,
                                    crtc.x,
                                    crtc.y,
                                    crtc.width as i32,
                                    crtc.height as i32,
                                ));
                                Ok(ac)
                            },
                        )?;
                        // Append the tmp vec of usable monitor info to the result and finally move
                        // onto the next Monitor.
                        acc.append(&mut tmp);
                        Ok(acc)
                    },
                )?;
                match mons.iter().find(|m| m.0 == self.monitor) {
                    Some(m) => {
                        self.x = m.1;
                        self.y = if self.top { m.2 } else { m.4 - self.height };
                        self.width = m.3;
                    }
                    None => eprintln!(
                        "Xrandr monitor -> {} <- not found, using full XDisplay!",
                        self.monitor
                    ),
                }
                (self.xlib.XCloseDisplay)(dpy);
            } else {
                eprintln!("XRandr not available, using full XDisplay!");
            }

            if let Some(width) = conf.width {
                self.width = width;
            }
            self.underline_height = conf.ul_height;
            self.fonts = conf.fonts.iter().try_fold(
                Vec::new(),
                |mut acc, fs| -> Result<Vec<*mut xft::XftFont>> {
                    acc.push(self.get_font(fs)?);
                    Ok(acc)
                },
            )?;
            self.font_y = conf.font_y;
            self.back_colour = self.get_xlib_color(&conf.back_color)?;
            type XftColorList = Vec<xft::XftColor>;
            self.palette.font =
                conf.ft_clrs
                    .iter()
                    .try_fold(Vec::new(), |mut acc, s| -> Result<XftColorList> {
                        acc.push(self.get_xft_colour(s)?);
                        Ok(acc)
                    })?;
            self.palette.background =
                conf.bg_clrs
                    .iter()
                    .try_fold(Vec::new(), |mut acc, s| -> Result<XftColorList> {
                        acc.push(self.get_xft_colour(s)?);
                        Ok(acc)
                    })?;
            self.palette.underline =
                conf.ul_clrs
                    .iter()
                    .try_fold(Vec::new(), |mut acc, s| -> Result<XftColorList> {
                        acc.push(self.get_xft_colour(s)?);
                        Ok(acc)
                    })?;
        }
        Ok(())
    }

    pub fn init(&mut self) -> Result<()> {
        unsafe {
            // Manually set the attributes here so we can get more fine grain control.
            let mut attributes: MaybeUninit<xlib::XSetWindowAttributes> = MaybeUninit::uninit();
            let atts = attributes.as_mut_ptr();
            (*atts).background_pixel = self.back_colour;
            (*atts).colormap = self.cmap;
            (*atts).override_redirect = xlib::False;
            (*atts).event_mask =
                xlib::ExposureMask | xlib::ButtonPressMask | xlib::VisibilityChangeMask;
            let mut attributes = attributes.assume_init();

            // Use the attributes we created to make a window.
            self.window_id = (self.xlib.XCreateWindow)(
                self.display,             // Display to use.
                self.root,                // Parent window.
                self.x,                   // X position (from top-left.
                self.y,                   // Y position (from top-left.
                self.width as u32,        // Length of the bar in x direction.
                self.height as u32,       // Height of the bar in y direction.
                0,                        // Border-width.
                xlib::CopyFromParent,     // Window depth.
                xlib::InputOutput as u32, // Window class.
                self.visual,              // Visual type to use.
                xlib::CWBackPixel | xlib::CWColormap | xlib::CWOverrideRedirect | xlib::CWEventMask, // Mask for which attributes are set.
                &mut attributes, // Pointer to the attributes to use.
            );
            self.draw =
                (self.xft.XftDrawCreate)(self.display, self.window_id, self.visual, self.cmap);

            self.set_atoms()?;

            // Map it up.
            (self.xlib.XMapWindow)(self.display, self.window_id);
        }
        Ok(())
    }

    pub fn event_loop(&mut self) -> Result<()> {
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
        ])?;

        loop {
            // Check signals.
            // All of the signals basically tell the program to shutdown, so we just get ahead and
            // make sure that we clean up properly.
            if signals.pending().count() > 0 {
                self.close(1);
            }

            // Check the input thread.
            if let Ok(string) = rx.try_recv() {
                // Small kill marker for when I can't click.
                if string == "QUIT NOW" {
                    break;
                }

                // messy way to check if kill me option is enabled
                if string.starts_with("PLEASE KILL:") {
                    if let Some(kill_me) = self.kill_me.as_mut() {
                        if let Some(id_str) = string.split(':').nth(1) {
                            if let Ok(id) = id_str.parse::<u32>() {
                                kill_me.push(id);
                            }
                        }
                    }
                    continue;
                }

                let split: Vec<String> = string.split("<|>").map(|s| s.to_owned()).collect();
                unsafe {
                    match split.len() {
                        // If there are no seperators then we assign the whole string to the left
                        // bar section.
                        1 => {
                            self.left_string.parse_string(
                                &self.xft,
                                self.display,
                                &self.fonts,
                                &mut self.font_map,
                                &self.palette,
                                &split[0],
                            )?;
                            self.center_string.clear();
                            self.right_string.clear();
                        }
                        // If there is only one seperator we assign the first bit to the left and
                        // the second to the right.
                        2 => {
                            self.left_string.parse_string(
                                &self.xft,
                                self.display,
                                &self.fonts,
                                &mut self.font_map,
                                &self.palette,
                                &split[0],
                            )?;
                            self.center_string.clear();
                            self.right_string.parse_string(
                                &self.xft,
                                self.display,
                                &self.fonts,
                                &mut self.font_map,
                                &self.palette,
                                &split[1],
                            )?;
                        }
                        // If there are two or more seperators then we are only gonna use the first
                        // three, assign the first to left, second to center, and third to right.
                        _ => {
                            self.left_string.parse_string(
                                &self.xft,
                                self.display,
                                &self.fonts,
                                &mut self.font_map,
                                &self.palette,
                                &split[0],
                            )?;
                            self.center_string.parse_string(
                                &self.xft,
                                self.display,
                                &self.fonts,
                                &mut self.font_map,
                                &self.palette,
                                &split[1],
                            )?;
                            self.right_string.parse_string(
                                &self.xft,
                                self.display,
                                &self.fonts,
                                &mut self.font_map,
                                &self.palette,
                                &split[2],
                            )?;
                        }
                    }
                    self.draw_display();
                }
            }

            unsafe {
                // Check events.
                if self.poll_events() {
                    #[allow(clippy::single_match)]
                    match self.event.assume_init_ref().get_type() {
                        // if the bar is show on the screen we draw content.
                        xlib::Expose => self.draw_display(),
                        // ignore all other events
                        _ => (),
                    }
                }
            }

            thread::sleep(time::Duration::from_millis(100));
        }
        Ok(())
    }

    unsafe fn clear_display(&self) {
        (self.xlib.XClearWindow)(self.display, self.window_id);
    }

    unsafe fn draw_display(&self) {
        // clear display before we redraw
        self.clear_display();
        // left string.
        self.left_string.draw(
            &self.xft,
            self.display,
            self.draw,
            &self.palette,
            &self.fonts,
            0,
            self.font_y,
            self.height as u32,
            self.underline_height as u32,
        );

        // center string.
        self.center_string.draw(
            &self.xft,
            self.display,
            self.draw,
            &self.palette,
            &self.fonts,
            (self.width - self.center_string.len(&self.xft, self.display, &self.fonts) as i32) / 2,
            self.font_y,
            self.height as u32,
            self.underline_height as u32,
        );

        // right string.
        self.right_string.draw(
            &self.xft,
            self.display,
            self.draw,
            &self.palette,
            &self.fonts,
            self.width - self.right_string.len(&self.xft, self.display, &self.fonts) as i32,
            self.font_y,
            self.height as u32,
            self.underline_height as u32,
        );
    }

    pub fn close(&mut self, code: i32) -> ! {
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
        if let Some(km) = self.kill_me.as_mut() {
            km.kill_all()
        }
        process::exit(code);
    }

    unsafe fn get_atom(&self, name: &str) -> Result<xlib::Atom> {
        let name = CString::new(name)?;
        Ok((self.xlib.XInternAtom)(
            self.display,
            name.as_ptr() as *const i8,
            xlib::False,
        ))
    }

    unsafe fn get_font(&self, name: &str) -> Result<*mut xft::XftFont> {
        let name = CString::new(name)?;
        let tmp = (self.xft.XftFontOpenName)(self.display, self.screen, name.as_ptr() as *const i8);
        if tmp.is_null() {
            panic!("Font {} not found!!", name.to_str()?)
        } else {
            Ok(tmp)
        }
    }

    unsafe fn get_xft_colour(&self, name: &str) -> Result<xft::XftColor> {
        let name = CString::new(name)?;

        let mut tmp: MaybeUninit<xft::XftColor> = MaybeUninit::uninit();
        (self.xft.XftColorAllocName)(
            self.display,
            self.visual,
            self.cmap,
            name.as_ptr() as *const i8,
            tmp.as_mut_ptr(),
        );
        let tmp = tmp.assume_init();
        Ok(tmp)
    }

    unsafe fn get_xlib_color(&self, name: &str) -> Result<u64> {
        let name = CString::new(name)?;
        let mut temp: MaybeUninit<xlib::XColor> = MaybeUninit::uninit();
        (self.xlib.XParseColor)(self.display, self.cmap, name.as_ptr(), temp.as_mut_ptr());
        (self.xlib.XAllocColor)(self.display, self.cmap, temp.as_mut_ptr());
        let temp = temp.assume_init();
        Ok(temp.pixel)
    }

    unsafe fn poll_events(&mut self) -> bool {
        (self.xlib.XCheckWindowEvent)(
            self.display,
            self.window_id,
            xlib::ButtonPressMask | xlib::ExposureMask,
            self.event.as_mut_ptr(),
        ) == 1
    }

    unsafe fn set_atoms(&mut self) -> Result<()> {
        // Set the WM_NAME.
        let name = format!("Unibar_{}", self.name);
        let title = CString::new(name)?;
        (self.xlib.XStoreName)(self.display, self.window_id, title.as_ptr() as *mut i8);
        // Set WM_CLASS
        let class: *mut xlib::XClassHint = (self.xlib.XAllocClassHint)();
        let cl_names = [CString::new("unibar")?, CString::new("Unibar")?];
        (*class).res_name = cl_names[0].as_ptr() as *mut i8;
        (*class).res_class = cl_names[1].as_ptr() as *mut i8;
        (self.xlib.XSetClassHint)(self.display, self.window_id, class);
        // Set WM_CLIENT_MACHINE
        let hn_size = libc::sysconf(libc::_SC_HOST_NAME_MAX) as libc::size_t;
        let hn_buffer: *mut i8 = vec![0i8; hn_size].as_mut_ptr();
        libc::gethostname(hn_buffer, hn_size);
        let mut hn_list = [hn_buffer];
        let mut hn_text_prop: std::mem::MaybeUninit<xlib::XTextProperty> = MaybeUninit::uninit();
        (self.xlib.XStringListToTextProperty)(hn_list.as_mut_ptr(), 1, hn_text_prop.as_mut_ptr());
        let mut hn_text_prop = hn_text_prop.assume_init();
        (self.xlib.XSetWMClientMachine)(self.display, self.window_id, &mut hn_text_prop);
        // Set _NET_WM_PID
        let pid = [process::id()].as_ptr();
        let wm_pid_atom = self.get_atom("_NET_WM_PID")?;
        (self.xlib.XChangeProperty)(
            self.display,
            self.window_id,
            wm_pid_atom,
            xlib::XA_CARDINAL,
            32,
            xlib::PropModeReplace,
            pid as *const u8,
            1,
        );

        // Set _NET_WM_DESKTOP
        let dk_num = [0xFFFFFFFFu64].as_ptr();
        let wm_dktp_atom = self.get_atom("_NET_WM_DESKTOP")?;
        (self.xlib.XChangeProperty)(
            self.display,
            self.window_id,
            wm_dktp_atom,
            xlib::XA_CARDINAL,
            32,
            xlib::PropModeReplace,
            dk_num as *const u8,
            1,
        );

        // Change _NET_WM_STATE
        let wm_state_atom = self.get_atom("_NET_WM_STATE")?;
        let state_atoms = [
            self.get_atom("_NET_WM_STATE_STICKY")?,
            self.get_atom("_NET_WM_STATE_ABOVE")?,
        ];
        (self.xlib.XChangeProperty)(
            self.display,
            self.window_id,
            wm_state_atom,
            xlib::XA_ATOM,
            32,
            xlib::PropModeAppend,
            state_atoms.as_ptr() as *const u8,
            2,
        );

        // Set the _NET_WM_STRUT[_PARTIAL]
        // TOP    = 2 -> height, 8 -> start x, 9 -> end x
        // BOTTOM = 3 -> height, 10 -> start x, 11 -> end x
        let mut strut: [i64; 12] = [0; 12];
        if self.top {
            strut[2] = self.height as i64;
            strut[8] = self.x as i64;
            strut[9] = (self.x + self.width - 1) as i64;
        } else {
            strut[3] = self.height as i64;
            strut[10] = self.x as i64;
            strut[11] = (self.x + self.width - 1) as i64;
        }
        let strut_atoms = [
            self.get_atom("_NET_WM_STRUT_PARTIAL")?,
            self.get_atom("_NET_WM_STRUT")?,
        ];
        (self.xlib.XChangeProperty)(
            self.display,
            self.window_id,
            strut_atoms[0],
            xlib::XA_CARDINAL,
            32,
            xlib::PropModeReplace,
            strut.as_ptr() as *const u8,
            12,
        );
        (self.xlib.XChangeProperty)(
            self.display,
            self.window_id,
            strut_atoms[1],
            xlib::XA_CARDINAL,
            32,
            xlib::PropModeReplace,
            strut.as_ptr() as *const u8,
            4,
        );

        // Set the _NET_WM_WINDOW_TYPE atom
        let win_type_atom = self.get_atom("_NET_WM_WINDOW_TYPE")?;
        let dock_atom = [self.get_atom("_NET_WM_WINDOW_TYPE_DOCK")?];
        (self.xlib.XChangeProperty)(
            self.display,
            self.window_id,
            win_type_atom,
            xlib::XA_ATOM,
            32,
            xlib::PropModeReplace,
            dock_atom.as_ptr() as *const u8,
            1,
        );
        Ok(())
    }
}
