// SPDX-License-Identifier: MIT OR Apache-2.0

//! How a step turns a derivative into algebra, and what it costs either way.
//!
//! A capacitor obeys `i = C dv/dt` and an inductor `v = L di/dt`. An MNA matrix
//! holds algebra and no derivatives at all, so every step replaces the
//! derivative with a difference over the step — and *which* difference is a
//! choice with consequences. Two are offered, selected per analysis (C-08).
//!
//! **Backward Euler** reads the rate at the far end of the step alone. It is
//! first order: halve the step and the error halves. Its amplification factor
//! never goes negative and shrinks towards zero as the step grows, so a step
//! that is far too long produces an answer that sags towards the settled one
//! rather than one that oscillates. The error it makes looks exactly like
//! resistance the circuit does not have: an LC tank that should ring for ever
//! rings down to nothing.
//!
//! **Trapezoidal** averages the rate at both ends. It is second order: halve
//! the step and the error quarters, so the same accuracy is had from a longer
//! step, and on an LC tank it holds the amplitude where backward Euler bleeds
//! it away. The price is the other half of the same coin — past a step of
//! twice the circuit's shortest time constant its amplification factor turns
//! negative, and the answer alternates above and below the truth. That
//! ringing belongs to the integrator, not to the circuit, and no shorter
//! time constant in the circuit announces itself first.
//!
//! **Backward Euler is the default, and stays the default**, for the reason
//! C-06 first chose it: a beginner cannot tell an artefact from a component.
//! Damping understates something that is really there, which is a reading;
//! ringing invents something that is not, which is a lie about their circuit.
//! Trapezoidal is for the person who knows which of the two they are looking
//! at — and for a resonant circuit, where the damping is the bigger lie.
//!
//! Both behaviours are demonstrated rather than asserted: see
//! `an_lc_tank_rings_on_under_trapezoidal_and_dies_under_backward_euler` and
//! `trapezoidal_rings_on_a_step_where_backward_euler_does_not` in
//! `solver::transient`.
//!
//! Both formulations are the standard ones from the published literature
//! (Vlach & Singhal, Najm), not from any GPL source — see ADR 0001.

/// Which difference a step puts in place of the derivative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Integration {
    /// The rate at the end of the step. Stable, damping, first order — the
    /// default, and the one whose errors understate rather than invent.
    #[default]
    BackwardEuler,
    /// The rate averaged over both ends. Second order, and it rings when the
    /// step outruns the circuit.
    Trapezoidal,
}

impl Integration {
    /// The rule this step can actually be taken by, given what the step before
    /// it left behind.
    ///
    /// Trapezoidal needs the rate the previous step ended at, and at the start
    /// of a run there was no previous step: an element placed holding a volt
    /// says nothing about how fast that volt is about to change, because the
    /// circuit around it has not been solved yet. Backward Euler asks for no
    /// such thing, so the first step of a run is taken by backward Euler and
    /// the rest by whichever rule was asked for. SPICE does the same at t = 0,
    /// for the same reason.
    ///
    /// Guessing zero instead is worse than it sounds: it is the claim that a
    /// capacitor just connected to a battery is not charging, and the error it
    /// makes decays only as fast as the circuit does. It was enough to make
    /// trapezoidal the *less* accurate of the two over the first few time
    /// constants of an RC charging curve, which is most of what anyone watches.
    pub fn usable(self, previous: Charge) -> Integration {
        match previous.slope {
            Some(_) => self,
            None => Integration::BackwardEuler,
        }
    }

    /// How fast the error falls as the step shortens: halving the step divides
    /// the error by two to this power.
    ///
    /// Read by the step-doubling estimator, which compares one long step
    /// against two short ones and has to know what the gap between them means
    /// (C-10).
    pub fn order(self) -> u32 {
        match self {
            Integration::BackwardEuler => 1,
            Integration::Trapezoidal => 2,
        }
    }

    /// The rate the value was changing at the end of a step that ended at
    /// `value` — a capacitor's dv/dt, read back once the step is solved.
    ///
    /// The integration rule read backwards: it says what the value at the end
    /// of the step is in terms of the rate there, so knowing the value says
    /// what the rate must have been.
    pub fn slope_after(self, seconds: f64, value: f64, previous: Charge) -> f64 {
        let difference = (value - previous.value) / seconds;
        match self.usable(previous) {
            Integration::BackwardEuler => difference,
            // v(t) = v(t−h) + h/2·(v̇(t) + v̇(t−h)), solved for v̇(t).
            Integration::Trapezoidal => 2.0 * difference - previous.rate(),
        }
    }

    /// The value at the end of a step whose rate ended at `slope` — an
    /// inductor's current, which the solved point never carries directly.
    ///
    /// The mirror of [`slope_after`](Self::slope_after), and its exact
    /// inverse: an inductor is solved for its rate where a capacitor is solved
    /// for its value, so each device needs the rule pointing a different way.
    pub fn value_after(self, seconds: f64, slope: f64, previous: Charge) -> f64 {
        match self.usable(previous) {
            Integration::BackwardEuler => previous.value + seconds * slope,
            Integration::Trapezoidal => previous.value + seconds * (slope + previous.rate()) / 2.0,
        }
    }
}

/// What a reactive element carries out of one step and into the next.
///
/// Backward Euler needs only the value — the voltage a capacitor ended the
/// step holding, the current an inductor ended it passing. Trapezoidal needs
/// how fast that value was changing as well, because it averages the rate at
/// both ends of the step rather than reading it only at the far one.
///
/// The rate is stored as the rate of the value itself, volts or amps per
/// second, rather than as the current or voltage it comes from: it then means
/// the same thing for both devices, and the rule above can be written once.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Charge {
    /// A capacitor's voltage, an inductor's current.
    pub value: f64,
    /// How fast that was changing at the moment the step ended, or `None` when
    /// no step has run yet and nothing knows.
    ///
    /// Unknown rather than zero, because the two are different claims and only
    /// one of them is true of an element that has just been placed. Saying so
    /// costs one backward Euler step at the start of a run; guessing zero
    /// costs what [`Integration::usable`] describes.
    pub slope: Option<f64>,
    /// A bit carried beside it: a 555's output high (ADR 0012), a relay's
    /// contact thrown (C-64). False for everything that carries only a value.
    ///
    /// Its own field rather than folded into `value`, because a relay carries
    /// both at once — the current in its coil, which is what an inductor
    /// carries, and where its contact is, which only hysteresis can say.
    pub latched: bool,
    /// Two numbers more, for a part whose state is more than a value and a
    /// bit: a servo's target angle, and how long its pulse has been high
    /// (C-90); a CD4017's, CD4013's or CD4026's supply, which sets its
    /// outputs' strength (C-124, C-125). Zero for everything else.
    ///
    /// Here rather than beside the run, because what a servo is doing decides
    /// what it draws from its supply, which is a stamp — and stamps read only
    /// what the run carries, so a rejected step takes it back with the rest.
    pub held: [f64; 2],
}

impl Charge {
    /// An element placed holding `value`, in a circuit nothing has solved yet:
    /// the rate is not known and will not be until a step has run.
    pub const fn placed(value: f64) -> Charge {
        Charge {
            value,
            slope: None,
            latched: false,
            held: [0.0; 2],
        }
    }

    /// An element in a circuit that has settled at its operating point, where
    /// the rate is known and is zero — that is what settled means.
    pub const fn settled(value: f64) -> Charge {
        Charge {
            value,
            slope: Some(0.0),
            latched: false,
            held: [0.0; 2],
        }
    }

    /// A bit and nothing else, which is all a 555 carries.
    pub const fn bit(latched: bool) -> Charge {
        Charge {
            value: 0.0,
            slope: Some(0.0),
            latched,
            held: [0.0; 2],
        }
    }

    /// The rate, treating "not known" as "not moving".
    ///
    /// The rules only reach this through [`Integration::usable`], which sends
    /// a step with nothing behind it to backward Euler — the rule that never
    /// asks. Elsewhere it means "as far as anything knows, nothing is moving".
    pub(crate) fn rate(self) -> f64 {
        self.slope.unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STEP: f64 = 1e-6;

    #[test]
    fn backward_euler_reads_the_rate_at_the_end_of_the_step() {
        // One volt gained over a microsecond is a million volts a second,
        // however the step began.
        let slope = Integration::BackwardEuler.slope_after(STEP, 1.0, Charge::settled(0.0));

        assert!((slope - 1e6).abs() < 1.0, "{slope} V/s");
    }

    #[test]
    fn trapezoidal_reads_the_rate_the_average_implies() {
        // The same volt gained over the same microsecond, so the average rate
        // is the same million volts a second — but this step began climbing at
        // twice that, and an average of a million means it ended flat.
        let previous = Charge {
            value: 0.0,
            slope: Some(2e6),
            latched: false,
            held: [0.0; 2],
        };
        let slope = Integration::Trapezoidal.slope_after(STEP, 1.0, previous);

        assert!(slope.abs() < 1.0, "{slope} V/s");
    }

    #[test]
    fn the_two_directions_of_each_rule_undo_each_other() {
        // A capacitor is solved for its value and an inductor for its rate, so
        // both directions are in use at once and a rule that did not invert
        // would have the two devices integrating differently.
        let previous = Charge {
            value: 0.3,
            slope: Some(-2e5),
            latched: false,
            held: [0.0; 2],
        };

        for method in [Integration::BackwardEuler, Integration::Trapezoidal] {
            let value = 0.7;
            let slope = method.slope_after(STEP, value, previous);
            let back = method.value_after(STEP, slope, previous);

            assert!(
                (back - value).abs() < 1e-12,
                "{method:?}: {back} vs {value}"
            );
        }
    }

    #[test]
    fn an_element_nothing_has_solved_yet_has_no_rate_to_give() {
        assert_eq!(Charge::placed(5.0).slope, None);
        // Which is what an element nobody mentioned starts as, too: never
        // charged is still never solved.
        assert_eq!(Charge::default(), Charge::placed(0.0));
        assert_eq!(Charge::settled(5.0).slope, Some(0.0));
    }

    #[test]
    fn the_first_step_of_a_run_is_taken_by_backward_euler() {
        // Whatever was asked for: there is no rate behind it to average
        // against, and inventing one is what makes the first step wrong.
        let nothing_behind_it = Charge::placed(0.0);

        assert_eq!(
            Integration::Trapezoidal.usable(nothing_behind_it),
            Integration::BackwardEuler
        );
        assert_eq!(
            Integration::Trapezoidal.usable(Charge::settled(0.0)),
            Integration::Trapezoidal
        );
    }
}
