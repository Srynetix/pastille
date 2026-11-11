// SPDX-License-Identifier: MIT OR Apache-2.0

//! An aluminium electrolytic capacitor, and what happens wired backwards
//! (C-86).
//!
//! Polarised: its + pin has to face the higher voltage. The capacitor this
//! simulator has had since C-06 is not, on purpose — two straight plates — and
//! a kit's capacitors above a microfarad are nearly all electrolytics, so this
//! is the one a beginner actually holds. Wired right and inside its rating it
//! *is* that capacitor, plus the leak its sheet allows.
//!
//! The part is Nichicon's UVR, a general-purpose radial series. Its sheet,
//! for 6.3 V to 100 V: leakage no more than 0.01·C·V µA after two minutes at
//! rated voltage, C in µF. As a resistance that is 10⁸ / C Ω, whatever the
//! voltage — 1 MΩ for 100 µF — so every one of them, at its limit, empties on
//! its own with the same time constant of 100 s. The model takes the limit,
//! which is what makes a large one sag visibly over a minute.
//!
//! **Wired backwards, or past its rating, it says so and goes on working**
//! (Denis, 2026-10-01): nothing is destroyed, so turning it round makes the
//! warning go. Reversed means more than a volt the wrong way, which is the
//! rule of thumb electrolytics are given rather than anything the UVR sheet
//! states. What it ignores: what a reversed one does in reality — it conducts,
//! heats and can vent — and the leak's fall to far less than the limit once
//! it has sat at voltage for a while; the surge rating above the rated one.

use crate::netlist::Element;
use crate::solver::OperatingPoint;

/// How far the wrong way round counts as wired backwards, in volts.
pub const REVERSED_FROM: f64 = 1.0;

/// How long a reversal or an over-voltage is remembered, in seconds of the
/// run. Long enough that one on an alternating drive — reversed half of every
/// cycle — is said steadily rather than flickering on and off with the cycle;
/// short enough that turning the part round is answered at once.
pub const HELD_FOR: f64 = 0.5;

/// The series, for the Inspector.
pub const SERIES: &str = "Nichicon UVR";

/// Its sheet.
pub const DATASHEET: &str = "https://www.nichicon.co.jp/english/series_items/catalog_pdf/e-uvr.pdf";

/// The voltages it can be rated for: three a kit carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Rated {
    V16,
    V25,
    V50,
}

impl Rated {
    /// In the order a chooser offers them: lowest first.
    pub const ALL: &'static [Rated] = &[Rated::V16, Rated::V25, Rated::V50];

    /// The name it crosses the boundary under.
    pub fn name(&self) -> &'static str {
        match self {
            Rated::V16 => "16 V",
            Rated::V25 => "25 V",
            Rated::V50 => "50 V",
        }
    }

    pub fn named(name: &str) -> Option<Rated> {
        Rated::ALL
            .iter()
            .copied()
            .find(|rated| rated.name() == name)
    }

    /// The most it may be given, the right way round, in volts.
    pub fn volts(&self) -> f64 {
        match self {
            Rated::V16 => 16.0,
            Rated::V25 => 25.0,
            Rated::V50 => 50.0,
        }
    }

    /// The part, as the Inspector names it.
    pub fn part(&self) -> &'static str {
        match self {
            Rated::V16 => "Nichicon UVR, 16 V",
            Rated::V25 => "Nichicon UVR, 25 V",
            Rated::V50 => "Nichicon UVR, 50 V",
        }
    }
}

/// The leak the sheet allows, as a resistance across it, in ohms: 0.01·C·V µA
/// at V volts is 10⁸ / C\[µF\] Ω.
pub fn leak_ohms(farads: f64) -> f64 {
    1.0 / (0.01 * farads)
}

/// What a capacitor has been put through lately.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Strain {
    /// When it was last more than [`REVERSED_FROM`] the wrong way round.
    reversed_at: Option<f64>,
    /// When it last had more across it than its rating.
    over_at: Option<f64>,
    /// The latest time it was told about.
    now: f64,
}

impl Strain {
    /// Takes in `volts` across it, + over −, at `time` in seconds of the run.
    pub fn advance(&mut self, volts: f64, rated: Rated, time: f64) {
        self.now = time;
        if volts < -REVERSED_FROM {
            self.reversed_at = Some(time);
        }
        if volts > rated.volts() {
            self.over_at = Some(time);
        }
    }

    /// Whether it has been wired backwards within the last [`HELD_FOR`].
    pub fn reversed(&self) -> bool {
        self.reversed_at.is_some_and(|at| self.now - at <= HELD_FOR)
    }

    /// Whether it has been past its rating within the last [`HELD_FOR`].
    pub fn over(&self) -> bool {
        self.over_at.is_some_and(|at| self.now - at <= HELD_FOR)
    }
}

/// The voltage across an electrolytic, + over −, and its rating, or nothing
/// for any other part or one not wired at both ends.
pub fn across(element: &Element, point: &OperatingPoint) -> Option<(f64, Rated)> {
    let crate::netlist::ElementKind::Electrolytic { rated, .. } = element.kind else {
        return None;
    };
    let plus = point.voltage(element.pins.first()?.net?)?;
    let minus = point.voltage(element.pins.get(1)?.net?)?;
    Some((plus - minus, rated))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_rating_is_found_by_its_own_name() {
        for rated in Rated::ALL {
            assert_eq!(Rated::named(rated.name()), Some(*rated));
        }
    }

    #[test]
    fn the_leak_is_the_sheet_s_limit_whatever_the_voltage() {
        // 0.01·C·V µA: 100 µF at 16 V is 16 µA, which is 1 MΩ at 16 V.
        let ohms = leak_ohms(100e-6);
        assert!((16.0 / ohms - 16e-6).abs() < 1e-12);
        assert!((ohms - 1e6).abs() < 1e-3);
        // And it empties with a 100 s time constant, whatever its size.
        for farads in [1e-6, 100e-6, 4700e-6] {
            assert!((leak_ohms(farads) * farads - 100.0).abs() < 1e-9);
        }
    }

    #[test]
    fn a_volt_the_wrong_way_is_tolerated_and_more_is_not() {
        let mut strain = Strain::default();
        strain.advance(-0.9, Rated::V16, 0.0);
        assert!(!strain.reversed());
        strain.advance(-1.5, Rated::V16, 0.1);
        assert!(strain.reversed());
    }

    #[test]
    fn past_its_rating_is_said_and_at_it_is_not() {
        let mut strain = Strain::default();
        strain.advance(16.0, Rated::V16, 0.0);
        assert!(!strain.over());
        strain.advance(17.0, Rated::V16, 0.1);
        assert!(strain.over());
        // A 25 V part takes it.
        let mut bigger = Strain::default();
        bigger.advance(17.0, Rated::V25, 0.0);
        assert!(!bigger.over());
    }

    #[test]
    fn a_reversal_on_every_cycle_is_said_steadily() {
        // ±5 V at 10 Hz: reversed half of each cycle, and said throughout.
        let mut strain = Strain::default();
        let mut said = Vec::new();
        for index in 0..600 {
            let time = index as f64 / 600.0;
            let volts = if (time * 10.0).fract() < 0.5 {
                5.0
            } else {
                -5.0
            };
            strain.advance(volts, Rated::V16, time);
            if time > 0.1 {
                said.push(strain.reversed());
            }
        }
        assert!(said.iter().all(|&reversed| reversed));
    }

    #[test]
    fn turned_round_it_stops_saying_so_shortly_after() {
        let mut strain = Strain::default();
        strain.advance(-9.0, Rated::V16, 0.0);
        strain.advance(9.0, Rated::V16, 0.2);
        assert!(strain.reversed(), "still within the hold");
        strain.advance(9.0, Rated::V16, 0.6);
        assert!(!strain.reversed());
    }
}
