// SPDX-License-Identifier: MIT OR Apache-2.0

//! An L293D: four half-H drivers, so two motors either way (C-91).
//!
//! One transistor can switch a motor on and off but not reverse it: that
//! takes both ends of the motor driven, each one able to go high or low. A
//! half-H is one such end, and two of them are an H-bridge. With 1A high and
//! 2A low the motor between 1Y and 2Y turns one way; swapped, the other;
//! both the same, it stops; and with 1,2EN low both outputs let go.
//!
//! The CD4511's shape (C-77): the inputs and enables are read after each step,
//! from the solved potentials, into what the run carries, and within a solve
//! the chip is linear — each enabled output a source towards VCC2 or a pull to
//! GND, each input a leak. The clamp diodes are the D in the name, and are
//! real junctions here: each output has one up to VCC2 and one up from GND,
//! handed to Newton like any diode's.
//!
//! Two supplies, as the sheet has them: VCC1 for the logic, which has to be at
//! least 4.5 V above GND or every output lets go, and VCC2 for the motor, from
//! 4.5 to 36 V. One GND pin for the sheet's four (ADR 0010).
//!
//! What it honours: the truth table; the enables; inputs read at TTL levels
//! whatever the supplies; the drop the sheet gives across each output at its
//! rated current, high and low; the clamps; the logic supply's own draw. What
//! it ignores: the motor supply's quiescent current, the output's current
//! limit and thermal shutdown, and switching times.

/// Pin order: the sheet's, round the package, with its four GND pins one.
pub const PINS: [&str; 13] = [
    "1,2EN", "1A", "1Y", "2Y", "2A", "VCC2", "3,4EN", "3A", "3Y", "4Y", "4A", "VCC1", "GND",
];

/// Each channel's input, 1A to 4A.
pub const INPUTS: [usize; 4] = [1, 4, 7, 10];
/// Each channel's output, 1Y to 4Y.
pub const OUTPUTS: [usize; 4] = [2, 3, 8, 9];
/// The two enables: 1,2EN for channels 1 and 2, 3,4EN for 3 and 4.
pub const ENABLES: [usize; 2] = [0, 6];
/// The motor supply.
pub const VCC2: usize = 5;
/// The logic supply.
pub const VCC1: usize = 11;
pub const GND: usize = 12;

/// What is on the part.
pub const PART: &str = "L293D";

/// Where an input is read as high, in volts above GND: between the sheet's
/// 1.5 V most for a low and 2.3 V least for a high.
pub const INPUT_HIGH_FROM: f64 = 1.9;

/// The least logic supply it runs on, VCC1 above GND.
pub const LOGIC_FROM: f64 = 4.5;

/// How far below VCC2 a high output sits with nothing drawn from it.
pub const HIGH_DROP: f64 = 1.0;

/// Behind a high output, in ohms: with [`HIGH_DROP`], the sheet's typical
/// 1.4 V under VCC2 at 0.6 A.
pub const HIGH_OHMS: f64 = 0.67;

/// What pulls a low output to GND, in ohms: the sheet's typical 1.2 V at
/// 0.6 A.
pub const LOW_OHMS: f64 = 2.0;

/// The logic supply's draw, in ohms from VCC1 to GND: the sheet's typical
/// 16 mA, at 5 V.
pub const LOGIC_OHMS: f64 = 312.0;

/// An input's leak to GND, and an output's while it is let go, in siemens:
/// as the CD4511's, a node with nothing on it is one the solver cannot place
/// (C-63).
pub const LEAK: f64 = 1e-9;

/// What one output is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Drive {
    /// Towards VCC2.
    High,
    /// Towards GND.
    Low,
    /// Let go: disabled, or no logic supply.
    Off,
}

/// What the chip read at the end of the last step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {
    /// Each channel's input, high or low.
    pub inputs: [bool; 4],
    /// Each pair's enable.
    pub enabled: [bool; 2],
    /// Whether VCC1 was enough to run on.
    pub powered: bool,
}

impl State {
    /// As one whole number, which is how it rides in the run's carried state
    /// (C-48): seven bits, exact.
    pub fn encode(self) -> f64 {
        let bits = self
            .inputs
            .iter()
            .chain(&self.enabled)
            .chain([&self.powered])
            .enumerate()
            .filter(|&(_, &on)| on)
            .map(|(bit, _)| 1u8 << bit)
            .sum::<u8>();
        f64::from(bits)
    }

    pub fn decode(value: f64) -> State {
        let word = value.round().clamp(0.0, 127.0) as u8;
        let bit = |at: usize| word & (1 << at) != 0;
        State {
            inputs: [bit(0), bit(1), bit(2), bit(3)],
            enabled: [bit(4), bit(5)],
            powered: bit(6),
        }
    }

    /// What `channel`'s output does: its input, if its pair is enabled and the
    /// logic is powered.
    pub fn drive(self, channel: usize) -> Drive {
        if !self.powered || !self.enabled[channel / 2] {
            Drive::Off
        } else if self.inputs[channel] {
            Drive::High
        } else {
            Drive::Low
        }
    }
}

/// What the chip reads from pins left at `potentials`. A pin on nothing is
/// passed as GND's potential, so it reads low.
pub fn read(potentials: &[f64; 13]) -> State {
    let ground = potentials[GND];
    let high = |pin: usize| potentials[pin] - ground >= INPUT_HIGH_FROM;
    State {
        inputs: INPUTS.map(high),
        enabled: ENABLES.map(high),
        powered: potentials[VCC1] - ground >= LOGIC_FROM,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Logic at 5 V, the first pair enabled, 1A and 2A as given.
    fn pins(one: bool, two: bool) -> [f64; 13] {
        let mut volts = [0.0; 13];
        volts[VCC1] = 5.0;
        volts[VCC2] = 9.0;
        volts[ENABLES[0]] = 5.0;
        volts[INPUTS[0]] = if one { 5.0 } else { 0.0 };
        volts[INPUTS[1]] = if two { 5.0 } else { 0.0 };
        volts
    }

    #[test]
    fn its_two_inputs_set_its_two_outputs() {
        // Forwards, backwards, and the two ways of stopping: both low, both
        // high — the motor's ends at one potential either way.
        for (one, two) in [(false, false), (true, false), (false, true), (true, true)] {
            let state = read(&pins(one, two));
            let level = |on| if on { Drive::High } else { Drive::Low };
            assert_eq!(
                [state.drive(0), state.drive(1)],
                [level(one), level(two)],
                "{one} {two}"
            );
        }
    }

    #[test]
    fn a_disabled_pair_lets_go() {
        let mut volts = pins(true, false);
        volts[ENABLES[0]] = 0.0;
        let state = read(&volts);
        assert_eq!([state.drive(0), state.drive(1)], [Drive::Off; 2]);
        // And the other pair was never enabled.
        assert_eq!(state.drive(2), Drive::Off);
    }

    #[test]
    fn without_its_logic_supply_every_output_lets_go() {
        let mut volts = pins(true, false);
        volts[VCC1] = 3.0;
        let state = read(&volts);
        assert!((0..4).all(|channel| state.drive(channel) == Drive::Off));
    }

    #[test]
    fn it_reads_its_inputs_at_ttl_levels() {
        // 2.3 V is a high and 1.5 V a low, whatever the supplies.
        let mut volts = pins(false, false);
        volts[INPUTS[0]] = 2.3;
        volts[INPUTS[1]] = 1.5;
        let state = read(&volts);
        assert_eq!([state.drive(0), state.drive(1)], [Drive::High, Drive::Low]);
    }

    #[test]
    fn its_outputs_drop_what_the_sheet_says_at_its_rated_current() {
        let amps = 0.6;
        let high = HIGH_DROP + amps * HIGH_OHMS;
        let low = amps * LOW_OHMS;
        assert!((high - 1.4).abs() < 0.01, "{high}");
        assert!((low - 1.2).abs() < 0.01, "{low}");
    }

    #[test]
    fn its_state_survives_being_carried() {
        for word in 0..128u8 {
            let state = State::decode(f64::from(word));
            assert_eq!(State::decode(state.encode()), state);
            assert_eq!(state.encode(), f64::from(word));
        }
    }
}
