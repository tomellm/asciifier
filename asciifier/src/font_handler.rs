use crate::chars::RasterizedChar;

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
    Even,
    Exact,
    #[default]
    ExactAdjustedBlacks,
    ExactAdjustedWhites,
}

impl CharDistributionType {
    pub(crate) fn adjust_coverage(&self, chars: &mut [RasterizedChar]) {
        println!("coverage has been adjusted");
        if matches!(self, CharDistributionType::Exact) {
            return chars
                .iter_mut()
                .for_each(|char| char.adjusted_coverage = char.actual_coverage);
        }
        if matches!(self, CharDistributionType::Even) {
            let increment = 1. / chars.len() as f64;
            chars.iter_mut().enumerate().for_each(|(index, rc)| {
                rc.adjusted_coverage = index as f64 * increment;
            });
            return;
        }
        let max = chars
            .iter()
            .max_by(|a, b| a.actual_coverage.partial_cmp(&b.actual_coverage).unwrap())
            .unwrap()
            .actual_coverage;
        if matches!(self, CharDistributionType::ExactAdjustedBlacks) {
            chars.iter_mut().for_each(|c| c.adjusted_coverage /= max);
            return;
        }
        let min = chars
            .iter()
            .min_by(|a, b| a.actual_coverage.partial_cmp(&b.actual_coverage).unwrap())
            .unwrap()
            .actual_coverage;

        chars
            .iter_mut()
            .for_each(|c| c.adjusted_coverage = (c.actual_coverage - min) / max);
    }
}
