// SPDX-License-Identifier: MIT OR Apache-2.0

//! A hobby servo, an SG90: a horn that turns to the angle its pulses ask for
//! (C-90).
//!
//! Three wires — signal, +5 V and GND — and a pulse on the signal every 20 ms
//! or so: about 1 ms high for one end of its travel, 2 ms for the other, 1.5 ms
//! for the middle. The servo measures each pulse, and turns its horn towards
//! that angle at its own speed; between pulses, and when they stop altogether,
//! it holds where it was sent. Behavioural under ADR 0010, its state carried
//! between steps as the CD4026's count is (C-78): the horn's angle, the angle
//! it is heading for, and whether the signal is high and how long it has been.
//! An edge is placed at the start of the step it is first seen in, because
//! that is where every edge here happens: a 555's output flips once a step
//! has ended (ADR 0012), a contact clicked takes effect from the next step,
//! and a signal generator's steps end on its edges (ADR 0011). So a pulse is
//! read as exactly as many steps as it was high, whatever their length. Only a
//! signal ramping slowly through the threshold crosses inside a step, and the
//! run takes short steps on a ramp. (Interpolating against the threshold was
//! tried first, and read 1.5 ms as 103° in 0.3 ms steps; half way through the
//! step was tried next, and is half a step out on every one of these edges —
//! for a click, up to half a millisecond, which is 90°.)
//!
//! From TowerPro's SG90 sheet: 0.1 s for 60° at 4.8 V, so 600° a second; 180°
//! of travel; 4.8 V to 6 V. Its current is not on the sheet: about 10 mA
//! holding still and a couple of hundred milliamps turning are what one draws
//! on a bench, and that is the model — a load on its supply that grows while
//! the horn moves. Its signal input is a CMOS-like leak, and reads high from
//! 2 V, so a 3.3 V signal drives it as a 5 V one does.
//!
//! What it ignores: its 10 µs dead band; the load on its horn; that it
//! stalls, and how hard it pushes back; and how it misbehaves on too little
//! supply, which here is simply that it does not move below 4 V.

use crate::solver::Charge;

/// Pin order: the order the registry names them in.
pub const SIGNAL: usize = 0;
pub const SUPPLY: usize = 1;
pub const GROUND: usize = 2;

/// The part, for the label and the Inspector.
pub const PART: &str = "SG90";

/// Degrees a second: 60° in 0.1 s.
pub const DEGREES_PER_SECOND: f64 = 600.0;

/// Its travel, in degrees, and where a fresh one sits: the middle.
pub const TRAVEL: f64 = 180.0;
pub const MIDDLE: f64 = 90.0;

/// The pulse widths for each end of its travel, in seconds.
pub const SHORTEST: f64 = 1e-3;
pub const LONGEST: f64 = 2e-3;

/// The widest pulse it takes as a pulse at all, in seconds, and the
/// narrowest: outside them it is a level or a glitch, and ignored.
const TOO_LONG: f64 = 3e-3;
const TOO_SHORT: f64 = 0.4e-3;

/// Where its signal reads high, and where it reads low again, in volts above
/// GND.
pub const HIGH_FROM: f64 = 2.0;
pub const LOW_BELOW: f64 = 1.8;

/// The least supply it turns on, in volts.
pub const LEAST_SUPPLY: f64 = 4.0;

/// Its signal input's leak to GND, in siemens: CMOS-like, as a chip's are.
pub const INPUT_LEAK: f64 = 1e-9;

/// Its supply, as a resistance from +5 V to GND, holding still and turning:
/// 10 mA and 200 mA at 5 V.
pub const HOLDING_OHMS: f64 = 500.0;
pub const TURNING_OHMS: f64 = 25.0;

/// Closer than this to where it is going, in degrees, it is there.
const THERE: f64 = 0.1;

/// What a servo carries from one step to the next.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State {
    /// The horn's angle, in degrees, 0 to 180.
    pub angle: f64,
    /// The angle the last pulse asked for.
    pub target: f64,
    /// Whether the signal was high at the end of the last step.
    pub high: bool,
    /// How long it has been high, in seconds, while it is.
    pub since: f64,
}

impl State {
    /// A servo nothing has driven yet: in the middle, and staying there.
    pub const FRESH: State = State {
        angle: MIDDLE,
        target: MIDDLE,
        high: false,
        since: 0.0,
    };

    /// As the run carries it. The angles go in from the middle, so a charge
    /// that has never been written — all zeros — is a fresh servo.
    pub fn encode(self) -> Charge {
        Charge {
            value: self.angle - MIDDLE,
            slope: Some(0.0),
            latched: self.high,
            held: [self.target - MIDDLE, self.since],
        }
    }

    pub fn decode(carried: Option<Charge>) -> State {
        let Some(carried) = carried else {
            return State::FRESH;
        };
        State {
            angle: carried.value + MIDDLE,
            target: carried.held[0] + MIDDLE,
            high: carried.latched,
            since: carried.held[1],
        }
    }

    /// Whether the horn is on its way somewhere.
    pub fn turning(&self) -> bool {
        (self.target - self.angle).abs() > THERE
    }

    /// Its supply, as a resistance, for the step about to be taken.
    pub fn supply_ohms(&self) -> f64 {
        if self.turning() {
            TURNING_OHMS
        } else {
            HOLDING_OHMS
        }
    }

    /// The state `seconds` later, with the signal at `signal` volts above GND
    /// and `supply` across its supply pins at the end of it.
    pub fn advance(self, signal: f64, supply: f64, seconds: f64) -> State {
        let mut next = self;
        let seconds = seconds.max(0.0);
        // An edge seen at the end of this step happened at its start.
        if !self.high && signal >= HIGH_FROM {
            next.high = true;
            next.since = seconds;
        } else if self.high && signal < LOW_BELOW {
            next.high = false;
            let width = self.since;
            if (TOO_SHORT..=TOO_LONG).contains(&width) {
                next.target = angle_for(width);
            }
            next.since = 0.0;
        } else if self.high {
            next.since = self.since + seconds;
        }

        if supply >= LEAST_SUPPLY {
            let room = next.target - self.angle;
            let reach = DEGREES_PER_SECOND * seconds;
            next.angle = self.angle + room.clamp(-reach, reach);
        }
        next
    }
}

/// The angle a pulse of `width` seconds asks for: 1 ms is 0°, 2 ms is 180°,
/// and anything beyond either end is that end.
pub fn angle_for(width: f64) -> f64 {
    (TRAVEL * (width - SHORTEST) / (LONGEST - SHORTEST)).clamp(0.0, TRAVEL)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pulses of `width` every 20 ms for `seconds`, sampled every `step`, on
    /// a 5 V supply.
    fn driven(mut state: State, width: f64, seconds: f64, step: f64) -> State {
        let mut time = 0.0;
        while time < seconds {
            let phase = time % 20e-3;
            let signal = if phase < width { 5.0 } else { 0.0 };
            state = state.advance(signal, 5.0, step);
            time += step;
        }
        state
    }

    #[test]
    fn a_pulse_width_is_an_angle() {
        assert_eq!(angle_for(1e-3), 0.0);
        assert!((angle_for(1.5e-3) - 90.0).abs() < 1e-9);
        assert_eq!(angle_for(2e-3), 180.0);
        assert_eq!(angle_for(2.4e-3), 180.0, "past the end is the end");
    }

    #[test]
    fn it_turns_to_the_angle_its_pulses_ask_for() {
        for (width, angle) in [(1e-3, 0.0), (1.25e-3, 45.0), (2e-3, 180.0)] {
            let state = driven(State::FRESH, width, 0.5, 10e-6);
            assert!(
                (state.angle - angle).abs() < 1.0,
                "{width} s: {} °",
                state.angle
            );
            assert!(!state.turning());
        }
    }

    #[test]
    fn a_coarse_step_reads_the_same_pulse() {
        // Edges placed between the readings, so steps of 0.3 ms — a third
        // of the pulse — still read 1.5 ms as the middle.
        let fine = driven(
            State {
                target: 0.0,
                angle: 0.0,
                ..State::FRESH
            },
            1.5e-3,
            0.5,
            10e-6,
        );
        let coarse = driven(
            State {
                target: 0.0,
                angle: 0.0,
                ..State::FRESH
            },
            1.5e-3,
            0.5,
            0.3e-3,
        );
        assert!((fine.angle - 90.0).abs() < 1.0, "{}", fine.angle);
        assert!(
            (coarse.angle - fine.angle).abs() < 5.0,
            "{} against {}",
            coarse.angle,
            fine.angle
        );
    }

    #[test]
    fn it_turns_at_its_sheet_s_speed() {
        // From the middle to one end, 90°, at 600° a second: 0.15 s.
        let early = driven(State::FRESH, 2e-3, 0.1, 10e-6);
        assert!(early.turning(), "still on its way at 0.1 s");
        assert!(
            (early.angle - 90.0 - 600.0 * (0.1 - 2e-3)).abs() < 5.0,
            "{}",
            early.angle
        );
        let later = driven(State::FRESH, 2e-3, 0.2, 10e-6);
        assert!((later.angle - 180.0).abs() < 0.5, "{}", later.angle);
    }

    #[test]
    fn it_holds_its_angle_when_the_pulses_stop() {
        let sent = driven(State::FRESH, 1e-3, 0.5, 10e-6);
        let mut held = sent;
        for _ in 0..1000 {
            held = held.advance(0.0, 5.0, 1e-3);
        }
        assert!((held.angle - sent.angle).abs() < 1e-9);
    }

    #[test]
    fn without_its_supply_it_does_not_move() {
        let mut state = State::FRESH;
        let mut time = 0.0;
        while time < 0.5 {
            let signal = if time % 20e-3 < 2e-3 { 5.0 } else { 0.0 };
            state = state.advance(signal, 0.0, 10e-6);
            time += 10e-6;
        }
        assert_eq!(state.angle, MIDDLE);
    }

    #[test]
    fn a_level_is_not_a_pulse() {
        // Held high, then let go: 0.5 s is not a servo pulse, and nothing moves.
        let mut state = State::FRESH;
        for _ in 0..500 {
            state = state.advance(5.0, 5.0, 1e-3);
        }
        state = state.advance(0.0, 5.0, 1e-3);
        assert_eq!(state.target, MIDDLE);
    }

    #[test]
    fn it_crosses_the_run_and_back_unchanged() {
        let state = State {
            angle: 33.0,
            target: 120.0,
            high: true,
            since: 1e-3,
        };
        assert_eq!(State::decode(Some(state.encode())), state);
        assert_eq!(State::decode(Some(Charge::default())), State::FRESH);
    }
}
