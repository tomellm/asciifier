use image::Luma;

use crate::font_handler::CharRasterizingError;

#[derive(Debug, Clone)]
pub struct GroupedImage {
    pub group_width: usize,
    pub group_height: usize,
    pub groups: Vec<Vec<PixelGroup>>
}

impl GroupedImage {
    pub fn new(w: usize, h: usize) -> GroupedImage {
        GroupedImage { group_width: w, group_height: h, groups: vec![] }
    }

    pub fn push(&mut self, pixel: (u32, u32, &Luma<u8>)) -> Result<(), CharRasterizingError>{
        let w_index = (pixel.0 as f32 / self.group_width as f32).floor() as usize;
        let h_index = (pixel.1 as f32 / self.group_height as f32).floor() as usize;
        let row = match self.groups.get_mut(h_index) {
            Some(r) => r,
            None => {
                self.groups.insert(h_index, vec![]); 
                self.groups.get_mut(h_index).expect("This should be impossible!")
            }
        };
        match row.get_mut(w_index) {
            Some(c) => c.push(pixel)?,
            None => {
                row.insert(w_index, PixelGroup::new()); 
                row.get_mut(w_index).expect("This should not be possible!").push(pixel)?;
            }
        };
        Ok(())
    }

    pub fn num_rows(&self) -> usize {
        self.groups.len()
    }

    pub fn num_cols(&self) -> Option<usize> {
        Some(self.groups.get(0)?.len())
    }

    pub fn num_pixels(&self) -> usize {
        self.groups.iter().map(|r| {
            r.into_iter().map(PixelGroup::num_pixels).fold(0, |a, b| a + b)
        }).fold(0, |a, b| a + b)
    }

    pub fn num_groups(&self) -> usize {
        self.groups.iter().map(Vec::len).fold(0, |a, b| a + b)
    }
}

#[derive(Debug, Clone)]
pub struct PixelGroup {
    pixels: Vec<Pixel>,
    coverage: Option<f64>
}

impl PixelGroup {
    pub fn new() -> PixelGroup {
        PixelGroup { pixels: vec![], coverage: None }
    }

    pub fn push(&mut self, pixel: (u32, u32, &Luma<u8>)) -> Result<(), CharRasterizingError>{
        self.pixels.push(Pixel::new(pixel)?);
        Ok(())
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

    pub fn update_coverage(&mut self) -> f64{
        self.calc_coverage();
        self.coverage.expect("This is not possible")
    }

    fn calc_coverage(&mut self) {
        self.coverage = Some(self.pixels.iter().fold(0.0, |a, b| a + b.lum()) / self.pixels.len() as f64);
    }

    pub fn num_pixels(&self) -> usize {
        self.pixels.len()
    }


}

#[derive(Debug, Clone, Copy)]
struct Pixel {
    x: u32,
    y: u32,
    luma: u8
}

impl Pixel {
    pub fn new(pixel: (u32, u32, &Luma<u8>)) -> Result<Pixel, CharRasterizingError>{
        Ok(Pixel { 
            x: pixel.0, 
            y: pixel.1, 
            luma: match pixel.2.0.get(0) {
                Some(p) => Ok(p.clone()),
                None => Err(CharRasterizingError::CouldNotGetBufferPixelError(1))
            }? 
        })
    }

    pub fn lum(&self) -> f64{
        self.luma as f64 / 255.0
    }
}