// Struct to load config from file and cli args.
// By: Curtis Jones <mail@curtisjones.ca>
// Started on: September 07, 2020

// gonna start by implementing the loading from file bits.

use anyhow::Result;
use clap::clap_app;
use dirs::config_dir;
use std::fs::read_to_string;
use std::os::raw::*;

#[derive(Debug, thiserror::Error)]
#[error("Could not generate config.")]
pub struct Error;

#[derive(Debug)]
pub struct Config {
    pub name: String,         // name of the bar
    pub top: bool,            // top or bottom
    pub monitor: String,      // xinerama montior list index for monitor
    pub height: c_int,        // width or height of bar depending on pos.
    pub ul_height: c_int,     // width or height of bar depending on pos.
    pub fonts: Vec<String>,   // Vec of strings listing the fonts in FcLookup form.
    pub font_y: c_int,        // pixel offset from the top of bar to bottom font.
    pub back_color: String,   // String of the hex color.
    pub ft_clrs: Vec<String>, // String of the hex color.
    pub bg_clrs: Vec<String>, // String of the hex color.
    pub ul_clrs: Vec<String>, // String of the hex color.
}

impl Config {
    pub fn default() -> Config {
        Config {
            name: String::new(),
            top: true,
            monitor: String::new(),
            height: 32,
            ul_height: 4,
            fonts: vec![String::from("mono:size=12")],
            font_y: 20,
            back_color: String::from("#000000"),
            ft_clrs: vec![String::from("#FFFFFF")],
            bg_clrs: vec![String::from("#0000FF")],
            ul_clrs: vec![String::from("#FF0000")],
        }
    }

    /// Making a full Config by parsing the CLI arguments, parsing the config file, and mashing
    /// them together to create whatever.
    ///
    /// # Output
    /// Main Config to be used for the Bar.
    pub fn from_args() -> Result<Config> {
        // Create an App object for parsing CLI args. Thankfully the library makes the code pretty
        // readable and there is no runtime penalty.
        let matches = clap_app!(Unibar =>
        (version: env!("CARGO_PKG_VERSION"))
        (author: "Curtis Jones <mail@curtisjones.ca>")
        (about: "Simple Xorg display bar!")
        (@arg NO_CONFIG:      -C --noconfig                   "Tells Unibar to skip loading a config file.")
        (@arg CONFIG:         -c --config        +takes_value "Sets a custom config file")
        (@arg NAME:           *                  +takes_value "Sets name and is required")
        (@arg POSITION:       -p --position      +takes_value "overrides config file position option")
        (@arg MONITOR:        -m --monitor       +takes_value "sets the monitor number to use. starts at 1")
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
        // IF we are loading a config file then we use the value generated from bar name, if not we use
        // the default Config.
        let mut tmp = if matches.is_present("NO_CONFIG") {
            Config::default()
        } else {
            Config::from_file(conf_opt)
        };
        // Set the name first as we got it earlier.
        tmp.change_option("NAME", name);
        // Now we alter the loaded Config object with the CLI args.
        // First we check all of the options that only take one val.
        for opt in &[
            "MONITOR",
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
        Ok(tmp)
    }

    pub fn from_file(file: &str) -> Config {
        let mut tmp = Config::default();

        // Read the config file to a string.
        let conf_file = match read_to_string(file) {
            Ok(s) => s,
            Err(e) => {
                // If we can't read it, let the user know but continue on with the default.
                eprintln!("Could not read config file!\nFile: {}\nError: {}", file, e);
                return tmp;
            }
        };

        for (i, line) in (1..).zip(conf_file.lines()) {
            // line to allow comments
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let mut line = line.splitn(2, '=');
            let opt = match line.next() {
                Some(o) => o,
                None => {
                    eprintln!("Invalid config on line {}.", i);
                    continue;
                }
            };
            let val = match line.next() {
                Some(v) => v,
                None => {
                    eprintln!("Invalid config on line {}.", i);
                    continue;
                }
            };
            tmp.change_option(opt, val);
        }

        // Clear out the defaults if anything else was set.
        if tmp.fonts.len() > 1 {
            tmp.fonts.remove(0);
        }
        if tmp.ft_clrs.len() > 1 {
            tmp.ft_clrs.remove(0);
        }
        if tmp.bg_clrs.len() > 1 {
            tmp.bg_clrs.remove(0);
        }
        if tmp.ul_clrs.len() > 1 {
            tmp.ul_clrs.remove(0);
        }

        // Return our temp variable.
        tmp
    }

    pub fn change_option(&mut self, opt: &str, val: &str) {
        // Doing a lot of direct comparisons so we gotta trim and set the values to lowercase.
        // Also grabbing just string slices because it makes the rest of the code look pretty.
        let opt = &opt.trim().to_lowercase()[..];
        let val = val.trim().to_string();

        // Can't get around a big ass match statement in a situation like this.
        // For args that take specific vals we check to see if the val given fits within the
        // constraints but otherwise we just push it into the Config.
        match opt {
            // skip name...
            "name" => self.name = val,
            "position" => match &val.to_lowercase()[..] {
                "top" => self.top = true,
                "bottom" => self.top = false,
                _ => eprintln!("Invaild position option!"),
            },
            "monitor" => self.monitor = val,
            "height" => {
                if let Ok(s) = val.parse::<c_int>() {
                    self.height = s;
                } else {
                    eprintln!("Invaild size option! Needs to be a digit representable by a 32-bit integer.");
                }
            }
            "underline_height" => {
                if let Ok(s) = val.parse::<c_int>() {
                    self.ul_height = s;
                } else {
                    eprintln!("Invaild highlight_size option! Needs to be a digit representable by a 32-bit integer.");
                }
            }
            "font" => self.fonts.push(val),
            "font_y" => {
                if let Ok(y) = val.parse::<c_int>() {
                    self.font_y = y;
                } else {
                    eprintln!("Invaild font_y option! Needs to be a digit representable by a 32-bit integer.");
                }
            }
            "default_background" => self.back_color = val,
            "ft_colour" => self.ft_clrs.push(val),
            "background_colour" => self.bg_clrs.push(val),
            "highlight_colour" => self.ul_clrs.push(val),
            _ => eprintln!("Invalid option -> {}", opt),
        }
    }

    pub fn replace_opt(&mut self, opt: &str, vals: Vec<String>) {
        let opt = &opt.trim().to_lowercase()[..];
        match opt {
            "fonts" => self.fonts = vals,
            "ft_colours" => self.ft_clrs = vals,
            "bg_colours" => self.bg_clrs = vals,
            "ul_colours" => self.ul_clrs = vals,
            _ => eprintln!("Invalid option."),
        }
    }
}
