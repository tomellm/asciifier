use std::time::Instant;

use asciifier::{asciifier::Asciifier, error::AsciiError};
use image::ImageFormat;

const IMAGE: &str = "assets/images/jojo-tom.jpg";
const SAVE_FILE: &str = "target/images/out.jpeg";

fn asciify_image() -> Result<(), AsciiError> {
    let start = Instant::now();
    Asciifier::load_image_with_format(IMAGE, ImageFormat::Jpeg)?
        .font(|mut builder| {
            builder.font_height(70);
            Ok(builder)
        })?
        .convert()?
        .save_to(SAVE_FILE)?;
    println!("final : {:?}", start.elapsed());
    Ok(())
}

fn main() {
    asciify_image().unwrap();
}
