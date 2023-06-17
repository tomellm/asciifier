
use image::{ImageBuffer, Rgb, RgbImage, Luma, GrayImage};


pub fn scale_down(
    img: &ImageBuffer<Rgb<u8>, Vec<u8>>, 
    scale: u32
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    
    let dim = img.dimensions();
    let mut new_img = RgbImage::new(dim.0 / scale + 1, dim.1 / scale + 1);

    for x in (0..dim.0).step_by(scale as usize) {
        for y in (0..dim.1).step_by(scale as usize) {
            new_img.put_pixel(x/scale, y/scale, img.get_pixel(x, y).clone());
        }
    }
    new_img
}

pub fn convert_to_luminance(
    img: &ImageBuffer<Rgb<u8>, Vec<u8>>
) -> ImageBuffer<Luma<u8>, Vec<u8>> {
  
    let img_dim = img.dimensions();
    let mut out = GrayImage::new(img_dim.0, img_dim.1);

    
    for x in 0..img_dim.0 {
        for y in 0..img_dim.1 {
            let pixel = img.get_pixel(x, y);
            out.put_pixel(x, y, Luma::from(
                    [(
                        0.299 * pixel.0[0] as f32 + 
                        0.587 * pixel.0[1] as f32 + 
                        0.114 * pixel.0[2] as f32 
                    ) as u8;1]
            ));
        }
    }

    //(0.299*R + 0.587*G + 0.114*B)
    out
}

pub fn get_image(path: &str) -> ImageBuffer<Rgb<u8>, Vec<u8>> { 
    image::open(path)
        .expect("ERROR: The image load failed!")
        .to_rgb8()
}
