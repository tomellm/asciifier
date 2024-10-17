use std::time::Instant;

use asciifier::{asciifier::Asciifier, error::AsciiError};

const IMAGE: &str = "assets/images/jojo-tom.jpg";
const SAVE_FILE: &str = "target/images/out.jpeg";

fn asciify_image() -> Result<(), AsciiError> {
    let start = Instant::now();
    Asciifier::load_image(IMAGE)?
        .char_height(30)
        .convert()?
        .save_to(SAVE_FILE)?;

    println!("time passed : {:?}", start.elapsed());
    Ok(())
}

fn main() {
    asciify_image().unwrap();
}
