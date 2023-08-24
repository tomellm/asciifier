use leptos::*;
use ab_glyph::FontRef;
use image::{ImageBuffer, Luma, GrayImage};

use crate::{font_handler::{RasterizedChar, rasterize_chars, CharAlignment, CharRasterizingError, CharacterBackground}, ascii_image::GroupedImage};



#[derive(Debug, Clone)]
pub struct CharDistributionMatcher<'a> {
    pub distance: f64,
    pub rasterized_char: Option<&'a RasterizedChar>,
}

#[derive(Debug, Clone)]
pub enum CharDistributionType {
    Even,
    Exact,
    ExactAdjustedBlacks,
    ExactAdjustedWhites,
}

#[derive(Debug, Clone)]
pub struct Asciifier {
    pub rasterized_chars: Vec<RasterizedChar>,
    pub f_size: (usize, usize)
}


impl Asciifier {
    pub fn new(
        chars: Vec<char>,
        font: FontRef<'_>, 
        char_height: usize,
        char_alignment: Option<CharAlignment>,
    ) -> Result<Asciifier, CharRasterizingError> {
        let mut rasterized_chars_r = rasterize_chars(chars, font, (None, char_height), char_alignment, Some(CharacterBackground::Black))?;
        
        Ok(Asciifier { rasterized_chars: rasterized_chars_r.0, f_size: (rasterized_chars_r.1.0, rasterized_chars_r.1.1) })
    }

    pub fn convert(&self, image: ImageBuffer<Luma<u8>, Vec<u8>>, char_distrubution: CharDistributionType) -> ImageBuffer<Luma<u8>, Vec<u8>> {
        let mut grouped_image = GroupedImage::new(
            self.f_size.0, 
            self.f_size.1
        );
        image.enumerate_pixels().for_each(|p| { 
            match grouped_image.push(p) {
                Err(err) => println!("A error happend while trying to add pixels to the grouped Matrix: {:?}", err),
                Ok(_) => (),
            }
        });
        let mut final_image = GrayImage::new(
            image.width(), 
            image.height()
        );

        let mut chars: Vec<RasterizedChar>= self.rasterized_chars.clone();
        chars.sort_by(|a, b| { a.coverage.partial_cmp(&b.coverage).expect("There has been an issues with the ordering!")});
        match char_distrubution {
            CharDistributionType::Even => {
                let inc = 1 / chars.len();
                for (i, rc) in chars.iter_mut().enumerate() {
                    rc.coverage = i as f64 * inc as f64;
                }
            },
            CharDistributionType::Exact => {
                ();
            },
            CharDistributionType::ExactAdjustedBlacks => {
                let max_v = chars.last().expect("There should be at least one char in the list!").coverage;
                chars.iter_mut().for_each(|c| c.coverage = c.coverage / max_v);
            },
            CharDistributionType::ExactAdjustedWhites => {
                let min_v = chars.first().expect("There should be at least one char in the list!").coverage;
                let max_v = chars.last().expect("There should be at least one char in the list!").coverage;
                chars.iter_mut().for_each(|c| c.coverage = (c.coverage - min_v) / max_v);
            }
        };
        for (row_i, group_row) in grouped_image.groups.iter_mut().enumerate() {
            for (col_i, group_col) in group_row.iter_mut().enumerate() {

                let r_char = self.best_char_match(
                    group_col.coverage(),
                    &chars
                ).raster_letter;
                r_char.enumerate_pixels().for_each(|(x, y, pixel)| {
                    let act_x = (self.f_size.0 * col_i) as u32 + x;
                    let act_y = (self.f_size.1 * row_i) as u32 + y;
                    if act_x < image.width() && act_y < image.height() {
                        final_image.put_pixel(act_x, act_y, pixel.clone());
                    }    
                })
            }
        };
        final_image
    }

    fn best_char_match(&self, target_cov: f64, chars: &Vec<RasterizedChar>) -> RasterizedChar {
        //TODO implement the dist_type thingy
        chars.into_iter()
            .map(|c| CharDistributionMatcher{
                distance: (target_cov - c.coverage).abs(), 
                rasterized_char: Some(&c)
            })
            .fold(
                CharDistributionMatcher { distance: f64::INFINITY, rasterized_char: None }, 
                |a, b| 
                    if a.distance < b.distance { a } else { b }
            )
            .rasterized_char.expect("There should always be a char bc of Infinity!").clone()    
    }  
}
