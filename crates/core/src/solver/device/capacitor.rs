// SPDX-License-Identifier: MIT OR Apache-2.0

//! The capacitor, as the solver sees it: a conductance and a current source.
//!
//! A capacitor is the first element whose behaviour depends on *when* you ask.
//! `i = C dv/dt` is a differential equation, and an MNA matrix holds algebra,
//! so the derivative is replaced by a difference over one timestep. The result
//! — a companion model — is a resistor in parallel with a current source that
//! remembers the last step, which is something the matrix can hold.
//!
//! Which difference, and what it costs, is [`Integration`]: backward Euler by
//! default because it damps rather than rings, trapezoidal on request because
//! it is second order and keeps a resonant circuit resonating (C-08). The
//! formulations are the standard ones from the published literature (Vlach &
//! Singhal, Najm), not from any GPL source — see ADR 0001.

use crate::solver::{Charge, Companion, Integration};

/// A capacitor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Capacitor {
    pub farads: f64,
}

impl Capacitor {
    /// The companion model for one step of `seconds`, given what the previous
    /// step left the capacitor holding.
    ///
    /// Backward Euler evaluates the derivative at the *end* of the step:
    ///
    /// ```text
    /// i(t) = C · (v(t) − v(t−h)) / h
    ///      = (C/h) · v(t)  −  (C/h) · v(t−h)
    ///        ╰────────╯       ╰────────────╯
    ///        conductance      a current the companion source supplies
    /// ```
    ///
    /// So a charged capacitor enters the step looking like a small resistor
    /// with a battery behind it, which is exactly what a charged capacitor is
    /// over a short enough interval.
    ///
    /// Trapezoidal averages the derivative over both ends, which stiffens the
    /// conductance to `2C/h` and gives the source a second thing to remember —
    /// the current the capacitor was passing when the step began:
    ///
    /// ```text
    /// i(t) = (2C/h) · v(t)  −  [ (2C/h) · v(t−h)  +  i(t−h) ]
    /// ```
    pub fn companion(&self, method: Integration, seconds: f64, previous: Charge) -> Companion {
        let (conductance, remembered) = match method.usable(previous) {
            Integration::BackwardEuler => {
                let conductance = self.farads / seconds;
                (conductance, conductance * previous.value)
            }
            Integration::Trapezoidal => {
                let conductance = 2.0 * self.farads / seconds;
                // C · v̇(t−h) is the current it was passing, which the average
                // carries into this step alongside the voltage.
                (
                    conductance,
                    conductance * previous.value + self.farads * previous.rate(),
                )
            }
        };

        Companion {
            conductance,
            // Negative because `Companion::current` is measured from `+` to `-`
            // through the source, and this one pushes the other way: it is what
            // keeps the capacitor's voltage where the last step left it.
            current: -remembered,
            point: previous.value,
        }
    }

    /// What the capacitor carries out of a step that ended with `volts` across
    /// it.
    ///
    /// A capacitor is solved for its value — the voltage is a node voltage the
    /// matrix hands back — so what is left to work out is how fast it was
    /// changing when the step ended, which is the half trapezoidal needs next
    /// time.
    pub fn advance(
        &self,
        method: Integration,
        seconds: f64,
        volts: f64,
        previous: Charge,
    ) -> Charge {
        Charge {
            value: volts,
            slope: Some(method.slope_after(seconds, volts, previous)),
            latched: false,
            held: [0.0; 2],
        }
    }

    /// Current through the capacitor, from what it is carrying: `i = C dv/dt`,
    /// read literally.
    ///
    /// Zero before the first step of a run, where the rate is not known yet
    /// and no current has been solved for that could contradict it.
    pub fn current(&self, charge: Charge) -> f64 {
        self.farads * charge.rate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE_MICROFARAD: Capacitor = Capacitor { farads: 1e-6 };

    #[test]
    fn an_uncharged_capacitor_is_a_plain_conductance() {
        // Nothing remembered means nothing pushed: at v(t−h) = 0 the companion
        // source is silent and the capacitor is C/h siemens and no more.
        let companion =
            ONE_MICROFARAD.companion(Integration::BackwardEuler, 1e-3, Charge::placed(0.0));

        assert_eq!(companion.conductance, 1e-3);
        assert_eq!(companion.current, 0.0);
    }

    #[test]
    fn a_charged_capacitor_pushes_back() {
        let companion =
            ONE_MICROFARAD.companion(Integration::BackwardEuler, 1e-3, Charge::settled(5.0));

        assert_eq!(companion.conductance, 1e-3);
        assert_eq!(companion.current, -5e-3);
    }

    #[test]
    fn a_shorter_step_looks_stiffer() {
        // The shorter the step, the less the voltage can move within it, which
        // is the whole of "a capacitor resists a change in voltage".
        let slow = ONE_MICROFARAD.companion(Integration::BackwardEuler, 1e-3, Charge::placed(0.0));
        let fast = ONE_MICROFARAD.companion(Integration::BackwardEuler, 1e-6, Charge::placed(0.0));

        assert!(fast.conductance > slow.conductance);
    }

    #[test]
    fn trapezoidal_is_twice_as_stiff_over_the_same_step() {
        // Averaging the two ends means only half the step is left for the
        // voltage to move in, and the conductance says so.
        let euler =
            ONE_MICROFARAD.companion(Integration::BackwardEuler, 1e-3, Charge::settled(0.0));
        let trapezoid =
            ONE_MICROFARAD.companion(Integration::Trapezoidal, 1e-3, Charge::settled(0.0));

        assert_eq!(trapezoid.conductance, 2.0 * euler.conductance);
    }

    #[test]
    fn a_capacitor_with_no_step_behind_it_is_stamped_by_backward_euler() {
        // Whatever was asked for. Nothing has solved this circuit yet, so
        // there is no rate to average against (C-08).
        let asked = ONE_MICROFARAD.companion(Integration::Trapezoidal, 1e-3, Charge::placed(2.0));
        let euler = ONE_MICROFARAD.companion(Integration::BackwardEuler, 1e-3, Charge::placed(2.0));

        assert_eq!(asked.conductance, euler.conductance);
        assert_eq!(asked.current, euler.current);
    }

    #[test]
    fn trapezoidal_remembers_the_current_as_well_as_the_voltage() {
        // Charged to 5 V and, at that moment, passing 1 mA: the source carries
        // (2C/h)·5 = 10 mA for the voltage and the 1 mA on top of it.
        let previous = Charge {
            value: 5.0,
            slope: Some(1e3), // 1 mA through 1 µF is a thousand volts a second.
            latched: false,
            held: [0.0; 2],
        };
        let companion = ONE_MICROFARAD.companion(Integration::Trapezoidal, 1e-3, previous);

        assert!(
            (companion.current + 11e-3).abs() < 1e-15,
            "{}",
            companion.current
        );
    }

    #[test]
    fn a_capacitor_holding_still_passes_nothing() {
        let held =
            ONE_MICROFARAD.advance(Integration::BackwardEuler, 1e-3, 5.0, Charge::settled(5.0));

        assert_eq!(ONE_MICROFARAD.current(held), 0.0);
    }

    #[test]
    fn current_follows_the_rate_of_change() {
        // 1 µF charging 1 V in 1 ms is 1 mA, by i = C dv/dt.
        let charged =
            ONE_MICROFARAD.advance(Integration::BackwardEuler, 1e-3, 1.0, Charge::placed(0.0));

        assert_eq!(charged.value, 1.0);
        assert!((ONE_MICROFARAD.current(charged) - 1e-3).abs() < 1e-15);
    }
}
