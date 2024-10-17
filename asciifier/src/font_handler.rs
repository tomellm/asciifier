use core::f32;

use ab_glyph::{Font, FontRef, Glyph, Point, PxScale};

use crate::error::{AsciiError, IntoGlyphOutlineMissingResult};

/// Returns the selected chars in rasterized form with the rectangle the rasterized
/// squares will take u.
///
/// # Arguments
///
/// * `chars` - The list of chars to rasterize
/// * `font` - The font to use for the rasterization
/// * `(font_width, font_height)` - The wanted size of the font, main component
///     here is the height since, but width can also be determined id wanted. Although
///     its only considered if wider then minimum width.
/// * `alignment` - since not each char is equally wide, this defines the chars
///     placement on the X axis
/// * `character_bg` - color of the background TODO: what the hell is this actually
pub fn rasterize_chars(
    chars: &[char],
    font: &FontRef<'_>,
    (font_width, font_height): (Option<usize>, usize),
    alignment: CharAlignment,
    character_bg: CharacterBackground,
) -> Result<(Vec<RasterizedChar>, (usize, usize)), AsciiError> {
    let builders = chars
        .iter()
        .map(|c| RasterizedCharBuilder::new(*c, font_height, font, &alignment, &character_bg))
        .collect::<Vec<_>>();

    let (possible_width, possible_height) =
        char_boxing(font, builders.iter().map(|t| &t.glyph).collect());

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

    let (rasterized_chars, rasterization_errors): (Vec<_>, Vec<_>) = builders
        .into_iter()
        .map(|builder| Ok(builder.rasterize(font_box)?.build()))
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

/// Finds the ideal box size for the font and the requested Glyphs
///
/// Removes any padding that may exist on the sides of the Glyphs Box and then
/// selects the widest and talles Box to create a final ideal box size.
fn char_boxing(font: &FontRef<'_>, glyphs: Vec<&Glyph>) -> (usize, usize) {
    glyphs
        .into_iter()
        .map(|g| {
            let rect = font.outline_glyph(g.clone()).unwrap().px_bounds();
            let (_, b) = remove_padding(rect.min, rect.max);
            (b.x.ceil() as usize, b.y.ceil() as usize)
        })
        .fold((0, 0), |mut widest, new| {
            widest.0 = if new.0 > widest.0 { new.0 } else { widest.0 };
            widest.1 = if new.1 > widest.1 { new.1 } else { widest.1 };
            widest
        })
}

fn rasterize_glyph(
    glyph: &Glyph,
    font: &FontRef<'_>,
    (bounding_width, bounding_height): (usize, usize),
    alignment: CharAlignment,
    character_bg: CharacterBackground,
) -> Result<Vec<u8>, AsciiError> {
    let q = font.outline_glyph(glyph.clone()).ok_or_ascii_err(glyph)?;

    let Point {
        x: char_width,
        y: char_height,
    } = {
        let char_bounds = q.px_bounds();
        let (char_min, char_max) = remove_padding(char_bounds.min, char_bounds.max);
        assert_eq!(char_min, Point { x: 0., y: 0. });
        char_max
    };

    let default_coverage = match character_bg {
        CharacterBackground::Black => 0,
        CharacterBackground::White => 255,
    };
    let mut letter: Vec<u8> = (0..bounding_width * bounding_height)
        .map(|_| default_coverage)
        .collect::<Vec<_>>();

    q.draw(|x, y, coverage| {
        let x = x as usize;
        let y = y as usize;
        let coverage = (match character_bg {
            CharacterBackground::Black => coverage,
            CharacterBackground::White => 1f32 / coverage,
        } * 255f32) as u8;
        let x = match alignment {
            CharAlignment::Left => x,
            CharAlignment::Center => x + ((bounding_width - char_width as usize) / 2),
            CharAlignment::Right => x + (bounding_width as f32 - char_height) as usize,
        };
        let y = y + (bounding_height as f32 - char_height) as usize;
        let pos = (x * bounding_width) + y;

        let Some(pixel) = letter.get_mut(pos) else {
            unreachable!()
        };
        *pixel = coverage;
    });

    Ok(letter)
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
    pub raster_letter: Vec<u8>,
    pub size: (usize, usize),
    pub alignment: CharAlignment,
    pub actual_coverage: f64,
    pub adjusted_coverage: f64,
}

impl RasterizedChar {
    fn new(
        character: char,
        glyph: Glyph,
        raster_letter: Vec<u8>,
        size: (usize, usize),
        alignment: CharAlignment,
    ) -> RasterizedChar {
        let coverage = raster_letter.iter().fold(0f64, |mut sum, p| {
            sum += *p as f64;
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
            distance: (target_coverage - self.actual_coverage).abs(),
            rasterized_char: self,
        }
    }
    pub(crate) fn width(&self) -> usize {
        self.size.0
    }
    pub(crate) fn height(&self) -> usize {
        self.size.1
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

#[derive(Debug, Clone)]
pub enum CharDistributionType {
    Even,
    Exact,
    ExactAdjustedBlacks,
    ExactAdjustedWhites,
}

impl CharDistributionType {
    pub(crate) fn adjust_coverage(&self, chars: &mut [RasterizedChar]) {
        if matches!(self, CharDistributionType::Exact) {
            return chars
                .iter_mut()
                .for_each(|char| char.adjusted_coverage = char.actual_coverage);
        }
        if matches!(self, CharDistributionType::Even) {
            let inc = 1 / chars.len();
            for (i, rc) in chars.iter_mut().enumerate() {
                rc.actual_coverage = i as f64 * inc as f64;
            }
            return;
        }
        let max = chars
            .iter()
            .max_by(|a, b| a.actual_coverage.partial_cmp(&b.actual_coverage).unwrap())
            .unwrap()
            .actual_coverage;
        if matches!(self, CharDistributionType::ExactAdjustedBlacks) {
            chars.iter_mut().for_each(|c| c.actual_coverage /= max);
            return;
        }
        let min = chars
            .iter()
            .min_by(|a, b| a.actual_coverage.partial_cmp(&b.actual_coverage).unwrap())
            .unwrap()
            .actual_coverage;

        chars
            .iter_mut()
            .for_each(|c| c.actual_coverage = (c.actual_coverage - min) / max);
    }
}

pub(crate) struct RasterizedCharBuilder<'builder> {
    pub(crate) char: char,
    pub(crate) glyph: Glyph,
    pub(crate) font: &'builder FontRef<'builder>,
    pub(crate) alignment: &'builder CharAlignment,
    pub(crate) background: &'builder CharacterBackground,
    pub(crate) glyph_box: Option<(usize, usize)>,
    pub(crate) rasterized_letter: Option<Vec<u8>>,
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
        *rasterized_letter = Some(rasterize_glyph(
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
