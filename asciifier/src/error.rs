use std::{fmt::Display, io};

use ab_glyph::{Glyph, InvalidFont, OutlinedGlyph};
use image::{flat, ImageBuffer, Rgb};

#[derive(Debug)]
pub enum AsciiError {
    FileError(io::Error),
    FontLoad(InvalidFont),
    FontParse(FontParseErrors),
    ImageError(ImageError),
    GroupedImage(GroupedImageError),
    ConvertNotCalled,
    ManyErrors(Vec<AsciiError>),
}

#[derive(Debug)]
pub enum FontParseErrors {
    GlyphOutlineMissing(Glyph),
    FontSizeTooSmall,
}

#[derive(Debug)]
pub enum GroupedImageError {
    RowIndexOutOfBounds { index: usize, row_len: usize },
}

#[derive(Debug)]
pub enum ImageError {
    Default(image::ImageError),
    Flat(flat::Error),
}

pub trait IntoAsciiError<V> {
    fn ascii_err(self) -> Result<V, AsciiError>;
}

impl<V, E: Into<AsciiError>> IntoAsciiError<V> for Result<V, E> {
    fn ascii_err(self) -> Result<V, AsciiError> {
        self.map_err(|err| err.into())
    }
}

impl From<io::Error> for AsciiError {
    fn from(value: io::Error) -> Self {
        Self::FileError(value)
    }
}

impl From<InvalidFont> for AsciiError {
    fn from(value: InvalidFont) -> Self {
        Self::FontLoad(value)
    }
}

impl From<image::ImageError> for AsciiError {
    fn from(value: image::ImageError) -> Self {
        Self::ImageError(ImageError::Default(value))
    }
}

impl From<flat::Error> for AsciiError {
    fn from(value: flat::Error) -> Self {
        Self::ImageError(ImageError::Flat(value))
    }
}

impl From<Vec<AsciiError>> for AsciiError {
    fn from(value: Vec<AsciiError>) -> Self {
        Self::ManyErrors(value)
    }
}

pub trait IntoConvertNotCalledResult {
    fn ok_or_ascii_err(&self) -> Result<&ImageBuffer<Rgb<u8>, Vec<u8>>, AsciiError>;
}

impl IntoConvertNotCalledResult for Option<ImageBuffer<Rgb<u8>, Vec<u8>>> {
    fn ok_or_ascii_err(&self) -> Result<&ImageBuffer<Rgb<u8>, Vec<u8>>, AsciiError> {
        self.as_ref().ok_or(AsciiError::ConvertNotCalled)
    }
}

pub trait IntoGlyphOutlineMissingResult {
    fn ok_or_ascii_err(self, glyph: &Glyph) -> Result<OutlinedGlyph, AsciiError>;
}

impl IntoGlyphOutlineMissingResult for Option<OutlinedGlyph> {
    fn ok_or_ascii_err(self, glyph: &Glyph) -> Result<OutlinedGlyph, AsciiError> {
        self.ok_or(AsciiError::FontParse(FontParseErrors::GlyphOutlineMissing(
            glyph.clone(),
        )))
    }
}

impl Display for AsciiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str: String = match self {
            Self::FileError(file_error) => file_error.to_string(),
            Self::FontLoad(invalid_font) => invalid_font.to_string(),
            Self::FontParse(font_parse_errors) => match font_parse_errors {
                FontParseErrors::GlyphOutlineMissing(glyph) => {
                    format!(
                        "Getting the glyph outline of glyph with id [{:?}] was not possible.",
                        glyph.id
                    )
                }
                FontParseErrors::FontSizeTooSmall => "The selected font size is too small.".into(),
            },
            Self::ImageError(image_errors) => match image_errors {
                ImageError::Default(default) => default.to_string(),
                ImageError::Flat(flat) => flat.to_string(),
            },
            Self::GroupedImage(groped_image_errors) => match groped_image_errors {
                GroupedImageError::RowIndexOutOfBounds { index, row_len } => {
                    format!("Grouping the image for asciification went out of bounds at index: [{index}] and with row len: [{row_len}]")
                }
            },
            Self::ConvertNotCalled => {
                "Convert was not called so there is no asciified Image.".into()
            }
            Self::ManyErrors(errors) => {
                // WARNING: could call some dangerous recursion
                let len = errors.len();
                let errors = errors
                    .iter()
                    .map(|error| error.to_string())
                    .collect::<Vec<_>>()
                    .join("\n\n");
                format!(
                    r#"A total of {len} errors occured:
                        
                        {errors}
                    "#
                )
            }
        };

        write!(f, "{str}")
    }
}
