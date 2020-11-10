# Unibar

Simple Xorg display bar written for speed and ease of use.

![Crates.io](https://img.shields.io/crates/v/unibar?color=%238be9fd)
![GitHub release (latest by date)](https://img.shields.io/github/v/release/ikubetoomuzik/unibar?color=%23ff79c6)

## CLI Interface

### Required Arg
* **[[NAME]]** ---> *Used to find config file, also used to create unique WMNAME.*

### Flags
* **-H, --help** ---> *Display help info*
* **-V, --version** ---> *Display version info*
* **-C, --noconfig** ---> _Do not try to load a conifg file, only use cli options._

### Options
* **-c, --config <CONFIG>** ---> *Specify custom config file to use.*
 
* **-p, --position <POSITION>** ---> *Choose bar position, options are* __TOP__ *or* __BOTTOM__*.*
* **-m, --monitor <MONITOR>** ---> *Monitor to use: can either be the Xrandr monitor name, or a number. If value is a number it is used to index the Xinerama displays. Valid index starts at 0.*

* **-h, --height <HEIGHT>** ---> *Choose bar height in pixels.*
* **-u, --underline <UNDERLINE>** ---> *Choose underline highlight height in pixels.*

* **-b, --background <DEFBACKGROUND>** ---> *Choose default bg colour in '#XXXXXX' hex format.*
* **-y, --fonty <FONTY>** ---> *Choose font offset from top of bar in pixels.*
* **-f, --fonts <FONTS>...** ---> *Comma seperated list of FcConfig font name strings. Ex. 'FontName:size=XX:antialias=true/false'*
 
* **-F, --ftcolours <FTCOLOURS>...** ---> *Comma seperated list of font colours in '#XXXXXX' hex format.*
* **-B, --bgcolours <BGCOLOURS>...** ---> *Comma seperated list of background highlight colours in '#XXXXXX' hex format.*
* **-U, --ulcolours <ULCOLOURS>...** ---> *Comma seperated list of underline highlight colours in '#XXXXXX' hex format.*

## Configuration
The bar looks for the config file at:
  * **$XDGCONFIGDIR**/unibar/**[[NAME]]**.conf
**OR**
  * **~/.config**/unibar/**[[NAME]]**.conf

## Defaults
Any configuration options set with command line arguements override options set in the config file.
The default config file provided lays out the default options for configuration and how to override them.

## Usage
The bar is only used to display text provided to it on *stdin*. 
Input is read and displayed everytime a new line is output. 
So make sure to use *echo -n* or some other print method that does not end in newline until you are ready to refresh the bar.
Text written to the bar can also include formatting blocks with the following format.

### Formatting
All formatting blocks are enclosed in *curly braces* **{}**.
All closing blocks are enclosed in *curly braces* and start with the *slash* **{/}**.

* {*f*__i__} {/*f*} => all characters within the blocks will be printed with the *font face* at index **i**. 
* {*F*__i__} {/*F*} => all characters within the blocks will be printed with the *font colour* at index **i**. 
* {*B*__i__} {/*B*} => all characters within the blocks will have a background highlight behind them with the *btcolour* at index **i**. 
* {*H*__i__} {/*H*} => all characters within the blocks will have an underline highlight behind them with the *htcolour* at index **i**. 

### Splitting Input
There is only one special block the is not in curly braces.
The *splitting block* is **<|>** and seperates between the left, right, and center displays.

* *0 splitting blocks* => the whole string will be considered **left-adjusted**.
* *1 splitting block* => the part of the string before the block will be **left-adjusted** and everything else will be **right-adjusted**.
* *2 or more splitting blocks* => the part of the string before the first block will be **left-adjusted** the part between the first and second will be **center-adjusted** and everything between the second and third will be **right-adjusted**. Any other *splitting blocks* and their strings will be ignored.

## Example
The bar running on my system by default, set up using the scripts in the files repo.

![Screenshot](https://github.com/ikubetoomuzik/unibar/blob/master/files/images/screenshot01.png)

## Installation
The project has been uploaded to crates.io and can be downloaded with:
```sh
cargo install unibar
```








## Author

By: **Curtis Jones** <*mail@curtisjones.ca*>
