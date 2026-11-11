// SPDX-License-Identifier: MIT OR Apache-2.0

//! The values a part can be bought in (B-08, design doc §6.2).
//!
//! A computed 217 Ω is not a resistor anybody sells; 220 Ω is. The E-series
//! are the IEC 60063 tables every maker's catalogue follows: each decade cut
//! into 6, 12 or 24 steps of near-equal ratio, so the next value up is always
//! about the same percentage away. Resistors come in E12 and E24 (an
//! assortment box is E12), capacitors in E6 and E12.
//!
//! Each step is written as its two significant figures, 10 to 91, and a value
//! built from it by an exact power of ten, so 4.7 kΩ is 4700 and not
//! 4700.000000000001 — which is what an editable field would otherwise show.

/// One of the E-series.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Series {
    E6,
    E12,
    E24,
}

/// What resistors are sold in, the commoner first.
pub const RESISTORS: [Series; 2] = [Series::E12, Series::E24];

/// What capacitors are sold in, the commoner first.
pub const CAPACITORS: [Series; 2] = [Series::E6, Series::E12];

const E6: [u8; 6] = [10, 15, 22, 33, 47, 68];
const E12: [u8; 12] = [10, 12, 15, 18, 22, 27, 33, 39, 47, 56, 68, 82];
const E24: [u8; 24] = [
    10, 11, 12, 13, 15, 16, 18, 20, 22, 24, 27, 30, 33, 36, 39, 43, 47, 51, 56, 62, 68, 75, 82, 91,
];

/// Two values this close are the same value: rounding, not a difference.
const SAME: f64 = 1e-9;

impl Series {
    /// Its steps in one decade, as two significant figures.
    pub fn steps(self) -> &'static [u8] {
        match self {
            Series::E6 => &E6,
            Series::E12 => &E12,
            Series::E24 => &E24,
        }
    }

    /// What it is called: "E12".
    pub fn name(self) -> &'static str {
        match self {
            Series::E6 => "E6",
            Series::E12 => "E12",
            Series::E24 => "E24",
        }
    }

    /// Its values from the decade below `value`'s to the one above, in
    /// order: everything the questions below can answer with.
    fn around(self, value: f64) -> Vec<f64> {
        let decade = value.log10().floor() as i32;
        (decade - 1..=decade + 1)
            .flat_map(|decade| {
                self.steps()
                    .iter()
                    .map(move |&step| scaled(step, decade - 1))
            })
            .chain(std::iter::once(scaled(10, decade + 1)))
            .collect()
    }
}

/// `step` times ten to the `exponent`, divided rather than multiplied for a
/// negative one, which is the exact way round: 47 / 10⁷ is the double nearest
/// 4.7 µ, and 47 × 10⁻⁷ is not.
fn scaled(step: u8, exponent: i32) -> f64 {
    if exponent >= 0 {
        f64::from(step) * 10f64.powi(exponent)
    } else {
        f64::from(step) / 10f64.powi(-exponent)
    }
}

/// A value that can be bought, and how far it is from the one asked for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pick {
    pub value: f64,
    /// Its difference from the value asked for, as a percentage of that
    /// value: positive when the part bought is larger.
    pub error_percent: f64,
}

/// Whether there is anything to look up: a positive, finite value.
fn usable(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

/// The value in `series` nearest `value`, and the percentage error: 217 Ω is
/// 220 Ω in E12, 1.4 % high. Nearest by that percentage, which is the number
/// it is reported with. `None` for a value that is not positive.
pub fn nearest(value: f64, series: Series) -> Option<Pick> {
    if !usable(value) {
        return None;
    }
    let error = |candidate: f64| (candidate - value) / value * 100.0;
    series
        .around(value)
        .into_iter()
        .min_by(|a, b| error(*a).abs().total_cmp(&error(*b).abs()))
        .map(|candidate| Pick {
            value: candidate,
            error_percent: error(candidate),
        })
}

/// The smallest value in `series` no smaller than `value`: what an LED's
/// resistor is rounded to, where more is safer (B-02). A hair under counts,
/// so 470.0000001 is 470.
pub fn at_least(value: f64, series: Series) -> Option<f64> {
    usable(value).then(|| {
        series
            .around(value)
            .into_iter()
            .find(|&candidate| candidate >= value * (1.0 - SAME))
    })?
}

/// The largest value in `series` no larger than `value`: what a base
/// resistor is rounded to, where less is more drive (B-04).
pub fn at_most(value: f64, series: Series) -> Option<f64> {
    usable(value).then(|| {
        series
            .around(value)
            .into_iter()
            .rev()
            .find(|&candidate| candidate <= value * (1.0 + SAME))
    })?
}

/// The next value in `series` above `value`, never `value` itself: what a
/// field's + steps to (C-136).
pub fn above(value: f64, series: Series) -> Option<f64> {
    usable(value).then(|| {
        series
            .around(value)
            .into_iter()
            .find(|&candidate| candidate > value * (1.0 + SAME))
    })?
}

/// The next value in `series` below `value`, never `value` itself: what a
/// field's − steps to (C-136).
pub fn below(value: f64, series: Series) -> Option<f64> {
    usable(value).then(|| {
        series
            .around(value)
            .into_iter()
            .rev()
            .find(|&candidate| candidate < value * (1.0 - SAME))
    })?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_computed_value_is_told_the_nearest_it_can_be_bought_in() {
        // The design doc's own case (§6.2): 217 Ω, buy 220 Ω.
        let pick = nearest(217.0, Series::E12).unwrap();
        assert_eq!(pick.value, 220.0);
        assert!((pick.error_percent - 1.382).abs() < 0.001, "{pick:?}");
        // E24 has 200 and 220 either side; 220 is still nearer.
        assert_eq!(nearest(217.0, Series::E24).unwrap().value, 220.0);
        // And low as well as high: 4.9 k is 4.7 k, 4.1 % under.
        let pick = nearest(4_900.0, Series::E12).unwrap();
        assert_eq!(pick.value, 4_700.0);
        assert!(pick.error_percent < 0.0);
    }

    #[test]
    fn values_are_exact_whatever_their_decade() {
        // What a field would show: 4700, not 4700.000000000001.
        assert_eq!(nearest(4_650.0, Series::E12).unwrap().value, 4_700.0);
        assert_eq!(nearest(1.0e6, Series::E12).unwrap().value, 1.0e6);
        assert_eq!(nearest(4.6e-6, Series::E6).unwrap().value, 47.0 / 1e7);
        assert_eq!(nearest(0.9e-9, Series::E6).unwrap().value, 1.0e-9);
    }

    #[test]
    fn it_crosses_into_the_next_decade() {
        // 9.6 is nearer 10 than 8.2, and 10 is the next decade's first.
        assert_eq!(nearest(9.6, Series::E12).unwrap().value, 10.0);
        assert_eq!(at_least(8.3, Series::E12), Some(10.0));
        assert_eq!(at_most(9.9, Series::E12), Some(8.2));
        assert_eq!(above(82.0, Series::E12), Some(100.0));
        assert_eq!(below(100.0, Series::E12), Some(82.0));
    }

    #[test]
    fn rounding_one_way_keeps_a_value_already_in_the_series() {
        assert_eq!(at_least(470.0, Series::E12), Some(470.0));
        assert_eq!(at_least(470.000_000_1, Series::E12), Some(470.0));
        assert_eq!(at_least(471.0, Series::E12), Some(560.0));
        assert_eq!(at_most(470.0, Series::E12), Some(470.0));
        assert_eq!(at_most(27_900.0, Series::E12), Some(27_000.0));
    }

    #[test]
    fn stepping_always_moves() {
        // C-136's − and +: from a value in the series, and from one between.
        assert_eq!(above(470.0, Series::E12), Some(560.0));
        assert_eq!(below(470.0, Series::E12), Some(390.0));
        assert_eq!(above(500.0, Series::E24), Some(510.0));
        assert_eq!(below(500.0, Series::E24), Some(470.0));
        assert_eq!(above(4.7e-6, Series::E6), Some(68.0 / 1e7));
    }

    #[test]
    fn nothing_to_look_up_is_nothing() {
        for value in [0.0, -220.0, f64::NAN, f64::INFINITY] {
            assert_eq!(nearest(value, Series::E12), None, "{value}");
            assert_eq!(at_least(value, Series::E12), None, "{value}");
        }
    }

    #[test]
    fn each_series_is_the_one_before_with_steps_between() {
        // E6 ⊂ E12 ⊂ E24: an assortment box is one of each of the coarser.
        assert!(E6.iter().all(|step| E12.contains(step)));
        assert!(E12.iter().all(|step| E24.contains(step)));
        for series in [Series::E6, Series::E12, Series::E24] {
            assert!(series.steps().windows(2).all(|pair| pair[0] < pair[1]));
        }
        assert_eq!(RESISTORS.map(Series::name), ["E12", "E24"]);
        assert_eq!(CAPACITORS.map(Series::name), ["E6", "E12"]);
    }
}
