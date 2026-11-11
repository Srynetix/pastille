// SPDX-License-Identifier: MIT OR Apache-2.0

//! The 555 timer: two thresholds, a latch, an output and a switch to ground.
//!
//! ADR 0012. Within a solve the latch is fixed, so what the solver sees is
//! smooth: the output is the saturating source of C-53 with no gain, driven to
//! near VCC or near GND by the latch; the discharge pin is a switch to GND that
//! is shut while the output is low; and the internal divider of three 5 kΩ
//! resistors draws its current from VCC. Between steps [`latch`] reads the two
//! thresholds from the solved point and moves the bit, which the stepper
//! carries as it carries a capacitor's charge.
//!
//! What it honours: the thresholds at a third and two thirds of the supply; the
//! latch, set by TRIG and reset by THR, with TRIG winning a tie; an output that
//! swings to 1.7 V below VCC and 0.1 V above GND, current-limited; a discharge
//! transistor that sinks. What it ignores: RESET (tied to VCC) and CONTROL
//! (left free), which is why they are not pins; rise and fall times; the
//! supply current beyond the divider's and a few milliamps of quiescent.
//!
//! One part, the NE555, in the DIP-8 every kit has.

use super::saturating::{HIGH_RAIL, INVERTING, LOW_RAIL, NON_INVERTING, OUTPUT, SaturatingSource};
use crate::solver::{Coupled, CoupledCompanion, MAX_TERMINALS};

/// The part it is, for the label and, later, the BOM.
pub const PART: &str = "NE555";

/// Pin order, which is the order the registry names them in.
pub const VCC: usize = 0;
pub const GND: usize = 1;
pub const TRIG: usize = 2;
pub const THR: usize = 3;
pub const DIS: usize = 4;
pub const OUT: usize = 5;

/// How many pins it has.
const PINS: usize = 6;

/// Which of the timer's pins each terminal of its output stage sits on.
///
/// The output stage has no inputs to compare — the latch decides — so both sit
/// on GND, where with a gain of zero they carry nothing.
const OUTPUT_STAGE: [usize; 5] = {
    let mut wiring = [0; 5];
    wiring[NON_INVERTING] = GND;
    wiring[INVERTING] = GND;
    wiring[OUTPUT] = OUT;
    wiring[HIGH_RAIL] = VCC;
    wiring[LOW_RAIL] = GND;
    wiring
};

/// The internal divider, three 5 kΩ resistors from VCC to GND.
const DIVIDER_OHMS: f64 = 15e3;

/// The discharge transistor, shut.
///
/// Ten ohms: it empties a timing capacitor through the resistor above it in
/// far less time than that resistor fills it, which is all a timing circuit
/// asks of it.
const DISCHARGE_ON_OHMS: f64 = 10.0;

/// And open: a leak far below anything a timing resistor would notice.
const DISCHARGE_OFF_SIEMENS: f64 = 1e-9;

/// Below this supply, in volts, the part is off and its output low.
const POWERED_VOLTS: f64 = 1.0;

/// A 555 with its latch set one way or the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Timer {
    /// The latch: true drives the output high and opens the discharge pin.
    pub high: bool,
}

impl Timer {
    fn output_stage(&self) -> SaturatingSource {
        SaturatingSource {
            gain: 0.0,
            // High: 1.7 V under VCC. Low: a tenth of a volt over GND. The same
            // headroom either way, so the clamp is one clamp.
            offset: if self.high { -1.7 } else { 0.1 },
            reference: if self.high { 1.0 } else { 0.0 },
            headroom_high: 1.7,
            headroom_low: 0.1,
            output_ohms: 10.0,
            source_limit: 0.2,
            sink_limit: 0.2,
            quiescent: 3e-3,
            input_ohms: f64::INFINITY,
        }
    }
}

/// Where the latch goes, given where it was and the potentials at the pins.
///
/// TRIG below a third of the supply sets it; THR above two thirds resets it;
/// the trigger wins when both are true, as it does in the part. An unpowered
/// timer is reset, which is what one reads as when it is switched on.
pub fn latch(was: bool, potentials: &[f64; PINS]) -> bool {
    let supply = potentials[VCC] - potentials[GND];
    if supply < POWERED_VOLTS {
        return false;
    }
    if potentials[TRIG] - potentials[GND] < supply / 3.0 {
        true
    } else if potentials[THR] - potentials[GND] > supply * 2.0 / 3.0 {
        false
    } else {
        was
    }
}

impl Timer {
    /// The timer's pin potentials, as its output stage's terminals see them.
    fn staged(potentials: &[f64; MAX_TERMINALS]) -> [f64; MAX_TERMINALS] {
        let mut staged = [0.0; MAX_TERMINALS];
        for (terminal, &pin) in OUTPUT_STAGE.iter().enumerate() {
            staged[terminal] = potentials[pin];
        }
        staged
    }
}

impl Coupled for Timer {
    /// Its output stage's, which is the only part of it with a history.
    fn settled(&self, potentials: &[f64; MAX_TERMINALS]) -> [f64; MAX_TERMINALS] {
        self.output_stage().settled(&Self::staged(potentials))
    }

    fn linearise(
        &self,
        potentials: &[f64; MAX_TERMINALS],
        previous: &[f64; MAX_TERMINALS],
    ) -> CoupledCompanion {
        // The output stage, seen through its own five terminals.
        let stage = self
            .output_stage()
            .linearise(&Self::staged(potentials), previous);

        let mut conductances = [[0.0; MAX_TERMINALS]; MAX_TERMINALS];
        let mut currents = [0.0; MAX_TERMINALS];
        for (i, &row) in OUTPUT_STAGE.iter().enumerate() {
            currents[row] += stage.currents[i];
            for (j, &column) in OUTPUT_STAGE.iter().enumerate() {
                conductances[row][column] += stage.conductances[i][j];
            }
        }

        // Two linear pieces, which need no companion current: the divider
        // across the supply, and the discharge switch from DIS to GND.
        let mut resistor = |a: usize, b: usize, siemens: f64| {
            conductances[a][a] += siemens;
            conductances[b][b] += siemens;
            conductances[a][b] -= siemens;
            conductances[b][a] -= siemens;
        };
        resistor(VCC, GND, 1.0 / DIVIDER_OHMS);
        resistor(
            DIS,
            GND,
            if self.high {
                DISCHARGE_OFF_SIEMENS
            } else {
                1.0 / DISCHARGE_ON_OHMS
            },
        );

        CoupledCompanion {
            terminals: PINS,
            conductances,
            currents,
            points: stage.points,
            limited: stage.limited,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(vcc: f64, trig: f64, thr: f64) -> [f64; PINS] {
        let mut potentials = [0.0; PINS];
        potentials[VCC] = vcc;
        potentials[TRIG] = trig;
        potentials[THR] = thr;
        potentials
    }

    #[test]
    fn the_latch_follows_the_datasheet() {
        // 9 V: thresholds at 3 V and 6 V.
        assert!(
            latch(false, &at(9.0, 2.9, 2.9)),
            "trigger under a third sets it"
        );
        assert!(
            !latch(true, &at(9.0, 6.1, 6.1)),
            "threshold over two thirds resets it"
        );
        assert!(latch(true, &at(9.0, 4.5, 4.5)), "between, it holds high");
        assert!(!latch(false, &at(9.0, 4.5, 4.5)), "between, it holds low");
        assert!(latch(false, &at(9.0, 2.0, 7.0)), "the trigger wins a tie");
        assert!(!latch(true, &at(0.0, 0.0, 0.0)), "unpowered, it is off");
    }

    #[test]
    fn what_enters_it_leaves_it() {
        // Kirchhoff inside the part: its currents sum to nothing, high or low.
        for high in [false, true] {
            let mut potentials = [0.0; MAX_TERMINALS];
            potentials[..PINS].copy_from_slice(&[9.0, 0.0, 4.0, 4.0, 3.0, 5.0]);
            let companion = Timer { high }.linearise(&potentials, &[0.0; MAX_TERMINALS]);
            let total: f64 = (0..PINS)
                .map(|i| {
                    companion.currents[i]
                        + (0..PINS)
                            .map(|j| companion.conductances[i][j] * potentials[j])
                            .sum::<f64>()
                })
                .sum();
            assert!(
                total.abs() < 1e-12,
                "high {high}: {total} A unaccounted for"
            );
        }
    }
}
