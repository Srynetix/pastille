// SPDX-License-Identifier: MIT OR Apache-2.0

//! The relay: a coil that, once it carries enough current, throws a contact.
//!
//! C-64. A single-pole, double-throw part: the common contact rests on NC and
//! is pulled over to NO while the coil is energised. Energised is a bit carried
//! from step to step, as a 555's latch is (ADR 0012), and it has the hysteresis
//! a real relay has: it pulls in at three quarters of its rated current and
//! does not let go until the coil is down to a tenth of it. Between those the
//! contact stays where it was.
//!
//! The coil is a resistance in series with an inductance (C-67), and that is
//! what a relay teaches beyond the click. Its current cannot stop at once, so
//! switched on it takes a millisecond or two to reach pull-in, and switched off
//! it goes on flowing into whatever the switch left it: with nothing to take
//! it, the voltage across the coil kicks far past the supply, and a diode
//! across the coil is what gives it somewhere to go. It is the coil's
//! *current* that decides the contact, as the magnet's pull does — the kick is
//! a large voltage the wrong way round, and reading the voltage would have it
//! throw the contact it is letting go of.
//!
//! How high the kick reaches is not something a datasheet says. A real one
//! puts its energy into the winding's capacitance, the iron's losses and the
//! breakdown of whatever switched it, none of which is modelled; a resistance
//! across the inductance stands in for them ([`LOSS`]), so the height is
//! the part's rather than the step's. What is honest is that it is many times
//! the supply without the diode, and a diode's drop above it with one. The
//! scope shows it lower than that when a frame's steps are too long to catch
//! a spike that lasts microseconds.
//!
//! Within a solve the bit is fixed and the contacts are linear: a closed one a
//! tenth of an ohm, an open one a leak of a gigaohm — the lesson of C-63, that
//! a part which can be off must still leave its node something to hold on to,
//! or the stepper pays for it. The coil is linear too, and stamped with the
//! rest of the matrix rather than here.
//!
//! What it honours: the coil's resistance and inductance; pull-in and
//! drop-out; which contact is made. What it ignores: the milliseconds a
//! contact takes to travel once the coil has let it, contact bounce, the
//! change in inductance as the armature closes, and what the kick breaks down
//! beyond the one resistance standing in for it.

use crate::solver::device::Inductor;
use crate::solver::{Charge, Companion, Coupled, CoupledCompanion, Integration, MAX_TERMINALS};

/// Terminal order, and the order the pins are named in.
pub const COIL_PLUS: usize = 0;
pub const COIL_MINUS: usize = 1;
pub const COM: usize = 2;
pub const NO: usize = 3;
pub const NC: usize = 4;

const PINS: usize = 5;

/// Which pins are on which side: the coil, and the contacts it throws, which
/// share no conductor with it — what a relay is for. Each its own circuit,
/// as an optocoupler's two are (C-92).
pub const SIDES: [&[usize]; 2] = [&[COIL_PLUS, COIL_MINUS], &[COM, NO, NC]];

/// A made contact, in ohms: what a relay's datasheet gives, a tenth of an ohm.
const CONTACT_OHMS: f64 = 0.1;

/// A broken one's leak, in siemens.
const OPEN_LEAK: f64 = 1e-9;

/// Which part: Songle's SRD, the relay on every Arduino relay board, by coil.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RelayPart {
    V5,
    V9,
    V12,
}

impl RelayPart {
    /// The 9 V coil first: the one a 9 V battery drives.
    pub const ALL: &'static [RelayPart] = &[RelayPart::V9, RelayPart::V5, RelayPart::V12];

    /// The name it crosses the boundary under: its coil voltage.
    pub fn name(&self) -> &'static str {
        match self {
            RelayPart::V5 => "5 V coil",
            RelayPart::V9 => "9 V coil",
            RelayPart::V12 => "12 V coil",
        }
    }

    pub fn named(name: &str) -> Option<RelayPart> {
        RelayPart::ALL
            .iter()
            .copied()
            .find(|part| part.name() == name)
    }

    /// What is printed on it.
    pub fn part_number(&self) -> &'static str {
        match self {
            RelayPart::V5 => "SRD-05VDC",
            RelayPart::V9 => "SRD-09VDC",
            RelayPart::V12 => "SRD-12VDC",
        }
    }

    /// The coil's rated voltage.
    pub fn volts(&self) -> f64 {
        match self {
            RelayPart::V5 => 5.0,
            RelayPart::V9 => 9.0,
            RelayPart::V12 => 12.0,
        }
    }

    /// The coil's resistance, from the datasheet's rated current.
    pub fn coil_ohms(&self) -> f64 {
        match self {
            RelayPart::V5 => 70.0,
            RelayPart::V9 => 200.0,
            RelayPart::V12 => 400.0,
        }
    }

    /// The coil's inductance, in henries.
    ///
    /// Not on Songle's datasheet, which gives no inductance at all. Taken from
    /// a time constant: a small relay's coil has an L/R of the order of a
    /// millisecond or two, and 1.5 ms for all three coils puts the 9 V one at
    /// 0.3 H. It sets how long the coil takes to pull in and how much it has
    /// to give back when it is switched off, and neither is a number a
    /// beginner's circuit depends on to better than a factor of two.
    pub fn henries(&self) -> f64 {
        TIME_CONSTANT * self.coil_ohms()
    }

    /// What the coil puts its current through once nothing else will take it.
    /// See [`LOSS`].
    pub fn loss_ohms(&self) -> f64 {
        LOSS * self.coil_ohms()
    }

    /// The coil's rated current: what its rated voltage drives through it.
    pub fn rated_amps(&self) -> f64 {
        self.volts() / self.coil_ohms()
    }

    /// The coil current it closes at: the datasheet's "must operate", 75 % of
    /// rated. Given as a voltage there, which at DC is the same thing.
    pub fn pull_in(&self) -> f64 {
        0.75 * self.rated_amps()
    }

    /// And the one it lets go at: "must release", 10 %.
    pub fn drop_out(&self) -> f64 {
        0.1 * self.rated_amps()
    }

    /// The coil over one step, as a Norton pair the matrix can stamp between
    /// its two pins: the current into `coil+` is `conductance · v + current`.
    ///
    /// The inductance's own companion, with [`loss_ohms`](Self::loss_ohms) across it and the
    /// winding's resistance in series, folded into one. For backward Euler
    /// that is roughly `L/h + R` while the step is long — a short step makes
    /// the coil look like a far larger resistance than its winding, which is
    /// what refusing a change in current means — and never more than the
    /// loss and the winding together, which is what gives the kick its height.
    pub fn coil(&self, method: Integration, seconds: f64, previous: Charge) -> Companion {
        let (inductance, share) = self.split(method, seconds, previous);
        Companion {
            conductance: (inductance.conductance + 1.0 / self.loss_ohms()) * share,
            current: inductance.current * share,
            point: previous.value,
        }
    }

    /// The inductance's companion alone, and the share of the voltage at the
    /// pins that reaches it past the winding.
    fn split(&self, method: Integration, seconds: f64, previous: Charge) -> (Companion, f64) {
        let inductor = Inductor {
            henries: self.henries(),
        };
        let alone = inductor.companion(method, seconds, previous);
        let across = alone.conductance + 1.0 / self.loss_ohms();
        (alone, 1.0 / (1.0 + across * self.coil_ohms()))
    }

    /// What the coil carries out of a step that ended with `volts` across its
    /// pins, or out of a settled point when there was no step.
    ///
    /// The current carried is the inductance's own, not what the pins pass:
    /// it is the one that cannot jump, and the one whose pull holds the
    /// contact. Through a kick the two part company — what the pins pass
    /// stops, and what the inductance carries goes round through the loss.
    pub fn advance(
        &self,
        step: Option<(Integration, f64)>,
        volts: f64,
        previous: Charge,
    ) -> Charge {
        let Some((method, seconds)) = step else {
            // At DC the inductance is a wire, the loss across it carries
            // nothing, and the winding is all there is.
            return Charge::settled(volts / self.coil_ohms());
        };
        let (inductance, share) = self.split(method, seconds, previous);
        // What the winding does not drop is across the inductance, and that
        // is its rate; the integration rule turns it back into the current
        // its companion says, which is the check that the two halves agree.
        let inner = volts * share - inductance.current * share * self.coil_ohms();
        let inductor = Inductor {
            henries: self.henries(),
        };
        let carried = inductor.advance(method, seconds, inner, previous);
        let expected = inductance.conductance * inner + inductance.current;
        debug_assert!(
            (carried.value - expected).abs() <= 1e-9 * (1.0 + expected.abs()),
            "the coil's companion and its integration disagree: {} A against {expected} A",
            carried.value
        );
        carried
    }
}

/// What a coil switched off puts its current through, as a multiple of its
/// winding's resistance.
///
/// A coil's current cannot stop, so it has to go somewhere: into the winding's
/// own capacitance, into the losses in its iron, into the breakdown of whatever
/// switched it. None of those is modelled, and without any of them the kick
/// would be as high as the step is short — a number the solver chose rather
/// than the part. This stands in for all of them, across the inductance where
/// it changes nothing at DC.
///
/// Three windings' worth puts the kick at three times the coil's rated voltage
/// above the supply — 36 V at the switch of a 9 V coil — dying in half a
/// millisecond. A real one reaches higher and dies sooner; this is the low end,
/// kept where the steps of a frame can still see it, and it is the difference
/// the diode makes rather than the exact height that there is to learn.
pub const LOSS: f64 = 3.0;

/// L/R for every coil, in seconds. See [`RelayPart::henries`].
const TIME_CONSTANT: f64 = 1.5e-3;

/// A relay, energised or not.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Relay {
    pub part: RelayPart,
    pub energised: bool,
    /// Which of its pins are on a net, in pin order. A relay is used with one
    /// of its two contacts left free as often as not, and a pin on no net has
    /// no row: stamping it would mean stamping to the reference, so nothing
    /// that touches it is stamped at all.
    pub wired: [bool; PINS],
}

/// How many pins it has.
pub const PIN_COUNT: usize = PINS;

/// Where the contact goes, given where it was and the current in its coil.
pub fn latch(part: RelayPart, was: bool, amps: f64) -> bool {
    // A coil does not mind which way its current goes: the pull is the same.
    let through = amps.abs();
    if through >= part.pull_in() {
        true
    } else if through <= part.drop_out() {
        false
    } else {
        was
    }
}

impl Coupled for Relay {
    fn linearise(
        &self,
        _potentials: &[f64; MAX_TERMINALS],
        _previous: &[f64; MAX_TERMINALS],
    ) -> CoupledCompanion {
        let mut conductances = [[0.0; MAX_TERMINALS]; MAX_TERMINALS];
        let wired = self.wired;
        let mut resistor = |a: usize, b: usize, siemens: f64| {
            if !(wired[a] && wired[b]) {
                return;
            }
            conductances[a][a] += siemens;
            conductances[b][b] += siemens;
            conductances[a][b] -= siemens;
            conductances[b][a] -= siemens;
        };
        let (made, broken) = if self.energised { (NO, NC) } else { (NC, NO) };
        resistor(COM, made, 1.0 / CONTACT_OHMS);
        resistor(COM, broken, OPEN_LEAK);

        CoupledCompanion {
            terminals: PINS,
            conductances,
            // Linear throughout, so there is nothing a slope does not say.
            currents: [0.0; MAX_TERMINALS],
            points: [0.0; MAX_TERMINALS],
            limited: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NINE: RelayPart = RelayPart::V9;

    #[test]
    fn it_pulls_in_late_and_lets_go_later_still() {
        // A 9 V coil of 200 Ω is rated at 45 mA: it closes past 33.75 mA and
        // opens under 4.5 mA.
        assert!(latch(NINE, false, 0.035), "past 75 % it closes");
        assert!(!latch(NINE, false, 0.025), "below it, open stays open");
        assert!(latch(NINE, true, 0.025), "and closed stays closed");
        assert!(!latch(NINE, true, 0.004), "under 10 % it lets go");
        assert!(latch(NINE, false, -0.035), "a coil does not mind which way");
    }

    #[test]
    fn every_part_is_found_by_its_own_name() {
        for part in RelayPart::ALL {
            assert_eq!(RelayPart::named(part.name()), Some(*part));
        }
    }

    #[test]
    fn at_dc_the_coil_is_its_winding() {
        let settled = NINE.advance(None, 9.0, Charge::default());
        assert!((settled.value - 0.045).abs() < 1e-12, "{}", settled.value);
    }

    #[test]
    fn switched_on_it_rises_with_its_time_constant() {
        // 9 V across the coil from nothing: the current climbs towards 45 mA,
        // and after one time constant it is 1 − 1/e of the way there. The loss
        // across the inductance lengthens that from L/R to L over the winding
        // and the loss in parallel, 2 ms. Ten thousand steps across it, so the
        // rule's own error is far below the 0.1 % asked.
        let (winding, loss) = (NINE.coil_ohms(), NINE.loss_ohms());
        let tau = NINE.henries() * (winding + loss) / (winding * loss);
        assert!((tau - 2e-3).abs() < 1e-12, "{tau} s");
        for method in [Integration::BackwardEuler, Integration::Trapezoidal] {
            let steps = 10_000;
            let seconds = tau / f64::from(steps);
            let mut carried = Charge::settled(0.0);
            for _ in 0..steps {
                carried = NINE.advance(Some((method, seconds)), 9.0, carried);
            }
            let expected = 0.045 * (1.0 - (-1.0f64).exp());
            assert!(
                (carried.value - expected).abs() < 1e-3 * expected,
                "{method:?}: {} A against {expected} A",
                carried.value
            );
        }
    }

    #[test]
    fn a_short_step_makes_the_coil_hold_on_to_its_current() {
        // Carrying 45 mA: over a microsecond the inductance's current hardly
        // moves, so the coil looks like its winding and its loss in series,
        // 800 Ω, with most of the 45 mA still pushed round. That is the kick:
        // open whatever fed it, and those milliamps go through the loss.
        let coil = NINE.coil(Integration::BackwardEuler, 1e-6, Charge::settled(0.045));
        let series = NINE.coil_ohms() + NINE.loss_ohms();
        assert!(
            (1.0 / coil.conductance - series).abs() < 0.01 * series,
            "{} Ω",
            1.0 / coil.conductance
        );
        assert!(coil.current > 0.03, "{} A", coil.current);
    }

    #[test]
    fn let_go_the_kick_is_three_times_the_rated_voltage() {
        // The pins opened with 45 mA in the inductance. Nothing outside takes
        // a current, so the pins sit where the companion passes none, and over
        // a short step the 45 mA goes round through the 600 Ω loss instead:
        // 27 V, three times the coil's 9 V, the other way round.
        let (seconds, before) = (1e-7, Charge::settled(0.045));
        let coil = NINE.coil(Integration::BackwardEuler, seconds, before);
        let open = -coil.current / coil.conductance;
        let carried = NINE.advance(Some((Integration::BackwardEuler, seconds)), open, before);

        assert!((open + 27.0).abs() < 0.1, "{open} V at the pins");
        assert!(
            (carried.value - 0.045).abs() < 1e-4,
            "and the inductance still carries it: {} A",
            carried.value
        );
    }
}
