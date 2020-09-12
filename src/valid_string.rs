// File to define the valid string type we are using for display.
// By: Curtis Jones <mail@curtisjones.ca>
// Started on: September 07, 2020
//
//
use super::init;
use std::{mem, os::raw::*};
use x11_dl::{xft, xlib, xrender::XGlyphInfo};

unsafe fn default_font_idx(
    xft: &xft::Xft,
    dpy: *mut xlib::Display,
    fonts: &[*mut xft::XftFont],
    chr: char,
) -> usize {
    match fonts
        .iter()
        .position(|&f| (xft.XftCharExists)(dpy, f, chr as c_uint) > 0)
    {
        Some(i) => i,
        None => 0,
    }
}

/// Utility funtion so get the pixel width of a chunk of characters in a given font.
/// Returns a u32 with that value.
///
/// # Arguments
/// * xft:   -> reference to the xft lib.
/// * dpy:   -> pointer to the XDisplay object.
/// * font   -> pointer to the font we are checking the width of the string in.
/// * string -> refernce to the string we want the width of.
///
unsafe fn string_pixel_width(
    xft: &xft::Xft,
    dpy: *mut xlib::Display,
    font: *mut xft::XftFont,
    string: &str,
) -> u32 {
    let mut extents: XGlyphInfo = init!();
    (xft.XftTextExtentsUtf8)(
        dpy,
        font,
        string.as_bytes().as_ptr() as *mut c_uchar,
        string.as_bytes().len() as c_int,
        &mut extents,
    );
    extents.width as u32
}

pub struct ColourPalette {
    pub background: Vec<xft::XftColor>,
    pub highlight: Vec<xft::XftColor>,
    pub font: Vec<xft::XftColor>,
}

impl ColourPalette {
    pub fn empty() -> ColourPalette {
        ColourPalette {
            background: Vec::new(),
            highlight: Vec::new(),
            font: Vec::new(),
        }
    }

    pub unsafe fn destroy(
        &mut self,
        xft: &xft::Xft,
        dpy: *mut xlib::Display,
        cmap: xlib::Colormap,
        visual: *mut xlib::Visual,
    ) {
        self.background
            .drain(..)
            .for_each(|mut col| (xft.XftColorFree)(dpy, visual, cmap, &mut col));
        self.highlight
            .drain(..)
            .for_each(|mut col| (xft.XftColorFree)(dpy, visual, cmap, &mut col));
        self.font
            .drain(..)
            .for_each(|mut col| (xft.XftColorFree)(dpy, visual, cmap, &mut col));
    }
}

enum IndexType {
    BackgroundColour,
    HighlightColour,
    FontColour,
    FontFace,
}

#[derive(Clone, Copy, Debug)]
struct DisplayType {
    idx: usize,
    start: usize,
    end: usize,
}

impl DisplayType {
    fn from(inp: (usize, usize, usize)) -> DisplayType {
        let (idx, start, end) = inp;
        DisplayType { idx, start, end }
    }
    fn merge_font_faces(mut default: Vec<usize>, explicit: Vec<DisplayType>) -> Vec<DisplayType> {
        explicit.iter().for_each(|dt| {
            if dt.idx != usize::MAX {
                for item in default.iter_mut().take(dt.end).skip(dt.start) {
                    *item = dt.idx;
                }
            }
        });
        let mut tmp: (usize, usize, usize) = (0, 0, 0);
        let mut count = 0;
        let mut result = default.iter().fold(Vec::new(), |mut acc, &i| {
            if i != tmp.0 {
                if count > 0 {
                    tmp.2 = count;
                    acc.push(DisplayType::from(tmp));
                }
                tmp = (i, count, 0);
            }
            count += 1;
            acc
        });
        if count - tmp.1 > 0 {
            tmp.2 = count;
            result.push(DisplayType::from(tmp));
        }
        result
    }
}

#[derive(Debug)]
struct RectDisplayInfo {
    idx: usize,
    start: usize,
    end: usize,
}

impl RectDisplayInfo {
    unsafe fn gen_list(
        xft: &xft::Xft,
        dpy: *mut xlib::Display,
        fonts: &[*mut xft::XftFont],
        rect_display_types: &[DisplayType],
        text_display: &[FontDisplayInfo],
        text: &str,
    ) -> Vec<RectDisplayInfo> {
        rect_display_types
            .iter()
            .fold(Vec::new(), |mut acc, rect_oj| {
                // Generate the start pixel val.
                let mut start = 0;
                for font_display in text_display.iter() {
                    if font_display.start == rect_oj.start {
                        break;
                    } else if font_display.end > rect_oj.start {
                        let chunk: String = text
                            .chars()
                            .skip(font_display.start)
                            .take(rect_oj.start - font_display.start)
                            .collect();
                        start += string_pixel_width(xft, dpy, fonts[font_display.face_idx], &chunk);
                        break;
                    } else {
                        let chunk: String = text
                            .chars()
                            .skip(font_display.start)
                            .take(font_display.end - font_display.start)
                            .collect();
                        start += string_pixel_width(xft, dpy, fonts[font_display.face_idx], &chunk);
                    }
                }
                // Generate the end pixel val.
                let mut end = 0;
                for font_display in text_display.iter() {
                    if font_display.start == rect_oj.end {
                        let chunk: String = text.chars().skip(font_display.start).take(1).collect();
                        end += string_pixel_width(xft, dpy, fonts[font_display.face_idx], &chunk);
                        break;
                    } else if font_display.end <= rect_oj.end {
                        let chunk: String = text
                            .chars()
                            .take(font_display.end)
                            .skip(font_display.start)
                            .collect();
                        end += string_pixel_width(xft, dpy, fonts[font_display.face_idx], &chunk);
                    } else {
                        let chunk: String = text
                            .chars()
                            .skip(font_display.start)
                            .take(rect_oj.end - font_display.start)
                            .collect();
                        end += string_pixel_width(xft, dpy, fonts[font_display.face_idx], &chunk);
                        break;
                    }
                }
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
/// Private struct used to hold the data for drawing the text of a ValidString to the display.
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
    fn generate_list(col_objs: &[DisplayType], face_objs: &[DisplayType]) -> Vec<FontDisplayInfo> {
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

#[derive(Debug)]
/// Main struct to hold display info for text on the bar.
/// Has references needed to display the text, backgrounds, and highlights.
pub struct ValidString {
    /// The actual text to be drawn.
    text: String,
    /// Reference for which chars to display in which font or colour.
    text_display: Vec<FontDisplayInfo>,
    /// Reference for background highlights to draw with pixel val start and ends.
    backgrounds: Vec<RectDisplayInfo>,
    /// Reference for underline highlights to draw with pixel val start and ends.
    highlights: Vec<RectDisplayInfo>,
}

impl ValidString {
    /// Small helper function to generate an emply ValidString.
    ///
    /// # Output
    /// Empty ValidString object to use as placeholder.
    pub fn empty() -> ValidString {
        ValidString {
            text: String::new(),
            text_display: Vec::new(),
            backgrounds: Vec::new(),
            highlights: Vec::new(),
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
    pub unsafe fn draw(
        &self,
        xft: &xft::Xft,
        dpy: *mut xlib::Display,
        draw: *mut xft::XftDraw,
        colours: &ColourPalette,
        fonts: &[*mut xft::XftFont],
        start_x: c_int,
        font_y: c_int,
        height: c_uint,
        hlt_hgt: c_uint,
    ) {
        // Displaying the backgrounds first.
        self.backgrounds.iter().for_each(|b| {
            (xft.XftDrawRect)(
                draw,
                &colours.background[b.idx],
                start_x + b.start as c_int,
                0,
                (b.end - b.start) as c_uint,
                height,
            );
        });

        // Display the highlights next.
        self.highlights.iter().for_each(|h| {
            (xft.XftDrawRect)(
                draw,
                &colours.highlight[h.idx],
                start_x + h.start as c_int,
                (height - hlt_hgt) as c_int,
                (h.end - h.start) as c_uint,
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
                chunk.as_bytes().as_ptr() as *const c_uchar,
                chunk.as_bytes().len() as c_int,
            );
            acc + string_pixel_width(xft, dpy, fonts[td.face_idx], &chunk) as c_int
        });
    }

    /// Small helper function to get the pixel length of a ValidString object.
    ///
    /// # Arguments
    /// * xft:   -> reference to the link to the Xft library.
    /// * dpy:   -> pointer to the XDisplay object we are displaying to.
    /// * fonts: -> list of pointers to our XftFont objects available to use.
    ///
    /// # Output
    /// u32 representing the pixel length of the self ValidString.
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

    /// Function to parse a string and develop a ValidString.
    /// Tries to do most of it's work in one loop over the input.
    ///
    /// # Arguments
    /// * xft:     -> reference to the link to the Xft library.
    /// * dpy:     -> pointer to the XDisplay object we are displaying to.
    /// * fonts:   -> list of pointers to our XftFont objects available to use.
    /// * colours: -> reference to the ColourPalette available to use.
    /// * input:   -> the string we are reading from to develop a ValidString.
    ///
    /// # Output
    /// ValidString made based on the input String object.
    pub fn parse_input_string(
        xft: &xft::Xft,
        dpy: *mut xlib::Display,
        fonts: &[*mut xft::XftFont],
        colours: &ColourPalette,
        input: &str,
    ) -> ValidString {
        // Loop vars.
        let mut in_format_block = false;
        let mut next_is_index = false;
        let mut closing_block = false;
        let mut index_type = IndexType::FontColour;

        // Result vars.
        let mut text = String::new();
        let mut background_vec: Vec<DisplayType> = Vec::new();
        let mut highlight_vec: Vec<DisplayType> = Vec::new();
        let mut font_colour_vec: Vec<DisplayType> = Vec::new();
        let mut font_face_vec: Vec<DisplayType> = Vec::new();
        let mut default_font_faces: Vec<usize> = Vec::new();

        // Temp vars.
        let mut count: usize = 0;
        let mut bckgrnd_tmp: (usize, usize, usize) = (usize::MAX, 0, 0);
        let mut highlht_tmp: (usize, usize, usize) = (usize::MAX, 0, 0);
        let mut fcol_tmp: (usize, usize, usize) = (0, 0, 0);
        let mut fface_tmp: (usize, usize, usize) = (usize::MAX, 0, 0);

        // Big ass loop to proces the input.
        for ch in input.chars() {
            if in_format_block {
                if closing_block {
                    match ch {
                        // B is the marker for the background highlight.
                        'B' => {
                            bckgrnd_tmp.2 = count;
                            background_vec.push(DisplayType::from(bckgrnd_tmp));
                            bckgrnd_tmp = (usize::MAX, count, 0);
                        }
                        // H is the marker for the underline highlight.
                        'H' => {
                            highlht_tmp.2 = count;
                            highlight_vec.push(DisplayType::from(highlht_tmp));
                            highlht_tmp = (usize::MAX, count, 0);
                        }
                        // F is the marker for the font colour.
                        'F' => {
                            fcol_tmp.2 = count;
                            font_colour_vec.push(DisplayType::from(fcol_tmp));
                            fcol_tmp = (0, count, 0);
                        }
                        // F is the marker for the font face.
                        'f' => {
                            fface_tmp.2 = count;
                            font_face_vec.push(DisplayType::from(fface_tmp));
                            fface_tmp = (usize::MAX, count, 0);
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
                                    bckgrnd_tmp.2 = count;
                                    background_vec.push(DisplayType::from(bckgrnd_tmp));
                                    bckgrnd_tmp = (d as usize, count, 0);
                                }
                            }
                            IndexType::HighlightColour => {
                                if d > (colours.highlight.len() - 1) as u32 {
                                    eprintln!("Invalid highlight colour index -- TOO LARGE.");
                                } else {
                                    highlht_tmp.2 = count;
                                    highlight_vec.push(DisplayType::from(highlht_tmp));
                                    highlht_tmp = (d as usize, count, 0);
                                }
                            }
                            IndexType::FontColour => {
                                if d > (colours.font.len() - 1) as u32 {
                                    eprintln!("Invalid font colour index -- TOO LARGE.");
                                } else {
                                    fcol_tmp.2 = count;
                                    font_colour_vec.push(DisplayType::from(fcol_tmp));
                                    fcol_tmp = (d as usize, count, 0);
                                }
                            }
                            IndexType::FontFace => {
                                if d > (fonts.len() - 1) as u32 {
                                    eprintln!("Invalid font face index -- TOO LARGE.");
                                } else {
                                    fface_tmp.2 = count;
                                    font_face_vec.push(DisplayType::from(fface_tmp));
                                    fface_tmp = (d as usize, count, 0);
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
                        unsafe {
                            default_font_faces.push(default_font_idx(&xft, dpy, fonts, ch));
                        }
                    }
                }
            }
        }

        // Set the end of the tmp var to the end count to finish off the tmp vars.
        bckgrnd_tmp.2 = count;
        highlht_tmp.2 = count;
        fcol_tmp.2 = count;
        fface_tmp.2 = count;

        // Push the last val onto all of our count vecs.
        if bckgrnd_tmp.2 - bckgrnd_tmp.1 != 0 {
            background_vec.push(DisplayType::from(bckgrnd_tmp));
        }
        if highlht_tmp.2 - highlht_tmp.1 != 0 {
            highlight_vec.push(DisplayType::from(highlht_tmp));
        }
        if fcol_tmp.2 - fcol_tmp.1 != 0 {
            font_colour_vec.push(DisplayType::from(fcol_tmp));
        }
        if fface_tmp.2 - fface_tmp.1 != 0 {
            font_face_vec.push(DisplayType::from(fface_tmp));
        }

        // usize::MAX is our default value we need to get rid of it from background vec.
        let background_vec: Vec<DisplayType> = background_vec
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
        let highlight_vec: Vec<DisplayType> = highlight_vec
            .iter()
            .filter_map(|&hi_oj| {
                if hi_oj.idx != usize::MAX {
                    Some(hi_oj)
                } else {
                    None
                }
            })
            .collect();

        // Override the default detected font faces.
        let merged_faces = DisplayType::merge_font_faces(default_font_faces, font_face_vec);

        // Gen the final FontDisplayInfo objects.
        let text_display = FontDisplayInfo::generate_list(&font_colour_vec, &merged_faces);
        // Gen the final RectDisplayInfo objects.
        let highlights = unsafe {
            RectDisplayInfo::gen_list(xft, dpy, fonts, &highlight_vec, &text_display, &text)
        };
        // Gen the final RectDisplayInfo objects.
        let backgrounds = unsafe {
            RectDisplayInfo::gen_list(xft, dpy, fonts, &background_vec, &text_display, &text)
        };

        // Return our valid string using the objects we generated previously.
        ValidString {
            text,
            text_display,
            backgrounds,
            highlights,
        }
    }
}
