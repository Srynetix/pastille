// SPDX-License-Identifier: MIT OR Apache-2.0

//! A phototransistor: a transistor whose base current is light (C-89).
//!
//! Two legs, a collector and an emitter. The base is there, inside, but not
//! pinned out: light falling on it frees a photocurrent into it, and the
//! transistor multiplies that up into its collector. So it is the transistor
//! already modelled (C-12, Ebers–Moll) with a floating base fed a set
//! current — and with the base floating, the base–emitter voltage is fixed by
//! the collector–emitter one in closed form, which makes the whole part a
//! two-terminal curve the solver linearises as it does a diode's. Nothing
//! inside needs a node of its own.
//!
//! **The closed form.** With x = e^(vbe/Vt) and y = e^(−vce/Vt), the base
//! current the light supplies is Is·[(x − 1)/βF + (x·y − 1)/βR], so
//! x = k / (1/βF + y/βR) with k = Ip/Is + 1/βF + 1/βR; and the collector
//! current is Is·[(x − 1) − (1 + 1/βR)(x·y − 1)]. Well clear of saturation
//! that is βF·Ip; towards vce = 0 it falls away over a few tenths of a volt,
//! which is the saturation a transistor has — so a phototransistor under a
//! resistor pulls its output down to the sheet's V_CE(sat) in light. Written with y for vce ≥ 0 and 1/y below, so
//! neither ever overflows.
//!
//! **The part is Vishay's TEPT4400**, an ambient-light sensor in a 3 mm
//! package: collector light current typically 200 µA at 100 lx, and 15 to
//! 70 µA at 20 lx across its bins, so about 2 µA a lux, linear; dark current
//! 3 nA typical, 50 nA most, at 5 V; V_CE(sat) 0.1 V; 6 V collector to
//! emitter, 20 mA, and only 1.5 V the other way round. Its gain is not on the
//! sheet and does not need to be: only βF·Ip shows outside saturation, and Ip
//! is set to make it the sheet's current.
//!
//! What it ignores: its spectral response — the dial is lux of daylight-ish
//! light — and its angle; its speed; breakdown, either way round; heat.

use crate::solver::{Companion, Nonlinear};

use super::THERMAL_VOLTAGE;

/// Collector current per lux, in amps: 200 µA at 100 lx.
pub const AMPS_PER_LUX: f64 = 2e-6;

/// Collector current in the dark, in amps, the sheet's typical.
pub const DARK_AMPS: f64 = 3e-9;

/// The part, and its sheet.
pub const PART: &str = "Vishay TEPT4400";
pub const DATASHEET: &str = "https://www.vishay.com/docs/81341/tept4400.pdf";

/// The transistor inside: a forward gain high enough that the base current is
/// a small thing, and the saturation current the discrete transistor has
/// (C-12). The reverse gain sets where it saturates, and is fitted to the
/// sheet's one point: 0.1 V across it carrying 1.2 µA at 20 lx, where
/// 0.3 gives 0.104 V (0.1 gave 0.13, 1 gave 0.07).
const FORWARD_BETA: f64 = 500.0;
const REVERSE_BETA: f64 = 0.3;
const SATURATION_CURRENT: f64 = 1e-14;

/// A phototransistor in so much light.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Phototransistor {
    /// What the light puts into its base, in amps.
    photocurrent: f64,
}

impl Phototransistor {
    /// One in `lux` of light: its collector current, clear of saturation, is
    /// the sheet's.
    pub fn new(lux: f64) -> Phototransistor {
        Phototransistor {
            photocurrent: (AMPS_PER_LUX * lux.max(0.0) + DARK_AMPS) / FORWARD_BETA,
        }
    }

    /// One fed so much that, clear of saturation, its collector carries
    /// `amps`: not light but whatever lights it — the LED beside it in an
    /// optocoupler (C-92).
    pub(super) fn fed(amps: f64) -> Phototransistor {
        Phototransistor {
            photocurrent: amps.max(0.0) / FORWARD_BETA,
        }
    }

    /// Current into the collector with `vce` across it, collector over
    /// emitter, and how fast it changes with `vce`.
    pub(super) fn current_and_slope(&self, vce: f64) -> (f64, f64) {
        let is = SATURATION_CURRENT;
        let k = self.photocurrent / is + 1.0 / FORWARD_BETA + 1.0 / REVERSE_BETA;
        // x and x·y, and their slopes with vce.
        let (x, dx, xy, dxy) = if vce >= 0.0 {
            let y = (-vce / THERMAL_VOLTAGE).exp();
            let d = 1.0 / FORWARD_BETA + y / REVERSE_BETA;
            let x = k / d;
            let dx = k * y / (THERMAL_VOLTAGE * REVERSE_BETA * d * d);
            let xy = k * y / d;
            let dxy = -k * y / (THERMAL_VOLTAGE * FORWARD_BETA * d * d);
            (x, dx, xy, dxy)
        } else {
            let u = (vce / THERMAL_VOLTAGE).exp();
            let e = u / FORWARD_BETA + 1.0 / REVERSE_BETA;
            let x = k * u / e;
            let dx = k * u / (THERMAL_VOLTAGE * REVERSE_BETA * e * e);
            let xy = k / e;
            let dxy = -k * u / (THERMAL_VOLTAGE * FORWARD_BETA * e * e);
            (x, dx, xy, dxy)
        };
        let share = 1.0 + 1.0 / REVERSE_BETA;
        let current = is * ((x - 1.0) - share * (xy - 1.0));
        let slope = is * (dx - share * dxy);
        (current, slope)
    }

    /// Current into the collector, in amps, with `vce` across it.
    pub fn current(&self, vce: f64) -> f64 {
        self.current_and_slope(vce).0
    }
}

/// How far one Newton iteration may move it: half a volt, and half again of
/// how far from zero it was — and never across zero in one step, only half
/// way to it.
///
/// Its curve is flat on both sides — clear of saturation it is a current
/// source, and reversed it is one again — and from either flat side Newton's
/// step lands on the other: in bright light under a resistor it swung
/// between ±20 V and never settled, and bounded alone it still swung between
/// ±⅓ V either side of the knee. Kept from crossing, it walks down into the
/// knee, where its slope is steep and the steps small, so the bound stops
/// mattering exactly where the answer is: ten iterations from cold, in every
/// case tried.
pub(super) fn limit(voltage: f64, previous: f64) -> f64 {
    let reach = 0.5 + 0.5 * previous.abs();
    let bounded = voltage.clamp(previous - reach, previous + reach);
    if previous * bounded < 0.0 {
        previous / 2.0
    } else {
        bounded
    }
}

impl Nonlinear for Phototransistor {
    fn linearise(&self, voltage_across: f64, previous_point: f64) -> Companion {
        let v = limit(voltage_across, previous_point);
        let (current, slope) = self.current_and_slope(v);
        Companion {
            conductance: slope,
            current: current - slope * v,
            point: v,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_of_saturation_it_carries_the_sheet_s_current() {
        // 200 µA at 100 lx, 40 µA at 20 lx, at 5 V across it.
        for (lux, expected) in [(100.0, 200e-6), (20.0, 40e-6)] {
            let current = Phototransistor::new(lux).current(5.0);
            assert!(
                (current / expected - 1.0).abs() < 0.01,
                "{lux} lx: {current} A"
            );
        }
        let dark = Phototransistor::new(0.0).current(5.0);
        assert!((dark / DARK_AMPS - 1.0).abs() < 0.05, "dark: {dark} A");
    }

    #[test]
    fn it_saturates_where_its_sheet_says() {
        // The sheet's V_CE(sat): 0.1 V across it carrying 1.2 µA at 20 lx.
        let lit = Phototransistor::new(20.0);
        let (mut low, mut high) = (0.0, 1.0);
        for _ in 0..60 {
            let middle = (low + high) / 2.0;
            if lit.current(middle) < 1.2e-6 {
                low = middle;
            } else {
                high = middle;
            }
        }
        assert!((low - 0.1).abs() < 0.01, "{low} V at 1.2 µA");
        // A knee, not a resistor's line: most of its current by 0.3 V.
        let full = lit.current(5.0);
        assert!(lit.current(0.3) > 0.8 * full);
        // And at nothing across, only its photocurrent, the base's, leaving
        // by the collector: a photodiode's, a five-hundredth of its full.
        assert!(lit.current(0.0) < 0.0 && lit.current(0.0).abs() < 0.01 * full);
    }

    #[test]
    fn its_slope_is_its_current_s_rate_of_change() {
        let lit = Phototransistor::new(50.0);
        for vce in [-1.0, -0.05, 0.0, 0.02, 0.1, 3.0] {
            let (_, slope) = lit.current_and_slope(vce);
            let h = 1e-7;
            let numeric = (lit.current(vce + h) - lit.current(vce - h)) / (2.0 * h);
            assert!(
                (slope - numeric).abs() <= 1e-6 * numeric.abs().max(1e-9),
                "at {vce} V: {slope} against {numeric}"
            );
        }
    }

    #[test]
    fn far_reversed_it_carries_little_and_does_not_overflow() {
        let lit = Phototransistor::new(1000.0);
        let reversed = lit.current(-20.0);
        assert!(reversed.is_finite());
        assert!(reversed < 0.0 && reversed.abs() < lit.current(5.0) / 100.0);
    }
}
