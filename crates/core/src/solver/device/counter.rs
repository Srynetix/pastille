// SPDX-License-Identifier: MIT OR Apache-2.0

//! A CD4026: a decade counter that drives a digit itself (C-78).
//!
//! Every rising edge on its clock counts one, 0 to 9 and round again, and its
//! seven outputs draw the count on a common-cathode digit — so a 555 and this
//! chip are a digit that counts on its own, and a second one on the first's
//! carry counts the tens.
//!
//! The same shape as the CD4511 (C-77), one step further: the count is state,
//! and so is the clock's level at the end of the last step, because an edge
//! is the level now against the level then. Both are read from the solved
//! potentials after each step, at half the supply, and carried; within a
//! solve the chip is linear, its outputs driven high or low and its inputs a
//! leak.
//!
//! What it honours: counting on the rising edge; reset, active high, back to
//! 0; clock inhibit, active high, holding the count; display enable, active
//! high, the outputs on at all; carry out, high for 0 to 4 and low for 5 to
//! 9, so its rising edge is the count going round. What it ignores: the
//! display-enable output and the ungated c output, which are not pinned out
//! here; the maximum clock rate; propagation delay.
//!
//! **From TI's CD4026B sheet, SCHS031B (revised July 2003)**, a scan, read off
//! its page images (C-122). Its 6 and 9 have tails: in its timing diagram
//! (Fig. 3) segment a is high through 6, and d is high again at 9, and the
//! digits drawn under the CD4033B's (Fig. 4), which shares the decoder, show
//! both. And its outputs' strength, at 5 V and 25 °C typical: 1 mA at 0.4 V
//! from either rail, but only 3.2 mA at 2.5 V below VDD — the curve bends
//! over, 400 Ω near the rail and 780 Ω two and a half volts from it. A
//! segment is driven a volt or two from the rail, so 780 Ω, the 2.5 V point's:
//! through 220 Ω into a red segment that is 3.2 mA, where the sheet's own bend
//! gives about 3.3 and 400 Ω would give 5.2. The 0.4 V point is a logic
//! load's, which draws next to nothing either way. And at the supply it is on
//! (C-125): its typical source curves (Fig. 8, the CD4017B's drawing) give
//! 9.9 mA at 2.7 V on 10 V and 12.0 mA on 15 V — 270 Ω and 220 Ω — so on 9 V
//! a segment is lit four times as hard as on 5 V (see [`super::cmos`]). Still
//! weaker than a CD4511's, which drives a segment as the battery through its
//! resistor would — on 9 V through 220 Ω about 13 mA against 30.

/// Pin order: the four inputs, the carry, the seven segments, the supply.
pub const PINS: [&str; 14] = [
    "CLK", "INH", "RST", "DEI", "CO", "a", "b", "c", "d", "e", "f", "g", "VDD", "VSS",
];

pub const CLOCK: usize = 0;
/// Clock inhibit, active high: edges are ignored.
pub const INHIBIT: usize = 1;
/// Reset, active high: the count goes to 0 and stays there.
pub const RESET: usize = 2;
/// Display enable in, active high: the segment outputs are on at all.
pub const DISPLAY_ENABLE: usize = 3;
/// Carry out: high for 0 to 4, low for 5 to 9.
pub const CARRY: usize = 4;
/// The first segment output, `a`; the seven follow in order.
pub const FIRST_SEGMENT: usize = 5;
pub const VDD: usize = 12;
pub const VSS: usize = 13;

/// What is on the part.
pub const PART: &str = "CD4026";

/// Behind every output, high or low: (supply, ohms), 2.5 V below VDD over
/// the sheet's current there (see the module).
pub const OUTPUT_CURVE: [(f64, f64); 3] = [(5.0, 780.0), (10.0, 270.0), (15.0, 220.0)];

/// An input's leak to VSS, in siemens, as the CD4511's (C-63).
pub const INPUT_LEAK: f64 = 1e-9;

/// Which segments each count lights, `a` in bit 0 through `g` in bit 6. With
/// tails on the 6 and the 9, as the sheet's timing diagram has them.
const FONT: [u8; 10] = [
    0b011_1111, // 0
    0b000_0110, // 1
    0b101_1011, // 2
    0b100_1111, // 3
    0b110_0110, // 4
    0b110_1101, // 5
    0b111_1101, // 6
    0b000_0111, // 7
    0b111_1111, // 8
    0b110_1111, // 9
];

/// What the chip is holding between steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {
    /// The count, 0 to 9.
    pub count: u8,
    /// Whether the clock was high at the end of the last step.
    pub clock_was_high: bool,
    /// Whether the display is enabled.
    pub enabled: bool,
}

impl State {
    /// As one whole number, carried beside every charge (C-48).
    pub fn encode(self) -> f64 {
        f64::from(self.count)
            + 16.0 * f64::from(u8::from(self.clock_was_high))
            + 32.0 * f64::from(u8::from(self.enabled))
    }

    pub fn decode(value: f64) -> State {
        let word = value.round().clamp(0.0, 63.0) as u8;
        State {
            count: (word & 0b1111).min(9),
            clock_was_high: word & 0b1_0000 != 0,
            enabled: word & 0b10_0000 != 0,
        }
    }

    /// Which segments are lit, `a` in bit 0: none while the display is off.
    pub fn segments(self) -> u8 {
        if self.enabled {
            FONT[usize::from(self.count)]
        } else {
            0
        }
    }

    /// Whether carry out is high: for 0 to 4.
    pub fn carry(self) -> bool {
        self.count < 5
    }
}

/// What the chip holds on its first reading, before anything is known of what
/// its clock was doing: that level, and no edge. A chip powered up with its
/// clock already high has not seen that clock rise.
pub fn first(potentials: &[f64; 14]) -> State {
    let threshold = (potentials[VDD] + potentials[VSS]) / 2.0;
    State {
        count: 0,
        clock_was_high: potentials[CLOCK] > threshold,
        enabled: potentials[DISPLAY_ENABLE] > threshold,
    }
}

/// What the chip holds after a step that left its pins at `potentials`.
pub fn advance(was: State, potentials: &[f64; 14]) -> State {
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
        enabled: high(DISPLAY_ENABLE),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn its_font_is_the_sheet_s_timing_diagram() {
        // Fig. 3 of SCHS031B, read count by count, each segment high (1) or
        // low (0) from 0 to 9 — the tails on 6 (a) and 9 (d) among them.
        let diagram: [(char, [u8; 10]); 7] = [
            ('a', [1, 0, 1, 1, 0, 1, 1, 1, 1, 1]),
            ('b', [1, 1, 1, 1, 1, 0, 0, 1, 1, 1]),
            ('c', [1, 1, 0, 1, 1, 1, 1, 1, 1, 1]),
            ('d', [1, 0, 1, 1, 0, 1, 1, 0, 1, 1]),
            ('e', [1, 0, 1, 0, 0, 0, 1, 0, 1, 0]),
            ('f', [1, 0, 0, 0, 1, 1, 1, 0, 1, 1]),
            ('g', [0, 0, 1, 1, 1, 1, 1, 0, 1, 1]),
        ];
        for (bit, (name, levels)) in diagram.iter().enumerate() {
            for (count, &level) in levels.iter().enumerate() {
                let lit = FONT[count] >> bit & 1;
                assert_eq!(lit, level, "segment {name} at {count}");
            }
        }
    }

    fn pins(clock: bool, inhibit: bool, reset: bool) -> [f64; 14] {
        let mut volts = [0.0; 14];
        volts[VDD] = 9.0;
        volts[DISPLAY_ENABLE] = 9.0;
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

    #[test]
    fn it_counts_rising_edges_and_goes_round_after_nine() {
        let start = advance(State::default(), &pins(false, false, false));
        assert_eq!(pulse(start, 3).count, 3);
        assert_eq!(pulse(start, 10).count, 0);
        assert_eq!(pulse(start, 12).count, 2);
    }

    #[test]
    fn a_clock_already_high_when_it_starts_is_not_an_edge() {
        let start = first(&pins(true, false, false));
        assert_eq!(advance(start, &pins(true, false, false)).count, 0);
    }

    #[test]
    fn a_level_is_not_an_edge() {
        // Held high across many steps, the clock counts once.
        let mut state = advance(State::default(), &pins(false, false, false));
        for _ in 0..5 {
            state = advance(state, &pins(true, false, false));
        }
        assert_eq!(state.count, 1);
    }

    #[test]
    fn inhibit_holds_and_reset_clears() {
        let three = pulse(advance(State::default(), &pins(false, false, false)), 3);
        let held = advance(
            advance(three, &pins(true, true, false)),
            &pins(false, true, false),
        );
        assert_eq!(held.count, 3, "inhibited");
        assert_eq!(advance(three, &pins(false, false, true)).count, 0, "reset");
    }

    #[test]
    fn carry_rises_as_the_count_goes_round() {
        // High for 0 to 4, low for 5 to 9: its rising edge is nine to nought,
        // which is what clocks the next decade.
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
    fn its_state_survives_being_carried() {
        for count in 0..10 {
            for clock_was_high in [false, true] {
                for enabled in [false, true] {
                    let state = State {
                        count,
                        clock_was_high,
                        enabled,
                    };
                    assert_eq!(State::decode(state.encode()), state);
                }
            }
        }
    }
}
