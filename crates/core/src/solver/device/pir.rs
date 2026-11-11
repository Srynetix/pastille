// SPDX-License-Identifier: MIT OR Apache-2.0

//! An HC-SR501 PIR motion sensor module (C-116).
//!
//! In nearly every Arduino kit, and it works without one: its output goes
//! high when something warm moves in front of it and stays high for a time
//! set on its own trimmer. Movement is a press on the part, as a push-button
//! is: holding it is someone walking past.
//!
//! **From the module's sheet** (MPJA's HC-SR501, 31227sc): 4.5 to 20 V, under
//! 50 µA standing, an output of 3.3 V high and 0 V low; a delay adjustable
//! from 5 s to 200 s; 2.5 s after each delay in which it will not trigger; a
//! jumper, L for a single trigger and H for repeatable — held movement then
//! keeps restarting the delay — and about a minute after power-up to settle,
//! during which it ignores movement here (the sheet says it may trigger up to
//! three times while it does; that is not modelled).
//!
//! **Not on the sheet, and said**: how hard its output drives. It is taken as
//! 3.3 V behind 1 kΩ, an estimate — a few milliamps into an LED, which is why
//! the example puts a MOSFET after it. Its sensitivity trimmer has nothing to
//! act on: movement here is there or it is not.
//!
//! The run carries how long it has been powered, how much of its delay is
//! left and how much of its block, as the servo carries its pulse (C-90).

use crate::solver::Charge;

/// Pin order: as the module's three pins run, the jumper side up.
pub const PINS: [&str; 3] = ["VCC", "OUT", "GND"];
pub const VCC: usize = 0;
pub const OUT: usize = 1;
pub const GND: usize = 2;

/// What is on the part.
pub const PART: &str = "HC-SR501";

/// Its supply range, in volts.
pub const LEAST_SUPPLY: f64 = 4.5;
pub const MOST_SUPPLY: f64 = 20.0;
/// Its output high, in volts, and what it is taken to drive through.
pub const HIGH: f64 = 3.3;
pub const OUT_OHMS: f64 = 1_000.0;
/// Its standing draw, as the resistance that draws 50 µA at 5 V.
pub const SUPPLY_OHMS: f64 = 5.0 / 50e-6;
/// Seconds after power-up before it answers movement.
pub const WARM_UP: f64 = 60.0;
/// Seconds after a delay in which it will not trigger again.
pub const BLOCK: f64 = 2.5;
/// Its delay trimmer's range, in seconds.
pub const SHORTEST_HOLD: f64 = 5.0;
pub const LONGEST_HOLD: f64 = 200.0;

/// What it holds between steps.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct State {
    /// Seconds it has been powered, up to the end of its warm-up.
    pub powered_for: f64,
    /// Seconds of its delay left: high while there are any.
    pub holding: f64,
    /// Seconds of its block left.
    pub blocked: f64,
}

impl State {
    /// As the run carries it.
    pub fn encode(self) -> Charge {
        Charge {
            value: self.holding,
            slope: Some(0.0),
            latched: self.warm(),
            held: [self.powered_for, self.blocked],
        }
    }

    pub fn decode(carried: Option<Charge>) -> State {
        carried.map_or(State::default(), |carried| State {
            powered_for: carried.held[0],
            holding: carried.value,
            blocked: carried.held[1],
        })
    }

    /// Whether its output is high.
    pub fn high(self) -> bool {
        self.holding > 0.0
    }

    /// Whether it has finished settling since power-up.
    pub fn warm(self) -> bool {
        self.powered_for >= WARM_UP
    }

    /// After `seconds` more, with someone moving in front of it or not,
    /// its delay at `hold`, retriggered by held movement or not, on `supply`.
    pub fn advance(
        self,
        moving: bool,
        hold: f64,
        retriggers: bool,
        supply: f64,
        seconds: f64,
    ) -> State {
        if supply < LEAST_SUPPLY {
            return State::default();
        }
        let powered_for = (self.powered_for + seconds).min(WARM_UP);
        if powered_for < WARM_UP {
            return State {
                powered_for,
                ..State::default()
            };
        }
        let mut blocked = (self.blocked - seconds).max(0.0);
        let mut holding = self.holding;
        if holding > 0.0 {
            holding = if moving && retriggers {
                hold
            } else {
                holding - seconds
            };
            if holding <= 0.0 {
                holding = 0.0;
                blocked = BLOCK;
            }
        } else if moving && blocked <= 0.0 {
            holding = hold;
        }
        State {
            powered_for,
            holding,
            blocked,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `seconds` at a twentieth of a second a step.
    fn run(mut state: State, moving: bool, retriggers: bool, seconds: f64) -> State {
        let steps = (seconds * 20.0).round() as usize;
        for _ in 0..steps {
            state = state.advance(moving, 5.0, retriggers, 5.0, 0.05);
        }
        state
    }

    fn warm() -> State {
        run(State::default(), false, true, WARM_UP)
    }

    #[test]
    fn it_ignores_movement_for_its_first_minute() {
        let early = run(State::default(), true, true, 30.0);
        assert!(!early.high() && !early.warm());
        let later = run(early, true, true, 31.0);
        assert!(later.high() && later.warm());
    }

    #[test]
    fn one_movement_holds_it_high_for_its_delay() {
        let seen = run(warm(), true, true, 0.1);
        assert!(seen.high());
        assert!(run(seen, false, true, 4.8).high(), "still within 5 s");
        assert!(!run(seen, false, true, 5.1).high(), "and low after");
    }

    #[test]
    fn held_movement_keeps_a_retriggering_one_high_and_a_single_one_not() {
        // Twelve seconds of movement against a 5 s delay.
        assert!(
            run(warm(), true, true, 12.0).high(),
            "retriggered all along"
        );
        let once = run(warm(), true, false, 5.2);
        assert!(!once.high(), "its delay ran out under the movement");
        assert!(!run(once, true, false, 2.0).high(), "and blocked for 2.5 s");
        assert!(run(once, true, false, 2.7).high(), "then triggered again");
    }

    #[test]
    fn without_its_supply_it_starts_again() {
        let seen = run(warm(), true, true, 1.0);
        let unpowered = seen.advance(true, 5.0, true, 3.0, 0.05);
        assert_eq!(unpowered, State::default());
    }

    #[test]
    fn its_state_survives_being_carried() {
        let state = State {
            powered_for: 60.0,
            holding: 3.25,
            blocked: 1.5,
        };
        assert_eq!(State::decode(Some(state.encode())), state);
        assert_eq!(State::decode(None), State::default());
    }
}
