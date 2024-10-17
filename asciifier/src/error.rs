use std::io;

use ab_glyph::{Glyph, InvalidFont, OutlinedGlyph};
use image::{flat, ImageBuffer, Luma};


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
}

#[derive(Debug)]
pub enum GroupedImageError {
    RowIndexOutOfBounds{
        index: usize,
        row_len: usize
    },
}

#[derive(Debug)]
pub enum ImageError {
    Default(image::ImageError),
    Flat(flat::Error)
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
    fn ok_or_ascii_err(&self) -> Result<&ImageBuffer<Luma<u8>, Vec<u8>>, AsciiError>;
}

impl IntoConvertNotCalledResult for Option<ImageBuffer<Luma<u8>, Vec<u8>>> {
    fn ok_or_ascii_err(&self) -> Result<&ImageBuffer<Luma<u8>, Vec<u8>>, AsciiError> {
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


