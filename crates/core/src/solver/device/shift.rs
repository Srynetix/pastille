// SPDX-License-Identifier: MIT OR Apache-2.0

//! A 74HC595: eight bits shifted in one at a time, then shown all at once
//! (C-93).
//!
//! Every Arduino kit drives eight LEDs from three pins with one, and clocking
//! it by hand first is the way to see what the code will be doing. Each rising
//! edge on SRCLK moves the eight bits of its shift register along one, QA
//! taking whatever SER is; a rising edge on RCLK copies all eight into the
//! storage register, which is what QA to QH show. So the LEDs do not ripple
//! while a byte goes in: they change once, on the latch. QH′ is the shift
//! register's last bit, wired to the SER of a second one to chain them.
//!
//! The CD4026's shape (C-78): both registers and both clocks' last levels are
//! read from the solved potentials after each step, at half the supply, and
//! carried; within a solve the chip is linear, each output driven high or low
//! behind its own resistance, or let go, and each input a leak. An edge is an
//! edge of a step, as every edge here is (C-90).
//!
//! **From TI's SN74HC595 sheet** (SCLS041J), at 4.5 V and 25 °C, typical:
//! QA–QH 4.3 V at −6 mA and 0.17 V at 6 mA; QH′ the same at 4 mA; inputs
//! switching between 1.35 V and 3.15 V, ±0.1 nA; a 3-state output ±0.01 µA
//! let go; 2 to 6 V; 35 mA an output and 70 mA through VCC or GND at most.
//! The outputs are taken as the resistances those points give — a CMOS
//! output's is not one number, and it rises as the current does, so an LED
//! pulled harder than the sheet's 6 mA is drawn a little brighter than it
//! would be.
//!
//! What it honours: shifting on SRCLK's rising edge and latching on RCLK's;
//! the storage register taking the shift register as it was before the same
//! step's shift, so with the two clocks tied it is one behind, as the sheet
//! says; SRCLR, active low, clearing the shift register and not the storage
//! one; OE, active low, letting go of QA to QH and not of QH′. What it
//! ignores: set-up and hold times, propagation delay, the clock rate, and the
//! total current limit.
//!
//! **Seen coming**: a real push-button bounces, and a real 595 shifts on every
//! bounce. These buttons do not, and the example says so.

/// Pin order: the sheet's, round the package.
pub const PINS: [&str; 16] = [
    "QB", "QC", "QD", "QE", "QF", "QG", "QH", "GND", "QH'", "SRCLR", "SRCLK", "RCLK", "OE", "SER",
    "QA", "VCC",
];

/// QA to QH, in order: QA is pin 15 and the rest start the package.
pub const OUTPUTS: [usize; 8] = [14, 0, 1, 2, 3, 4, 5, 6];
pub const GND: usize = 7;
/// The shift register's last bit, for the next chip's SER.
pub const CARRY: usize = 8;
/// Shift register clear, active low.
pub const CLEAR: usize = 9;
/// Shift register clock.
pub const SHIFT_CLOCK: usize = 10;
/// Storage register clock: the latch.
pub const LATCH_CLOCK: usize = 11;
/// Output enable, active low.
pub const ENABLE: usize = 12;
/// Serial data in.
pub const DATA: usize = 13;
pub const VCC: usize = 15;

/// Its inputs, for the leaks.
pub const INPUTS: [usize; 5] = [CLEAR, SHIFT_CLOCK, LATCH_CLOCK, ENABLE, DATA];

/// What is on the part.
pub const PART: &str = "74HC595";

/// Behind QA to QH driven high, in ohms: 0.2 V under 4.5 V at 6 mA.
pub const HIGH_OHMS: f64 = 33.0;
/// Behind them driven low: 0.17 V at 6 mA.
pub const LOW_OHMS: f64 = 28.0;
/// Behind QH′ driven high and low: the same drops at 4 mA.
pub const CARRY_HIGH_OHMS: f64 = 50.0;
pub const CARRY_LOW_OHMS: f64 = 42.0;

/// An input's leak, and an output's let go, in siemens: a node with nothing
/// on it is one the solver cannot place (C-63).
pub const LEAK: f64 = 1e-9;

/// What the chip holds between steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {
    /// The shift register, QA's stage in bit 0.
    pub shifting: u8,
    /// The storage register, which QA to QH show: QA in bit 0.
    pub stored: u8,
    /// Whether SRCLK was high at the end of the last step.
    pub shift_was_high: bool,
    /// Whether RCLK was.
    pub latch_was_high: bool,
    /// Whether OE was low, so QA to QH are driven.
    pub enabled: bool,
}

impl State {
    /// As one whole number, carried beside every charge (C-48): nineteen
    /// bits, which a double holds exactly.
    pub fn encode(self) -> f64 {
        f64::from(
            u32::from(self.shifting)
                | u32::from(self.stored) << 8
                | u32::from(self.shift_was_high) << 16
                | u32::from(self.latch_was_high) << 17
                | u32::from(self.enabled) << 18,
        )
    }

    pub fn decode(value: f64) -> State {
        let word = value.round().clamp(0.0, f64::from((1u32 << 19) - 1)) as u32;
        State {
            shifting: (word & 0xff) as u8,
            stored: (word >> 8 & 0xff) as u8,
            shift_was_high: word >> 16 & 1 != 0,
            latch_was_high: word >> 17 & 1 != 0,
            enabled: word >> 18 & 1 != 0,
        }
    }

    /// What QA to QH show, QA in bit 0, or nothing while they are let go.
    pub fn shown(self) -> Option<u8> {
        self.enabled.then_some(self.stored)
    }

    /// Whether QH′ is high: the shift register's last stage.
    pub fn carry(self) -> bool {
        self.shifting & 0x80 != 0
    }
}

/// What the chip holds on its first reading: the clocks' levels, and no edge.
/// A chip powered up with a clock already high has not seen it rise.
pub fn first(potentials: &[f64; 16]) -> State {
    let high = levels(potentials);
    State {
        shift_was_high: high(SHIFT_CLOCK),
        latch_was_high: high(LATCH_CLOCK),
        enabled: !high(ENABLE),
        ..State::default()
    }
}

/// What the chip holds after a step that left its pins at `potentials`.
pub fn advance(was: State, potentials: &[f64; 16]) -> State {
    let high = levels(potentials);
    let shift = high(SHIFT_CLOCK);
    let latch = high(LATCH_CLOCK);
    let shifting = if !high(CLEAR) {
        0
    } else if shift && !was.shift_was_high {
        was.shifting << 1 | u8::from(high(DATA))
    } else {
        was.shifting
    };
    // From the shift register as it stood before this step's edge: with the
    // two clocks tied, the storage register is one behind (the sheet's note).
    let stored = if latch && !was.latch_was_high {
        was.shifting
    } else {
        was.stored
    };
    State {
        shifting,
        stored,
        shift_was_high: shift,
        latch_was_high: latch,
        enabled: !high(ENABLE),
    }
}

/// Whether each pin is high: above half the supply, as a CMOS input reads.
fn levels(potentials: &[f64; 16]) -> impl Fn(usize) -> bool + '_ {
    let threshold = (potentials[VCC] + potentials[GND]) / 2.0;
    move |pin| potentials[pin] > threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pins at 5 V: clear and enable inactive, both clocks and the data as
    /// given.
    fn pins(data: bool, shift: bool, latch: bool) -> [f64; 16] {
        let mut volts = [0.0; 16];
        let level = |on: bool| if on { 5.0 } else { 0.0 };
        volts[VCC] = 5.0;
        volts[CLEAR] = 5.0;
        volts[DATA] = level(data);
        volts[SHIFT_CLOCK] = level(shift);
        volts[LATCH_CLOCK] = level(latch);
        volts
    }

    /// One bit clocked in, the clock up then down.
    fn clock_in(state: State, bit: bool) -> State {
        let up = advance(state, &pins(bit, true, false));
        advance(up, &pins(bit, false, false))
    }

    fn latched(state: State) -> State {
        let up = advance(state, &pins(false, false, true));
        advance(up, &pins(false, false, false))
    }

    fn byte(bits: &[bool]) -> State {
        let start = first(&pins(false, false, false));
        bits.iter().fold(start, |state, &bit| clock_in(state, bit))
    }

    #[test]
    fn a_byte_goes_in_one_edge_at_a_time_and_shows_on_the_latch() {
        // 1, 0, 1, 1: the last bit in is QA's, the first has moved to QD.
        let state = byte(&[true, false, true, true]);
        assert_eq!(state.shifting, 0b1011);
        assert_eq!(state.shown(), Some(0), "nothing shown before the latch");
        assert_eq!(latched(state).shown(), Some(0b1011));
    }

    #[test]
    fn a_level_is_not_an_edge() {
        let mut state = first(&pins(true, false, false));
        for _ in 0..5 {
            state = advance(state, &pins(true, true, false));
        }
        assert_eq!(state.shifting, 0b1, "one bit for one rising edge");
    }

    #[test]
    fn a_clock_already_high_when_it_starts_is_not_an_edge() {
        let start = first(&pins(true, true, true));
        let next = advance(start, &pins(true, true, true));
        assert_eq!((next.shifting, next.stored), (0, 0));
    }

    #[test]
    fn with_both_clocks_tied_the_storage_register_is_one_behind() {
        let state = byte(&[true, true]);
        let up = advance(state, &pins(true, true, true));
        assert_eq!(up.shifting, 0b111);
        assert_eq!(up.stored, 0b11, "the register before this edge");
    }

    #[test]
    fn clear_empties_the_shift_register_and_not_the_storage_one() {
        let shown = latched(byte(&[true, true, true]));
        let mut volts = pins(false, false, false);
        volts[CLEAR] = 0.0;
        let cleared = advance(shown, &volts);
        assert_eq!(cleared.shifting, 0);
        assert_eq!(cleared.shown(), Some(0b111));
    }

    #[test]
    fn output_enable_lets_go_of_the_outputs_and_not_of_the_carry() {
        let full = latched(byte(&[true; 8]));
        let mut volts = pins(false, false, false);
        volts[ENABLE] = 5.0;
        let off = advance(full, &volts);
        assert_eq!(off.shown(), None);
        assert!(off.carry(), "QH′ still carries the last bit");
    }

    #[test]
    fn the_ninth_bit_pushes_the_first_out_of_qh_prime() {
        let eight = byte(&[true, false, false, false, false, false, false, false]);
        assert!(eight.carry(), "the first bit is in QH's stage");
        assert!(!clock_in(eight, false).carry(), "and gone with the ninth");
    }

    #[test]
    fn its_state_survives_being_carried() {
        for word in [0u32, 1, 0xff, 0x1234, 0x7_ffff, 0x5_a5a5] {
            let state = State::decode(f64::from(word));
            assert_eq!(state.encode(), f64::from(word));
        }
    }
}
