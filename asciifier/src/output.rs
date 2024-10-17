use crate::basic::*;

use image::{ImageBuffer, Rgb, Pixel};
use ansi_rgb::Foreground;
use rgb::RGB8;

pub fn print_to_terminal_pixel(img: &ImageBuffer<Rgb<u8>, Vec<u8>>, scale: u32) {
    let new_img = scale_down(img, scale);

    let new_dim = new_img.dimensions();

    //U+2588
    for x in 0..new_dim.0 {
        for y in 0..new_dim.1 {
            let r = new_img.get_pixel(x, y).to_rgb().0[0];
            let g = new_img.get_pixel(x, y).to_rgb().0[1];
            let b = new_img.get_pixel(x, y).to_rgb().0[2];
            let fg = RGB8::new(r, g, b);
            print!(
                "{}{}", 
                String::from('\u{2588}').fg(fg), 
                String::from('\u{2588}').fg(fg)
            );
        }
        println!();  
    }

 
}
