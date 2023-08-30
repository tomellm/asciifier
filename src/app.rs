use image:: {ImageBuffer, Luma};
use ab_glyph::FontRef;

use crate::asciifier::{Asciifier, CharDistributionType};
use crate::basic::{get_image, convert_to_luminance};

fn get_img(img: Vec<u8>) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    //let chars: Vec<char> = vec!['a', '7', '#', 'Q', '?', '.', 'm', 'h', '@', '%', '+'];
    let chars = "^°<>|{}≠¿'][¢¶`.,:;-_#'+*?=)(/&%$§qwertzuiopasdfghjklyxcvbnmQWERTZUIOPASDFGHJKLYXCVBNM".chars().collect();
    //let chars = ".-_:,;?=#+*@".chars().collect();
    //let chars = ".,-*/+=%^1234567890".chars().collect();
    let f_height: usize = 12;

    let font = FontRef::try_from_slice(include_bytes!("../fonts/Hasklug-2.otf"))
        .expect("ERROR: The font parsing failed!");
    let asciifier = match Asciifier::new(chars, font, f_height, None) {
        Ok(a) => a,
        Err(err) => {
            panic!("There seems to be an error with the asciifier creation: {:?}", err);
        }
    };
    //let img = rotate90(&get_image("images/2.ektar 100.tif"));
    let img_r = get_image(img.clone());
    let img = match img_r {
        Ok(img) => img,
        Err(err) => panic!("There was an error {:?} with the image {:?}", err, img)
    };
    let luma_img = convert_to_luminance(&img);
    asciifier.convert(luma_img, CharDistributionType::ExactAdjustedBlacks)
}
