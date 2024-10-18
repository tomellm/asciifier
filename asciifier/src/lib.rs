use image::{GenericImageView, ImageBuffer, Luma, SubImage};

pub mod asciifier;
pub mod chars;
pub mod error;
pub mod grouped_image;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Coverage {
    top_left_coverage: f64,
    top_right_coverage: f64,
    bottom_left_coverage: f64,
    bottom_right_coverage: f64,
}

impl Coverage {
    pub fn new(view: SubImage<&ImageBuffer<Luma<u8>, Vec<u8>>>) -> Self {
        let section_width = view.width() / 2;
        let section_height = view.height() / 2;

        let get_coverage = |view: SubImage<&ImageBuffer<Luma<u8>, Vec<u8>>>| -> f64 {
            let len = view.width() * view.height();
            view.pixels()
                .map(|(_, _, pixel)| pixel.0[0] as f64 / 255.)
                .sum::<f64>()
                / len as f64
        };

        Self {
            top_left_coverage: get_coverage(view.view(0, 0, section_width, section_height)),
            top_right_coverage: get_coverage(view.view(
                section_width,
                0,
                section_width,
                section_height,
            )),
            bottom_left_coverage: get_coverage(view.view(
                0,
                section_height,
                section_width,
                section_height,
            )),
            bottom_right_coverage: get_coverage(view.view(
                section_width,
                section_height,
                section_width,
                section_height,
            )),
        }
    }

    pub fn dist(&self, other: &Self) -> f64 {
        (dist(self.top_left_coverage, other.top_left_coverage)
            + dist(self.top_right_coverage, other.top_right_coverage)
            + dist(self.bottom_left_coverage, other.bottom_left_coverage)
            + dist(self.bottom_right_coverage, other.bottom_right_coverage))
            / 4.
    }

    pub fn from_func(&self, mut func: impl FnMut(f64) -> f64) -> Self {
        Self {
            top_left_coverage: func(self.top_left_coverage),
            top_right_coverage: func(self.top_right_coverage),
            bottom_left_coverage: func(self.bottom_left_coverage),
            bottom_right_coverage: func(self.bottom_right_coverage),
        }
    }

    pub fn max(&self) -> f64 {
        vec![
            self.top_left_coverage,
            self.top_right_coverage,
            self.bottom_left_coverage,
            self.bottom_right_coverage,
        ]
        .into_iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
    }

    pub fn min(&self) -> f64 {
        vec![
            self.top_left_coverage,
            self.top_right_coverage,
            self.bottom_left_coverage,
            self.bottom_right_coverage,
        ]
        .into_iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
    }
}

fn dist(left: f64, right: f64) -> f64 {
    (left - right).abs()
}
