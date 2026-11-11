// SPDX-License-Identifier: MIT OR Apache-2.0

//! A B-series CMOS output's strength at the supply it is on (C-124).
//!
//! The sheets give an output's source current two and a half volts below VDD
//! at 5 V in their table, and at 10 V and 15 V in their typical curves; a
//! chip driving an LED sits about there. The resistance behind an output is
//! that 2.5 V over the current, which falls steeply from 5 V to 10 V — a
//! CD4017 on 9 V is three times the driver it is on 5 V. Between the sheet's
//! supplies the conductance is taken in a straight line; outside them, the
//! nearest one's, since the sheets draw nothing there.

/// The resistance behind an output on `supply` volts, from `curve`: the
/// sheet's (supply, ohms) points, lowest supply first. A supply not known yet
/// — nothing has been solved — takes the lowest point's.
pub fn output_ohms(curve: &[(f64, f64)], supply: Option<f64>) -> f64 {
    let (first, last) = (curve[0], curve[curve.len() - 1]);
    let Some(volts) = supply else {
        return first.1;
    };
    if volts <= first.0 {
        return first.1;
    }
    if volts >= last.0 {
        return last.1;
    }
    let at = curve
        .windows(2)
        .find(|pair| volts <= pair[1].0)
        .expect("between the first point and the last");
    let ((low, below), (high, above)) = (at[0], at[1]);
    let along = (volts - low) / (high - low);
    1.0 / (1.0 / below + along * (1.0 / above - 1.0 / below))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CURVE: [(f64, f64); 3] = [(5.0, 780.0), (10.0, 270.0), (15.0, 220.0)];

    #[test]
    fn at_each_of_the_sheet_s_supplies_it_is_the_sheet_s_figure() {
        for (volts, ohms) in CURVE {
            assert!((output_ohms(&CURVE, Some(volts)) - ohms).abs() < 1e-9);
        }
    }

    #[test]
    fn between_them_its_conductance_is_in_a_straight_line() {
        // 9 V, four fifths of the way from 5 V to 10 V.
        let g = 1.0 / 780.0 + 0.8 * (1.0 / 270.0 - 1.0 / 780.0);
        assert!((output_ohms(&CURVE, Some(9.0)) - 1.0 / g).abs() < 1e-9);
    }

    #[test]
    fn outside_them_or_unknown_it_is_the_nearest_point_s() {
        assert_eq!(output_ohms(&CURVE, Some(3.0)), 780.0);
        assert_eq!(output_ohms(&CURVE, Some(18.0)), 220.0);
        assert_eq!(output_ohms(&CURVE, None), 780.0);
    }
}
