use std::{sync::Arc, thread};

use image::{GenericImageView, ImageBuffer, Rgb, SubImage};

use crate::{
    asciifier::{convert_to_gray, get_adjusted_size},
    error::AsciiError,
    Coverage,
};

#[derive(Debug, Clone)]
pub struct GroupedImage {
    pub group_width: usize,
    pub group_height: usize,
    pub(crate) groups: Vec<Vec<PixelGroup>>,
}

impl GroupedImage {
    pub fn new(
        group_width: usize,
        group_height: usize,
        image: ImageBuffer<Rgb<u8>, Vec<u8>>,
    ) -> Result<GroupedImage, AsciiError> {
        let (adjusted_width, adjusted_height) =
            get_adjusted_size(&image, &(group_width, group_height));

        let mut grouped_image = GroupedImage {
            group_width,
            group_height,
            groups: vec![],
        };

        let arc_image = Arc::new(image);

        let threads = (0..(adjusted_width / group_width))
            .map(|group_row_start| {
                let image = arc_image.clone();
                thread::spawn(move || {
                    let mut row = vec![];
                    for group_col_start in 0..(adjusted_height / group_height) {
                        let sub_image = image.view(
                            (group_row_start * group_width) as u32,
                            (group_col_start * group_height) as u32,
                            group_width as u32,
                            group_height as u32,
                        );
                        row.push(PixelGroup::new(sub_image)?);
                    }
                    Ok::<Vec<PixelGroup>, AsciiError>(row)
                })
            })
            .collect::<Vec<_>>();

        for handle in threads {
            let row = handle.join().unwrap()?;
            grouped_image.groups.push(row)
        }

        Ok(grouped_image)
    }

    pub fn num_rows(&self) -> usize {
        self.groups.len()
    }

    pub fn num_cols(&self) -> Option<usize> {
        Some(self.groups.first()?.len())
    }

    pub fn num_groups(&self) -> usize {
        self.groups.iter().map(Vec::len).sum::<usize>()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PixelGroup {
    pub color: Rgb<u8>,
    pub coverage: Coverage,
}

impl PixelGroup {
    pub(crate) fn new(image: SubImage<&ImageBuffer<Rgb<u8>, Vec<u8>>>) -> Result<Self, AsciiError> {
        let gray_image = convert_to_gray(&image.to_image());
        let coverage =
            Coverage::new(gray_image.view(0, 0, gray_image.width(), gray_image.height()))?;

        let len = (image.width() * image.height()) as f64;
        let (r, g, b) = image
            .pixels()
            .map(|(_, _, pixel)| [pixel.0[0] as f64, pixel.0[1] as f64, pixel.0[2] as f64])
            .fold((0f64, 0f64, 0f64), |(r_sum, g_sum, b_sum), [r, g, b]| {
                (r_sum + r, g_sum + g, b_sum + b)
            });
        let r = (r / len) as u8;
        let g = (g / len) as u8;
        let b = (b / len) as u8;

        let max = Ord::max(Ord::max(r, g), b);

        let add = ((255 - max) as f64 * (1. - coverage.avg())) as u8;
        let color = [r + add, g + add, b + add].into();

        Ok(Self { color, coverage })
    }
}
