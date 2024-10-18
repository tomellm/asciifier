pub mod char;
pub mod font_handler;

use ab_glyph::FontRef;
use char::{RasterizedChar, RasterizedCharBuilder};
use font_handler::{CharAlignment, CharDistributionType, CharacterBackground};

use crate::error::AsciiError;

#[derive(Debug, Clone)]
pub(crate) struct Chars<'font> {
    chars: Vec<char>,
    font: FontRef<'font>,
    font_height: usize,
    alignment: CharAlignment,
    distribution: CharDistributionType,
    background: CharacterBackground,
    char_box: (usize, usize),
    pub rasterized_chars: Vec<RasterizedChar>,
}

impl<'font> Chars<'font> {
    pub(crate) fn new(
        chars: Vec<char>,
        font: FontRef<'font>,
        font_height: usize,
        alignment: CharAlignment,
        distribution: CharDistributionType,
        background: CharacterBackground,
    ) -> Result<Self, AsciiError> {
        let (mut rasterized_chars, char_box) =
            Self::rasterize_chars(&chars, &font, font_height, alignment, background)?;

        distribution.adjust_coverage(&mut rasterized_chars);
        Ok(Self {
            chars,
            font,
            font_height,
            alignment,
            distribution,
            background,
            char_box,
            rasterized_chars,
        })
    }

    pub(crate) fn change_font_heigh(&mut self, font_height: usize) -> Result<(), AsciiError> {
        self.font_height = font_height;
        self.re_rasterize()?;
        Ok(())
    }

    pub(crate) fn change_distribution(&mut self, distribution: CharDistributionType) {
        self.distribution = distribution;
        self.distribution
            .adjust_coverage(&mut self.rasterized_chars);
    }

    pub(crate) fn best_match(&self, target_coverage: f64) -> &RasterizedChar {
        self.rasterized_chars
            .iter()
            .map(|char| char.match_coverage(target_coverage))
            .min_by(|match_a, match_b| match_a.partial_cmp(match_b).unwrap())
            .unwrap()
            .rasterized_char
    }

    fn re_rasterize(&mut self) -> Result<(), AsciiError> {
        let (rasterized_chars, char_box) = Self::rasterize_chars(
            &self.chars,
            &self.font,
            self.font_height,
            self.alignment,
            self.background,
        )?;
        self.rasterized_chars = rasterized_chars;
        self.char_box = char_box;
        self.distribution
            .adjust_coverage(&mut self.rasterized_chars);
        Ok(())
    }

    /// Returns the selected chars in rasterized form with the rectangle the rasterized
    /// squares will take u.
    ///
    /// # Arguments
    ///
    /// * `chars` - The list of chars to rasterize
    /// * `font` - The font to use for the rasterization
    /// * `(font_width, font_height)` - The wanted size of the font, main component
    ///     here is the height since, but width can also be determined id wanted. Although
    ///     its only considered if wider then minimum width.
    /// * `alignment` - since not each char is equally wide, this defines the chars
    ///     placement on the X axis
    /// * `character_bg` - color of the background TODO: what the hell is this actually
    fn rasterize_chars(
        chars: &[char],
        font: &FontRef<'_>,
        font_height: usize,
        alignment: CharAlignment,
        character_bg: CharacterBackground,
    ) -> Result<(Vec<RasterizedChar>, (usize, usize)), AsciiError> {
        let builders = chars
            .iter()
            .map(|c| RasterizedCharBuilder::new(*c, font_height, font, &alignment, &character_bg))
            .collect::<Vec<_>>();

        let font_box =
            RasterizedChar::char_boxing(font, builders.iter().map(|t| &t.glyph).collect());

        //let font_box = (
        //    possible_width,
        //    if font_height > possible_height {
        //        font_height
        //    } else {
        //        possible_height
        //    },
        //);

        let mut rasterized_chars = vec![];
        for builder in builders {
            let rasterized_char = builder.rasterize(font_box)?.build();
            rasterized_chars.push(rasterized_char);
        }

        Ok((rasterized_chars, font_box))
    }

    pub(crate) fn char_box(&self) -> (usize, usize) {
        self.char_box
    }
}
