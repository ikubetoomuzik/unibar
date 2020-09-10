// this file is basically just to weave together the seperate mod files.
// also the pub use statements bring objects over to our main file.
// By: Curtis Jones <mail@curtisjones.ca>
// Started on: September 07, 2020
//
use clap::clap_app;
use config::{Config, ReplaceOpt};
use dirs::config_dir;

mod bar;
mod config;
mod valid_string;

pub use bar::Bar;

pub fn gen_config() -> Config {
    let matches = clap_app!(Unibar =>
        (version: "0.1.0")
        (author: "Curtis Jones <mail@curtisjones.ca>")
        (about: "Simple Xorg display bar!")
        (@arg CONFIG: -c --config +takes_value "Sets a custom config file")
        (@arg POSITION: -p --position  +takes_value "overrides config file position option")
        (@arg SIZE: -s --size  +takes_value "overrides config file bar size option")
        (@arg FONTS: -f --fonts +multiple +takes_value "overrides config file font options")
        (@arg DEF_BACKGROUND: -b --background +takes_value "overrides config file default background")
        (@arg FT_COLOURS: -F --ftcolours +multiple +takes_value "overrides config file font colours")
        (@arg BG_COLOURS: -B --bgcolours +multiple +takes_value "overrides config file background highlight colours")
        (@arg HT_COLOURS: -H --htcolours +multiple +takes_value "overrides config file underline highlight colours")
        (@arg UNDERLINE: -u --underline  +takes_value "overrides config file underline size option")
    )
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
