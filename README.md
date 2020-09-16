# Unibar

Simple Xorg display bar written for speed and ease of use.

## CLI Interface

### Flags
* **-H, --help** ---> *Display help info*
* **-V, --version** ---> *Display version info*

### Options
* **-c, --config <CONFIG>** ---> *Specify custom config file to use.*
* **-p, --position <POSITION>** ---> *Choose bar position, options are* __TOP__ *or* __BOTTOM__*.*
* **-h, --height <HEIGHT>** ---> *Choose bar height in pixels.*
* **-b, --background <DEFBACKGROUND>** ---> *Choose default bg colour in '#XXXXXX' hex format.*
* **-u, --underline <UNDERLINE>** ---> *Choose underline highlight height in pixels.*
* **-y, --fonty <FONTY>** ---> *Choose font offset from top of bar in pixels.*
* **-f, --fonts <FONTS>...** ---> *Comma seperated list of FcConfig font name strings. Ex. 'FontName:size=XX:antialias=true/false'.*
* **-F, --ftcolours <FTCOLOURS>...** ---> *Comma seperated list of font colours in '#XXXXXX' hex format.*
* **-B, --bgcolours <BGCOLOURS>...** ---> *Comma seperated list of background highlight colours in '#XXXXXX' hex format.*
* **-H, --htcolours <HTCOLOURS>...** ---> *Comma seperated list of underline highlight colours in '#XXXXXX' hex format.*

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








## Author

By: **Curtis Jones** <*mail@curtisjones.ca*>
