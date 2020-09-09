// Super simple status bar written in rust with direct Xlib.
// By: Curtis Jones
// Started on Ausust 06, 2020

use unibar::*;

fn main() {
    unsafe {
        let conf = Config::from_file("unibar.conf");
        let mut bar = Bar::new();
        bar.load_config(conf);
        bar.init();
        bar.event_loop();
        bar.close();
    }
}
