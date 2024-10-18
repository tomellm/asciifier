use crate::chars::char::RasterizedChar;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CharAlignment {
    Left,
    #[default]
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum CharacterBackground {
    #[default]
    Black,
    White,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CharDistributionMatch<'a> {
    pub distance: f64,
    pub rasterized_char: &'a RasterizedChar,
}

impl<'a> PartialOrd for CharDistributionMatch<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.distance.partial_cmp(&other.distance)
    }
}

#[derive(Debug, Clone, Default, Copy)]
pub enum CharDistributionType {
    //Even,
    Exact,
    #[default]
    ExactAdjustedBlacks,
    ExactAdjustedWhites,
}

impl CharDistributionType {
    pub(crate) fn adjust_coverage(&self, chars: &mut [RasterizedChar]) {
        if matches!(self, CharDistributionType::Exact) {
            return chars
                .iter_mut()
                .for_each(|char| char.adjusted_coverage = char.coverage.clone());
        }
        //if matches!(self, CharDistributionType::Even) {
        //    let increment = 1. / chars.len() as f64;
        //    chars.iter_mut().enumerate().for_each(|(index, rc)| {
        //        rc.adjusted_coverage = index as f64 * increment;
        //    });
        //    return;
        //}
        let max = chars
            .iter()
            .map(|char| &char.coverage)
            .max_by(|a, b| a.max().partial_cmp(&b.max()).unwrap())
            .unwrap()
            .max();

        if matches!(self, CharDistributionType::ExactAdjustedBlacks) {
            chars
                .iter_mut()
                .for_each(|c| c.adjusted_coverage = c.coverage.from_func(|val| {
                    val / max
                }));
            return;
        }
        let min = chars
            .iter()
            .map(|char| &char.coverage)
            .min_by(|a, b| a.min().partial_cmp(&b.min()).unwrap())
            .unwrap()
            .min();

        chars
            .iter_mut()
            .for_each(|c| c.adjusted_coverage = c.coverage.from_func(|val| {
                (val - min) / max
            }));
    }
}
