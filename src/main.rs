// Super simple status bar written in rust with direct Xlib.
// By: Curtis Jones
// Started on Ausust 06, 2020

use dirs::config_dir;
use unibar::*;

fn get_config_opt() -> String {
    let mut args = std::env::args().skip_while(|a| !a.starts_with("-c"));
    let first = match args.next() {
        Some(a) => a,
        None => return String::new(),
    };
    if let Some(f) = first.split('=').nth(1) {
        String::from(f)
    } else {
        match args.next() {
            Some(file) => file,
            None => String::new(),
        }
    }
}

fn parse_args(conf: &mut Config) {
    let mut is_value = false;
    let mut skip = false;
    let mut key = String::new();
    for arg in std::env::args().skip(1) {
        if is_value {
            conf.change_option(&key, &arg);
            is_value = false;
        } else if skip {
            skip = false;
            continue;
        } else {
            match &arg[..] {
                "-c" => {
                    skip = true;
                    continue;
                }
                "-f" => {
                    key = String::from("font");
                    is_value = true;
                }
                "-F" => {
                    key = String::from("ft_colour");
                    is_value = true;
                }
                "-B" => {
                    key = String::from("background_colour");
                    is_value = true;
                }
                "-H" => {
                    key = String::from("highlight_colour");
                    is_value = true;
                }
                _ => println!("invalid arg! -> {}", arg),
            }
        }
    }
}

fn main() {
    let conf_opt = get_config_opt();
    let default_conf = match config_dir() {
        Some(mut d) => {
            d.push("unibar/unibar.conf");
            println!("{}", d.to_str().unwrap());
            String::from(d.to_str().unwrap())
        }
        None => String::new(),
    };
    let mut conf = Config::from_file(if conf_opt.is_empty() {
        &default_conf
    } else {
        &conf_opt
    });
    parse_args(&mut conf);
    unsafe {
        let mut bar = Bar::new();
        bar.load_config(conf);
        bar.init();
        bar.event_loop();
        bar.close();
    }
}
