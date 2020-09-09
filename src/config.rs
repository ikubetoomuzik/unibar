#![allow(dead_code, unused_variables)]
// Struct to load config from file and cli args.
// By: Curtis Jones <mail@curtisjones.ca>
// Started on: September 07, 2020

// gonna start by implementing the loading from file bits.

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
    Left,
    Right,
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

    pub fn from_file(file: &str) -> Config {
        let mut tmp = Config::default();

        // Loop to add to default options.
        let conf_file = match read_to_string(file) {
            Ok(s) => s,
            Err(_) => return tmp,
        };
        for (i, line) in (1..).zip(conf_file.lines()) {
            // line to allow comments
            if line.starts_with('#') {
                continue;
            }
            let mut line = line.split('=');
            let opt = match line.next() {
                Some(o) => o,
                None => {
                    println!("Invalid config on line {}.", i);
                    continue;
                }
            };
            let val = match line.next() {
                Some(v) => v,
                None => {
                    println!("Invalid config on line {}.", i);
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

    pub fn change_option(&mut self, opt: &str, val: &str) {
        // Doing a lot of direct comparisons so we gotta trim.
        let opt = opt.trim();
        let val = val.trim();

        if OPTIONS.contains(&opt) {
            // Can't get around a big ass match statement in a situation like this.
            match opt {
                "position" => match &val.to_lowercase()[..] {
                    "top" => self.position = BarPos::Top,
                    "bottom" => self.position = BarPos::Bottom,
                    "left" => self.position = BarPos::Left,
                    "right" => self.position = BarPos::Right,
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
                _ => println!("Invalid option."),
            }
        }
    }
}
