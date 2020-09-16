// Super simple status bar written in rust with direct Xlib.
// By: Curtis Jones
// Started on Ausust 06, 2020

use unibar::*;

fn main() {
    // Generate configuration from a file and any command line args.
    let conf = gen_config();

    // Generate a new empty bar object.
    let mut bar = Bar::new();

    // Alter the bar based on the config.
    bar.load_config(conf);

    // Initialize the window and set any specific Atoms needed to get the bar displayed correctly.
    bar.init();

    // Here is where the real work is done:
    //  -> checking for any exit signals that may have come in from the os.
    //  -> parsing any input on stdin to generate new text to display.
    //  -> deal with any XEvents like clicks or messages.
    bar.event_loop();

    // Because we are using C libraries alot of the objects we load need to be freed manually, so
    // we do that here before exiting with the code provided as arg.
    bar.close(0);
}
