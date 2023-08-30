use image:: { ImageBuffer, GrayImage, Luma};
use ab_glyph::{FontRef, Font, Glyph, Point, PxScale};

pub fn rasterize_chars(
    chars: Vec<char>,
    font: FontRef<'_>, 
    font_size:(Option<usize>, usize),
    alignment: Option<CharAlignment>,
    character_bg: Option<CharacterBackground>
) -> Result<(Vec<RasterizedChar>, (usize, usize)), CharRasterizingError>{

    let alignment = match alignment {
        Some(a) => a,
        None => CharAlignment::Center
    };

    let character_bg = match character_bg {
        Some(b) => b,
        None => CharacterBackground::Black
    };

    let glyphs: Vec<(char, Glyph)> = chars.into_iter()
        .map(|c| (c, font.glyph_id(c).with_scale(PxScale{x: font_size.1 as f32, y: font_size.1 as f32})))
        .collect();

    let possible_size = char_boxing( &font, glyphs.iter().map(|t| &t.1).collect());

    let font_box = (
        match font_size.0 {
            Some(x) => if x > possible_size.0 { x } else { possible_size.0 },
            None => possible_size.0
        },
        if font_size.1 > possible_size.1 { font_size.1 } else { possible_size.1 }
    );

    let rasterization_result: (Vec<_>, Vec<_>)= glyphs.into_iter().map(|t| {
        let buffer = glyph_to_buffer(
            &t.1, 
            &font, 
            font_box, 
            alignment,
            character_bg
        );
        RasterizedChar::new(t.0, t.1, buffer?, font_box, alignment)
    }).partition(Result::is_ok);
    
    let mut c_err: Vec<CharRasterizingError> = vec![]; 
    let rasterized_chars: Vec<RasterizedChar> = rasterization_result.0
        .into_iter()
        .filter_map(|r| r.map_err(|e| c_err.push(e)).ok())
        .collect();
    if c_err.len() > 0 {
        return Err(CharRasterizingError::RasterizedCharIsError);
    }

    let mut e_err: Vec<RasterizedChar> = vec![];
    let rasterization_errors: Vec<CharRasterizingError> = rasterization_result.1
        .into_iter()
        .filter_map(|r: Result<RasterizedChar, CharRasterizingError>| r.map(|c| e_err.push(c)).err())
        .collect();
    if e_err.len() > 0 {
        return Err(CharRasterizingError::GlyphErrorsIsNotError(e_err))
    }

    match rasterization_errors.len() {
        //FIXME: there can still be erros in the safe list so check needs to be done
        0 => Ok((rasterized_chars, font_box)),
        _ => {
            let out: Vec<Glyph> = rasterization_errors.into_iter().map(|e| {
                match e {
                    CharRasterizingError::CouldNotGetGlyphOutlineError(g) => Ok(g),
                    _ => Err(CharRasterizingError::RasterizationProducedManyErrorsError)
                }
            }).collect::<Result<Vec<Glyph>, CharRasterizingError>>()?;
            Err(CharRasterizingError::CouldNotGetManyGlyphsOutlineError(out))
        }

    }
}

fn char_boxing(
    font: &FontRef<'_>, 
    glyphs: Vec<&Glyph>
) -> (usize, usize) {

    let out = glyphs.into_iter().map(|g| {
        let rect = font.outline_glyph(g.clone()).expect("The outline of the glyph could not be found!").px_bounds();
        let (_, b) = normalize_bounds(rect.min, rect.max);
        (b.x.ceil(), b.y.ceil())
    }).fold((0.0, 0.0), |a, b| {
        let mut fin = (0.0, 0.0);
        fin.0 = if b.0 > a.0 {  b.0 } else { a.0 }; 
        fin.1 = if b.1 > a.1 {  b.1 } else { a.1 };
        fin 
    });

    (out.0.ceil() as usize, out.1.ceil() as usize)
}

fn glyph_to_buffer(
    glyph: &Glyph,
    font: &FontRef<'_>, 
    bounding_box: (usize, usize),
    alignment: CharAlignment,
    character_bg: CharacterBackground
) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, CharRasterizingError>{

    // Draw it.
    let q = match font.outline_glyph(glyph.clone()) {
        Some(q) => q,
        None => return Err(CharRasterizingError::CouldNotGetGlyphOutlineError(glyph.clone()))
    };

    let char_bounds = q.px_bounds();
    let (_, char_max) = normalize_bounds(char_bounds.min, char_bounds.max);


    let mut letter = GrayImage::new(
        (bounding_box.0) as u32,
        (bounding_box.1) as u32
    );
    q.draw(|x, y, c| {
        let cov = ( match character_bg {
            CharacterBackground::Black => c,
            CharacterBackground::White => 1f32 / c
        } * 255f32) as u8;
        let x = match alignment {
            CharAlignment::Left => x,
            CharAlignment::Center => x + ((bounding_box.0 as u32 - char_max.x as u32) / 2),
            CharAlignment::Right => x + (bounding_box.0 as f32 - char_max.x) as u32
        };
        let y = y + (bounding_box.1 as f32 - char_max.y) as u32;
        if x < letter.width() && y < letter.height() {
            letter.put_pixel(x, y, Luma::from([cov;1]));            
        }
    });
    
    Ok(letter)
}


fn normalize_bounds(
    mut min: Point, 
    mut max: Point
) -> (Point, Point) {
    if min.x != 0f32 {
        let distance = 0f32 - min.x;
        min.x += distance;
        max.x += distance;
    }
    if min.y != 0f32 {
        let distance = 0f32 - min.y;
        min.y += distance;
        max.y += distance;
    }

    (min, max)
}


#[derive(Debug, Clone)]
pub enum CharRasterizingError {
    /// General error only for very special cases
    DefaultRasterizingError,
    /// Error while trying to determine the max width of the char array
    CouldNotFindMaxWidthError,
    /// Error while getting the outline of a `Glyph`
    CouldNotGetGlyphOutlineError(Glyph),
    /// Failed to get the outline of Multiple `Glyph`s
    CouldNotGetManyGlyphsOutlineError(Vec<Glyph>),
    /// A apparent sucessful Rasterization is actually an Error
    RasterizedCharIsError,
    /// A apparent errors are actually successful rasterizations
    GlyphErrorsIsNotError(Vec<RasterizedChar>),
    /// When the rasterization process retruns a number of errors but they are not all part of type `CouldNotGetGlyphOutlineError(Glyph)`
    RasterizationProducedManyErrorsError,
    /// Failed to get the value of a `Pixel` while calculating the coverage
    CouldNotGetBufferPixelError(u64)

}

#[derive(Debug, Clone, Copy)]
pub enum CharAlignment {
    Left,
    Center,
    Right
}

#[derive(Debug, Clone)]
pub struct RasterizedChar {
    pub character: char,
    pub glyph: Glyph,
    pub raster_letter: ImageBuffer<Luma<u8>, Vec<u8>>,
    pub size: (usize, usize),
    pub alignment: CharAlignment,
    pub coverage: f64
}

impl RasterizedChar {
    fn new(
        character: char,
        glyph: Glyph,
        raster_letter: ImageBuffer<Luma<u8>, Vec<u8>>,
        size: (usize, usize),
        alignment: CharAlignment
    ) -> Result<RasterizedChar, CharRasterizingError> {
        let num_pixels = raster_letter.width() * raster_letter.height();
        let mut p_err: Vec<CharRasterizingError> = vec![];
        let converage = raster_letter
            .pixels()
            .filter_map(|p| match p.0.get(0) {
                Some(p) => Some(p),
                None => {
                    p_err.push(CharRasterizingError::CouldNotGetBufferPixelError(1));
                    None
                }
            })
            .fold(0.0, |a, b| a + *b as f64)  / num_pixels as f64 / 255f64;
        
        if p_err.len() > 0 {
            return Err(CharRasterizingError::CouldNotGetBufferPixelError(p_err.len() as u64));
        }
        Ok(RasterizedChar { character: character, glyph: glyph, raster_letter: raster_letter, size: size, alignment: alignment, coverage: converage})
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CharacterBackground {
    Black, 
    White
}