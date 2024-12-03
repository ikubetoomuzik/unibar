// File to define the valid string type we are using for display.
// By: Curtis Jones <mail@curtisjones.ca>
// Started on: September 07, 2020
//

use anyhow::Result;
use std::{
    collections::{hash_map::Entry, HashMap},
    mem::MaybeUninit,
};
use x11_dl::{xft, xlib, xrender::XGlyphInfo};

/// Utility funtion so get the index of the first font that has a glyph for the provided char.
///
/// # Arguments
/// * xft:   -> reference to the xft lib.
/// * dpy:   -> pointer to the XDisplay object.
/// * fonts: -> list of fonts available to the bar.
/// * chr:   -> character we are checking.
///
/// # Output
/// Index of the default font in the Bar.fonts field as a usize.
unsafe fn default_font_idx(
    xft: &xft::Xft,
    dpy: *mut xlib::Display,
    fonts: &[*mut xft::XftFont],
    chr: char,
) -> usize {
    fonts
        .iter()
        // Get the position of the first font with a glyph for chr.
        .position(|&f| (xft.XftCharExists)(dpy, f, chr as u32) > 0)
        // If none are found then we default to 0.
        .unwrap_or(0)
}

/// Utility funtion so get the pixel width of a chunk of characters in a given font.
///
/// # Arguments
/// * xft:   -> reference to the xft lib.
/// * dpy:   -> pointer to the XDisplay object.
/// * font   -> pointer to the font we are checking the width of the string in.
/// * string -> refernce to the string we want the width of.
///
/// # Output
/// Returns a c_uint representing the pixel with of the <string> arg.
unsafe fn string_pixel_width(
    xft: &xft::Xft,
    dpy: *mut xlib::Display,
    font: *mut xft::XftFont,
    string: &str,
) -> u32 {
    // Rust gets mad if you don't initialize a variable before providing it as a function arg so we
    // lie to the rust compiler.
    let mut extents: MaybeUninit<XGlyphInfo> = MaybeUninit::uninit();
    // let mut extents: XGlyphInfo = init!();

    // Getting just so much info about the glyphs to be printed for the string arg when using the
    // font provided.
    (xft.XftTextExtentsUtf8)(
        dpy,
        font,
        string.as_bytes().as_ptr() as *mut u8,
        string.as_bytes().len() as i32,
        // &mut extents,
        extents.as_mut_ptr(),
    );

    let extents = extents.assume_init();

    // All that nice info and we just need the width.
    extents.width as u32
}

/// Private struct to contain colour information for the status bar.
/// Simpler than storing seperate fields as individual Vecs.
pub struct ColourPalette {
    /// Colours for the background highlight.
    pub background: Vec<xft::XftColor>,
    /// Colours for the underline highlight.
    pub underline: Vec<xft::XftColor>,
    /// Colours for the fonts.
    pub font: Vec<xft::XftColor>,
}

impl ColourPalette {
    /// Simple helper function to get an empty ColourPalette object.
    ///
    /// # Output
    /// ColourPalette than can be added to later.
    pub fn empty() -> ColourPalette {
        ColourPalette {
            background: Vec::new(),
            underline: Vec::new(),
            font: Vec::new(),
        }
    }

    /// Simple helper function to free all of the colours contained in seperate vecs.
    ///
    /// # Arguments
    /// * xft:    -> Reference to the Xft library for XftColorFree.
    /// * dpy:    -> Pointer to the Display that the bar is using.
    /// * cmap:   -> Pointer to the Colormap for the active Display.
    /// * visual: -> Pointer to the Visual for the active Display.
    pub unsafe fn destroy(
        &mut self,
        xft: &xft::Xft,
        dpy: *mut xlib::Display,
        cmap: xlib::Colormap,
        visual: *mut xlib::Visual,
    ) {
        // Do the background colours first.
        self.background
            .drain(..)
            .for_each(|mut col| (xft.XftColorFree)(dpy, visual, cmap, &mut col));
        // Next the underline highlight colours.
        self.underline
            .drain(..)
            .for_each(|mut col| (xft.XftColorFree)(dpy, visual, cmap, &mut col));
        // Finally we free our font colours.
        self.font
            .drain(..)
            .for_each(|mut col| (xft.XftColorFree)(dpy, visual, cmap, &mut col));
    }
}

#[derive(Clone, Copy, Debug)]
/// Private struct to use instead of a tuple of usize vals. Just helps to make things more explicit
/// during parsing of inputs. Used for font colours and faces, as well as background and underline
/// highlights.
struct DisplayTemp {
    /// Index into the vector of available fonts or colours.
    idx: usize,
    /// Start character within the parsed input string.
    start: usize,
    /// End character within the parsed input string.
    end: usize,
}

impl DisplayTemp {
    /// Some pathological need I have for constructors is being filled here I guess.
    ///
    /// # Arguments
    /// * idx:   -> see struct def.
    /// * start: -> see struct def.
    /// * end:   -> see struct def.
    ///
    /// # Output
    /// DisplayTemp that has the values provided in the arguements.
    fn from(idx: usize, start: usize, end: usize) -> DisplayTemp {
        DisplayTemp { idx, start, end }
    }

    /// The parsing function when reading in font faces doesn't check for the default font that
    /// each char belongs to. So this function sorts through and checks for only the sections that
    /// were not previously explicitly set.
    ///
    /// # Arguments
    /// def: -> reference to a HashMap that is acting like a lookup table for the default face
    /// index for each char.
    /// expl: -> reference to the explicitly set font face values from the parse. With usize::MAX
    /// set for the sections to be filled in.
    /// text: -> the actual printed text for the Input to be made.
    ///
    /// # Output
    /// A vec of DisplayTemp representing the font_faces for the Input.
    fn default_font_faces(
        def: &HashMap<char, usize>,
        expl: &[DisplayTemp],
        text: &str,
    ) -> Result<Vec<DisplayTemp>> {
        expl.iter()
            .try_fold(Vec::new(), |mut acc, dt| -> Result<Vec<DisplayTemp>> {
                // usize::MAX is our key value to mean use default here.
                if dt.idx == usize::MAX {
                    // Get the parts of the string within the DisplayTemp start and end.
                    let chunk: String = text.chars().take(dt.end).skip(dt.start).collect();

                    // Temp value to use while counting.
                    let mut tmp = DisplayTemp::from(0, dt.start, dt.start);

                    // Generating the Vec of DisplayTemp values by folding over the chars.
                    let mut res = chunk.chars().fold(Vec::new(), |mut ac, ch| {
                        // Get a copy of the default index for the char
                        let ch_idx = *def.get(&ch).expect("cant fail.");

                        // If the default index is different from the tmp val index then we push our
                        // tmp and start the new count.
                        if ch_idx != tmp.idx {
                            // We only do the push if we actually counted something. Otherwise just
                            // start over by changing the font index.
                            if tmp.start != tmp.end {
                                ac.push(tmp);
                                tmp.start = tmp.end;
                                tmp.end = tmp.start + 1;
                            }
                            tmp.idx = ch_idx;
                        } else {
                            // If the default is the same as the idx for our tmp val then we just increment
                            // the end value.
                            tmp.end += 1;
                        }

                        // Gotta return something every loop.
                        ac
                    });

                    // Once the loop is done we push the tmp value on the end if it counted and
                    // characters.
                    if tmp.start != tmp.end {
                        res.push(tmp);
                    }

                    // We use append here because it is possible to have generated multiple
                    // DisplayTemps when using the default indexes.
                    acc.append(&mut res);
                } else {
                    // If we are not using the defaults then we just push the value.
                    acc.push(*dt);
                }

                // Once we pushed the old DisplayTemp or generated default ones we return our acc Vec.
                Ok(acc)
            })
    }
}

#[derive(Debug)]
/// Private struct to contain display info for the underline and background highlight objects.
/// No reason to have different structs as they would just end up repeating code.
struct RectDisplayInfo {
    /// Index into the ColourPalette.{background or underline} vectors of colours for this section.
    idx: usize,
    /// Pixel x-value to start.
    start: usize,
    /// Pixel x-value to end.
    end: usize,
}

impl RectDisplayInfo {
    /// Function to generate a list of background or underline highlight instructions from the text
    /// and the text_display instructions.
    ///
    /// # Arguments
    /// * xft: -> Reference to the xft library for the string_pixel_width function.
    /// * dpy: -> Pointer to the xlib Display for the string_pixel_width function.
    /// * fonts: -> List of fonts available to use, for the string_pixel_width function.
    /// * rect_display_types: -> List of DisplayTemps for the backgrounds/highlights we are
    /// converting.
    /// * text_display: -> List of reference FontDisplayInfo objects used to get the string chunks.
    /// * text: -> Actual chars that will be displayed.
    ///
    /// # Output
    /// Returns the converted list of RectDisplayInfo objects for the highlights or backgrounds.
    unsafe fn gen_list(
        xft: &xft::Xft,
        dpy: *mut xlib::Display,
        fonts: &[*mut xft::XftFont],
        rect_display_types: &[DisplayTemp],
        text_display: &[FontDisplayInfo],
        text: &str,
    ) -> Vec<RectDisplayInfo> {
        // Only need one loop, going over the chars and converting from char indexes to pixel vals.
        rect_display_types
            .iter()
            .fold(Vec::new(), |mut acc, rect_oj| {
                // Generate the start pixel val.
                let mut start = 0;
                for font_display in text_display.iter() {
                    if font_display.start == rect_oj.start {
                        // If the starts are equal we are done.
                        break;
                    } else if font_display.end > rect_oj.start {
                        // If the end is larger than our start we can get the width up to our start
                        // and we are done.
                        let chunk: String = text
                            .chars()
                            .skip(font_display.start)
                            .take(rect_oj.start - font_display.start)
                            .collect();
                        start += string_pixel_width(xft, dpy, fonts[font_display.face_idx], &chunk);
                        break;
                    } else {
                        // Other wise just add in the pixel withd of the whole font display.
                        let chunk: String = text
                            .chars()
                            .take(font_display.end)
                            .skip(font_display.start)
                            .collect();
                        start += string_pixel_width(xft, dpy, fonts[font_display.face_idx], &chunk);
                    }
                }
                // Generate the end pixel val.
                let mut end = 0;
                for font_display in text_display.iter() {
                    if font_display.start == rect_oj.end {
                        // If the start matches our end we grab one char width and we are done.
                        let chunk: String = text.chars().skip(font_display.start).take(1).collect();
                        end += string_pixel_width(xft, dpy, fonts[font_display.face_idx], &chunk);
                        break;
                    } else if font_display.end <= rect_oj.end {
                        // If the ends match or the objects is less than ours we get the width of
                        // the whole font object in pixels.
                        let chunk: String = text
                            .chars()
                            .take(font_display.end)
                            .skip(font_display.start)
                            .collect();
                        end += string_pixel_width(xft, dpy, fonts[font_display.face_idx], &chunk);
                    } else {
                        // Otherwise we just get the width up to the end value our our object and
                        // convert it to pixels and finish up.
                        let chunk: String = text
                            .chars()
                            .skip(font_display.start)
                            .take(rect_oj.end - font_display.start)
                            .collect();
                        end += string_pixel_width(xft, dpy, fonts[font_display.face_idx], &chunk);
                        break;
                    }
                }
                // Once we have our start and end vals moved to pixels we can make the
                // RectDisplayInfo object with the idx from the current ref val.
                acc.push(RectDisplayInfo {
                    idx: rect_oj.idx,
                    start: start as usize,
                    end: end as usize,
                });
                acc
            })
    }
}

#[derive(Debug)]
/// Private struct used to hold the data for drawing the text of a Input to the display.
struct FontDisplayInfo {
    /// Index in the faces field of a Bar struct to be used.
    face_idx: usize,
    /// Index in the palette.font field of a Bar struct to be used.
    col_idx: usize,
    /// First char to include in formatting.
    start: usize,
    /// Include upto but not this value.
    end: usize,
}

impl FontDisplayInfo {
    /// General helper function to transition from our basic tuples into info we can use for
    /// font display.
    ///
    /// # Arguments
    /// * col_objs:  -> The instructions for the different font colors to use on different sections
    /// of the text.
    /// * face_objs: -> The instructions for different font faces to use on different sections of
    /// the text.
    ///
    /// # Output
    /// List of instructions for both font colour and font face to use. Seperated into the minimum
    /// different sets of instructions to use.
    fn generate_list(col_objs: &[DisplayTemp], face_objs: &[DisplayTemp]) -> Vec<FontDisplayInfo> {
        let mut strt_idx = 0;
        // Loop through the color objects and for each one generate the mixed FontDisplayInfo
        // object until the start of the face object is larger than the start of the colour object.
        col_objs.iter().fold(Vec::new(), |mut acc, cl_oj| {
            // Temp var is a vector because sometimes we create multiple objects.
            let mut tmp: Vec<FontDisplayInfo> = Vec::new();
            // We skip until the end of the previous iteration to try and save compute.
            for (i, fc_oj) in face_objs.iter().enumerate().skip(strt_idx) {
                let start = if fc_oj.start <= cl_oj.start {
                    cl_oj.start
                } else {
                    fc_oj.start
                };
                // The end is either the end of the face val if we need more loops or the end of
                // the colour val if we are gonna be done.
                let end = if fc_oj.end >= cl_oj.end {
                    cl_oj.end
                } else {
                    fc_oj.end
                };
                // If start and end are the same then there is no point in pushing.
                if start != end {
                    tmp.push(FontDisplayInfo {
                        face_idx: fc_oj.idx,
                        col_idx: cl_oj.idx,
                        start,
                        end,
                    });
                }
                // If we are at the end of the color object then we can break the face object loop
                // and set the start val for next iteration.
                if fc_oj.end >= cl_oj.end {
                    strt_idx = i;
                    break;
                }
            }
            // Appending is just push for another vec of similar objects.
            acc.append(&mut tmp);
            acc
        })
    }
}

/// Small private enum to help when parsing the inital input.
enum IndexType {
    BackgroundColour,
    HighlightColour,
    FontColour,
    FontFace,
}

#[derive(Debug)]
/// Main struct to hold display info for text on the bar.
/// Has references needed to display the text, backgrounds, and underlines.
pub struct Input {
    // The actual text to be drawn.
    text: String,
    // Reference for which chars to display in which font or colour.
    text_display: Vec<FontDisplayInfo>,
    // Reference for background highlights to draw with pixel val start and ends.
    backgrounds: Vec<RectDisplayInfo>,
    // Reference for underline highlights to draw with pixel val start and ends.
    underlines: Vec<RectDisplayInfo>,
}

impl Input {
    /// Helper to clear
    pub fn clear(&mut self) {
        self.text.clear();
        self.text_display.clear();
        self.backgrounds.clear();
        self.underlines.clear();
    }
    /// Small helper function to generate an emply Input.
    ///
    /// # Output
    /// Empty Input object to use as placeholder.
    pub fn empty() -> Input {
        Input {
            text: String::new(),
            text_display: Vec::new(),
            backgrounds: Vec::new(),
            underlines: Vec::new(),
        }
    }

    /// Draw the valid string onto the XftDraw object using the different struct fields. Starting
    /// with the background rectangles and then finishing with the text.
    ///
    /// # Arguments
    /// * xft:     -> Reference to the xft library and it's functions.
    /// * dpy:     -> Pointer to the XDisplay we are drawing to.
    /// * draw:    -> Pointer to the XftDraw object we are drawing to.
    /// * colours: -> Reference to the ColourPalette object holding the colours available.
    /// * fonts:   -> Reference to the list of fonts available.
    /// * start_x: -> X-value to start drawing at.
    /// * font_y:  -> Y-value to draw the text at.
    /// * height:  -> Height of the bar.
    /// * hlt_hgt: -> Height of the underline highlights.
    ///
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn draw(
        &self,
        xft: &xft::Xft,
        dpy: *mut xlib::Display,
        draw: *mut xft::XftDraw,
        colours: &ColourPalette,
        fonts: &[*mut xft::XftFont],
        start_x: i32,
        font_y: i32,
        height: u32,
        hlt_hgt: u32,
    ) {
        // Displaying the backgrounds first.
        self.backgrounds.iter().for_each(|b| {
            (xft.XftDrawRect)(
                draw,
                &colours.background[b.idx],
                start_x + b.start as i32,
                0,
                (b.end - b.start) as u32,
                height,
            );
        });

        // Display the highlights next.
        self.underlines.iter().for_each(|h| {
            (xft.XftDrawRect)(
                draw,
                &colours.underline[h.idx],
                start_x + h.start as i32,
                (height - hlt_hgt) as i32,
                (h.end - h.start) as u32,
                hlt_hgt,
            );
        });

        // Do the font bits last.
        self.text_display.iter().fold(0, |acc, td| {
            let chunk: String = self.text.chars().take(td.end).skip(td.start).collect();
            (xft.XftDrawStringUtf8)(
                draw,
                &colours.font[td.col_idx],
                fonts[td.face_idx],
                start_x + acc,
                font_y,
                chunk.as_bytes().as_ptr() as *const u8,
                chunk.as_bytes().len() as i32,
            );
            acc + string_pixel_width(xft, dpy, fonts[td.face_idx], &chunk) as i32
        });
    }

    /// Small helper function to get the pixel length of a Input object.
    ///
    /// # Arguments
    /// * xft:   -> reference to the link to the Xft library.
    /// * dpy:   -> pointer to the XDisplay object we are displaying to.
    /// * fonts: -> list of pointers to our XftFont objects available to use.
    ///
    /// # Output
    /// c_uint representing the pixel length of the self Input.
    pub unsafe fn len(
        &self,
        xft: &xft::Xft,
        dpy: *mut xlib::Display,
        fonts: &[*mut xft::XftFont],
    ) -> u32 {
        self.text_display.iter().fold(0, |acc, fd| {
            let chunk: String = self.text.chars().take(fd.end).skip(fd.start).collect();
            acc + string_pixel_width(xft, dpy, fonts[fd.face_idx], &chunk)
        })
    }

    /// Function to parse a string and develop a Input.
    /// Tries to do most of it's work in one loop over the input.
    ///
    /// # Arguments
    /// * xft:     -> reference to the link to the Xft library.
    /// * dpy:     -> pointer to the XDisplay object we are displaying to.
    /// * fonts:   -> list of pointers to our XftFont objects available to use.
    /// * colours: -> reference to the ColourPalette available to use.
    /// * input:   -> the string we are reading from to develop a Input.
    ///
    /// # Output
    /// Input made based on the input String object.
    pub fn parse_string(
        &mut self,
        xft: &xft::Xft,
        dpy: *mut xlib::Display,
        fonts: &[*mut xft::XftFont],
        def_font_map: &mut HashMap<char, usize>,
        colours: &ColourPalette,
        input: &str,
    ) -> Result<()> {
        // clear self to start.
        self.clear();

        // Loop vars.
        let mut in_format_block = false;
        let mut next_is_index = false;
        let mut closing_block = false;
        let mut index_type = IndexType::FontColour;

        // Result vars.
        let mut text = String::new();
        let mut background_vec: Vec<DisplayTemp> = Vec::new();
        let mut underline_vec: Vec<DisplayTemp> = Vec::new();
        let mut font_colour_vec: Vec<DisplayTemp> = Vec::new();
        let mut font_face_vec: Vec<DisplayTemp> = Vec::new();

        // Temp vars.
        let mut count: usize = 0;
        let mut bckgrnd_tmp: DisplayTemp = DisplayTemp::from(usize::MAX, 0, 0);
        let mut underln_tmp: DisplayTemp = DisplayTemp::from(usize::MAX, 0, 0);
        let mut fcol_tmp: DisplayTemp = DisplayTemp::from(0, 0, 0);
        let mut fface_tmp: DisplayTemp = DisplayTemp::from(usize::MAX, 0, 0);

        // Big ass loop to proces the input.
        for ch in input.chars() {
            if in_format_block {
                if closing_block {
                    match ch {
                        // B is the marker for the background highlight.
                        'B' => {
                            bckgrnd_tmp.end = count;
                            background_vec.push(bckgrnd_tmp);
                            bckgrnd_tmp = DisplayTemp::from(usize::MAX, count, 0);
                        }
                        // H is the marker for the underline highlight.
                        'H' => {
                            underln_tmp.end = count;
                            underline_vec.push(underln_tmp);
                            underln_tmp = DisplayTemp::from(usize::MAX, count, 0);
                        }
                        // F is the marker for the font colour.
                        'F' => {
                            fcol_tmp.end = count;
                            font_colour_vec.push(fcol_tmp);
                            fcol_tmp = DisplayTemp::from(0, count, 0);
                        }
                        // F is the marker for the font face.
                        'f' => {
                            fface_tmp.end = count;
                            font_face_vec.push(fface_tmp);
                            fface_tmp = DisplayTemp::from(usize::MAX, count, 0);
                        }
                        // End the block if we hit a close bracket.
                        '}' => {
                            in_format_block = false;
                            closing_block = false;
                        }
                        _ => (),
                    }
                // When we hit any of the markers the next char will be an index val so we
                // start to process it.
                } else if next_is_index {
                    // Converting to a base 10 digit creates a nice limit of 10 fonts,
                    // font-colours, background-colours, and highlight-colours.
                    if let Some(d) = ch.to_digit(10) {
                        // All four index types are basically the same, check to make sure the
                        // index is valid & if it is we push our current count onto the vec and
                        // start a new tmp count.
                        match index_type {
                            IndexType::BackgroundColour => {
                                if d > (colours.background.len() - 1) as u32 {
                                    eprintln!("Invalid background colour index -- TOO LARGE.");
                                } else {
                                    bckgrnd_tmp.end = count;
                                    background_vec.push(bckgrnd_tmp);
                                    bckgrnd_tmp = DisplayTemp::from(d as usize, count, 0);
                                }
                            }
                            IndexType::HighlightColour => {
                                if d > (colours.underline.len() - 1) as u32 {
                                    eprintln!("Invalid underline colour index -- TOO LARGE.");
                                } else {
                                    underln_tmp.end = count;
                                    underline_vec.push(underln_tmp);
                                    underln_tmp = DisplayTemp::from(d as usize, count, 0);
                                }
                            }
                            IndexType::FontColour => {
                                if d > (colours.font.len() - 1) as u32 {
                                    eprintln!("Invalid font colour index -- TOO LARGE.");
                                } else {
                                    fcol_tmp.end = count;
                                    font_colour_vec.push(fcol_tmp);
                                    fcol_tmp = DisplayTemp::from(d as usize, count, 0);
                                }
                            }
                            IndexType::FontFace => {
                                if d > (fonts.len() - 1) as u32 {
                                    eprintln!("Invalid font face index -- TOO LARGE.");
                                } else {
                                    fface_tmp.end = count;
                                    font_face_vec.push(fface_tmp);
                                    fface_tmp = DisplayTemp::from(d as usize, count, 0);
                                }
                            }
                        }
                        next_is_index = false;
                    }
                } else {
                    // If we are in a format block and have no other info we sort through and
                    // determine if we are in a closing or opening block. And what kind of format
                    // specifically.
                    match ch {
                        '/' => closing_block = true,
                        'B' => {
                            next_is_index = true;
                            index_type = IndexType::BackgroundColour;
                        }
                        'H' => {
                            next_is_index = true;
                            index_type = IndexType::HighlightColour;
                        }
                        'F' => {
                            next_is_index = true;
                            index_type = IndexType::FontColour;
                        }
                        'f' => {
                            next_is_index = true;
                            index_type = IndexType::FontFace;
                        }
                        '}' => in_format_block = false,
                        _ => (),
                    }
                }
            } else {
                // If we are not in a format block either it's a valid char or the beginning of a
                // new format block. We also take the chance to get the default valid font for the
                // char.
                match ch {
                    '{' => in_format_block = true,
                    _ => {
                        count += 1;
                        text.push(ch);
                        if let Entry::Vacant(v) = def_font_map.entry(ch) {
                            v.insert(unsafe { default_font_idx(xft, dpy, fonts, ch) });
                        }
                    }
                }
            }
        }

        // Set the end of the tmp var to the end count to finish off the tmp vars.
        bckgrnd_tmp.end = count;
        underln_tmp.end = count;
        fcol_tmp.end = count;
        fface_tmp.end = count;

        // Push the last val onto all of our count vecs.
        if bckgrnd_tmp.end != bckgrnd_tmp.start {
            background_vec.push(bckgrnd_tmp);
        }
        if underln_tmp.end != underln_tmp.start {
            underline_vec.push(underln_tmp);
        }
        if fcol_tmp.end != fcol_tmp.start {
            font_colour_vec.push(fcol_tmp);
        }
        if fface_tmp.end != fface_tmp.start {
            font_face_vec.push(fface_tmp);
        }

        // usize::MAX is our default value we need to get rid of it from background vec.
        let background_vec: Vec<DisplayTemp> = background_vec
            .iter()
            .filter_map(|&bk_oj| {
                if bk_oj.idx != usize::MAX {
                    Some(bk_oj)
                } else {
                    None
                }
            })
            .collect();

        // usize::MAX is our default value we need to get rid of it from highlight vec.
        let underline_vec: Vec<DisplayTemp> = underline_vec
            .iter()
            .filter_map(|&ul_oj| {
                if ul_oj.idx != usize::MAX {
                    Some(ul_oj)
                } else {
                    None
                }
            })
            .collect();

        // Fill in the default font faces.
        let merg_fcs = DisplayTemp::default_font_faces(def_font_map, &font_face_vec, &text)?;

        // Gen the final FontDisplayInfo objects.
        let text_display = FontDisplayInfo::generate_list(&font_colour_vec, &merg_fcs);

        // Gen the final underline RectDisplayInfo objects.
        let underlines = unsafe {
            RectDisplayInfo::gen_list(xft, dpy, fonts, &underline_vec, &text_display, &text)
        };

        // Gen the final background RectDisplayInfo objects.
        let backgrounds = unsafe {
            RectDisplayInfo::gen_list(xft, dpy, fonts, &background_vec, &text_display, &text)
        };

        // Return our valid string using the objects we generated previously.
        self.text = text;
        self.text_display = text_display;
        self.underlines = underlines;
        self.backgrounds = backgrounds;
        Ok(())
    }
}
