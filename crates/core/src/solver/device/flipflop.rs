// SPDX-License-Identifier: MIT OR Apache-2.0

//! A CD4013: two D flip-flops (C-111).
//!
//! Each one's Q takes whatever D is on its clock's rising edge and keeps it
//! until the next: the first part here that remembers a bit because it was
//! told to, rather than because two gates hold each other up (C-108). With
//! /Q wired back to D it toggles on every edge — one button, on and off — and
//! two of those in a row halve a frequency and halve it again.
//!
//! The CD4026's shape (C-78): each flip-flop's outputs and its clock's level at
//! the end of the last step are read from the solved potentials after each
//! step, at half the supply, and carried; within a solve the chip is linear,
//! its outputs driven high or low behind a resistance and its inputs a leak.
//! /Q is carried beside Q rather than worked out from it, because with SET
//! and RESET both high the sheet has both outputs high.
//!
//! What it honours: Q taking D on the rising edge and holding through the
//! falling one; SET and RESET acting at once whatever the clock, each active
//! high; both at once, both outputs high. What it ignores: set-up and hold
//! times, the clock rate, propagation delay.
//!
//! **From TI's CD4013B sheet** (SCHS023E): 3 to 18 V; at 5 V and 25 °C,
//! typical, an output gives 1 mA at 0.4 V from either rail and 3.2 mA at 2.5 V
//! below VDD — the B series' standard output, the CD4017's and the CD4026's.
//! The 2.5 V point's, because an LED is driven a volt or two from the rail
//! (C-124); the 0.4 V point's 400 Ω is a logic load's. Its own typical source
//! curves (Fig. 3, read off the rendered page) give 9.1 mA there on 10 V and
//! 11.0 mA on 15 V: 780 Ω on 5 V, 275 Ω on 10 V, 230 Ω on 15 V, the CD4017's
//! near enough, taken at the supply it is on (see [`super::cmos`]).

/// Pin order: the sheet's, round the package.
pub const PINS: [&str; 14] = [
    "Q1", "/Q1", "CLK1", "RST1", "D1", "SET1", "VSS", "SET2", "D2", "RST2", "CLK2", "/Q2", "Q2",
    "VDD",
];

/// One flip-flop's pins.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Half {
    pub q: usize,
    pub not_q: usize,
    pub clock: usize,
    pub reset: usize,
    pub data: usize,
    pub set: usize,
}

/// The two, in the sheet's numbering.
pub const HALVES: [Half; 2] = [
    Half {
        q: 0,
        not_q: 1,
        clock: 2,
        reset: 3,
        data: 4,
        set: 5,
    },
    Half {
        q: 12,
        not_q: 11,
        clock: 10,
        reset: 9,
        data: 8,
        set: 7,
    },
];

pub const VSS: usize = 6;
pub const VDD: usize = 13;

/// What is on the part.
pub const PART: &str = "CD4013";

/// Behind every output, high or low: (supply, ohms), 2.5 V below VDD over
/// the sheet's current there (see the module).
pub const OUTPUT_CURVE: [(f64, f64); 3] = [(5.0, 780.0), (10.0, 275.0), (15.0, 230.0)];

/// An input's leak to VSS, in siemens, as the CD4017's (C-63).
pub const INPUT_LEAK: f64 = 1e-9;

/// It runs on 3 V to 18 V.
pub const MOST_SUPPLY: f64 = 18.0;

/// What one flip-flop holds between steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bit {
    pub q: bool,
    pub not_q: bool,
    /// Whether its clock was high at the end of the last step.
    pub clock_was_high: bool,
}

impl Default for Bit {
    /// Q low, /Q high: how it is drawn before anything has happened.
    fn default() -> Self {
        Bit {
            q: false,
            not_q: true,
            clock_was_high: false,
        }
    }
}

/// What the chip holds between steps: both flip-flops.
pub type State = [Bit; 2];

/// As one whole number, carried beside every charge (C-48): six bits.
pub fn encode(state: State) -> f64 {
    let bits =
        |bit: Bit| u8::from(bit.q) | u8::from(bit.not_q) << 1 | u8::from(bit.clock_was_high) << 2;
    f64::from(bits(state[0]) | bits(state[1]) << 3)
}

pub fn decode(value: f64) -> State {
    let word = value.round().clamp(0.0, 63.0) as u8;
    let bit = |bits: u8| Bit {
        q: bits & 1 != 0,
        not_q: bits & 2 != 0,
        clock_was_high: bits & 4 != 0,
    };
    [bit(word & 0b111), bit(word >> 3)]
}

/// What a chip holds before it has been read: both Q low.
pub fn resting() -> State {
    [Bit::default(); 2]
}

/// What the chip holds on its first reading: each clock's level, and no
/// edge. A chip powered up with a clock already high has not seen it rise.
pub fn first(potentials: &[f64; 14]) -> State {
    let threshold = (potentials[VDD] + potentials[VSS]) / 2.0;
    HALVES.map(|half| Bit {
        clock_was_high: potentials[half.clock] > threshold,
        ..settle(Bit::default(), potentials, half, threshold, false)
    })
}

/// What the chip holds after a step that left its pins at `potentials`.
pub fn advance(was: State, potentials: &[f64; 14]) -> State {
    let threshold = (potentials[VDD] + potentials[VSS]) / 2.0;
    [0, 1].map(|index| {
        let half = HALVES[index];
        let clock = potentials[half.clock] > threshold;
        let rose = clock && !was[index].clock_was_high;
        Bit {
            clock_was_high: clock,
            ..settle(was[index], potentials, half, threshold, rose)
        }
    })
}

/// One flip-flop after a step: SET and RESET first, at once whatever the
/// clock; otherwise D on a rising edge; otherwise as it was.
fn settle(was: Bit, potentials: &[f64; 14], half: Half, threshold: f64, rose: bool) -> Bit {
    let high = |pin: usize| potentials[pin] > threshold;
    let (q, not_q) = match (high(half.set), high(half.reset)) {
        (true, true) => (true, true),
        (true, false) => (true, false),
        (false, true) => (false, true),
        (false, false) if rose => (high(half.data), !high(half.data)),
        // Let go of SET and RESET together, it keeps Q and puts /Q back
        // opposite it.
        (false, false) => (was.q, !was.q),
    };
    Bit { q, not_q, ..was }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The first flip-flop's inputs at 9 V; the second's all low.
    fn pins(clock: bool, data: bool, set: bool, reset: bool) -> [f64; 14] {
        let mut volts = [0.0; 14];
        volts[VDD] = 9.0;
        let level = |on: bool| if on { 9.0 } else { 0.0 };
        let half = HALVES[0];
        volts[half.clock] = level(clock);
        volts[half.data] = level(data);
        volts[half.set] = level(set);
        volts[half.reset] = level(reset);
        volts
    }

    /// A rising edge and the fall after it, with `data` on D.
    fn clocked(state: State, data: bool) -> State {
        let up = advance(state, &pins(true, data, false, false));
        advance(up, &pins(false, data, false, false))
    }

    #[test]
    fn q_takes_d_on_the_rising_edge_and_holds_it() {
        let start = first(&pins(false, false, false, false));
        assert!(!start[0].q);
        let one = clocked(start, true);
        assert!(one[0].q && !one[0].not_q);
        // D changing with no edge changes nothing.
        let held = advance(one, &pins(false, false, false, false));
        assert!(held[0].q, "held");
        assert!(!clocked(held, false)[0].q, "and the next edge takes the 0");
    }

    #[test]
    fn a_level_is_not_an_edge() {
        let mut state = first(&pins(true, false, false, false));
        for _ in 0..3 {
            state = advance(state, &pins(true, true, false, false));
        }
        assert!(!state[0].q, "a clock high from the start never rose");
    }

    #[test]
    fn set_and_reset_act_at_once_whatever_the_clock() {
        let start = first(&pins(false, false, false, false));
        let set = advance(start, &pins(false, false, true, false));
        assert!(set[0].q && !set[0].not_q, "set");
        let reset = advance(set, &pins(false, false, false, true));
        assert!(!reset[0].q && reset[0].not_q, "reset");
        let both = advance(reset, &pins(false, false, true, true));
        assert!(
            both[0].q && both[0].not_q,
            "both: both outputs high, as the sheet has it"
        );
        // Reset held, a rising edge with D high takes nothing.
        let held = advance(reset, &pins(true, true, false, true));
        assert!(!held[0].q);
    }

    #[test]
    fn the_other_flip_flop_is_left_alone() {
        let state = clocked(first(&pins(false, false, false, false)), true);
        assert_eq!(state[1], Bit::default());
    }

    #[test]
    fn the_pins_are_where_the_sheet_puts_them() {
        for (index, half) in HALVES.iter().enumerate() {
            let n = index + 1;
            assert_eq!(
                [
                    PINS[half.q],
                    PINS[half.not_q],
                    PINS[half.clock],
                    PINS[half.reset],
                    PINS[half.data],
                    PINS[half.set]
                ],
                [
                    format!("Q{n}"),
                    format!("/Q{n}"),
                    format!("CLK{n}"),
                    format!("RST{n}"),
                    format!("D{n}"),
                    format!("SET{n}")
                ]
            );
        }
        assert_eq!((PINS[VSS], PINS[VDD]), ("VSS", "VDD"));
    }

    #[test]
    fn its_state_survives_being_carried() {
        for word in 0..64u32 {
            assert_eq!(encode(decode(f64::from(word))), f64::from(word));
        }
    }
}
