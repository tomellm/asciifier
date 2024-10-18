use std::{fs::File, io::BufReader, ops::Deref, path::PathBuf};

use ab_glyph::FontRef;
use image::{
    GenericImage, GenericImageView, GrayImage, ImageBuffer, ImageFormat, ImageReader, Luma, Pixel,
    Rgb,
};
use rgb::FromSlice;

use crate::{
    chars::{
        font_handler::{CharAlignment, CharDistributionType, CharacterBackground},
        Chars,
    },
    error::{AsciiError, IntoAsciiError, IntoConvertNotCalledResult},
    grouped_image::GroupedImage,
};

const DEFAULT_FONT: &[u8] = include_bytes!("../../assets/fonts/Hasklug-2.otf");
const DEFAULT_CHARS: &str =
    "^°<>|{}≠¿'][¢¶`.,:;-_#'+*?=)(/&%$§qwertzuiopasdfghjklyxcvbnmQWERTZUIOPASDFGHJKLYXCVBNM∇∕∑∏∇∆∃∫∬∮≋⊋⊂⊃⊞⊟⊠⊪⊩∸∷∶∶∵∴∾⊢⊯⊮⊭⊬⊫⊪⊩⊨⊧⊦⊥⊤⊣⊡";
//const DEFAULT_CHARS: &str = "∇∕∑∏∇∆∃∫∬∮≋⊋⊂⊃⊞⊟⊠⊪⊩∸∷∶∶∵∴∾⊢⊯⊮⊭⊬⊫⊪⊩⊨⊧⊦⊥⊤⊣⊡";

pub struct Asciifier {
    image: ImageBuffer<Rgb<u8>, Vec<u8>>,
}

impl Asciifier {
    pub fn load_image(path: impl Into<PathBuf>) -> Result<Self, AsciiError> {
        Ok(Self {
            image: ImageReader::open(path.into())?.decode()?.into(),
        })
    }

    pub fn load_image_with_format(
        path: impl Into<PathBuf>,
        format: ImageFormat,
    ) -> Result<Self, AsciiError> {
        let reader = BufReader::new(File::open(path.into()).ascii_err()?);
        let mut buffer = ImageReader::with_format(reader, format);
        buffer.set_format(format);
        Ok(Self {
            image: buffer.decode()?.into(),
        })
    }

    pub fn font<'font>(
        self,
        mut font_builder: impl FnMut(FontBuilder<'font>) -> Result<FontBuilder<'font>, AsciiError>,
    ) -> Result<ImageBuilder<'font>, AsciiError> {
        let mut builder = font_builder(FontBuilder::new()?)?;
        builder.build(self.image)
    }
}

#[derive(Debug, Clone)]
pub struct ImageBuilder<'font> {
    chars: Chars<'font>,
    image: ImageBuffer<Rgb<u8>, Vec<u8>>,
    asciified_image: Option<ImageBuffer<Luma<u8>, Vec<u8>>>,
}

impl<'font> ImageBuilder<'font> {
    pub fn char_height(&mut self, new_height: usize) -> Result<&mut Self, AsciiError> {
        self.chars.change_font_heigh(new_height)?;
        Ok(self)
    }

    pub fn distribution_type(&mut self, new_distribution: CharDistributionType) -> &mut Self {
        self.chars.change_distribution(new_distribution);
        self
    }

    fn convert_to_gray(&self) -> ImageBuffer<Luma<u8>, Vec<u8>> {
        let pixels = self
            .image
            .as_rgb()
            .iter()
            .map(|p| (0.299 * p.r as f32 + 0.587 * p.g as f32 + 0.114 * p.b as f32) as u8)
            .collect();
        GrayImage::from_raw(self.image.width(), self.image.height(), pixels).unwrap()
    }

    pub fn convert(&mut self) -> Result<&mut Self, AsciiError> {
        let (font_width, font_height) = self.chars.char_box();

        let image = self.convert_to_gray();

        let grouped_image = GroupedImage::new(font_width, font_height, image);

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

        for (row_i, group_row) in grouped_image.groups.iter().enumerate() {
            for (col_i, coverage) in group_row.iter().enumerate() {
                let rasterized_char = self.chars.best_match(coverage);

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
            .save(path.into())
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

pub struct FontBuilder<'font> {
    chars: Vec<char>,
    font: FontRef<'font>,
    font_height: usize,
    alignment: CharAlignment,
    distribution: CharDistributionType,
    background: CharacterBackground,
}

impl<'font> FontBuilder<'font> {
    pub fn new() -> Result<Self, AsciiError> {
        Ok(Self {
            chars: DEFAULT_CHARS.chars().collect(),
            font: FontRef::try_from_slice(DEFAULT_FONT).ascii_err()?,
            font_height: 12,
            alignment: CharAlignment::default(),
            distribution: CharDistributionType::default(),
            background: CharacterBackground::default(),
        })
    }

    pub fn font_height(&mut self, font_height: usize) -> &mut Self {
        self.font_height = font_height;
        self
    }

    pub fn alignment(&mut self, alignment: CharAlignment) -> &mut Self {
        self.alignment = alignment;
        self
    }

    pub fn distribution(&mut self, distribution: CharDistributionType) -> &mut Self {
        self.distribution = distribution;
        self
    }

    pub fn background(&mut self, background: CharacterBackground) -> &mut Self {
        self.background = background;
        self
    }

    pub fn build(
        &mut self,
        image: ImageBuffer<Rgb<u8>, Vec<u8>>,
    ) -> Result<ImageBuilder<'font>, AsciiError> {
        let FontBuilder {
            chars,
            font,
            font_height,
            alignment,
            distribution,
            background,
        } = self;
        let chars = Chars::new(
            chars.clone(),
            font.clone(),
            *font_height,
            *alignment,
            *distribution,
            *background,
        )?;
        Ok(ImageBuilder {
            chars,
            image,
            asciified_image: None,
        })
    }
}
