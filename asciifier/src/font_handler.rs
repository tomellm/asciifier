use ab_glyph::{Font, FontRef, Glyph, Point, PxScale};
use image::{GrayImage, ImageBuffer, Luma};

use crate::error::{AsciiError, IntoGlyphOutlineMissingResult};

pub fn rasterize_chars(
    chars: &Vec<char>,
    font: &FontRef<'_>,
    (font_width, font_height): (Option<usize>, usize),
    alignment: CharAlignment,
    character_bg: CharacterBackground,
) -> Result<(Vec<RasterizedChar>, (usize, usize)), AsciiError> {
    let glyphs: Vec<(char, Glyph)> = chars
        .into_iter()
        .map(|c| {
            (
                *c,
                font.glyph_id(*c).with_scale(PxScale {
                    x: font_height as f32,
                    y: font_height as f32,
                }),
            )
        })
        .collect();

    let (possible_width, possible_height) =
        char_boxing(&font, glyphs.iter().map(|t| &t.1).collect());

    let font_box = (
        font_width
            .map(|width| {
                if width > possible_width {
                    width
                } else {
                    possible_width
                }
            })
            .unwrap_or(possible_width),
        if font_height > possible_height {
            font_height
        } else {
            possible_height
        },
    );

    let (rasterized_chars, rasterization_errors): (Vec<_>, Vec<_>) = glyphs
        .into_iter()
        .map(|t| {
            let buffer = glyph_to_buffer(&t.1, &font, font_box, alignment, character_bg)?;
            Ok(RasterizedChar::new(t.0, t.1, buffer, font_box, alignment))
        })
        .partition(Result::is_ok);

    if !rasterization_errors.is_empty() {
        return Err(rasterization_errors
            .into_iter()
            .map(Result::unwrap_err)
            .collect::<Vec<_>>()
            .into());
    }

    let rasterized_chars = rasterized_chars
        .into_iter()
        .map(Result::unwrap)
        .collect::<Vec<_>>();
    Ok((rasterized_chars, font_box))
}

fn char_boxing(font: &FontRef<'_>, glyphs: Vec<&Glyph>) -> (usize, usize) {
    let out = glyphs
        .into_iter()
        .map(|g| {
            let rect = font
                .outline_glyph(g.clone())
                .expect("The outline of the glyph could not be found!")
                .px_bounds();
            let (_, b) = normalize_bounds(rect.min, rect.max);
            (b.x.ceil(), b.y.ceil())
        })
        .fold((0.0, 0.0), |a, b| {
            let mut fin = (0.0, 0.0);
            fin.0 = if b.0 > a.0 { b.0 } else { a.0 };
            fin.1 = if b.1 > a.1 { b.1 } else { a.1 };
            fin
        });

    (out.0.ceil() as usize, out.1.ceil() as usize)
}

fn glyph_to_buffer(
    glyph: &Glyph,
    font: &FontRef<'_>,
    bounding_box: (usize, usize),
    alignment: CharAlignment,
    character_bg: CharacterBackground,
) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, AsciiError> {
    let q = font.outline_glyph(glyph.clone()).ok_or_ascii_err(glyph)?;

    let char_bounds = q.px_bounds();
    let (_, char_max) = normalize_bounds(char_bounds.min, char_bounds.max);

    let mut letter = GrayImage::new((bounding_box.0) as u32, (bounding_box.1) as u32);
    q.draw(|x, y, c| {
        let cov = (match character_bg {
            CharacterBackground::Black => c,
            CharacterBackground::White => 1f32 / c,
        } * 255f32) as u8;
        let x = match alignment {
            CharAlignment::Left => x,
            CharAlignment::Center => x + ((bounding_box.0 as u32 - char_max.x as u32) / 2),
            CharAlignment::Right => x + (bounding_box.0 as f32 - char_max.x) as u32,
        };
        let y = y + (bounding_box.1 as f32 - char_max.y) as u32;
        if x < letter.width() && y < letter.height() {
            letter.put_pixel(x, y, Luma::from([cov; 1]));
        }
    });

    Ok(letter)
}

fn normalize_bounds(mut min: Point, mut max: Point) -> (Point, Point) {
    if min.x != 0f32 {
        let distance = 0f32 - min.x;
        min.x += distance;
        max.x += distance;
    }
    if min.y != 0f32 {
        let distance = 0f32 - min.y;
        min.y += distance;
        max.y += distance;
    }

    (min, max)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CharAlignment {
    Left,
    #[default]
    Center,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterizedChar {
    pub character: char,
    pub glyph: Glyph,
    pub raster_letter: ImageBuffer<Luma<u8>, Vec<u8>>,
    pub size: (usize, usize),
    pub alignment: CharAlignment,
    pub coverage: f64,
}

impl RasterizedChar {
    fn new(
        character: char,
        glyph: Glyph,
        raster_letter: ImageBuffer<Luma<u8>, Vec<u8>>,
        size: (usize, usize),
        alignment: CharAlignment,
    ) -> RasterizedChar {
        let num_pixels = (raster_letter.width() * raster_letter.height()) as f64;
        let coverage = raster_letter
            .pixels()
            .map(|Luma(pixel_luma)| pixel_luma[0] as f64)
            .fold(0.0, |a, b| a + b)
            / num_pixels
            / 255f64;

        RasterizedChar {
            character,
            glyph,
            raster_letter,
            size,
            alignment,
            coverage,
        }
    }

    pub(crate) fn match_coverage<'a>(&'a self, target_coverage: f64) -> CharDistributionMatch<'a> {
        CharDistributionMatch {
            distance: (target_coverage - self.coverage).abs(),
            rasterized_char: self,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum CharacterBackground {
    #[default]
    Black,
    White,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CharDistributionMatch<'a> {
    distance: f64,
    pub rasterized_char: &'a RasterizedChar,
}

impl<'a> PartialOrd for CharDistributionMatch<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.distance.partial_cmp(&other.distance)
    }
} 

