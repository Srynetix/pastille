// SPDX-License-Identifier: MIT OR Apache-2.0

//! The signal generator: a source whose voltage changes with time (ADR 0011).
//!
//! Three shapes, each swinging symmetrically between −A and +A about the `−`
//! terminal, because alternating is the idea the part exists to show. Nothing
//! here is stateful: the voltage is a function of the time the solver asks
//! about, which is what lets a rejected step be taken again shorter without
//! anything having moved on (C-10).
//!
//! A square wave jumps, and a jump has two values at one instant. The solver
//! never takes a step across a jump — [`next_breakpoint`] is where it stops —
//! so a step ending exactly on one wants the value from *before* it, and the
//! step after wants the value from after. Hence [`Waveform::before`] and
//! [`Waveform::after`], which differ only there.

use crate::netlist::{ElementKind, Netlist};

/// How close to an edge, in cycles, counts as being on it.
///
/// A step clipped to land on an edge arrives there by floating-point
/// arithmetic, a few ulps either side. A billionth of a cycle is far above
/// that and far below any step the solver would take.
const ON_EDGE: f64 = 1e-9;

/// The shape of a signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Waveform {
    /// High for the first half of each cycle, low for the second.
    Square,
    /// The smooth one, starting at zero and rising.
    Sine,
    /// Straight lines between the peaks, starting at zero and rising.
    Triangle,
}

impl Waveform {
    /// Every shape, in the order a chooser should offer them: the one an LED
    /// shows most plainly first.
    pub const ALL: &'static [Waveform] = &[Waveform::Square, Waveform::Sine, Waveform::Triangle];

    /// The name it crosses the boundary under.
    pub fn name(&self) -> &'static str {
        match self {
            Waveform::Square => "Square",
            Waveform::Sine => "Sine",
            Waveform::Triangle => "Triangle",
        }
    }

    /// The shape of that name, or nothing if it is not one.
    pub fn named(name: &str) -> Option<Waveform> {
        Waveform::ALL
            .iter()
            .copied()
            .find(|waveform| waveform.name() == name)
    }

    /// Between −1 and +1, `cycles` of the way through the signal.
    fn unit(&self, cycles: f64) -> f64 {
        let phase = cycles - cycles.floor();
        match self {
            Waveform::Square => {
                if phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Waveform::Sine => (core::f64::consts::TAU * cycles).sin(),
            Waveform::Triangle => {
                if phase < 0.25 {
                    4.0 * phase
                } else if phase < 0.75 {
                    2.0 - 4.0 * phase
                } else {
                    4.0 * phase - 4.0
                }
            }
        }
    }

    /// The value just before `seconds`: what a step ending there has seen.
    pub fn before(&self, amplitude: f64, hertz: f64, seconds: f64) -> f64 {
        amplitude * self.unit(seconds * hertz - ON_EDGE)
    }

    /// The value just after `seconds`: what a run starting there begins with.
    pub fn after(&self, amplitude: f64, hertz: f64, seconds: f64) -> f64 {
        amplitude * self.unit(seconds * hertz + ON_EDGE)
    }

    /// The first moment after `seconds` where the signal jumps or turns a
    /// corner, or nothing for a shape that does neither.
    pub fn breakpoint_after(&self, hertz: f64, seconds: f64) -> Option<f64> {
        if !(hertz.is_finite() && hertz > 0.0) {
            return None;
        }
        let cycles = seconds * hertz + ON_EDGE;
        let next = match self {
            // Every half cycle.
            Waveform::Square => ((cycles * 2.0).floor() + 1.0) / 2.0,
            // The peaks, a quarter and three quarters of the way through.
            Waveform::Triangle => ((cycles * 2.0 - 0.5).floor() + 1.5) / 2.0,
            Waveform::Sine => return None,
        };
        Some(next / hertz)
    }
}

/// The first jump or corner of any signal in the circuit after `seconds`.
///
/// What the stepper clips each step to, so no step straddles an edge
/// (ADR 0011, term 5).
pub fn next_breakpoint(netlist: &Netlist, seconds: f64) -> Option<f64> {
    netlist
        .elements()
        .filter_map(|(_, element)| match element.kind {
            ElementKind::SignalGenerator {
                waveform, hertz, ..
            } => waveform.breakpoint_after(hertz, seconds),
            _ => None,
        })
        .reduce(f64::min)
}

/// Whether anything in the circuit changes with time on its own.
pub fn has_signal(netlist: &Netlist) -> bool {
    netlist
        .elements()
        .any(|(_, element)| matches!(element.kind, ElementKind::SignalGenerator { .. }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_shape_is_found_by_its_own_name() {
        for waveform in Waveform::ALL {
            assert_eq!(Waveform::named(waveform.name()), Some(*waveform));
        }
        assert_eq!(Waveform::named("Sawtooth"), None);
    }

    #[test]
    fn a_square_wave_is_high_then_low_and_takes_a_side_at_the_edge() {
        let square = Waveform::Square;
        // 1 Hz, 5 V: high for the first half second, low for the second.
        assert_eq!(square.after(5.0, 1.0, 0.0), 5.0, "it starts high");
        assert_eq!(square.before(5.0, 1.0, 0.25), 5.0);
        assert_eq!(square.before(5.0, 1.0, 0.75), -5.0);
        // At the edge, before is the half that is ending and after the one
        // that is starting.
        assert_eq!(square.before(5.0, 1.0, 0.5), 5.0);
        assert_eq!(square.after(5.0, 1.0, 0.5), -5.0);
        assert_eq!(square.before(5.0, 1.0, 1.0), -5.0);
        assert_eq!(square.after(5.0, 1.0, 1.0), 5.0);
    }

    #[test]
    fn a_sine_and_a_triangle_peak_at_a_quarter_cycle() {
        for waveform in [Waveform::Sine, Waveform::Triangle] {
            assert!(
                waveform.after(5.0, 2.0, 0.0).abs() < 1e-6,
                "{waveform:?} starts at 0"
            );
            assert!(
                (waveform.before(5.0, 2.0, 0.125) - 5.0).abs() < 1e-6,
                "{waveform:?}"
            );
            assert!(
                (waveform.before(5.0, 2.0, 0.375) + 5.0).abs() < 1e-6,
                "{waveform:?}"
            );
        }
    }

    #[test]
    fn a_breakpoint_is_the_next_edge_and_never_the_one_it_is_on() {
        let square = Waveform::Square;
        assert_eq!(square.breakpoint_after(1.0, 0.0), Some(0.5));
        assert_eq!(square.breakpoint_after(1.0, 0.2), Some(0.5));
        // Sitting on an edge — as a step clipped to it does — the next one.
        assert_eq!(square.breakpoint_after(1.0, 0.5), Some(1.0));
        assert_eq!(square.breakpoint_after(1.0, 0.5 - 1e-15), Some(1.0));

        let triangle = Waveform::Triangle;
        assert_eq!(triangle.breakpoint_after(1.0, 0.0), Some(0.25));
        assert_eq!(triangle.breakpoint_after(1.0, 0.25), Some(0.75));
        assert_eq!(triangle.breakpoint_after(1.0, 0.8), Some(1.25));

        assert_eq!(Waveform::Sine.breakpoint_after(1.0, 0.3), None);
    }
}
