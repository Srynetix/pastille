// SPDX-License-Identifier: MIT OR Apache-2.0

//! A 4N35: an LED and a phototransistor in one package, facing each other
//! across a gap nothing conducts through (C-92).
//!
//! Current through the LED lights the transistor, which lets through as much
//! again from its own circuit — the current transfer ratio — and that is the
//! only thing that crosses. The two circuits need not share a ground: the
//! signal crosses, the current does not. So the part declares two sides
//! (`ElementKind::sides`), and each side's nets are a circuit of their own
//! with a reference of their own, as two experiments side by side are. The
//! coupling is a current, which is the same whatever either side's zero is.
//!
//! One coupled device over four terminals, anode, cathode, collector,
//! emitter. The LED is a junction fitted to the sheet's forward drop; the
//! transistor is the phototransistor already modelled (C-89), fed not by
//! light but by the LED's current times the transfer ratio. Within one
//! linearisation the collector current depends on the LED's voltage and its
//! own, and both slopes are stamped, so Newton sees the coupling.
//!
//! **The part is Vishay's 4N35** (its sheet read, rev. 1.2): a GaAs infrared
//! LED, 1.3 V typical at 10 mA, 6 V reversed, 50 mA; a transfer ratio of at
//! least 100 % at 10 mA and 10 V — the sheet gives no typical, so the least
//! is taken, which is what a design can count on; dark current 5 nA typical
//! at 10 V; 30 V collector to emitter, 7 V the other way, 50 mA.
//!
//! What it ignores: the base, which the package pins out and the ticket
//! leaves inside; how the ratio falls off at low and high LED current and with
//! heat, and how much lower it is in saturation; the saturation itself, which
//! is the TEPT4400's transistor's, not fitted to this sheet; switching times;
//! the coupling capacitance; and breakdown, of the isolation or anything else.

use crate::solver::{Companion, Coupled, CoupledCompanion, MAX_TERMINALS, Nonlinear};

use super::photo::{Phototransistor, limit};
use super::{Diode, THERMAL_VOLTAGE};

/// Pin order: the LED's two, then the transistor's.
pub const PINS: [&str; 4] = ["A", "K", "C", "E"];
pub const ANODE: usize = 0;
pub const CATHODE: usize = 1;
pub const COLLECTOR: usize = 2;
pub const EMITTER: usize = 3;

/// Which pins are on which side of the gap.
pub const SIDES: [&[usize]; 2] = [&[ANODE, CATHODE], &[COLLECTOR, EMITTER]];

/// What is on the part, and its sheet.
pub const PART: &str = "4N35";
pub const DATASHEET: &str = "https://www.vishay.com/docs/81181/4n35.pdf";

/// Collector current per LED current, clear of saturation: the sheet's least.
pub const TRANSFER_RATIO: f64 = 1.0;

/// Collector current with the LED dark, in amps: the sheet's typical.
pub const DARK_AMPS: f64 = 5e-9;

/// The LED's forward drop at [`TEST_AMPS`], the sheet's typical.
pub const FORWARD_VOLTS: f64 = 1.3;

/// The current the sheet quotes its LED and its ratio at.
pub const TEST_AMPS: f64 = 10e-3;

/// An infrared GaAs junction is as shallow as a visible LED's.
const EMISSION_COEFFICIENT: f64 = 2.0;

/// The optocoupler, which has nothing to set.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Optocoupler;

impl Optocoupler {
    /// Its LED: a junction at [`FORWARD_VOLTS`] carrying [`TEST_AMPS`].
    pub fn led() -> Diode {
        let scale = EMISSION_COEFFICIENT * THERMAL_VOLTAGE;
        Diode {
            saturation_current: TEST_AMPS / ((FORWARD_VOLTS / scale).exp() - 1.0),
            emission_coefficient: EMISSION_COEFFICIENT,
        }
    }

    /// Its transistor, with `led_amps` through the LED.
    fn transistor(led_amps: f64) -> Phototransistor {
        Phototransistor::fed(TRANSFER_RATIO * led_amps.max(0.0) + DARK_AMPS)
    }

    /// The LED's current and the collector's, with `led_volts` across the one
    /// and `vce` across the other.
    pub fn currents(led_volts: f64, vce: f64) -> (f64, f64) {
        let led = Self::led().current(led_volts);
        (led, Self::transistor(led).current(vce))
    }
}

impl Coupled for Optocoupler {
    fn settled(&self, potentials: &[f64; MAX_TERMINALS]) -> [f64; MAX_TERMINALS] {
        let mut points = [0.0; MAX_TERMINALS];
        points[0] = potentials[ANODE] - potentials[CATHODE];
        points[1] = potentials[COLLECTOR] - potentials[EMITTER];
        points
    }

    fn linearise(
        &self,
        potentials: &[f64; MAX_TERMINALS],
        previous: &[f64; MAX_TERMINALS],
    ) -> CoupledCompanion {
        let led_volts = potentials[ANODE] - potentials[CATHODE];
        let vce = potentials[COLLECTOR] - potentials[EMITTER];

        // The LED, limited as any junction is, and its current where it was
        // linearised.
        let led = Self::led();
        let Companion {
            conductance: g,
            current: led_rest,
            point: at,
        } = led.linearise(led_volts, previous[0]);
        let led_amps = led.current(at);

        // The transistor, fed that current. Its collector current is affine
        // in what feeds it, so its slope with the LED's current is exact from
        // two evaluations; through the LED's own slope, that is a slope with
        // the LED's voltage.
        let vce_at = limit(vce, previous[1]);
        let (collector, go) = Self::transistor(led_amps).current_and_slope(vce_at);
        let gm = if led_amps > 0.0 {
            let more = Self::transistor(led_amps + TEST_AMPS).current(vce_at);
            g * (more - collector) / TEST_AMPS
        } else {
            0.0
        };
        let collector_rest = collector - gm * at - go * vce_at;

        let mut conductances = [[0.0; MAX_TERMINALS]; MAX_TERMINALS];
        let mut currents = [0.0; MAX_TERMINALS];
        // Into the anode, out of the cathode.
        conductances[ANODE][ANODE] = g;
        conductances[ANODE][CATHODE] = -g;
        conductances[CATHODE][ANODE] = -g;
        conductances[CATHODE][CATHODE] = g;
        currents[ANODE] = led_rest;
        currents[CATHODE] = -led_rest;
        // Into the collector, out of the emitter, with a slope on each side.
        for (row, sign) in [(COLLECTOR, 1.0), (EMITTER, -1.0)] {
            conductances[row][ANODE] = sign * gm;
            conductances[row][CATHODE] = -sign * gm;
            conductances[row][COLLECTOR] = sign * go;
            conductances[row][EMITTER] = -sign * go;
            currents[row] = sign * collector_rest;
        }

        let mut points = [0.0; MAX_TERMINALS];
        points[0] = at;
        points[1] = vce_at;
        CoupledCompanion {
            terminals: PINS.len(),
            conductances,
            currents,
            points,
            limited: at != led_volts || vce_at != vce,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn its_led_drops_what_the_sheet_says() {
        let amps = Optocoupler::led().current(FORWARD_VOLTS);
        assert!((amps / TEST_AMPS - 1.0).abs() < 1e-9, "{amps}");
    }

    #[test]
    fn at_the_sheet_s_test_point_it_passes_its_transfer_ratio() {
        // 10 mA through the LED and 10 V across the transistor: 10 mA.
        let (led, collector) = Optocoupler::currents(FORWARD_VOLTS, 10.0);
        assert!(
            (collector / led - TRANSFER_RATIO).abs() < 0.01,
            "{collector}"
        );
    }

    #[test]
    fn dark_it_lets_through_only_its_leak() {
        let (_, collector) = Optocoupler::currents(0.0, 10.0);
        assert!((collector - DARK_AMPS).abs() < 1e-9, "{collector}");
    }

    #[test]
    fn its_slopes_are_the_slopes_of_its_currents() {
        // Linearised at a point, the companion's slopes match a numerical
        // derivative of the currents: with the LED's voltage, across the gap.
        let mut potentials = [0.0; MAX_TERMINALS];
        potentials[ANODE] = 1.25;
        potentials[COLLECTOR] = 5.0;
        let mut previous = [0.0; MAX_TERMINALS];
        previous[0] = 1.25;
        previous[1] = 5.0;
        let companion = Optocoupler.linearise(&potentials, &previous);
        let step = 1e-6;
        let (_, low) = Optocoupler::currents(1.25 - step, 5.0);
        let (_, high) = Optocoupler::currents(1.25 + step, 5.0);
        let across = (high - low) / (2.0 * step);
        let gm = companion.conductances[COLLECTOR][ANODE];
        assert!((gm / across - 1.0).abs() < 1e-3, "{gm} against {across}");
    }
}
