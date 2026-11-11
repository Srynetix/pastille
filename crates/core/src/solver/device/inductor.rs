// SPDX-License-Identifier: MIT OR Apache-2.0

//! The inductor: the capacitor's mirror image, and it behaves like one.
//!
//! Where a capacitor resists a change in voltage, an inductor resists a change
//! in current, and `v = L di/dt` yields to the same treatment `i = C dv/dt`
//! does — a difference over one timestep, and a companion the matrix can hold.
//! The roles swap: the companion source carries a *current* the inductor
//! insists on continuing to pass, where the capacitor's carried a voltage it
//! insisted on continuing to hold.
//!
//! The mirror goes one step further than the companion. A capacitor is solved
//! for the thing it carries — its voltage is a node voltage — while an
//! inductor is solved for the *rate* it carries it at, since what the matrix
//! hands back is the voltage across it and `v/L` is di/dt. So the same
//! integration rule is read forwards for one and backwards for the other; both
//! directions live in [`Integration`] (C-08).

use crate::solver::{Charge, Companion, Integration};

/// An inductor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Inductor {
    pub henries: f64,
}

impl Inductor {
    /// The companion model for one step of `seconds`, given what the previous
    /// step left the inductor carrying.
    ///
    /// ```text
    /// i(t) = i(t−h)  +  (h/L) · v(t)
    ///        ╰──────╯     ╰─────────╯
    ///        a current    conductance
    ///        the source
    ///        supplies
    /// ```
    ///
    /// A longer step makes it a *better* conductor, which is the opposite of
    /// the capacitor and the right way round: given time, current in an
    /// inductor has time to change.
    ///
    /// Trapezoidal averages the two ends, which halves that conductance and
    /// adds to the remembered current the half-step the old voltage is still
    /// owed:
    ///
    /// ```text
    /// i(t) = [ i(t−h) + (h/2) · i̇(t−h) ]  +  (h/2L) · v(t)
    /// ```
    pub fn companion(&self, method: Integration, seconds: f64, previous: Charge) -> Companion {
        let (conductance, remembered) = match method.usable(previous) {
            Integration::BackwardEuler => (seconds / self.henries, previous.value),
            Integration::Trapezoidal => (
                seconds / (2.0 * self.henries),
                previous.value + seconds * previous.rate() / 2.0,
            ),
        };

        Companion {
            conductance,
            current: remembered,
            point: previous.value,
        }
    }

    /// What the inductor carries out of a step that ended with `volts` across
    /// it.
    ///
    /// The rate is the easy half — `v/L` is di/dt by definition — and the
    /// current it adds up to is what the rule has to be read backwards for.
    pub fn advance(
        &self,
        method: Integration,
        seconds: f64,
        volts: f64,
        previous: Charge,
    ) -> Charge {
        let slope = volts / self.henries;
        Charge {
            value: method.value_after(seconds, slope, previous),
            slope: Some(slope),
            latched: false,
            held: [0.0; 2],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE_MILLIHENRY: Inductor = Inductor { henries: 1e-3 };

    #[test]
    fn an_inductor_carrying_nothing_is_a_plain_conductance() {
        let companion =
            ONE_MILLIHENRY.companion(Integration::BackwardEuler, 1e-6, Charge::placed(0.0));

        assert_eq!(companion.conductance, 1e-3);
        assert_eq!(companion.current, 0.0);
    }

    #[test]
    fn an_inductor_carrying_current_keeps_pushing_it() {
        // The whole of "an inductor resists a change in current": whatever it
        // was passing, it goes on passing into the next step.
        let companion =
            ONE_MILLIHENRY.companion(Integration::BackwardEuler, 1e-6, Charge::settled(0.25));

        assert_eq!(companion.current, 0.25);
    }

    #[test]
    fn a_shorter_step_looks_stiffer() {
        // Mirror of the capacitor: less time means less room for the current
        // to change, which is a worse conductor rather than a better one.
        let slow = ONE_MILLIHENRY.companion(Integration::BackwardEuler, 1e-3, Charge::placed(0.0));
        let fast = ONE_MILLIHENRY.companion(Integration::BackwardEuler, 1e-6, Charge::placed(0.0));

        assert!(fast.conductance < slow.conductance);
    }

    #[test]
    fn trapezoidal_is_half_as_conductive_over_the_same_step() {
        let euler =
            ONE_MILLIHENRY.companion(Integration::BackwardEuler, 1e-6, Charge::settled(0.0));
        let trapezoid =
            ONE_MILLIHENRY.companion(Integration::Trapezoidal, 1e-6, Charge::settled(0.0));

        assert_eq!(trapezoid.conductance, euler.conductance / 2.0);
    }

    #[test]
    fn trapezoidal_carries_the_voltage_it_was_under_as_well() {
        // Passing 0.25 A and, at that moment, climbing at 1000 A/s — which is
        // 1 V across a millihenry. Half a microsecond of that is 0.5 mA more.
        let previous = Charge {
            value: 0.25,
            slope: Some(1e3),
            latched: false,
            held: [0.0; 2],
        };
        let companion = ONE_MILLIHENRY.companion(Integration::Trapezoidal, 1e-6, previous);

        assert!(
            (companion.current - 0.2505).abs() < 1e-12,
            "{}",
            companion.current
        );
    }

    #[test]
    fn no_voltage_across_it_means_no_change_in_current() {
        let carried =
            ONE_MILLIHENRY.advance(Integration::BackwardEuler, 1e-6, 0.0, Charge::settled(0.25));

        assert_eq!(carried.value, 0.25);
        assert_eq!(carried.slope, Some(0.0));
    }

    #[test]
    fn current_follows_the_voltage_across_it() {
        // 1 V across 1 mH for 1 ms is 1 A, by v = L di/dt.
        let carried =
            ONE_MILLIHENRY.advance(Integration::BackwardEuler, 1e-3, 1.0, Charge::placed(0.0));

        assert!((carried.value - 1.0).abs() < 1e-12, "{}", carried.value);
    }
}
