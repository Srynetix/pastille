// SPDX-License-Identifier: MIT OR Apache-2.0

//! The Zener diode: a diode forwards, and a fixed voltage backwards.
//!
//! A discrete part, so a physical model (ADR 0010, term 1): two pn junctions
//! back to back in one component. Forwards it is the Shockley diode of M-10.
//! Backwards a second junction takes over at the breakdown voltage, the same
//! exponential mirrored and shifted so that at the part's rated voltage it
//! carries the datasheet's test current — which gives it the few ohms of slope
//! a real one has, from the same `n·Vt / I` a forward diode's has.
//!
//! What it honours: the forward drop; the breakdown voltage at the test current;
//! how steeply the breakdown turns on. What it ignores: temperature, which moves
//! a real Zener's voltage by millivolts a degree; the soft knee of a low-voltage
//! part, which is mostly avalanche rather than Zener effect.
//!
//! Reimplemented from the published SPICE junction model with breakdown (BV and
//! IBV), not from any GPL source (ADR 0001).

use super::Diode;
use crate::solver::{Companion, Nonlinear};

/// Which part: three of the 1N47xx one-watt series, named for their voltage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ZenerPart {
    /// 1N4728A.
    V3_3,
    /// 1N4733A: the one to make 5 V from a battery without a chip.
    V5_1,
    /// 1N4739A.
    V9_1,
}

impl ZenerPart {
    /// Every part, in the order a chooser should offer them.
    pub const ALL: &'static [ZenerPart] = &[ZenerPart::V3_3, ZenerPart::V5_1, ZenerPart::V9_1];

    /// The name it crosses the boundary under: its voltage, which is how
    /// anybody asks for one.
    pub fn name(&self) -> &'static str {
        match self {
            ZenerPart::V3_3 => "3.3 V",
            ZenerPart::V5_1 => "5.1 V",
            ZenerPart::V9_1 => "9.1 V",
        }
    }

    /// The part of that name, or nothing if it is not one.
    pub fn named(name: &str) -> Option<ZenerPart> {
        ZenerPart::ALL
            .iter()
            .copied()
            .find(|part| part.name() == name)
    }

    /// What is printed on it, for the BOM.
    pub fn part_number(&self) -> &'static str {
        match self {
            ZenerPart::V3_3 => "1N4728A",
            ZenerPart::V5_1 => "1N4733A",
            ZenerPart::V9_1 => "1N4739A",
        }
    }

    /// The rated breakdown voltage, at [`ZenerPart::test_amps`].
    pub fn volts(&self) -> f64 {
        match self {
            ZenerPart::V3_3 => 3.3,
            ZenerPart::V5_1 => 5.1,
            ZenerPart::V9_1 => 9.1,
        }
    }

    /// The current the datasheet rates the voltage at: a quarter of what the
    /// part's watt allows, which is how the 1N47xx series is specified.
    pub fn test_amps(&self) -> f64 {
        match self {
            ZenerPart::V3_3 => 76e-3,
            ZenerPart::V5_1 => 49e-3,
            ZenerPart::V9_1 => 28e-3,
        }
    }
}

/// Most power a 1N47xx may dissipate, in watts.
pub const RATED_WATTS: f64 = 1.0;

/// A Zener diode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Zener {
    pub part: ZenerPart,
    junction: Diode,
}

impl Zener {
    pub fn new(part: ZenerPart) -> Self {
        Zener {
            part,
            junction: Diode::default(),
        }
    }

    /// How far below the rated voltage the breakdown junction's own zero sits:
    /// where its exponential reaches the test current.
    fn shift(&self) -> f64 {
        let scale = self.junction.emission_coefficient * super::THERMAL_VOLTAGE;
        self.part.volts()
            - scale * (1.0 + self.part.test_amps() / self.junction.saturation_current).ln()
    }

    /// The breakdown junction's own voltage, for an anode-to-cathode `v`.
    fn reverse(&self, v: f64) -> f64 {
        -v - self.shift()
    }

    /// Current from anode to cathode at `v` — negative in breakdown.
    pub fn current(&self, v: f64) -> f64 {
        self.junction.current(v) - self.junction.current(self.reverse(v))
    }

    /// Slope dI/dV at `v`.
    pub fn conductance(&self, v: f64) -> f64 {
        self.junction.conductance(v) + self.junction.conductance(self.reverse(v))
    }
}

impl Nonlinear for Zener {
    fn linearise(&self, voltage_across: f64, previous_point: f64) -> Companion {
        // Only one junction is ever conducting, and it is the one whose step
        // wants bounding: the forward one above half the breakdown voltage in
        // reverse, the breakdown one below it.
        let v = if voltage_across > -self.part.volts() / 2.0 {
            self.junction.limit(voltage_across, previous_point)
        } else {
            let limited = self
                .junction
                .limit(self.reverse(voltage_across), self.reverse(previous_point));
            -limited - self.shift()
        };
        let i = self.current(v);
        let g = self.conductance(v);
        Companion {
            conductance: g,
            current: i - g * v,
            point: v,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_carries_its_test_current_at_its_rated_voltage() {
        for part in ZenerPart::ALL {
            let zener = Zener::new(*part);
            let at = zener.current(-part.volts());
            assert!(
                (at + part.test_amps()).abs() / part.test_amps() < 1e-9,
                "{}: {at} A at −{} V",
                part.name(),
                part.volts()
            );
        }
    }

    #[test]
    fn forwards_it_is_a_diode() {
        let zener = Zener::new(ZenerPart::V5_1);
        let diode = Diode::default();
        assert!((zener.current(0.65) - diode.current(0.65)).abs() / diode.current(0.65) < 1e-9);
    }

    #[test]
    fn between_the_two_it_passes_nothing() {
        let zener = Zener::new(ZenerPart::V5_1);
        assert!(
            zener.current(-3.0).abs() < 1e-9,
            "{} A at −3 V",
            zener.current(-3.0)
        );
    }

    #[test]
    fn every_part_is_found_by_its_own_name() {
        for part in ZenerPart::ALL {
            assert_eq!(ZenerPart::named(part.name()), Some(*part));
        }
    }
}
