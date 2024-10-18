use ab_glyph::{Font, FontRef, Glyph, Point, PxScale};
use image::{GrayImage, ImageBuffer, Luma};

use crate::error::{AsciiError, IntoGlyphOutlineMissingResult};

use super::font_handler::{CharAlignment, CharDistributionMatch, CharacterBackground};

#[derive(Debug, Clone, PartialEq)]
pub struct RasterizedChar {
    pub character: char,
    pub glyph: Glyph,
    pub raster_letter: ImageBuffer<Luma<u8>, Vec<u8>>,
    pub size: (usize, usize),
    pub alignment: CharAlignment,
    pub actual_coverage: f64,
    pub adjusted_coverage: f64,
}

impl RasterizedChar {
    fn new(
        character: char,
        glyph: Glyph,
        raster_letter: ImageBuffer<Luma<u8>, Vec<u8>>,
        size: (usize, usize),
        alignment: CharAlignment,
    ) -> RasterizedChar {
        let coverage = raster_letter.iter().fold(0f64, |mut sum, p| {
            sum += *p as f64 / 255.;
            sum
        }) / raster_letter.len() as f64;

        RasterizedChar {
            character,
            glyph,
            raster_letter,
            size,
            alignment,
            actual_coverage: coverage,
            adjusted_coverage: coverage,
        }
    }

    pub(crate) fn match_coverage(&self, target_coverage: f64) -> CharDistributionMatch {
        CharDistributionMatch {
            distance: (target_coverage - self.adjusted_coverage).abs(),
            rasterized_char: self,
        }
    }

    fn rasterize_glyph(
        glyph: &Glyph,
        font: &FontRef<'_>,
        (bounding_width, bounding_height): (usize, usize),
        alignment: CharAlignment,
        character_bg: CharacterBackground,
    ) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, AsciiError> {
        let q = font.outline_glyph(glyph.clone()).ok_or_ascii_err(glyph)?;

        let Point {
            x: char_width,
            y: char_height,
        } = {
            let char_bounds = q.px_bounds();
            let (char_min, char_max) = Self::remove_padding(char_bounds.min, char_bounds.max);
            assert_eq!(char_min, Point { x: 0., y: 0. });
            char_max
        };

        let mut letter = GrayImage::new((bounding_width) as u32, (bounding_height) as u32);
        q.draw(|x, y, c| {
            let cov = (match character_bg {
                CharacterBackground::Black => c,
                CharacterBackground::White => 1f32 / c,
            } * 255f32) as u8;
            let x = match alignment {
                CharAlignment::Left => x,
                CharAlignment::Center => x + ((bounding_width as u32 - char_width as u32) / 2),
                CharAlignment::Right => x + (bounding_width as f32 - char_width) as u32,
            };
            let y = y + (bounding_height as f32 - char_height) as u32;
            if x < letter.width() && y < letter.height() {
                letter.put_pixel(x, y, Luma::from([cov; 1]));
            }
        });

        Ok(letter)
    }

    /// Finds the ideal box size for the font and the requested Glyphs
    ///
    /// Removes any padding that may exist on the sides of the Glyphs Box and then
    /// selects the widest and talles Box to create a final ideal box size.
    pub(crate) fn char_boxing(font: &FontRef<'_>, glyphs: Vec<&Glyph>) -> (usize, usize) {
        glyphs
            .into_iter()
            .map(|g| {
                let rect = font.outline_glyph(g.clone()).unwrap().px_bounds();
                let (_, b) = Self::remove_padding(rect.min, rect.max);
                (b.x.ceil() as usize, b.y.ceil() as usize)
            })
            .fold((0, 0), |mut widest, new| {
                widest.0 = if new.0 > widest.0 { new.0 } else { widest.0 };
                widest.1 = if new.1 > widest.1 { new.1 } else { widest.1 };
                widest
            })
    }

    fn remove_padding(mut min: Point, mut max: Point) -> (Point, Point) {
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
}

pub(crate) struct RasterizedCharBuilder<'builder> {
    pub(crate) char: char,
    pub(crate) glyph: Glyph,
    pub(crate) font: &'builder FontRef<'builder>,
    pub(crate) alignment: &'builder CharAlignment,
    pub(crate) background: &'builder CharacterBackground,
    pub(crate) glyph_box: Option<(usize, usize)>,
    pub(crate) rasterized_letter: Option<ImageBuffer<Luma<u8>, Vec<u8>>>,
}

impl<'builder> RasterizedCharBuilder<'builder> {
    pub(crate) fn new(
        char: char,
        font_height: usize,
        font: &'builder FontRef,
        alignment: &'builder CharAlignment,
        background: &'builder CharacterBackground,
    ) -> Self {
        let glyph = font
            .glyph_id(char)
            .with_scale(PxScale::from(font_height as f32));
        Self {
            char,
            glyph,
            font,
            alignment,
            background,
            glyph_box: None,
            rasterized_letter: None,
        }
    }

    pub(crate) fn rasterize(mut self, font_box: (usize, usize)) -> Result<Self, AsciiError> {
        let RasterizedCharBuilder {
            glyph,
            font,
            alignment,
            background,
            glyph_box,
            rasterized_letter,
            ..
        } = &mut self;
        *rasterized_letter = Some(RasterizedChar::rasterize_glyph(
            glyph,
            font,
            font_box,
            **alignment,
            **background,
        )?);
        *glyph_box = Some(font_box);
        Ok(self)
    }

    pub(crate) fn build(self) -> RasterizedChar {
        let RasterizedCharBuilder {
            char,
            glyph,
            alignment,
            glyph_box,
            rasterized_letter,
            ..
        } = self;
        let (size, raster_letter) = match (glyph_box, rasterized_letter) {
            (Some(size), Some(raster_letter)) => (size, raster_letter),
            _ => unreachable!("please use the function rasterize before calling build"),
        };
        RasterizedChar::new(char, glyph, raster_letter, size, *alignment)
    }
}
