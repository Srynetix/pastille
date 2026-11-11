// SPDX-License-Identifier: MIT OR Apache-2.0

//! A small solar panel: light in, current out (C-117).
//!
//! Two legs, + and −. Inside, a string of silicon cells, each a photodiode:
//! light frees a current in it that runs out of its + whatever is across it,
//! and the junction it is made of takes some of that current back the higher
//! the voltage climbs. So the panel is a current source in parallel with a
//! diode — the single-diode model every panel sheet's curve is drawn from —
//! and the current into its + is
//!
//!   I(V) = I₀·(e^(V/a) − 1) − Iph
//!
//! with Iph the photocurrent, proportional to the light, and a the string's
//! n·Vt. Short-circuited, it gives Iph; open, it sits where the diode takes
//! all of it back, at a·ln(Iph/I₀ + 1); with a load, where the load's line
//! crosses that curve. A two-terminal curve, so the solver linearises it as
//! it does a diode's, with the diode's limiting.
//!
//! **The panel is MikroE's SW0.4M**, 70 × 65 mm of monocrystalline cells, at
//! STC (1000 W/m², AM1.5, 25 °C): open-circuit 4.6 V, short-circuit 105 mA,
//! 4.0 V and 100 mA at its maximum power, 0.4 W, each ±8 %. Those four fix
//! the curve's two unknowns: a from how far the current has fallen at the
//! maximum-power point, (Voc − Vmp)/ln(Isc/(Isc − Imp)) = 0.197 V, and I₀
//! from the open-circuit voltage. That a is eight cells' worth at an emission
//! coefficient of 0.95 — eight cells of 0.58 V is the 4.6 V — so the sheet's
//! numbers are those of a real string of junctions. And the curve's own
//! maximum-power point, found rather than assumed, lands on the sheet's 4.0 V.
//!
//! **Light** is a fraction of the sun the sheet was measured in, 0 to 1, and
//! the photocurrent is that fraction of 105 mA: a short-circuit current
//! proportional to the light, which is what silicon does, and an open-circuit
//! voltage that falls only by a·ln 10, 0.45 V, for every tenth.
//!
//! What it ignores: series and shunt resistance, which round the knee off in
//! a real panel (the sheet gives no figures for either, and its 0.83 fill
//! factor leaves little room for them); temperature, which lowers the voltage
//! some 2 mV a cell for every degree; reverse breakdown; the light's colour
//! and angle.

use crate::solver::{Companion, Nonlinear};

use super::{Diode, THERMAL_VOLTAGE};

/// The part, and its sheet.
pub const PART: &str = "MikroE SW0.4M";
pub const DATASHEET: &str =
    "https://download.mikroe.com/documents/datasheets/solar-panel-datasheet.pdf";

/// The sheet's four numbers, at full sun.
pub const OPEN_CIRCUIT_VOLTS: f64 = 4.6;
pub const SHORT_CIRCUIT_AMPS: f64 = 0.105;
pub const PEAK_VOLTS: f64 = 4.0;
pub const PEAK_AMPS: f64 = 0.1;

/// The string's n·Vt, in volts, from the maximum-power point.
fn scale() -> f64 {
    (OPEN_CIRCUIT_VOLTS - PEAK_VOLTS) / (SHORT_CIRCUIT_AMPS / (SHORT_CIRCUIT_AMPS - PEAK_AMPS)).ln()
}

/// The junction inside: the string of cells as one diode.
fn junction() -> Diode {
    let scale = scale();
    Diode {
        saturation_current: SHORT_CIRCUIT_AMPS / ((OPEN_CIRCUIT_VOLTS / scale).exp() - 1.0),
        emission_coefficient: scale / THERMAL_VOLTAGE,
    }
}

/// A panel in so much sun.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolarCell {
    junction: Diode,
    /// What the light frees, in amps.
    photocurrent: f64,
}

impl SolarCell {
    /// One in `sun` of the sheet's light, 0 to 1.
    pub fn new(sun: f64) -> SolarCell {
        SolarCell {
            junction: junction(),
            photocurrent: SHORT_CIRCUIT_AMPS * sun.clamp(0.0, 1.0),
        }
    }

    /// Current into its +, in amps, with `volts` across it, + over −: below
    /// zero while it is giving power.
    pub fn current(&self, volts: f64) -> f64 {
        self.junction.current(volts) - self.photocurrent
    }
}

impl Nonlinear for SolarCell {
    fn linearise(&self, voltage_across: f64, previous_point: f64) -> Companion {
        // The diode's curve moved down by a constant: the same slope, so the
        // same limiting, and the photocurrent rides in the companion's source.
        let v = self.junction.limit(voltage_across, previous_point);
        let conductance = self.junction.conductance(v);
        Companion {
            conductance,
            current: self.current(v) - conductance * v,
            point: v,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Where the current into it is `amps`, by bisection: its curve rises.
    fn volts_at(cell: &SolarCell, amps: f64) -> f64 {
        let (mut low, mut high) = (-1.0, 10.0);
        for _ in 0..100 {
            let middle = (low + high) / 2.0;
            if cell.current(middle) < amps {
                low = middle;
            } else {
                high = middle;
            }
        }
        low
    }

    #[test]
    fn in_full_sun_it_gives_the_sheet_s_open_and_short_circuit() {
        let sun = SolarCell::new(1.0);
        assert!((sun.current(0.0) + SHORT_CIRCUIT_AMPS).abs() < 1e-12);
        assert!((volts_at(&sun, 0.0) - OPEN_CIRCUIT_VOLTS).abs() < 1e-6);
        // And the sheet's maximum-power point is on the curve.
        assert!((sun.current(PEAK_VOLTS) + PEAK_AMPS).abs() < 1e-6);
    }

    #[test]
    fn its_short_circuit_current_is_proportional_to_the_light() {
        for sun in [0.1, 0.25, 0.5, 0.8] {
            let short = -SolarCell::new(sun).current(0.0);
            assert!((short / (sun * SHORT_CIRCUIT_AMPS) - 1.0).abs() < 1e-9);
        }
        // In the dark it gives nothing, and is a diode.
        assert_eq!(SolarCell::new(0.0).current(0.0), 0.0);
        assert!(SolarCell::new(0.0).current(4.0) > 1e-3);
    }

    #[test]
    fn its_open_circuit_voltage_falls_slowly_with_the_light() {
        // A tenth of the sun: 0.45 V less, not a tenth of the voltage.
        let tenth = volts_at(&SolarCell::new(0.1), 0.0);
        assert!(
            (OPEN_CIRCUIT_VOLTS - tenth - 0.454).abs() < 0.01,
            "{tenth} V"
        );
    }

    #[test]
    fn its_maximum_power_is_the_sheet_s_and_lies_between_its_ends() {
        let sun = SolarCell::new(1.0);
        let (mut best, mut at) = (0.0, 0.0);
        for step in 0..=4600 {
            let volts = step as f64 / 1000.0;
            let watts = -volts * sun.current(volts);
            if watts > best {
                best = watts;
                at = volts;
            }
        }
        assert!((at - PEAK_VOLTS).abs() < 0.05, "best at {at} V");
        assert!((best - PEAK_VOLTS * PEAK_AMPS).abs() < 0.005, "{best} W");
        // Eight cells of a real junction, near enough.
        let per_cell = junction().emission_coefficient / 8.0;
        assert!((0.9..1.0).contains(&per_cell), "n = {per_cell} a cell");
    }
}
