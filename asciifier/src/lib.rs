use error::{AsciiError, FontParseErrors};
use image::{GenericImageView, ImageBuffer, Luma, SubImage};

pub mod asciifier;
pub mod chars;
pub mod error;
pub mod grouped_image;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Coverage {
    squares: [f64; 16],
}

impl Coverage {
    pub fn new(view: SubImage<&ImageBuffer<Luma<u8>, Vec<u8>>>) -> Result<Self, AsciiError> {
        if view.width() < 4 || view.height() < 4 {
            return Err(AsciiError::FontParse(FontParseErrors::FontSizeTooSmall))
        }


        let section_width = view.width() / 4;
        let section_height = view.height() / 4;

        let get_coverage = |view: SubImage<&ImageBuffer<Luma<u8>, Vec<u8>>>| -> f64 {
            let len = view.width() * view.height();
            view.pixels()
                .map(|(_, _, pixel)| pixel.0[0] as f64 / 255.)
                .sum::<f64>()
                / len as f64
        };

        let mut squares = [0.; 16];

        for row in 0..4 {
            for col in 0..4 {
                let pos = (row * 4) + col;
                squares[pos] = get_coverage(view.view(
                    row as u32 * section_width,
                    col as u32 * section_height,
                    section_width,
                    section_height,
                ))
            }
        }

        Ok(Self { squares })
    }

    pub fn dist(&self, other: &Self) -> f64 {
        (0..16)
            .map(|i| (self.squares[i] - other.squares[i]).powi(2))
            .sum::<f64>()
            .sqrt()
    }

    pub fn from_func(&self, mut func: impl FnMut(f64) -> f64) -> Self {
        let mut squares = [0.; 16];
        (0..16).for_each(|i| squares[i] = func(self.squares[i]));
        Self { squares }
    }

    pub fn avg(&self) -> f64 {
        self.squares.iter().sum::<f64>() / 16.
    }

    pub fn max(&self) -> f64 {
        *self
            .squares
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }

    pub fn min(&self) -> f64 {
        *self
            .squares
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }
}
