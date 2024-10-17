use std::{fs::File, io::Read, path::PathBuf};

use ab_glyph::FontRef;
use image::{GenericImage, GenericImageView, GrayImage, ImageBuffer, ImageFormat, Luma, Rgb};

use crate::{
    ascii_image::GroupedImage,
    basic::convert_to_luminance,
    error::{AsciiError, IntoAsciiError, IntoConvertNotCalledResult},
    font_handler::{
        rasterize_chars, CharAlignment, CharDistributionType, CharacterBackground, RasterizedChar,
    },
};

pub struct Asciifier;

impl Asciifier {
    pub fn load_image<'font>(path: impl Into<PathBuf>) -> Result<ImageBuilder<'font>, AsciiError> {
        let mut file = File::open(path.into()).ascii_err()?;
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).ascii_err()?;

        let buffer = image::load_from_memory(&bytes).ascii_err()?.to_rgb8();
        ImageBuilder::default_from_image(buffer)
    }
}

#[derive(Debug, Clone)]
pub struct ImageBuilder<'font> {
    chars: Vec<char>,
    font: FontRef<'font>,
    char_height: usize,
    char_alignment: CharAlignment,
    char_distribution: CharDistributionType,
    char_background: CharacterBackground,
    image: ImageBuffer<Rgb<u8>, Vec<u8>>,
    asciified_image: Option<ImageBuffer<Luma<u8>, Vec<u8>>>,
}

impl<'font> TryFrom<ImageBuffer<Rgb<u8>, Vec<u8>>> for ImageBuilder<'font> {
    type Error = AsciiError;
    fn try_from(value: ImageBuffer<Rgb<u8>, Vec<u8>>) -> Result<Self, Self::Error> {
        let chars = "^°<>|{}≠¿'][¢¶`.,:;-_#'+*?=)(/&%$§qwertzuiopasdfghjklyxcvbnmQWERTZUIOPASDFGHJKLYXCVBNM".chars().collect();
        let f_height: usize = 12;

        let font = FontRef::try_from_slice(include_bytes!("../../assets/fonts/Hasklug-2.otf"))
            .ascii_err()?;

        Self::new(
            chars,
            font,
            f_height,
            CharAlignment::Center,
            CharDistributionType::ExactAdjustedBlacks,
            CharacterBackground::Black,
            value,
        )
    }
}

impl<'font> ImageBuilder<'font> {
    fn default_from_image(buffer: ImageBuffer<Rgb<u8>, Vec<u8>>) -> Result<Self, AsciiError> {
        buffer.try_into()
    }
    fn new(
        chars: Vec<char>,
        font: FontRef<'font>,
        char_height: usize,
        char_alignment: CharAlignment,
        char_distribution: CharDistributionType,
        char_background: CharacterBackground,
        image: ImageBuffer<Rgb<u8>, Vec<u8>>,
    ) -> Result<Self, AsciiError> {
        Ok(Self {
            chars,
            font,
            char_height,
            char_alignment,
            char_distribution,
            char_background,
            image,
            asciified_image: None,
        })
    }

    pub fn char_height(&mut self, new_height: usize) -> &mut Self {
        self.char_height = new_height;
        self
    }

    pub fn convert(&mut self) -> Result<&mut Self, AsciiError> {
        let (mut rasterized_chars, (font_width, font_height)) = rasterize_chars(
            &self.chars,
            &self.font,
            (None, self.char_height),
            self.char_alignment,
            self.char_background,
        )?;

        // TODO: how should I handle the luminance situation
        let image = convert_to_luminance(&self.image);

        let mut grouped_image = GroupedImage::new(font_width, font_height);
        image.enumerate_pixels().for_each(|p| grouped_image.push(p));

        let mut final_image = GrayImage::new(self.image.width(), self.image.height());

        self.char_distribution
            .adjust_coverage(&mut rasterized_chars);

        let mut flat_sample = final_image.as_flat_samples_mut();
        let mut buffer_view = flat_sample.as_view_mut::<Luma<u8>>().ascii_err()?;
        let buffer_width = buffer_view.width() as usize;
        let pixels_mut = buffer_view.image_mut_slice();

        for (row_i, group_row) in grouped_image.groups.iter_mut().enumerate() {
            for (col_i, group) in group_row.iter_mut().enumerate() {
                let glyph = &self.best_char_match(group.coverage(), &rasterized_chars);
                let letter_bytes = &glyph.raster_letter;

                let top_left_x = col_i * font_width;
                let top_left_y = row_i * font_height;
                let section_start = (top_left_x * font_width) + top_left_y;

                for row in 0..glyph.height() {
                    let start = section_start + (buffer_width * row);
                    let glyph_start = row * glyph.width();
                    pixels_mut[start..(start + glyph.width())]
                        .as_mut()
                        .copy_from_slice(&letter_bytes[glyph_start..(glyph_start + glyph.width())]);
                }

                //raster_letter.iter().enumerate().for_each(|(index, pixel)| {
                //    let x = (index as f64 / best_match.size.0 as f64).floor() as usize;
                //    let y = index % best_match.size.0;
                //
                //    let act_x = ((font_width * col_i) + x) as u32;
                //    let act_y = ((font_height * row_i) + y) as u32;
                //    if act_x < self.image.width() && act_y < self.image.height() {
                //        final_image.put_pixel(act_x, act_y, [*pixel;1].into());
                //    }
                //})
            }
        }
        self.asciified_image = Some(final_image);
        Ok(self)
    }

    pub fn save_to(&mut self, path: impl Into<PathBuf>) -> Result<&mut Self, AsciiError> {
        self.asciified_image
            .ok_or_ascii_err()?
            .save_with_format(path.into(), ImageFormat::Jpeg)
            .ascii_err()?;

        Ok(self)
    }

    fn best_char_match<'a>(
        &self,
        target_coverage: f64,
        chars: &'a [RasterizedChar],
    ) -> &'a RasterizedChar {
        //TODO implement the dist_type thingy
        chars
            .iter()
            .map(|char| char.match_coverage(target_coverage))
            .min_by(|match_a, match_b| match_a.partial_cmp(match_b).unwrap())
            .unwrap()
            .rasterized_char
    }
}
