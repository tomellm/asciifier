use std::time::Instant;

use asciifier::{asciifier::Asciifier, error::AsciiError};

const IMAGE: &str = "assets/images/jojo-tom.jpg";
const SAVE_FILE: &str = "target/images/out.jpeg";

fn asciify_image() -> Result<(), AsciiError> {
    let start = Instant::now();
    let a = Asciifier::load_image(IMAGE)?;
    println!("image load : {:?}", start.elapsed());
    let mut a = a.font(|mut builder| {
        builder.font_height(30);
        Ok(builder)
    })?;
    println!("font set : {:?}", start.elapsed());
    a.convert()?;
    println!("convert : {:?}", start.elapsed());
    a.save_to(SAVE_FILE)?;
    println!("save : {:?}", start.elapsed());
    Ok(())
}

fn main() {
    asciify_image().unwrap();
}
