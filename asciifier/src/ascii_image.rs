use image::Luma;

use crate::error::AsciiError;

#[derive(Debug, Clone)]
pub struct GroupedImage {
    pub group_width: usize,
    pub group_height: usize,
    pub groups: Vec<Vec<PixelGroup>>,
}

impl GroupedImage {
    pub fn new(w: usize, h: usize) -> GroupedImage {
        GroupedImage {
            group_width: w,
            group_height: h,
            groups: vec![],
        }
    }

    pub fn push(&mut self, pixel: (u32, u32, &Luma<u8>)) {
        let w_index = (pixel.0 as f32 / self.group_width as f32).floor() as usize;
        let h_index = (pixel.1 as f32 / self.group_height as f32).floor() as usize;
        let row = if let Some(row) = self.groups.get_mut(h_index) {
            row
        } else {
            self.groups.insert(h_index, vec![]);
            self.groups.get_mut(h_index).unwrap()
        };

        match row.get_mut(w_index) {
            Some(c) => c.push(pixel),
            None => {
                row.insert(w_index, PixelGroup::default());
                row.get_mut(w_index)
                    .expect("This should not be possible!")
                    .push(pixel);
            }
        };
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
    coverage: Option<f64>,
}

impl PixelGroup {
    pub fn push(&mut self, pixel: (u32, u32, &Luma<u8>)) {
        self.pixels.push(Pixel::new(pixel));
    }

    pub fn coverage(&mut self) -> f64 {
        match self.coverage {
            Some(c) => c,
            None => {
                self.calc_coverage();
                self.coverage.expect("This is not possible")
            }
        }
    }

    pub fn update_coverage(&mut self) -> f64 {
        self.calc_coverage();
        self.coverage.expect("This is not possible")
    }

    fn calc_coverage(&mut self) {
        self.coverage =
            Some(self.pixels.iter().fold(0.0, |a, b| a + b.lum()) / self.pixels.len() as f64);
    }

    pub fn num_pixels(&self) -> usize {
        self.pixels.len()
    }
}

#[derive(Debug, Clone, Copy)]
struct Pixel {
    x: u32,
    y: u32,
    luma: u8,
}

impl Pixel {
    pub fn new(pixel: (u32, u32, &Luma<u8>)) -> Pixel {
        Pixel {
            x: pixel.0,
            y: pixel.1,
            luma: pixel.2 .0[0],
        }
    }

    pub fn lum(&self) -> f64 {
        self.luma as f64 / 255.0
    }
}

