mod basic;
mod output;
mod font_handler;
mod asciifier;
mod ascii_image;

use ab_glyph::FontRef;

use image::{ImageFormat, imageops::rotate90};
use asciifier::{Asciifier, CharDistributionType};
use basic::{get_image,convert_to_luminance};





fn main() {
    //let chars: Vec<char> = vec!['a', '7', '#', 'Q', '?', '.', 'm', 'h', '@', '%', '+'];
    //let chars = "^°<>|{}≠¿'][¢¶`.,:;-_#'+*?=)(/&%$§qwertzuiopasdfghjklyxcvbnmQWERTZUIOPASDFGHJKLYXCVBNM".chars().collect();
    //let chars = ".-_:,;?=#+*@".chars().collect();
    let chars = ".,-*/+=%^1234567890".chars().collect();
    let f_height: usize = 36;

    let font = FontRef::try_from_slice(include_bytes!("../fonts/Hasklug-2.otf"))
        .expect("ERROR: The font parsing failed!");

    let asciifier = match Asciifier::new(chars, font, f_height, None) {
        Ok(a) => a,
        Err(err) => {
            println!("There seems to be an error with the asciifier creation: {:?}", err);
            panic!("{:?}", err);
        }
    };

    //let img = rotate90(&get_image("images/2.ektar 100.tif")); 
    let img = &get_image("images/5.ektar 100.tif"); 
    let luma_img = convert_to_luminance(&img);


    let final_image = asciifier.convert(luma_img, CharDistributionType::ExactAdjustedBlacks);

    match final_image.save_with_format("images/test.png", ImageFormat::Png) {
        Ok(_) => println!("Image saved successfully!"),
        Err(err) => println!("Image saving failed because of {:?}", err)
    }
}

