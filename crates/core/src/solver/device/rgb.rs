// SPDX-License-Identifier: MIT OR Apache-2.0

//! An RGB LED: a red, a green and a blue LED sharing one pin (C-84).
//!
//! The seven-segment digit's shape with three dice in place of eight bars
//! (C-75): each die is a junction of its own between its pin and the common,
//! a die whose pin is on no net is simply dark, and which end they share is a
//! choice on the part. What is new is that the dice are not all the same LED:
//! each is the LED of its colour already modelled, so a red die through a
//! resistor is exactly as bright as a red LED through the same one, and lights
//! from less voltage than the other two — which is why a kit gives the red a
//! larger resistor.
//!
//! What it honours: each die's junction, drop and brightness; which way round
//! the common is. What it ignores: that a real RGB part's green is an InGaN
//! die (about 3 V, where the green LED here sits at 2.1 V), that its three
//! dice are not equally bright to the eye at the same current, and the
//! package's own heat.

use crate::netlist::LedColour;

/// The dice, in pin order: the colour each pin lights.
pub const DICE: [LedColour; 3] = [LedColour::Red, LedColour::Green, LedColour::Blue];

/// The common pin's index, after the three dice.
pub const COMMON: usize = DICE.len();

/// A generic 5 mm diffused RGB LED, the one every starter kit has: no part
/// number anybody would recognise, and figures that are the single LEDs'.
pub const PART: &str = "5 mm diffused RGB LED";
