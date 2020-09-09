#![allow(dead_code, unused_variables)]
// File to define the valid string type we are using for display.
// By: Curtis Jones <mail@curtisjones.ca>
// Started on: September 07, 2020
//
//
use std::{mem, os::raw::*};
use x11_dl::{xft, xlib, xrender::XGlyphInfo};

unsafe fn default_font_idx(
    xft: &xft::Xft,
    dpy: *mut xlib::Display,
    fonts: &Vec<*mut xft::XftFont>,
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

unsafe fn string_pixel_width(
    xft: &xft::Xft,
    dpy: *mut xlib::Display,
    font: *mut xft::XftFont,
    string: &str,
) -> u32 {
    let mut extents: XGlyphInfo = mem::MaybeUninit::uninit().assume_init();
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
        mut self,
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
                for i in dt.start..dt.end {
                    default[i] = dt.idx;
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
pub struct BackDisplay {
    pub idx: usize,
    pub start: usize,
    pub end: usize,
}

impl BackDisplay {
    unsafe fn gen_list(
        xft: &xft::Xft,
        dpy: *mut xlib::Display,
        fonts: &Vec<*mut xft::XftFont>,
        background_objs: &Vec<DisplayType>,
        text_display: &Vec<FontDisplay>,
        text: &str,
    ) -> Vec<BackDisplay> {
        background_objs.iter().fold(Vec::new(), |mut acc, back_oj| {
            // Generate the start pixel val.
            let mut start = 0;
            for i in 0..text_display.len() {
                let font_display = &text_display[i];
                if font_display.start == back_oj.start {
                    break;
                } else if font_display.end > back_oj.start {
                    let chunk: String = text
                        .chars()
                        .skip(font_display.start)
                        .take(back_oj.start - font_display.start)
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
            for i in 0..text_display.len() {
                let font_display = &text_display[i];
                if font_display.start == back_oj.end {
                    let chunk: String = text.chars().skip(font_display.start).take(1).collect();
                    end += string_pixel_width(xft, dpy, fonts[font_display.face_idx], &chunk);
                    break;
                } else if font_display.end <= back_oj.end {
                    let chunk: String = text
                        .chars()
                        .skip(font_display.start)
                        .take(font_display.end - font_display.start)
                        .collect();
                    end += string_pixel_width(xft, dpy, fonts[font_display.face_idx], &chunk);
                } else {
                    let chunk: String = text
                        .chars()
                        .skip(font_display.start)
                        .take(back_oj.end - font_display.start)
                        .collect();
                    end += string_pixel_width(xft, dpy, fonts[font_display.face_idx], &chunk);
                    break;
                }
            }
            acc.push(BackDisplay {
                idx: back_oj.idx,
                start: start as usize,
                end: end as usize,
            });
            acc
        })
    }
}

#[derive(Debug)]
pub struct HighDisplay {
    pub idx: usize,
    pub start: usize,
    pub end: usize,
}

impl HighDisplay {
    unsafe fn gen_list(
        xft: &xft::Xft,
        dpy: *mut xlib::Display,
        fonts: &Vec<*mut xft::XftFont>,
        highlight_objs: &Vec<DisplayType>,
        text_display: &Vec<FontDisplay>,
        text: &str,
    ) -> Vec<HighDisplay> {
        highlight_objs.iter().fold(Vec::new(), |mut acc, high_oj| {
            // Generate the start pixel val.
            let mut start = 0;
            for i in 0..text_display.len() {
                let font_display = &text_display[i];
                if font_display.start == high_oj.start {
                    break;
                } else if font_display.end > high_oj.start {
                    let chunk: String = text
                        .chars()
                        .skip(font_display.start)
                        .take(high_oj.start - font_display.start)
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
            for i in 0..text_display.len() {
                let font_display = &text_display[i];
                if font_display.start == high_oj.end {
                    let chunk: String = text.chars().skip(font_display.start).take(1).collect();
                    end += string_pixel_width(xft, dpy, fonts[font_display.face_idx], &chunk);
                    break;
                } else if font_display.end <= high_oj.end {
                    let chunk: String = text
                        .chars()
                        .skip(font_display.start)
                        .take(font_display.end - font_display.start)
                        .collect();
                    end += string_pixel_width(xft, dpy, fonts[font_display.face_idx], &chunk);
                } else {
                    let chunk: String = text
                        .chars()
                        .skip(font_display.start)
                        .take(high_oj.end - font_display.start)
                        .collect();
                    end += string_pixel_width(xft, dpy, fonts[font_display.face_idx], &chunk);
                    break;
                }
            }
            acc.push(HighDisplay {
                idx: high_oj.idx,
                start: start as usize,
                end: end as usize,
            });
            acc
        })
    }
}

#[derive(Debug)]
pub struct FontDisplay {
    pub face_idx: usize,
    pub col_idx: usize,
    pub start: usize,
    pub end: usize,
}

impl FontDisplay {
    fn generate_list(
        col_objs: &Vec<DisplayType>,
        face_objs: &Vec<DisplayType>,
    ) -> Vec<FontDisplay> {
        let mut strt_idx = 0;
        col_objs.iter().fold(Vec::new(), |mut acc, cl_oj| {
            let col_idx = cl_oj.idx;
            let mut tmp: Vec<FontDisplay> = Vec::new();
            for i in strt_idx..face_objs.len() {
                let fc_oj = &face_objs[i];
                let face_idx = fc_oj.idx;
                let start = if fc_oj.start <= cl_oj.start {
                    cl_oj.start
                } else {
                    fc_oj.start
                };
                let end = if fc_oj.end >= cl_oj.end {
                    cl_oj.end
                } else {
                    fc_oj.end
                };
                if start != end {
                    tmp.push(FontDisplay {
                        face_idx,
                        col_idx,
                        start,
                        end,
                    });
                }
                if fc_oj.end >= cl_oj.end {
                    strt_idx = i;
                    break;
                }
            }
            acc.append(&mut tmp);
            acc
        })
    }
}

#[derive(Debug)]
pub struct ValidString {
    pub text: String,
    pub text_display: Vec<FontDisplay>,
    pub backgrounds: Vec<BackDisplay>,
    pub highlights: Vec<HighDisplay>,
}

impl ValidString {
    pub fn empty() -> ValidString {
        ValidString {
            text: String::new(),
            text_display: Vec::new(),
            backgrounds: Vec::new(),
            highlights: Vec::new(),
        }
    }
    pub unsafe fn parse_input_string(
        xft: &xft::Xft,
        dpy: *mut xlib::Display,
        fonts: &Vec<*mut xft::XftFont>,
        colours: &ColourPalette,
        input: String,
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
        input.chars().for_each(|ch| {
            if in_format_block {
                if closing_block {
                    match ch {
                        'B' => {
                            bckgrnd_tmp.2 = count;
                            background_vec.push(DisplayType::from(bckgrnd_tmp));
                            bckgrnd_tmp = (usize::MAX, count, 0);
                        }
                        'H' => {
                            highlht_tmp.2 = count;
                            highlight_vec.push(DisplayType::from(highlht_tmp));
                            highlht_tmp = (usize::MAX, count, 0);
                        }
                        'F' => {
                            fcol_tmp.2 = count;
                            font_colour_vec.push(DisplayType::from(fcol_tmp));
                            fcol_tmp = (0, count, 0);
                        }
                        'f' => {
                            fface_tmp.2 = count;
                            font_face_vec.push(DisplayType::from(fface_tmp));
                            fface_tmp = (usize::MAX, count, 0);
                        }
                        '}' => {
                            in_format_block = false;
                            closing_block = false;
                        }
                        _ => (),
                    }
                } else {
                    if next_is_index {
                        if let Some(d) = ch.to_digit(10) {
                            match index_type {
                                IndexType::BackgroundColour => {
                                    if d > (colours.background.len() - 1) as u32 {
                                        println!("Invalid background colour index -- TOO LARGE.");
                                    } else {
                                        bckgrnd_tmp.2 = count;
                                        background_vec.push(DisplayType::from(bckgrnd_tmp));
                                        bckgrnd_tmp = (d as usize, count, 0);
                                    }
                                }
                                IndexType::HighlightColour => {
                                    if d > (colours.highlight.len() - 1) as u32 {
                                        println!("Invalid highlight colour index -- TOO LARGE.");
                                    } else {
                                        highlht_tmp.2 = count;
                                        highlight_vec.push(DisplayType::from(highlht_tmp));
                                        highlht_tmp = (d as usize, count, 0);
                                    }
                                }
                                IndexType::FontColour => {
                                    if d > (colours.font.len() - 1) as u32 {
                                        println!("Invalid font colour index -- TOO LARGE.");
                                    } else {
                                        fcol_tmp.2 = count;
                                        font_colour_vec.push(DisplayType::from(fcol_tmp));
                                        fcol_tmp = (d as usize, count, 0);
                                    }
                                }
                                IndexType::FontFace => {
                                    if d > (fonts.len() - 1) as u32 {
                                        println!("Invalid font face index -- TOO LARGE.");
                                    } else {
                                        fface_tmp.2 = count;
                                        font_face_vec.push(DisplayType::from(fface_tmp));
                                        fface_tmp = (d as usize, count, 0);
                                    }
                                }
                            }
                        }
                        next_is_index = false;
                    } else {
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
                }
            } else {
                match ch {
                    '{' => in_format_block = true,
                    _ => {
                        count += 1;
                        default_font_faces.push(default_font_idx(&xft, dpy, fonts, ch));
                        text.push(ch);
                    }
                }
            }
        });
        bckgrnd_tmp.2 = count;
        highlht_tmp.2 = count;
        fcol_tmp.2 = count;
        fface_tmp.2 = count;
        // If our temp vars are 0 then we don't need this last push.
        if bckgrnd_tmp.2 - bckgrnd_tmp.1 != 0 {
            background_vec.push(DisplayType::from(bckgrnd_tmp))
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
        let merged_faces = DisplayType::merge_font_faces(default_font_faces, font_face_vec);
        let text_display = FontDisplay::generate_list(&font_colour_vec, &merged_faces);
        let highlights =
            HighDisplay::gen_list(xft, dpy, fonts, &highlight_vec, &text_display, &text);
        let backgrounds =
            BackDisplay::gen_list(xft, dpy, fonts, &background_vec, &text_display, &text);
        ValidString {
            text,
            text_display,
            backgrounds,
            highlights,
        }
    }

    pub unsafe fn draw(
        &self,
        xft: &xft::Xft,
        dpy: *mut xlib::Display,
        draw: *mut xft::XftDraw,
        palette: &ColourPalette,
        fonts: &Vec<*mut xft::XftFont>,
        x_start: i32,
        y_font: i32,
        highlight_height: u32,
    ) {
        let bar_height = 32;
        // Displaying the backgrounds first.
        self.backgrounds.iter().for_each(|b| {
            (xft.XftDrawRect)(
                draw,
                &palette.background[b.idx],
                x_start + b.start as c_int,
                0,
                (b.end - b.start) as c_uint,
                bar_height,
            );
        });
        // End of the background bit.
        // Display the highlights next.
        self.highlights.iter().for_each(|h| {
            (xft.XftDrawRect)(
                draw,
                &palette.highlight[h.idx],
                x_start + h.start as c_int,
                (bar_height - highlight_height) as c_int,
                (h.end - h.start) as c_uint,
                highlight_height as c_uint,
            );
        });
        // End of highlight bit.
        // Do the font bits last.
        let mut offset = 0;
        self.text_display.iter().for_each(|td| {
            let chunk: String = self
                .text
                .chars()
                .skip(td.start)
                .take(td.end - td.start)
                .collect();
            let font = fonts[td.face_idx];
            let font_colour = palette.font[td.col_idx];
            (xft.XftDrawStringUtf8)(
                draw,
                &font_colour,
                font,
                x_start + offset,
                y_font,
                chunk.as_bytes().as_ptr() as *const c_uchar,
                chunk.as_bytes().len() as c_int,
            );
            offset += string_pixel_width(xft, dpy, font, &chunk) as c_int;
        });
        // End of font bit.
    }
}
