// SPDX-License-Identifier: MIT OR Apache-2.0

//! A CD4511: four bits in, seven segments out (C-77).
//!
//! BCD to seven-segment, latched, with a lamp test and a blanking input, and
//! outputs strong enough to drive a common-cathode LED digit through a
//! resistor per segment — the chip a seven-segment digit is bought beside.
//!
//! Sixteen pins, and every output depends on every input, so unlike the
//! digit it cannot be split into independent parts. Nor is it coupled: it is
//! a 555 in shape (ADR 0012). The inputs are read *after* each step, from the
//! solved potentials, into a state the run carries — the number latched, and
//! whether the lamp test or the blanking is held — and within a solve the part
//! is linear: each output a source towards one supply pin or the other, each
//! input a CMOS gate that takes nothing but a leak. No placement touches more
//! than two pins, so `MAX_TERMINALS` stays where it is. What that costs is
//! one step between an input changing and the segments following, shorter
//! than the chip's own propagation delay at any step the run takes.
//!
//! What it honours: the truth table, including the CD4511's tail-less 6 and 9
//! and its blank display for 10 to 15; the latch; lamp test over blanking over
//! the number; inputs read at half the supply; outputs that source current
//! from VDD and sink to VSS. What it ignores: the output's current limit, its
//! exact drop at every current, and propagation delay.

/// Pin order: the four bits, the three controls, the seven segments, the
/// supply.
pub const PINS: [&str; 16] = [
    "A", "B", "C", "D", "LT", "BI", "LE", "a", "b", "c", "d", "e", "f", "g", "VDD", "VSS",
];

/// The inputs' pin indices: the bits, least significant first.
pub const BITS: [usize; 4] = [0, 1, 2, 3];
/// Lamp test, active low: every segment lit.
pub const LAMP_TEST: usize = 4;
/// Blanking, active low: every segment dark.
pub const BLANKING: usize = 5;
/// Latch enable: low, the outputs follow the inputs; high, they hold.
pub const LATCH_ENABLE: usize = 6;
/// The first output, `a`; the seven follow in order.
pub const FIRST_SEGMENT: usize = 7;
pub const VDD: usize = 14;
pub const VSS: usize = 15;

/// What is on the part.
pub const PART: &str = "CD4511";

/// How far below VDD a lit output sits: one base–emitter drop, since each
/// output is an NPN emitter follower.
pub const HIGH_DROP: f64 = 0.7;

/// Behind a lit output, in ohms. Not a number the datasheet quotes: read off
/// the slope of its output curves, which fall about a volt over the 20 mA a
/// segment is driven at. Enough to say that a resistor is still needed.
pub const HIGH_OHMS: f64 = 50.0;

/// What pulls a dark output down to VSS, in ohms: its n-channel transistor.
pub const LOW_OHMS: f64 = 1_000.0;

/// An input's leak to VSS, in siemens. A CMOS gate takes nothing, and a node
/// with nothing on it is one the solver cannot place (C-63), so a gigaohm.
pub const INPUT_LEAK: f64 = 1e-9;

/// Which segments each digit lights, as `a` in bit 0 through `g` in bit 6.
///
/// The CD4511's own font: its 6 has no top bar and its 9 no bottom one, the
/// thing that tells a CD4511 display from any other at a glance.
const FONT: [u8; 10] = [
    0b011_1111, // 0: a b c d e f
    0b000_0110, // 1: b c
    0b101_1011, // 2: a b d e g
    0b100_1111, // 3: a b c d g
    0b110_0110, // 4: b c f g
    0b110_1101, // 5: a c d f g
    0b111_1100, // 6: c d e f g, no tail
    0b000_0111, // 7: a b c
    0b111_1111, // 8: all seven
    0b110_0111, // 9: a b c f g, no tail
];

/// What the chip is holding between steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {
    /// The number latched, 0 to 15.
    pub number: u8,
    /// Lamp test held low.
    pub testing: bool,
    /// Blanking held low.
    pub blanked: bool,
}

impl State {
    /// As one whole number, which is how it rides in the run's carried state
    /// beside every charge and bit there (C-48): exact, since it is small.
    pub fn encode(self) -> f64 {
        f64::from(self.number)
            + 16.0 * f64::from(u8::from(self.testing))
            + 32.0 * f64::from(u8::from(self.blanked))
    }

    pub fn decode(value: f64) -> State {
        let word = value.round().clamp(0.0, 63.0) as u8;
        State {
            number: word & 0b1111,
            testing: word & 0b1_0000 != 0,
            blanked: word & 0b10_0000 != 0,
        }
    }

    /// Which segments are lit, `a` in bit 0: lamp test over blanking over the
    /// number, as the datasheet orders them.
    pub fn segments(self) -> u8 {
        if self.testing {
            0b111_1111
        } else if self.blanked {
            0
        } else {
            FONT.get(usize::from(self.number)).copied().unwrap_or(0)
        }
    }
}

/// What the chip holds after a step that left its pins at `potentials`,
/// given what it held before.
pub fn latch(was: State, potentials: &[f64; 16]) -> State {
    let low = potentials[VSS];
    let threshold = (potentials[VDD] + low) / 2.0;
    let high = |pin: usize| potentials[pin] > threshold;
    let number = if high(LATCH_ENABLE) {
        was.number
    } else {
        BITS.iter()
            .enumerate()
            .filter(|&(_, &pin)| high(pin))
            .map(|(bit, _)| 1u8 << bit)
            .sum()
    };
    State {
        number,
        testing: !high(LAMP_TEST),
        blanked: !high(BLANKING),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pins(number: u8, lamp_test: bool, blanking: bool, latch_enable: bool) -> [f64; 16] {
        let mut volts = [0.0; 16];
        volts[VDD] = 5.0;
        for (bit, &pin) in BITS.iter().enumerate() {
            volts[pin] = if number & (1 << bit) != 0 { 5.0 } else { 0.0 };
        }
        // The controls are active low, so "held" is 0 V.
        volts[LAMP_TEST] = if lamp_test { 0.0 } else { 5.0 };
        volts[BLANKING] = if blanking { 0.0 } else { 5.0 };
        volts[LATCH_ENABLE] = if latch_enable { 5.0 } else { 0.0 };
        volts
    }

    #[test]
    fn it_draws_every_digit_in_its_own_font() {
        let shown: Vec<u8> = (0..10)
            .map(|number| latch(State::default(), &pins(number, false, false, false)).segments())
            .collect();
        assert_eq!(shown, FONT);
        // The CD4511's giveaway: no tail on the 6 or the 9.
        assert_eq!(FONT[6] & 0b1, 0, "no top bar on the 6");
        assert_eq!(FONT[9] & 0b1000, 0, "no bottom bar on the 9");
    }

    #[test]
    fn it_blanks_past_nine() {
        for number in 10..16 {
            assert_eq!(
                latch(State::default(), &pins(number, false, false, false)).segments(),
                0
            );
        }
    }

    #[test]
    fn lamp_test_wins_over_blanking_and_blanking_over_the_number() {
        assert_eq!(
            latch(State::default(), &pins(1, true, true, false)).segments(),
            0b111_1111
        );
        assert_eq!(
            latch(State::default(), &pins(1, false, true, false)).segments(),
            0
        );
    }

    #[test]
    fn the_latch_holds_what_it_had_while_enable_is_high() {
        let seven = latch(State::default(), &pins(7, false, false, false));
        let held = latch(seven, &pins(2, false, false, true));
        assert_eq!(held.number, 7);
        assert_eq!(latch(held, &pins(2, false, false, false)).number, 2);
    }

    #[test]
    fn its_state_survives_being_carried() {
        for number in 0..16 {
            for testing in [false, true] {
                for blanked in [false, true] {
                    let state = State {
                        number,
                        testing,
                        blanked,
                    };
                    assert_eq!(State::decode(state.encode()), state);
                }
            }
        }
    }
}
