// SPDX-License-Identifier: MIT OR Apache-2.0

//! A CD4017: a decade counter with ten outputs, one high at a time (C-110).
//!
//! Every rising edge on its clock moves the one high output along by one, Q0
//! to Q9 and round again. With a 555 on its clock and an LED on each output,
//! it is the running light every kit builds; with an output wired back to its
//! reset, it counts to that output and starts again — a counter designed by
//! a wire.
//!
//! The CD4026's shape (C-78): the count and the clock's level at the end of
//! the last step are read from the solved potentials after each step, at half
//! the supply, and carried; within a solve the chip is linear, its outputs
//! driven high or low behind a resistance and its inputs a leak.
//!
//! What it honours: counting on the clock's rising edge; reset, active high,
//! back to Q0 and held there; clock inhibit, active high, holding the count;
//! carry out, high for 0 to 4 and low for 5 to 9, so its rising edge is the
//! count going round. What it ignores: the Schmitt trigger on its clock input
//! (it reads at half the supply as the other inputs do), the maximum clock
//! rate, propagation delay, and counting on the inhibit pin's falling edge
//! with the clock held high.
//!
//! **From TI's CD4017B sheet** (SCHS027C, a scan, read off its images): 3 to
//! 18 V; at 5 V and 25 °C, typical, an output gives 1 mA at 0.4 V from either
//! rail, but only 3.2 mA at 2.5 V below VDD — the curve bends over, 400 Ω near
//! the rail and 780 Ω two and a half volts from it. An LED is driven a volt or
//! two from the rail, so the 2.5 V point's, as the CD4026's (C-124): through
//! 330 Ω into a red LED at 5 V that is about 2.9 mA, where 400 Ω would give
//! 4.3. The 0.4 V point is a logic load's, which draws next to nothing either
//! way. And at the supply it is on: its typical source curves (Fig. 7) give
//! 9.8 mA at 2.66 V on 10 V and 12.0 mA on 15 V — 270 Ω and 220 Ω — so on 9 V
//! it drives an LED half as hard again as on 5 V, and an output loaded by one
//! still reads high at the reset it is wired to (see [`super::cmos`]).

/// Pin order: the sheet's, round the package.
pub const PINS: [&str; 16] = [
    "Q5", "Q1", "Q0", "Q2", "Q6", "Q7", "Q3", "VSS", "Q8", "Q4", "Q9", "CO", "INH", "CLK", "RST",
    "VDD",
];

/// Q0 to Q9, in order.
pub const OUTPUTS: [usize; 10] = [2, 1, 3, 6, 9, 0, 4, 5, 8, 10];
pub const VSS: usize = 7;
/// Carry out: high for 0 to 4, low for 5 to 9.
pub const CARRY: usize = 11;
/// Clock inhibit, active high: edges are ignored.
pub const INHIBIT: usize = 12;
pub const CLOCK: usize = 13;
/// Reset, active high: back to Q0, and held there.
pub const RESET: usize = 14;
pub const VDD: usize = 15;

/// Its inputs, for the leaks.
pub const INPUTS: [usize; 3] = [INHIBIT, CLOCK, RESET];

/// What is on the part.
pub const PART: &str = "CD4017";

/// Behind every output, high or low: (supply, ohms), 2.5 V below VDD over
/// the sheet's current there (see the module).
pub const OUTPUT_CURVE: [(f64, f64); 3] = [(5.0, 780.0), (10.0, 270.0), (15.0, 220.0)];

/// An input's leak to VSS, in siemens, as the CD4026's (C-63).
pub const INPUT_LEAK: f64 = 1e-9;

/// It runs on 3 V to 18 V.
pub const MOST_SUPPLY: f64 = 18.0;

/// What the chip is holding between steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {
    /// The count, 0 to 9: which output is high.
    pub count: u8,
    /// Whether the clock was high at the end of the last step.
    pub clock_was_high: bool,
}

impl State {
    /// As one whole number, carried beside every charge (C-48).
    pub fn encode(self) -> f64 {
        f64::from(self.count) + 16.0 * f64::from(u8::from(self.clock_was_high))
    }

    pub fn decode(value: f64) -> State {
        let word = value.round().clamp(0.0, 31.0) as u8;
        State {
            count: (word & 0b1111).min(9),
            clock_was_high: word & 0b1_0000 != 0,
        }
    }

    /// Whether carry out is high: for 0 to 4.
    pub fn carry(self) -> bool {
        self.count < 5
    }
}

/// What the chip holds on its first reading: the clock's level and no edge,
/// at Q0. A chip powered up with its clock already high has not seen it rise.
pub fn first(potentials: &[f64; 16]) -> State {
    let threshold = (potentials[VDD] + potentials[VSS]) / 2.0;
    State {
        count: 0,
        clock_was_high: potentials[CLOCK] > threshold,
    }
}

/// What the chip holds after a step that left its pins at `potentials`.
pub fn advance(was: State, potentials: &[f64; 16]) -> State {
    let threshold = (potentials[VDD] + potentials[VSS]) / 2.0;
    let high = |pin: usize| potentials[pin] > threshold;
    let clock = high(CLOCK);
    let count = if high(RESET) {
        0
    } else if clock && !was.clock_was_high && !high(INHIBIT) {
        (was.count + 1) % 10
    } else {
        was.count
    };
    State {
        count,
        clock_was_high: clock,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pins(clock: bool, inhibit: bool, reset: bool) -> [f64; 16] {
        let mut volts = [0.0; 16];
        volts[VDD] = 9.0;
        let level = |on: bool| if on { 9.0 } else { 0.0 };
        volts[CLOCK] = level(clock);
        volts[INHIBIT] = level(inhibit);
        volts[RESET] = level(reset);
        volts
    }

    /// `edges` rising edges from `state`.
    fn pulse(mut state: State, edges: usize) -> State {
        for _ in 0..edges {
            state = advance(state, &pins(true, false, false));
            state = advance(state, &pins(false, false, false));
        }
        state
    }

    fn start() -> State {
        first(&pins(false, false, false))
    }

    #[test]
    fn each_edge_moves_the_high_output_along_and_round_after_q9() {
        assert_eq!(pulse(start(), 3).count, 3);
        assert_eq!(pulse(start(), 10).count, 0);
        assert_eq!(pulse(start(), 12).count, 2);
    }

    #[test]
    fn a_clock_already_high_when_it_starts_is_not_an_edge() {
        let start = first(&pins(true, false, false));
        assert_eq!(advance(start, &pins(true, false, false)).count, 0);
    }

    #[test]
    fn a_level_is_not_an_edge() {
        let mut state = start();
        for _ in 0..5 {
            state = advance(state, &pins(true, false, false));
        }
        assert_eq!(state.count, 1);
    }

    #[test]
    fn inhibit_holds_and_reset_clears_and_holds() {
        let three = pulse(start(), 3);
        let held = advance(
            advance(three, &pins(true, true, false)),
            &pins(false, true, false),
        );
        assert_eq!(held.count, 3, "inhibited");
        let reset = advance(three, &pins(false, false, true));
        assert_eq!(reset.count, 0, "reset");
        let still = advance(reset, &pins(true, false, true));
        assert_eq!(still.count, 0, "an edge under reset counts nothing");
    }

    #[test]
    fn carry_is_high_for_the_first_five() {
        let carries: Vec<bool> = (0..10)
            .map(|count| {
                State {
                    count,
                    ..State::default()
                }
                .carry()
            })
            .collect();
        assert_eq!(
            carries,
            [
                true, true, true, true, true, false, false, false, false, false
            ]
        );
    }

    #[test]
    fn the_outputs_are_where_the_sheet_puts_them() {
        for (count, &pin) in OUTPUTS.iter().enumerate() {
            assert_eq!(PINS[pin], format!("Q{count}"));
        }
        assert_eq!(
            [
                PINS[VSS],
                PINS[CARRY],
                PINS[INHIBIT],
                PINS[CLOCK],
                PINS[RESET],
                PINS[VDD]
            ],
            ["VSS", "CO", "INH", "CLK", "RST", "VDD"]
        );
    }

    #[test]
    fn its_state_survives_being_carried() {
        for count in 0..10 {
            for clock_was_high in [false, true] {
                let state = State {
                    count,
                    clock_was_high,
                };
                assert_eq!(State::decode(state.encode()), state);
            }
        }
    }
}
