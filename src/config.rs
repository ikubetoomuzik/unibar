// Struct to load config from file and cli args.
// By: Curtis Jones <mail@curtisjones.ca>
// Started on: September 07, 2020

// gonna start by implementing the loading from file bits.

use clap::clap_app;
use dirs::config_dir;
use std::fs::read_to_string;
use std::os::raw::*;

const OPTIONS: [&str; 8] = [
    "position",
    "size",
    "highlight_size",
    "font",
    "default_background",
    "ft_colour",
    "background_colour",
    "highlight_colour",
];

pub enum BarPos {
    Top,
    Bottom,
}

pub enum ReplaceOpt {
    Fonts,
    FtColours,
    BgColours,
    HtColours,
}

pub struct Config {
    pub position: BarPos,
    pub size: c_int,          // width or height of bar depending on pos.
    pub hlt_size: c_int,      // width or height of bar depending on pos.
    pub fonts: Vec<String>,   // Vec of strings listing the fonts in FcLookup form.
    pub back_color: String,   // String of the hex color.
    pub ft_clrs: Vec<String>, // String of the hex color.
    pub bg_clrs: Vec<String>, // String of the hex color.
    pub ht_clrs: Vec<String>, // String of the hex color.
}

impl Config {
    fn default() -> Config {
        Config {
            position: BarPos::Top,
            size: 32,
            hlt_size: 4,
            fonts: vec![String::from("mono:size=12")],
            back_color: String::from("#000000"),
            ft_clrs: vec![String::from("#FFFFFF")],
            bg_clrs: vec![String::from("#0000FF")],
            ht_clrs: vec![String::from("#FF0000")],
        }
    }

    pub fn gen_config() -> Config {
        let matches = clap_app!(Unibar =>
        (version: "0.1.0")
        (author: "Curtis Jones <mail@curtisjones.ca>")
        (about: "Simple Xorg display bar!")
        (@arg CONFIG: -c --config +takes_value "Sets a custom config file")
        (@arg POSITION: -p --position  +takes_value "overrides config file position option")
        (@arg SIZE: -s --size  +takes_value "overrides config file bar size option")
        (@arg FONTS: -f --fonts ... +takes_value "overrides config file font options")
        (@arg DEF_BACKGROUND: -b --background +takes_value "overrides config file default background")
        (@arg FT_COLOURS: -F --ftcolours ... +takes_value "overrides config file font colours")
        (@arg BG_COLOURS: -B --bgcolours ... +takes_value "overrides config file background highlight colours")
        (@arg HT_COLOURS: -H --htcolours ... +takes_value "overrides config file underline highlight colours")
        (@arg UNDERLINE: -u --underline  +takes_value "overrides config file underline size option")
    )
    .setting(clap::AppSettings::ColoredHelp)
    .get_matches();
        let default_conf = match config_dir() {
            Some(mut d) => {
                d.push("unibar/unibar.conf");
                String::from(d.to_str().unwrap())
            }
            None => String::new(),
        };
        let conf_opt = matches.value_of("CONFIG").unwrap_or(&default_conf);
        let mut tmp = Config::from_file(conf_opt);
        if let Some(p) = matches.value_of("POSITION") {
            tmp.change_option("position", p);
        }
        if let Some(h) = matches.value_of("SIZE") {
            tmp.change_option("size", h);
        }
        if let Some(h) = matches.value_of("DEF_BACKGROUND") {
            tmp.change_option("default_background", h);
        }
        if let Some(h) = matches.value_of("UNDERLINE") {
            tmp.change_option("highlight_size", h);
        }
        if let Some(fs) = matches.values_of("FONTS") {
            tmp.replace_opt(ReplaceOpt::Fonts, fs.map(|s| s.to_string()).collect());
        }
        if let Some(fcs) = matches.values_of("FT_COLOURS") {
            tmp.replace_opt(ReplaceOpt::FtColours, fcs.map(|s| s.to_string()).collect());
        }
        if let Some(bgs) = matches.values_of("BG_COLOURS") {
            tmp.replace_opt(ReplaceOpt::BgColours, bgs.map(|s| s.to_string()).collect());
        }
        if let Some(hts) = matches.values_of("HT_COLOURS") {
            tmp.replace_opt(ReplaceOpt::HtColours, hts.map(|s| s.to_string()).collect());
        }
        tmp
    }

    fn from_file(file: &str) -> Config {
        let mut tmp = Config::default();

        // Loop to add to default options.
        let conf_file = match read_to_string(file) {
            Ok(s) => s,
            Err(_) => return tmp,
        };
        for (i, line) in (1..).zip(conf_file.lines()) {
            // line to allow comments
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let mut line = line.split('=');
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
        if tmp.ht_clrs.len() > 1 {
            tmp.ht_clrs.remove(0);
        }

        // Return our temp variable.
        tmp
    }

    fn change_option(&mut self, opt: &str, val: &str) {
        // Doing a lot of direct comparisons so we gotta trim.
        let opt = opt.trim();
        let val = val.trim();

        if OPTIONS.contains(&opt) {
            // Can't get around a big ass match statement in a situation like this.
            match opt {
                "position" => match &val.to_lowercase()[..] {
                    "top" => self.position = BarPos::Top,
                    "bottom" => self.position = BarPos::Bottom,
                    _ => (),
                },
                "size" => {
                    if let Ok(s) = val.parse::<c_int>() {
                        self.size = s;
                    }
                }
                "highlight_size" => {
                    if let Ok(s) = val.parse::<c_int>() {
                        self.hlt_size = s;
                    }
                }
                "font" => self.fonts.push(val.to_string()),
                "default_background" => self.back_color = val.to_string(),
                "ft_colour" => self.ft_clrs.push(val.to_string()),
                "background_colour" => self.bg_clrs.push(val.to_string()),
                "highlight_colour" => self.ht_clrs.push(val.to_string()),
                _ => eprintln!("Invalid option."),
            }
        }
    }

    fn replace_opt(&mut self, opt: ReplaceOpt, vals: Vec<String>) {
        match opt {
            ReplaceOpt::Fonts => self.fonts = vals,
            ReplaceOpt::FtColours => self.ft_clrs = vals,
            ReplaceOpt::BgColours => self.bg_clrs = vals,
            ReplaceOpt::HtColours => self.ht_clrs = vals,
        }
    }
}
