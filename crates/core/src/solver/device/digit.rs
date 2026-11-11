// SPDX-License-Identifier: MIT OR Apache-2.0

//! A seven-segment LED digit: eight LEDs sharing one pin (C-75).
//!
//! Seven bars and a decimal point, each an LED between its own pin and the
//! common one. A common-cathode digit has every cathode joined, so a segment
//! lights when its pin is driven high; a common-anode one is the same with
//! every LED reversed, lit by pulling its pin low. Which it is is a choice on
//! the part, as an LED's colour is (C-46).
//!
//! Nine pins, past what a coupled device may have — but nothing couples the
//! segments to each other, so none is needed: each segment is stamped as a
//! junction of its own, and a segment whose pin is on no net is simply dark,
//! the way the relay leaves a free contact alone (C-64). Each is the red LED
//! already modelled, because a digit's segments are red LEDs in one package:
//! the same drop, the same brightness for the same current, the same need for
//! a resistor.
//!
//! What it honours: each segment's junction and brightness; which way round
//! the common is. What it ignores: the few segments of some parts that are two
//! LEDs in series (larger digits), and the package's own heat.

/// The segments' pin names, in the order they are numbered: the seven bars
/// clockwise from the top, the middle one last, then the point.
pub const SEGMENTS: [&str; 8] = ["a", "b", "c", "d", "e", "f", "g", "dp"];

/// The common pin's index, after the eight segments.
pub const COMMON: usize = 8;

/// How many pins it has.
pub const PIN_COUNT: usize = 9;

/// Which of its ends the eight LEDs share.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Common {
    Cathode,
    Anode,
}

impl Common {
    /// Common cathode first: the one a starter kit has, lit by driving a
    /// segment high, which is what a decoder like the CD4511 does (C-77).
    pub const ALL: &'static [Common] = &[Common::Cathode, Common::Anode];

    /// The name it crosses the boundary under.
    pub fn name(&self) -> &'static str {
        match self {
            Common::Cathode => "Common cathode",
            Common::Anode => "Common anode",
        }
    }

    pub fn named(name: &str) -> Option<Common> {
        Common::ALL
            .iter()
            .copied()
            .find(|common| common.name() == name)
    }

    /// A common-cathode 5161AS or a common-anode 5161BS: the 0.56-inch red
    /// digit in every kit.
    pub fn part_number(&self) -> &'static str {
        match self {
            Common::Cathode => "5161AS",
            Common::Anode => "5161BS",
        }
    }

    /// Which of an LED's two pins is its anode, and which its cathode, as pin
    /// indices — its own and the shared one, one way round or the other. The
    /// shared pin is a digit's [`COMMON`] or an RGB LED's (C-84).
    pub fn junction(&self, own: usize, shared: usize) -> (usize, usize) {
        match self {
            Common::Cathode => (own, shared),
            Common::Anode => (shared, own),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_choice_is_found_by_its_own_name() {
        for common in Common::ALL {
            assert_eq!(Common::named(common.name()), Some(*common));
        }
    }

    #[test]
    fn a_common_anode_is_every_segment_turned_round() {
        for segment in 0..SEGMENTS.len() {
            let (anode, cathode) = Common::Cathode.junction(segment, COMMON);
            assert_eq!(Common::Anode.junction(segment, COMMON), (cathode, anode));
        }
    }
}
