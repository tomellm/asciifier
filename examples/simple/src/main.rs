use std::time::Instant;

use asciifier::{asciifier::Asciifier, error::AsciiError};

const IMAGE: &str = "assets/images/matti.jpg";
const SAVE_FILE: &str = "target/images/out.jpeg";

fn asciify_image() -> Result<(), AsciiError> {
    let start = Instant::now();
    Asciifier::load_image(IMAGE)?
        .font(|mut builder| {
            builder.font_height(100);
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
