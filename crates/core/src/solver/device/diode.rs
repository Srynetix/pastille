// SPDX-License-Identifier: MIT OR Apache-2.0

//! The Shockley diode, with the junction limiting that makes it converge.
//!
//! Reimplemented from the published SPICE formulation (Berkeley SPICE2/3, and
//! the standard treatments in Vlach & Singhal and Najm), not from any GPL
//! source — see `docs/adrs/0001-permissive-dual-licence.md`.

use crate::solver::{Companion, Nonlinear};

/// Thermal voltage kT/q at about 27 °C. Temperature sweeps are out of scope
/// (design doc §3), so this is a constant rather than a parameter.
pub const THERMAL_VOLTAGE: f64 = 0.025_852;

/// A pn junction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Diode {
    /// Reverse saturation current, in amps. Around 1e-14 for small signal
    /// silicon; an LED's is far smaller, which is what moves its forward drop.
    pub saturation_current: f64,
    /// Emission coefficient — 1 for an ideal junction, up to about 2 for real
    /// ones.
    pub emission_coefficient: f64,
}

impl Default for Diode {
    fn default() -> Self {
        Diode {
            saturation_current: 1e-14,
            emission_coefficient: 1.0,
        }
    }
}

/// A 1N5819 Schottky barrier rectifier (C-114).
///
/// From onsemi's 1N5817/D: at most 0.34 V at 0.1 A, 0.6 V at 1 A and 0.9 V at
/// 3 A. No single exponential follows all three — above an amp it is the
/// part's series resistance that climbs, which a junction alone does not
/// have — so this one passes through the 0.1 A point with an emission
/// coefficient of 2: 0.22 V at 10 mA, 0.46 V at 1 A, 0.52 V at 3 A, under
/// every most the sheet gives, against a 1N4148's 0.71 V at 10 mA. Its
/// saturation current, 140 µA, is also what it leaks backwards, inside the
/// sheet's 1 mA at 40 V: a Schottky leaks far more than a silicon junction,
/// and that is part of what it is.
pub const SCHOTTKY: Diode = Diode {
    saturation_current: 1.4e-4,
    emission_coefficient: 2.0,
};

impl Diode {
    /// Scaled thermal voltage n·Vt: the voltage change that multiplies current
    /// by e.
    fn scale(&self) -> f64 {
        self.emission_coefficient * THERMAL_VOLTAGE
    }

    /// Current through the junction at `v`, by the Shockley equation.
    pub fn current(&self, v: f64) -> f64 {
        self.saturation_current * ((v / self.scale()).exp() - 1.0)
    }

    /// Slope dI/dV at `v`.
    pub fn conductance(&self, v: f64) -> f64 {
        self.saturation_current / self.scale() * (v / self.scale()).exp()
    }

    /// The voltage above which the exponential starts to run away, and beyond
    /// which a raw Newton step is not to be trusted.
    fn critical_voltage(&self) -> f64 {
        let scale = self.scale();
        scale * (scale / (core::f64::consts::SQRT_2 * self.saturation_current)).ln()
    }

    /// Bounds a Newton step across the junction.
    ///
    /// Newton on an exponential overshoots violently: one unlucky step asks for
    /// the current at 90 V, `exp(90/0.026)` overflows, and the matrix fills with
    /// infinities. Limiting keeps the new point on the same side of the curve
    /// while moving it by an amount the linearisation can justify — the step is
    /// damped, never reversed, so the iteration still makes progress.
    pub(crate) fn limit(&self, v: f64, previous: f64) -> f64 {
        let scale = self.scale();
        let critical = self.critical_voltage();

        // Below the critical voltage, or moving by less than a couple of
        // e-foldings, the raw step is harmless.
        if v <= critical || (v - previous).abs() <= 2.0 * scale {
            return v;
        }

        if previous > 0.0 {
            // Move to the voltage whose current matches what the previous
            // linearisation predicted, which is a step the model can support.
            let arg = 1.0 + (v - previous) / scale;
            if arg > 0.0 {
                previous + scale * arg.ln()
            } else {
                critical
            }
        } else {
            // Coming up from zero or below, land on the exponential rather than
            // leaping past it.
            scale * (v / scale).ln()
        }
    }
}

impl Nonlinear for Diode {
    fn linearise(&self, voltage_across: f64, previous_point: f64) -> Companion {
        let v = self.limit(voltage_across, previous_point);
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
    use crate::netlist::{ElementKind, NetId, Netlist};
    use crate::solver::{Device, MnaSystem, NewtonError, NewtonOptions, assemble, solve_nonlinear};

    use super::*;

    /// The forward voltage a junction needs for `amps`.
    fn drop(diode: &Diode, amps: f64) -> f64 {
        diode.scale() * (amps / diode.saturation_current + 1.0).ln()
    }

    #[test]
    fn a_schottky_drops_under_every_most_its_sheet_gives() {
        // onsemi's 1N5819: at most 0.34 V at 0.1 A, 0.6 V at 1 A, 0.9 V at
        // 3 A — through the first and under the other two (C-114).
        assert!((drop(&SCHOTTKY, 0.1) - 0.34).abs() < 0.005);
        assert!(drop(&SCHOTTKY, 1.0) < 0.6);
        assert!(drop(&SCHOTTKY, 3.0) < 0.9);
        // And well under a 1N4148 at what an LED draws.
        assert!(drop(&Diode::default(), 0.01) - drop(&SCHOTTKY, 0.01) > 0.4);
    }

    /// A source through a series resistor into a diode to the reference — the
    /// clamp of the acceptance, and the circuit of the first milestone with the
    /// LED swapped for a plain junction.
    fn clamp(volts: f64, ohms: f64) -> (MnaSystem, usize) {
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts,
            series_ohms: 0.0,
        });
        let resistor = netlist.add_element(ElementKind::Resistor { ohms });

        let top = netlist.add_net();
        let middle = netlist.add_net();
        let bottom = netlist.add_net();
        netlist.connect_named(source, "+", top).unwrap();
        netlist.connect_named(resistor, "a", top).unwrap();
        netlist.connect_named(resistor, "b", middle).unwrap();
        netlist.connect_named(source, "-", bottom).unwrap();

        row_of(netlist, middle)
    }

    fn row_of(netlist: Netlist, net: NetId) -> (MnaSystem, usize) {
        let map = netlist.compile().unwrap();
        let system = assemble(&netlist, &map);
        let row = system.row_of_node(map.index_of(net).unwrap()).unwrap();
        (system, row)
    }

    fn solve_clamp(volts: f64, ohms: f64, diode: &Diode) -> (f64, usize) {
        let (system, row) = clamp(volts, ohms);
        let solution = solve_nonlinear(
            &system,
            &[Device {
                plus: Some(row),
                minus: None,
                model: diode,
            }],
            &NewtonOptions::default(),
        )
        .unwrap();
        (solution.x[row], solution.iterations)
    }

    #[test]
    fn a_clamp_lands_on_its_independently_computed_answer() {
        // 5 V through 1 kΩ into a 1e-14 A junction. The reference was found by
        // bisecting (5 − v)/1000 = Is·(exp(v/nVt) − 1) in 50-digit arithmetic,
        // so it does not come from this code.
        const EXPECTED_V: f64 = 0.692_543_633_180_531;
        const EXPECTED_I: f64 = 4.307_456_366_819_469e-3;

        let diode = Diode::default();
        let (v, _) = solve_clamp(5.0, 1_000.0, &diode);

        assert!((v - EXPECTED_V).abs() < 1e-9, "v = {v}");
        assert!(((5.0 - v) / 1_000.0 - EXPECTED_I).abs() < 1e-9);
    }

    #[test]
    fn the_forward_drop_barely_moves_when_the_supply_does() {
        // What makes a diode a clamp: four times the supply, and the drop moves
        // by tens of millivolts. Both references computed by bisection.
        let diode = Diode::default();
        let (low, _) = solve_clamp(3.3, 220.0, &diode);
        let (high, _) = solve_clamp(12.0, 220.0, &diode);

        assert!((low - 0.718_451_719_111_459).abs() < 1e-9, "low = {low}");
        assert!((high - 0.756_490_396_124_927).abs() < 1e-9, "high = {high}");
        assert!(high - low < 0.05);
    }

    #[test]
    fn a_violent_forward_drive_converges_instead_of_overflowing() {
        // 100 V across 1 Ω. Without limiting, an early Newton step asks for the
        // current at tens of volts, exp(40/0.026) overflows, and the matrix
        // fills with infinities — this is precisely the case limiting exists for.
        let diode = Diode::default();
        let (v, iterations) = solve_clamp(100.0, 1.0, &diode);

        assert!((v - 0.952_175_541_351_052).abs() < 1e-9, "v = {v}");
        assert!(v.is_finite());
        assert!(iterations < NewtonOptions::default().max_iterations);
    }

    #[test]
    fn reverse_bias_settles_at_the_saturation_current() {
        // Source reversed: the junction blocks, and all but a leakage current of
        // Is flows nowhere.
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts: -5.0,
            series_ohms: 0.0,
        });
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: 1_000.0 });

        let top = netlist.add_net();
        let middle = netlist.add_net();
        let bottom = netlist.add_net();
        netlist.connect_named(source, "+", top).unwrap();
        netlist.connect_named(resistor, "a", top).unwrap();
        netlist.connect_named(resistor, "b", middle).unwrap();
        netlist.connect_named(source, "-", bottom).unwrap();

        let (system, row) = row_of(netlist, middle);
        let diode = Diode::default();
        let solution = solve_nonlinear(
            &system,
            &[Device {
                plus: Some(row),
                minus: None,
                model: &diode,
            }],
            &NewtonOptions::default(),
        )
        .unwrap();

        let v = solution.x[row];
        // Almost the whole supply stands across the junction, and the current is
        // the saturation current to within a fraction of a percent.
        assert!(v < -4.99, "v = {v}");
        assert!((diode.current(v) + diode.saturation_current).abs() < 1e-16);
    }

    #[test]
    fn limiting_leaves_gentle_steps_alone() {
        // A device that damped every step would slow the whole solver down. Well
        // inside the exponential's tame region, the raw Newton point is returned
        // unchanged.
        let diode = Diode::default();
        let companion = diode.linearise(0.3, 0.29);

        assert_eq!(companion.point, 0.3);
    }

    #[test]
    fn limiting_damps_a_step_that_would_overflow() {
        let diode = Diode::default();
        let companion = diode.linearise(40.0, 0.7);

        assert!(companion.point < 1.0, "point = {}", companion.point);
        assert!(companion.point > 0.7, "the step is damped, not reversed");
        assert!(companion.conductance.is_finite());
        assert!(companion.current.is_finite());
    }

    #[test]
    fn a_limited_iteration_is_never_the_converged_one() {
        // The companion was built somewhere other than where it was asked
        // about, so the residual computed at x would be meaningless. The loop
        // has to keep going, which is what Companion::point reports.
        let diode = Diode::default();
        let asked_about = 40.0;

        assert_ne!(diode.linearise(asked_about, 0.7).point, asked_about);
    }

    #[test]
    fn a_higher_emission_coefficient_raises_the_drop() {
        // n = 2 doubles the voltage per decade of current, so the same circuit
        // sits noticeably higher up the curve.
        let ideal = Diode::default();
        let real = Diode {
            emission_coefficient: 2.0,
            ..Diode::default()
        };

        let (ideal_v, _) = solve_clamp(5.0, 1_000.0, &ideal);
        let (real_v, _) = solve_clamp(5.0, 1_000.0, &real);

        assert!(real_v > ideal_v * 1.5, "{real_v} vs {ideal_v}");
    }

    #[test]
    fn an_unreachable_supply_does_not_stall_the_solver() {
        // A tiny supply that never turns the junction on: the answer is almost
        // the whole supply across the diode, and it still converges.
        let diode = Diode::default();
        let (v, iterations) = solve_clamp(0.1, 1_000.0, &diode);

        assert!(v > 0.0 && v < 0.1, "v = {v}");
        assert!(iterations < NewtonOptions::default().max_iterations);
    }

    #[test]
    fn the_solver_reports_rather_than_hangs_when_it_cannot_converge() {
        // Sanity: the cap still applies with a real device in the loop.
        let (system, row) = clamp(5.0, 1_000.0);
        let diode = Diode::default();
        let error = solve_nonlinear(
            &system,
            &[Device {
                plus: Some(row),
                minus: None,
                model: &diode,
            }],
            &NewtonOptions {
                max_iterations: 1,
                ..NewtonOptions::default()
            },
        )
        .unwrap_err();

        assert!(matches!(error, NewtonError::NotConverged { .. }));
    }
}
