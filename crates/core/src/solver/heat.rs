// SPDX-License-Identifier: MIT OR Apache-2.0

//! How hot a part runs (C-74).
//!
//! First order and nothing more: a part dissipating `P` watts in a room at
//! `room` °C heads for `room + P·θja` along
//!
//! ```text
//! T′ = (room + P·θja − T) / τ
//! ```
//!
//! and falls back to the room when the power goes. θja and τ are the
//! package's, from [`Thermal`]; a part with none has no temperature.
//!
//! The heat changes nothing electrical. A resistor's tempco, an LED dimming as
//! it warms, a thermistor heated by its own current are all real and all left
//! out, so the temperature is carried beside the solve rather than inside it,
//! and costs a resistive circuit nothing it did not already pay.
//!
//! Each step is taken exactly rather than integrated: over a step the power is
//! the step's, constant, and the equation above then has a closed form. A
//! step of a microsecond and a frame of a sixtieth of a second land on the
//! same curve, and neither can overshoot.

use crate::netlist::{Element, ElementKind};
use crate::registry::Thermal;

use super::device::regulator::{GROUND_PIN, INPUT_PIN, OUTPUT_PIN};
use super::device::{Bjt, Diode, Mosfet, Regulator, RegulatorPart, Zener};
use super::{Coupled, MAX_TERMINALS, OperatingPoint};

/// Where a part heads for, in °C, dissipating `watts` in a room at `room`.
pub fn settles_at(thermal: &Thermal, room: f64, watts: f64) -> f64 {
    room + watts * thermal.junction_to_ambient
}

/// A part's temperature `seconds` after it was `was`, dissipating `watts`
/// the whole time.
pub fn warm(thermal: &Thermal, was: f64, room: f64, watts: f64, seconds: f64) -> f64 {
    let target = settles_at(thermal, room, watts);
    if thermal.time_constant <= 0.0 {
        return target;
    }
    target + (was - target) * (-seconds / thermal.time_constant).exp()
}

/// Watts `element` turns into heat at `point`, for a part that has a
/// temperature; nothing for any other, or for one not wired enough to say.
pub fn watts(element: &Element, point: &OperatingPoint) -> Option<f64> {
    let volts = |pin: usize| point.voltage(element.pins.get(pin)?.net?);
    match element.kind {
        ElementKind::Resistor { ohms } => {
            let across = volts(0)? - volts(1)?;
            Some(across * across / ohms)
        }
        ElementKind::Diode => {
            let across = volts(0)? - volts(1)?;
            Some(across * Diode::default().current(across))
        }
        ElementKind::Schottky => {
            let across = volts(0)? - volts(1)?;
            Some(across * super::device::SCHOTTKY.current(across))
        }
        // Its four diodes, each from its anode to its cathode, all in one
        // package (C-118): an AC pin up to +, and − up to an AC pin.
        ElementKind::Bridge => {
            let diode = |anode: usize, cathode: usize| {
                let across = volts(anode)? - volts(cathode)?;
                Some(across * Diode::default().current(across))
            };
            Some(diode(0, 2)? + diode(1, 2)? + diode(3, 0)? + diode(3, 1)?)
        }
        ElementKind::Zener { part } => {
            let across = volts(0)? - volts(1)?;
            Some(across * Zener::new(part).current(across))
        }
        ElementKind::Mosfet { channel } => {
            let mut potentials = [0.0; MAX_TERMINALS];
            for (pin, slot) in potentials.iter_mut().enumerate().take(3) {
                *slot = volts(pin)?;
            }
            // Linearised where it is, so nothing is held back and what it is
            // linearised at is what it is doing — the power part's body
            // diode included (C-112).
            let fet = Mosfet::new(channel);
            let companion = fet.linearise(&potentials, &fet.settled(&potentials));
            let watts: f64 = (0..3)
                .map(|i| {
                    let into = companion.currents[i]
                        + (0..3)
                            .map(|j| companion.conductances[i][j] * potentials[j])
                            .sum::<f64>();
                    potentials[i] * into
                })
                .sum();
            Some(watts.max(0.0))
        }
        ElementKind::Bjt { polarity } => {
            let mut potentials = [0.0; MAX_TERMINALS];
            for (pin, slot) in potentials.iter_mut().enumerate().take(3) {
                *slot = volts(pin)?;
            }
            // What flows in at each pin times the potential it flows in at
            // (C-112).
            let currents = Bjt::new(polarity).terminal_currents(&potentials);
            let watts: f64 = (0..3).map(|i| potentials[i] * currents[i]).sum();
            Some(watts.max(0.0))
        }
        ElementKind::Regulator { .. } | ElementKind::Adjustable => {
            let part = match element.kind {
                ElementKind::Regulator { part } => part,
                _ => RegulatorPart::Lm317,
            };
            let mut pins = [0.0; 3];
            for pin in [INPUT_PIN, GROUND_PIN, OUTPUT_PIN] {
                pins[pin] = volts(pin)?;
            }
            // Never below nothing: a part cannot be cooled by its circuit,
            // and a rounding at no load is not a refrigerator.
            Some(Regulator::new(part).watts(pins).max(0.0))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::netlist::Netlist;
    use crate::registry::rating_of;
    use crate::solver::device::RegulatorPart;
    use crate::solver::operating_point;

    const ROOM: f64 = 25.0;

    fn resistor() -> Thermal {
        rating_of(&ElementKind::Resistor { ohms: 220.0 })
            .thermal
            .expect("a resistor has a package")
    }

    /// The temperature after `duration`, taken in steps of `step`.
    fn after(thermal: &Thermal, watts: f64, duration: f64, step: f64) -> f64 {
        let mut celsius = ROOM;
        let mut at = 0.0;
        while at < duration - step / 2.0 {
            celsius = warm(thermal, celsius, ROOM, watts, step);
            at += step;
        }
        celsius
    }

    #[test]
    fn it_settles_at_the_room_plus_the_power_times_the_package() {
        // A quarter of a watt in a quarter-watt resistor: 85 K above the room.
        let thermal = resistor();
        let settled = after(&thermal, 0.25, 20.0 * thermal.time_constant, 0.1);
        let expected = ROOM + 0.25 * 340.0;
        assert!(
            (settled - expected).abs() < 0.01 * expected,
            "{settled} °C, expected {expected} °C"
        );
    }

    #[test]
    fn it_is_63_percent_of_the_way_there_after_one_time_constant() {
        let thermal = resistor();
        let rise = 0.25 * thermal.junction_to_ambient;
        let reached = after(&thermal, 0.25, thermal.time_constant, 0.01) - ROOM;
        let fraction = reached / rise;
        let expected = 1.0 - (-1.0f64).exp();
        assert!(
            (fraction - expected).abs() < 0.001,
            "{:.1} % of the way",
            fraction * 100.0
        );
    }

    #[test]
    fn the_curve_does_not_depend_on_how_finely_it_is_stepped() {
        // A frame and a microsecond step have to agree: the live loop takes
        // whichever the circuit asks for.
        let thermal = resistor();
        let coarse = after(&thermal, 0.3, 10.0, 1.0 / 60.0);
        let fine = after(&thermal, 0.3, 10.0, 1e-3);
        assert!((coarse - fine).abs() < 1e-6, "{coarse} against {fine}");
    }

    #[test]
    fn it_falls_back_to_the_room_when_the_power_goes() {
        let thermal = resistor();
        let hot = settles_at(&thermal, ROOM, 0.25);
        let cooled = warm(&thermal, hot, ROOM, 0.0, 20.0 * thermal.time_constant);
        assert!((cooled - ROOM).abs() < 1e-6);
    }

    #[test]
    fn a_warmer_room_is_where_everything_starts_and_ends() {
        let thermal = resistor();
        assert_eq!(settles_at(&thermal, 40.0, 0.0), 40.0);
        assert_eq!(settles_at(&thermal, 40.0, 0.1), 40.0 + 34.0);
    }

    /// A 12 V battery into a 7805 into 47 Ω.
    fn regulator_into(ohms: f64, supply: f64) -> (Netlist, f64) {
        let mut netlist = Netlist::new();
        let battery = netlist.add_element(ElementKind::DcSource {
            volts: supply,
            series_ohms: 0.0,
        });
        let regulator = netlist.add_element(ElementKind::Regulator {
            part: RegulatorPart::L7805,
        });
        let load = netlist.add_element(ElementKind::Resistor { ohms });
        let (input, output, ground) = (netlist.add_net(), netlist.add_net(), netlist.add_net());
        let pin = |element, index| crate::netlist::PinRef { element, index };
        netlist.connect(pin(battery, 0), input).unwrap();
        netlist.connect(pin(regulator, INPUT_PIN), input).unwrap();
        netlist.connect(pin(regulator, OUTPUT_PIN), output).unwrap();
        netlist.connect(pin(load, 0), output).unwrap();
        netlist.connect(pin(battery, 1), ground).unwrap();
        netlist.connect(pin(regulator, GROUND_PIN), ground).unwrap();
        netlist.connect(pin(load, 1), ground).unwrap();

        let point = operating_point(&netlist).unwrap();
        let watts = watts(netlist.element(regulator).unwrap(), &point).unwrap();
        (netlist, watts)
    }

    #[test]
    fn a_regulator_dissipates_its_drop_times_its_load() {
        // 7 V across it at about 106 mA, and its 5 mA of quiescent current
        // across the whole 12 V.
        let (_, watts) = regulator_into(47.0, 12.0);
        let load = 5.0 / 47.0;
        let expected = (12.0 - 5.0) * load + 12.0 * 5e-3;
        assert!(
            (watts - expected).abs() < 0.01 * expected,
            "{watts} W, expected {expected} W"
        );
    }

    #[test]
    fn a_regulator_with_less_to_drop_runs_cooler() {
        let (_, hot) = regulator_into(47.0, 12.0);
        let (_, cool) = regulator_into(47.0, 7.5);
        assert!(
            cool < hot / 2.0,
            "{cool} W at 7.5 V against {hot} W at 12 V"
        );
    }

    /// A battery, a series resistor and `kind`, with the pin `high` of it on
    /// the resistor and its other end on ground; `gate`, if there is one,
    /// held at the battery. Returns what the model says the part dissipates,
    /// and what its drop times the resistor's current says.
    fn dissipated_in(
        kind: ElementKind,
        high: usize,
        low: usize,
        gate: Option<usize>,
    ) -> (f64, f64) {
        const SUPPLY: f64 = 9.0;
        const OHMS: f64 = 220.0;
        let mut netlist = Netlist::new();
        let battery = netlist.add_element(ElementKind::DcSource {
            volts: SUPPLY,
            series_ohms: 0.0,
        });
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: OHMS });
        let part = netlist.add_element(kind);
        let (top, middle, ground) = (netlist.add_net(), netlist.add_net(), netlist.add_net());
        let pin = |element, index| crate::netlist::PinRef { element, index };
        netlist.connect(pin(battery, 0), top).unwrap();
        netlist.connect(pin(resistor, 0), top).unwrap();
        netlist.connect(pin(resistor, 1), middle).unwrap();
        netlist.connect(pin(part, high), middle).unwrap();
        netlist.connect(pin(part, low), ground).unwrap();
        if let Some(gate) = gate {
            netlist.connect(pin(part, gate), top).unwrap();
        }
        netlist.connect(pin(battery, 1), ground).unwrap();
        let point = operating_point(&netlist).unwrap();
        let across = point.voltage(middle).unwrap();
        let expected = across * (SUPPLY - across) / OHMS;
        (
            watts(netlist.element(part).unwrap(), &point).unwrap(),
            expected,
        )
    }

    fn assert_close(watts: f64, expected: f64) {
        assert!(
            (watts - expected).abs() < 1e-3 * expected,
            "{watts} W against {expected} W"
        );
    }

    #[test]
    fn a_diode_dissipates_its_drop_times_its_current() {
        let (watts, expected) = dissipated_in(ElementKind::Diode, 0, 1, None);
        assert_close(watts, expected);
        assert!(watts > 0.02, "tens of milliwatts, not {watts} W");
    }

    #[test]
    fn a_zener_in_breakdown_dissipates_its_voltage_times_its_current() {
        // Cathode up: 5.1 V across it at some 18 mA.
        let zener = ElementKind::Zener {
            part: crate::solver::device::ZenerPart::V5_1,
        };
        let (watts, expected) = dissipated_in(zener, 1, 0, None);
        assert_close(watts, expected);
        assert!(watts > 0.08, "{watts} W");
    }

    #[test]
    fn a_mosfet_dissipates_its_drop_times_its_drain_current() {
        use crate::solver::device::mosfet::{DRAIN, GATE, SOURCE};
        let n = ElementKind::Mosfet {
            channel: crate::solver::device::Channel::N,
        };
        let (watts, expected) = dissipated_in(n, DRAIN, SOURCE, Some(GATE));
        assert_close(watts, expected);
        assert!(watts > 1e-3, "fully on, a little: {watts} W");
    }

    #[test]
    fn a_resistor_dissipates_what_ohms_law_says() {
        let (netlist, _) = regulator_into(47.0, 12.0);
        let point = operating_point(&netlist).unwrap();
        let (_, load) = netlist
            .elements()
            .find(|(_, element)| matches!(element.kind, ElementKind::Resistor { .. }))
            .unwrap();
        let watts = watts(load, &point).unwrap();
        assert!(
            (watts - 25.0 / 47.0).abs() < 0.01 * 25.0 / 47.0,
            "{watts} W"
        );
    }
}
