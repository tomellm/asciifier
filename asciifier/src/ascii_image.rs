use image::{GenericImage, GenericImageView, ImageBuffer, Luma};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::asciifier::get_adjusted_size;

#[derive(Debug, Clone)]
pub struct GroupedImage {
    pub group_width: usize,
    pub group_height: usize,
    pub groups: Vec<Vec<PixelGroup>>,
}

impl GroupedImage {
    pub fn new(
        group_width: usize,
        group_height: usize,
        image: ImageBuffer<Luma<u8>, Vec<u8>>,
    ) -> GroupedImage {
        let (adjusted_width, adjusted_height) =
            get_adjusted_size(&image, &(group_width, group_height));

        let mut grouped_image = GroupedImage {
            group_width,
            group_height,
            groups: vec![],
        };

        grouped_image.groups = (0..(adjusted_width / group_width))
            .into_par_iter()
            .map(|group_row_start| {
                (0..(adjusted_height / group_height))
                    .into_par_iter()
                    .map(|group_col_start| {
                        let sub_image = image.view(
                            (group_row_start * group_width) as u32,
                            (group_col_start * group_height) as u32,
                            group_width as u32,
                            group_height as u32,
                        );
                        let group_pixels = sub_image
                            .pixels()
                            .map(|(_, _, luma)| Pixel::new(&luma))
                            .collect::<Vec<_>>();
                        PixelGroup::new(group_pixels)
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        grouped_image
    }

    pub fn num_rows(&self) -> usize {
        self.groups.len()
    }

    pub fn num_cols(&self) -> Option<usize> {
        Some(self.groups.first()?.len())
    }

    pub fn num_pixels(&self) -> usize {
        self.groups
            .iter()
            .map(|r| r.iter().map(PixelGroup::num_pixels).sum::<usize>())
            .sum::<usize>()
    }

    pub fn num_groups(&self) -> usize {
        self.groups.iter().map(Vec::len).sum::<usize>()
    }
}

#[derive(Debug, Clone, Default)]
pub struct PixelGroup {
    pixels: Vec<Pixel>,
    coverage: f64,
}

impl PixelGroup {
    fn new(pixels: Vec<Pixel>) -> Self {
        let coverage = pixels.iter().fold(0f64, |a, b| a + b.cov()) / pixels.len() as f64;

        Self { pixels, coverage }
    }

    pub fn coverage(&mut self) -> f64 {
        self.coverage
    }

    pub fn num_pixels(&self) -> usize {
        self.pixels.len()
    }
}

#[derive(Debug, Clone, Copy)]
struct Pixel {
    luma: u8,
}

impl Pixel {
    pub fn new(pixel: &Luma<u8>) -> Pixel {
        Pixel { luma: pixel.0[0] }
    }

    /// How covered the pixel is from 0. - 1.
    pub fn cov(&self) -> f64 {
        self.luma as f64 / 255.0
    }
}
