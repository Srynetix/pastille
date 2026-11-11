// SPDX-License-Identifier: MIT OR Apache-2.0

//! A transformer: two windings on one core, and nothing else between them
//! (C-118).
//!
//! Four pins, two a winding, and no conductor from one winding to the other:
//! its two sides are separate circuits, each with its own reference, as a
//! relay's coil and contacts are (C-92). What joins them is the core's flux,
//! which is two inductors coupled:
//!
//! ```text
//! v₁ = R₁·i₁ + L₁·di₁/dt + M·di₂/dt
//! v₂ = R₂·i₂ + M·di₁/dt  + L₂·di₂/dt        M = k·√(L₁L₂)
//! ```
//!
//! with both currents into each winding's first pin. With L₂ = L₁/n² a sine
//! across the primary appears n times smaller across the secondary, and a
//! current drawn from the secondary is n times smaller in the primary — the
//! turns ratio both ways, out of the flux rather than written in.
//!
//! **The companion.** A step replaces the two derivatives with differences,
//! as an inductor's does (C-07), and the two equations solved together give
//! each winding's current from *both* voltages: a 2×2 conductance and a pair
//! of currents the step remembers. The off-diagonal terms are the coupling —
//! the voltage on one winding driving current in the other — which is the
//! mutual-inductance stamp nothing needed before this. Backward Euler, with
//! L the 2×2 inductance and h the step:
//!
//! ```text
//! (L/h + R)·i = v + (L/h)·i(t−h)
//! ```
//!
//! and trapezoidal, averaging both ends:
//!
//! ```text
//! (L/h + R/2)·i = v/2 + v(t−h)/2 + (L/h − R/2)·i(t−h)
//! ```
//!
//! At DC the flux does not change and each winding is its own copper, so a
//! battery on the primary drives its resistance's current and nothing comes
//! out of the secondary — which is what a real one does, and why a
//! transformer is no use on a battery.
//!
//! **Coupled perfectly.** With k = 1 the inductance matrix has no inverse —
//! all the flux is shared, so the two currents cannot be told apart from it
//! alone — but L/h + R always has one, since the copper sits on its
//! diagonal, so the companion is still well made. And each winding's rate is
//! read back by the integration rule rather than through L⁻¹. A real core
//! leaks a little of its flux, and that leakage is an inductance in series
//! with the secondary: tried at k = 0.999, it had to stop dead each time a
//! bridge after it stopped conducting, a kick with nowhere to go but the
//! diodes' leak, and the solver spent a frame's budget on every half cycle
//! chasing it. Leaving it out costs a little of each charging pulse's spread,
//! which a beginner does not see; the ripple the scope shows comes out at most
//! the textbook's I/(2f·C).
//!
//! **Not from a sheet.** A small mains transformer's sheet gives its VA, its
//! voltages and its regulation; none gives its inductances. These are a
//! 2 VA PCB transformer's, typical rather than any one part's, and said:
//! a primary of 10 H — a magnetising current of a few milliamps at 50 Hz from
//! 24 V, under a tenth of its full-load current — and 10 Ω of primary copper,
//! the secondary's scaled by n² so each side loses the same. What it ignores:
//! leakage, the core's saturation and its losses, the windings' capacitance,
//! and every rating — there is no mains here (see the example's note).

use crate::solver::{Charge, Integration};

/// The primary's inductance, in henries.
pub const PRIMARY_HENRIES: f64 = 10.0;

/// How much of each winding's flux the other shares: all of it.
pub const COUPLING: f64 = 1.0;

/// The primary's copper, in ohms; the secondary's is this over n².
pub const PRIMARY_OHMS: f64 = 10.0;

/// The ratios it can be wound to, as turns on the primary for one on the
/// secondary: below one steps up.
pub const FEWEST: f64 = 0.1;
pub const MOST: f64 = 100.0;

/// Pins: the primary's two ends, then the secondary's, each first pin the
/// dotted one.
pub const P1: usize = 0;
pub const P2: usize = 1;
pub const S1: usize = 2;
pub const S2: usize = 3;

/// The two windings, nothing conducting between them.
pub const SIDES: [&[usize]; 2] = [&[P1, P2], &[S1, S2]];

/// A 2×2 matrix, row-major.
type Matrix = [[f64; 2]; 2];

fn inverse(m: Matrix) -> Matrix {
    let det = m[0][0] * m[1][1] - m[0][1] * m[1][0];
    [
        [m[1][1] / det, -m[0][1] / det],
        [-m[1][0] / det, m[0][0] / det],
    ]
}

fn times(m: Matrix, v: [f64; 2]) -> [f64; 2] {
    [
        m[0][0] * v[0] + m[0][1] * v[1],
        m[1][0] * v[0] + m[1][1] * v[1],
    ]
}

/// The two windings over one step, as the matrix holds them: the current into
/// each winding's first pin is `conductance · [v₁, v₂] + current`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Companion {
    pub conductance: Matrix,
    pub current: [f64; 2],
}

/// A transformer wound `ratio` to one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transformer {
    pub ratio: f64,
}

impl Transformer {
    fn inductance(&self) -> Matrix {
        let primary = PRIMARY_HENRIES;
        let secondary = primary / (self.ratio * self.ratio);
        let mutual = COUPLING * (primary * secondary).sqrt();
        [[primary, mutual], [mutual, secondary]]
    }

    /// Each winding's copper, in ohms.
    pub fn resistance(&self) -> [f64; 2] {
        [PRIMARY_OHMS, PRIMARY_OHMS / (self.ratio * self.ratio)]
    }

    /// The two currents a step starts from, and how fast each was changing.
    fn carried(previous: Charge) -> ([f64; 2], Option<[f64; 2]>) {
        let currents = [previous.value, previous.held[0]];
        (
            currents,
            previous.slope.map(|slope| [slope, previous.held[1]]),
        )
    }

    /// The windings over a step of `seconds` from what the last one left.
    pub fn companion(&self, method: Integration, seconds: f64, previous: Charge) -> Companion {
        let l = self.inductance();
        let r = self.resistance();
        let (was, rates) = Self::carried(previous);
        let over_h = |m: Matrix| m.map(|row| row.map(|x| x / seconds));
        let lh = over_h(l);
        match (method.usable(previous), rates) {
            (Integration::Trapezoidal, Some(rates)) => {
                let a = inverse([
                    [lh[0][0] + r[0] / 2.0, lh[0][1]],
                    [lh[1][0], lh[1][1] + r[1] / 2.0],
                ]);
                // What the windings were across, from what they carried and
                // how fast: v = R·i + L·di/dt.
                let flux = times(l, rates);
                let volts = [r[0] * was[0] + flux[0], r[1] * was[1] + flux[1]];
                let kept = times(
                    [
                        [lh[0][0] - r[0] / 2.0, lh[0][1]],
                        [lh[1][0], lh[1][1] - r[1] / 2.0],
                    ],
                    was,
                );
                Companion {
                    conductance: a.map(|row| row.map(|x| x / 2.0)),
                    current: times(a, [volts[0] / 2.0 + kept[0], volts[1] / 2.0 + kept[1]]),
                }
            }
            _ => {
                let a = inverse([[lh[0][0] + r[0], lh[0][1]], [lh[1][0], lh[1][1] + r[1]]]);
                Companion {
                    conductance: a,
                    current: times(a, times(lh, was)),
                }
            }
        }
    }

    /// What the windings carry out of a step that ended with `volts` across
    /// them, or out of a settled point when there was no step.
    pub fn advance(
        &self,
        step: Option<(Integration, f64)>,
        volts: [f64; 2],
        previous: Charge,
    ) -> Charge {
        let r = self.resistance();
        let Some((method, seconds)) = step else {
            // Settled: no flux changing, only copper.
            let mut settled = Charge::settled(volts[0] / r[0]);
            settled.held = [volts[1] / r[1], 0.0];
            return settled;
        };
        let companion = self.companion(method, seconds, previous);
        let flowing = times(companion.conductance, volts);
        let currents = [
            flowing[0] + companion.current[0],
            flowing[1] + companion.current[1],
        ];
        // Each winding's rate, by the rule the step was taken by read
        // backwards — L has no inverse to read it through.
        let secondary = Charge {
            value: previous.held[0],
            slope: previous.slope.map(|_| previous.held[1]),
            ..previous
        };
        let rates = [
            method.slope_after(seconds, currents[0], previous),
            method.slope_after(seconds, currents[1], secondary),
        ];
        Charge {
            value: currents[0],
            slope: Some(rates[0]),
            latched: false,
            held: [currents[1], rates[1]],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A sine of `peak` volts at `hertz` on the primary, the secondary across
    /// `load` ohms, run for `cycles` by `method`. Returns the secondary's peak
    /// over the last cycle and the primary's peak current over it.
    fn run(ratio: f64, peak: f64, load: f64, method: Integration) -> (f64, f64) {
        let transformer = Transformer { ratio };
        let hertz = 50.0;
        let steps_a_cycle = 400;
        let h = 1.0 / (hertz * steps_a_cycle as f64);
        let mut charge = Charge::default();
        let (mut out, mut drawn) = (0.0f64, 0.0f64);
        for step in 1..=(20 * steps_a_cycle) {
            let v1 = peak * (2.0 * core::f64::consts::PI * hertz * step as f64 * h).sin();
            let c = transformer.companion(method, h, charge);
            // The secondary's own row: i₂ = g₂₁v₁ + g₂₂v₂ + j₂ = −v₂/load.
            let v2 =
                -(c.conductance[1][0] * v1 + c.current[1]) / (c.conductance[1][1] + 1.0 / load);
            charge = transformer.advance(Some((method, h)), [v1, v2], charge);
            if step > 19 * steps_a_cycle {
                out = out.max(v2.abs());
                drawn = drawn.max(charge.value.abs());
            }
        }
        (out, drawn)
    }

    #[test]
    fn it_steps_a_sine_down_by_its_ratio() {
        for method in [Integration::BackwardEuler, Integration::Trapezoidal] {
            // Open, near enough: 1 MΩ.
            let (out, _) = run(2.0, 24.0, 1e6, method);
            assert!((out / 12.0 - 1.0).abs() < 0.01, "{method:?}: {out} V");
            let (out, _) = run(10.0, 24.0, 1e6, method);
            assert!((out / 2.4 - 1.0).abs() < 0.01, "{method:?}: {out} V");
        }
    }

    #[test]
    fn a_load_on_the_secondary_draws_its_ratio_s_share_from_the_primary() {
        // 100 Ω on the 12 V side: 120 mA there, so about 60 mA in the
        // primary — plus a few milliamps of its own magnetising current.
        let (out, drawn) = run(2.0, 24.0, 100.0, Integration::Trapezoidal);
        let secondary = out / 100.0;
        assert!(out > 11.0 && out < 12.0, "{out} V on 100 Ω");
        assert!(
            (drawn / (secondary / 2.0) - 1.0).abs() < 0.1,
            "{drawn} A drawn"
        );
    }

    #[test]
    fn at_dc_it_is_two_lengths_of_copper() {
        let settled = Transformer { ratio: 2.0 }.advance(None, [5.0, 0.0], Charge::default());
        assert!((settled.value - 0.5).abs() < 1e-12);
        assert_eq!(settled.held[0], 0.0);
    }

    #[test]
    fn a_steady_voltage_on_the_primary_gives_nothing_on_the_secondary() {
        // A battery switched on across it: a kick on the secondary while the
        // primary's current rises, then nothing.
        let transformer = Transformer { ratio: 2.0 };
        let h = 1e-3;
        let mut charge = Charge::default();
        let mut v2 = 0.0;
        for _ in 0..20_000 {
            let c = transformer.companion(Integration::BackwardEuler, h, charge);
            v2 = -(c.conductance[1][0] * 5.0 + c.current[1]) / (c.conductance[1][1] + 1e-3);
            charge = transformer.advance(Some((Integration::BackwardEuler, h)), [5.0, v2], charge);
        }
        assert!(v2.abs() < 1e-3, "{v2} V still on the secondary");
        assert!(
            (charge.value - 0.5).abs() < 1e-3,
            "{} A in the primary",
            charge.value
        );
    }
}
