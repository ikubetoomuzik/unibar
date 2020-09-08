#![allow(dead_code, unused_variables)]
// Struct to load config from file and cli args.
// By: Curtis Jones <mail@curtisjones.ca>
// Started on: September 07, 2020

// gonna start by implementing the loading from file bits.

use std::fs::read_to_string;
use std::os::raw::*;

enum BarPos {
    Top,
    Bottom,
    Left,
    Right,
}

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

pub struct Config {
    position: BarPos,
    size: c_int,          // width or height of bar depending on pos.
    hlt_size: c_int,      // width or height of bar depending on pos.
    fonts: Vec<String>,   // Vec of strings listing the fonts in FcLookup form.
    back_color: String,   // String of the hex color.
    ft_clrs: Vec<String>, // String of the hex color.
    bg_clrs: Vec<String>, // String of the hex color.
    ht_clrs: Vec<String>, // String of the hex color.
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
        let conf_file = match read_to_string(file) {
            Ok(s) => s,
            Err(_) => return tmp,
        };
        for line in conf_file.lines() {
            let mut option = line.split('=');
            let key = match option.next() {
                Some(k) => {
                    let k = k.trim();
                    if OPTIONS.contains(&k) {
                        k
                    } else {
                        continue;
                    }
                }
                None => continue,
            };
            let val = match option.next() {
                Some(v) => v.trim(),
                None => continue,
            };
            match key {
                "position" => match &val.to_lowercase()[..] {
                    "top" => tmp.position = BarPos::Top,
                    "bottom" => tmp.position = BarPos::Bottom,
                    "left" => tmp.position = BarPos::Left,
                    "right" => tmp.position = BarPos::Right,
                    _ => continue,
                },
                "size" => match val.parse::<c_int>() {
                    Ok(s) => tmp.size = s,
                    Err(_) => continue,
                },
                "highlight_size" => match val.parse::<c_int>() {
                    Ok(s) => tmp.hlt_size = s,
                    Err(_) => continue,
                },
                "font" => tmp.fonts.push(val.to_string()),
                "default_background" => tmp.back_color = val.to_string(),
                "ft_colour" => tmp.ft_clrs.push(val.to_string()),
                "background_colour" => tmp.bg_clrs.push(val.to_string()),
                "highlight_colour" => tmp.ht_clrs.push(val.to_string()),
                _ => continue,
            }
        }
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
        tmp
    }
}
