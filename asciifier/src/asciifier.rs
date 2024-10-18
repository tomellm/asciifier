use std::{fs::File, io::Read, ops::Deref, path::PathBuf};

use ab_glyph::FontRef;
use image::{
    GenericImage, GenericImageView, GrayImage, ImageBuffer, ImageFormat, Luma, Pixel, Rgb,
};

use crate::{
    ascii_image::GroupedImage,
    basic::convert_to_luminance,
    chars::Chars,
    error::{AsciiError, IntoAsciiError, IntoConvertNotCalledResult},
    font_handler::CharDistributionType,
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
    chars: Chars<'font>,
    image: ImageBuffer<Rgb<u8>, Vec<u8>>,
    asciified_image: Option<ImageBuffer<Luma<u8>, Vec<u8>>>,
}

impl<'font> ImageBuilder<'font> {
    fn default_from_image(buffer: ImageBuffer<Rgb<u8>, Vec<u8>>) -> Result<Self, AsciiError> {
        let chars = "^°<>|{}≠¿'][¢¶`.,:;-_#'+*?=)(/&%$§qwertzuiopasdfghjklyxcvbnmQWERTZUIOPASDFGHJKLYXCVBNM".chars().collect();
        let font = FontRef::try_from_slice(include_bytes!("../../assets/fonts/Hasklug-2.otf"))
            .ascii_err()?;

        Self::new(chars, font, buffer)
    }
    fn new(
        chars: Vec<char>,
        font: FontRef<'font>,
        image: ImageBuffer<Rgb<u8>, Vec<u8>>,
    ) -> Result<Self, AsciiError> {
        let chars = Chars::new(chars, font)?;
        Ok(Self {
            chars,
            image,
            asciified_image: None,
        })
    }

    pub fn char_height(&mut self, new_height: usize) -> Result<&mut Self, AsciiError> {
        self.chars.change_font_heigh(new_height)?;
        Ok(self)
    }

    pub fn distribution_type(
        &mut self,
        new_distribution: CharDistributionType,
    ) -> &mut Self {
        self.chars.change_distribution(new_distribution);
        self
    }

    pub fn convert(&mut self) -> Result<&mut Self, AsciiError> {

        let (font_width, font_height) = self.chars.char_box();

        // TODO: how should I handle the luminance situation
        let image = convert_to_luminance(&self.image);
        let mut grouped_image = GroupedImage::new(font_width, font_height, image);

        let (adjusted_width, adjusted_height) =
            get_adjusted_size(&self.image, &(font_width, font_height));

        assert_eq!(
            adjusted_width as f64 / font_width as f64,
            grouped_image.num_rows() as f64
        );
        assert_eq!(
            adjusted_height as f64 / font_height as f64,
            grouped_image.num_cols().unwrap() as f64
        );

        let mut final_image = GrayImage::new(adjusted_width as u32, adjusted_height as u32);

        for (row_i, group_row) in grouped_image.groups.iter_mut().enumerate() {
            for (col_i, group) in group_row.iter_mut().enumerate() {
                let rasterized_char = self.chars.best_match(group.coverage());

                let start_glyph_x = (font_width * row_i) as u32;
                let start_glyph_y = (font_height * col_i) as u32;

                let mut sub_image = final_image.sub_image(
                    start_glyph_x,
                    start_glyph_y,
                    font_width as u32,
                    font_height as u32,
                );
                assert_eq!(sub_image.width(), rasterized_char.raster_letter.width());
                assert_eq!(sub_image.height(), rasterized_char.raster_letter.height());
                sub_image.copy_from(&rasterized_char.raster_letter, 0, 0)?
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
}

pub fn get_adjusted_size<P, Container>(
    image: &ImageBuffer<P, Container>,
    (char_width, char_height): &(usize, usize),
) -> (usize, usize)
where
    // Bounds from impl:
    P: Pixel,
    Container: Deref<Target = [P::Subpixel]>,
{
    let groups_width = (image.width() as f64 / *char_width as f64).floor();
    let adjusted_width = groups_width as usize * char_width;

    let groups_height = (image.height() as f64 / *char_height as f64).floor();
    let adjusted_height = groups_height as usize * char_height;

    (adjusted_width, adjusted_height)
}
