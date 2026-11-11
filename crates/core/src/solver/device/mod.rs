// SPDX-License-Identifier: MIT OR Apache-2.0

//! Device models.
//!
//! The nonlinear ones the solver linearises about an operating point, and the
//! reactive ones it linearises about a moment in time. Both hand back the same
//! `Companion`, which is why the matrix does not have to know the difference.

mod bjt;
mod capacitor;
pub mod cell;
pub mod cmos;
pub mod comparator;
pub mod counter;
pub mod decade;
pub mod decoder;
pub mod digit;
mod diode;
pub mod driver;
pub mod electrolytic;
pub mod flipflop;
pub mod fuse;
pub mod gate;
mod inductor;
pub mod lcd;
mod led;
pub mod lm3914;
pub mod load;
pub mod magnet;
pub mod mosfet;
pub mod opamp;
pub mod opto;
pub mod photo;
pub mod pir;
pub mod pitch;
pub mod regulator;
pub mod relay;
pub mod rgb;
mod saturating;
pub mod sensor;
pub mod servo;
pub mod shift;
pub mod signal;
pub mod solar;
pub mod timer;
pub mod tmp36;
pub mod transformer;
pub mod uno;
mod zener;

pub use bjt::{BASE, Bjt, COLLECTOR, EMITTER, Polarity};
pub use capacitor::Capacitor;
pub use comparator::Comparator;
pub use diode::{Diode, SCHOTTKY, THERMAL_VOLTAGE};
pub use inductor::Inductor;
pub use led::{Led, RATED_CURRENT};
pub use mosfet::{Channel, Mosfet};
pub use opamp::OpAmp;
pub use regulator::{Datasheet, Regulator, RegulatorPart};
pub use relay::{Relay, RelayPart};
pub use saturating::{
    HIGH_RAIL, INVERTING, LOW_RAIL, NON_INVERTING, OUTPUT, SMOOTHING, SaturatingSource,
};
pub use signal::Waveform;
pub use timer::Timer;
pub use zener::{RATED_WATTS as ZENER_WATTS, Zener, ZenerPart};

#[cfg(test)]
mod pin_order {
    //! C-105. Each of these parts names its pins in the registry and indexes
    //! them by number here, and the solver stamps by the numbers. Only a
    //! comment tied the two; swapping a pin in the registry would have shown
    //! up, if at all, as a part that behaved strangely. Held here instead.

    use crate::registry::definition;

    /// The pin of `kind` at `index`, by the registry's name.
    fn named(kind: &str, index: usize) -> &'static str {
        definition(kind)
            .unwrap_or_else(|| panic!("{kind} is registered"))
            .pins[index]
    }

    #[test]
    fn every_index_the_solver_uses_is_the_pin_the_registry_names() {
        let expected: &[(&str, usize, &str)] = &[
            ("Timer", super::timer::VCC, "VCC"),
            ("Timer", super::timer::GND, "GND"),
            ("Timer", super::timer::TRIG, "TRIG"),
            ("Timer", super::timer::THR, "THR"),
            ("Timer", super::timer::DIS, "DIS"),
            ("Timer", super::timer::OUT, "OUT"),
            ("Relay", super::relay::COIL_PLUS, "coil+"),
            ("Relay", super::relay::COIL_MINUS, "coil-"),
            ("Relay", super::relay::COM, "com"),
            ("Relay", super::relay::NO, "no"),
            ("Relay", super::relay::NC, "nc"),
            ("Regulator", super::regulator::INPUT_PIN, "in"),
            ("Regulator", super::regulator::GROUND_PIN, "gnd"),
            ("Regulator", super::regulator::OUTPUT_PIN, "out"),
            ("Bjt", super::BASE, "b"),
            ("Bjt", super::COLLECTOR, "c"),
            ("Bjt", super::EMITTER, "e"),
            ("Mosfet", super::mosfet::GATE, "g"),
            ("Mosfet", super::mosfet::DRAIN, "d"),
            ("Mosfet", super::mosfet::SOURCE, "s"),
            ("Servo", super::servo::SIGNAL, "SIG"),
            ("Servo", super::servo::SUPPLY, "+5V"),
            ("Servo", super::servo::GROUND, "GND"),
            ("Tmp36", super::tmp36::SUPPLY_PIN, "+Vs"),
            ("Tmp36", super::tmp36::OUTPUT_PIN, "Vout"),
            ("Tmp36", super::tmp36::GROUND_PIN, "GND"),
            // An op-amp's and a comparator's pins are the saturating source's
            // terminals, in its order (C-55, C-56).
            ("OpAmp", super::saturating::NON_INVERTING, "in+"),
            ("OpAmp", super::saturating::INVERTING, "in-"),
            ("OpAmp", super::saturating::OUTPUT, "out"),
            ("OpAmp", super::saturating::HIGH_RAIL, "V+"),
            ("OpAmp", super::saturating::LOW_RAIL, "V-"),
            ("Comparator", super::saturating::NON_INVERTING, "in+"),
            ("Comparator", super::saturating::INVERTING, "in-"),
            ("Comparator", super::saturating::OUTPUT, "out"),
            ("Comparator", super::saturating::HIGH_RAIL, "V+"),
            ("Comparator", super::saturating::LOW_RAIL, "V-"),
        ];
        for &(kind, index, name) in expected {
            assert_eq!(named(kind, index), name, "{kind} pin {index}");
        }
    }

    #[test]
    fn the_chips_are_registered_with_their_own_pin_lists() {
        // The chips hand the registry their `PINS` array, so their indices
        // and names are one list; held so a later copy cannot creep in.
        for (kind, pins) in [
            ("Decoder", &super::decoder::PINS[..]),
            ("Counter", &super::counter::PINS[..]),
            ("MotorDriver", &super::driver::PINS[..]),
            ("ShiftRegister", &super::shift::PINS[..]),
            ("Gate", &super::gate::QUAD_PINS[..]),
            ("Inverter", &super::gate::HEX_PINS[..]),
            ("Schmitt", &super::gate::HEX_PINS[..]),
            ("Decade", &super::decade::PINS[..]),
            ("FlipFlop", &super::flipflop::PINS[..]),
            ("Optocoupler", &super::opto::PINS[..]),
        ] {
            assert_eq!(definition(kind).unwrap().pins, pins, "{kind}");
        }
    }
}
