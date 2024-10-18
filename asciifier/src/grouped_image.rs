use std::{sync::Arc, thread};

use image::{GenericImageView, ImageBuffer, Luma};

use crate::{asciifier::get_adjusted_size, Coverage};

#[derive(Debug, Clone)]
pub struct GroupedImage {
    pub group_width: usize,
    pub group_height: usize,
    pub groups: Vec<Vec<Coverage>>,
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

        let arc_image = Arc::new(image);

        let threads = (0..(adjusted_width / group_width))
            .map(|group_row_start| {
                let image = arc_image.clone();
                thread::spawn(move || {
                    (0..(adjusted_height / group_height))
                        .map(|group_col_start| {
                            let sub_image = image.view(
                                (group_row_start * group_width) as u32,
                                (group_col_start * group_height) as u32,
                                group_width as u32,
                                group_height as u32,
                            );
                            Coverage::new(sub_image)
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect::<Vec<_>>();

        grouped_image.groups = threads
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect();

        grouped_image
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
