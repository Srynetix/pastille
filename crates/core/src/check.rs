// SPDX-License-Identifier: MIT OR Apache-2.0

//! What a part can be set to, said once for every way it can be set (C-96).
//!
//! A value typed in the Inspector and a value read from a file are the same
//! value, and a part that cannot exist is the same part whichever way it
//! arrived. Before this, the Inspector refused a frequency of nothing and a
//! negative internal resistance and took a resistor of 0 Ω — a beginner's
//! "make it a wire" — which the solver then left out of the circuit, killing
//! the loop without a word and reporting an infinite current; and a file took
//! anything at all. Every setter and every load now asks here first.
//!
//! The answer is a sentence for the person who typed it: what is wrong and
//! what is accepted, and for the two values whose nothing is a wire, where the
//! wire is.

use crate::netlist::ElementKind;

/// Why a part cannot be what it was asked to be, or nothing if it can.
pub fn check(kind: &ElementKind) -> Result<(), String> {
    let positive = |value: f64, sentence: &str| {
        if value > 0.0 && value.is_finite() {
            Ok(())
        } else {
            Err(sentence.to_string())
        }
    };
    let finite = |value: f64| {
        if value.is_finite() {
            Ok(())
        } else {
            Err("That is not a number this can be set to.".to_string())
        }
    };
    let within = |value: f64, low: f64, high: f64, sentence: &str| {
        if (low..=high).contains(&value) {
            Ok(())
        } else {
            Err(sentence.to_string())
        }
    };
    match *kind {
        ElementKind::Cell { charge, .. } => {
            within(charge, 0.0, 1.0, "A battery is from empty to full.")
        }
        ElementKind::Fuse { amps, part, .. } => {
            use crate::solver::device::fuse;
            finite(amps)?;
            if fuse::model(part, amps).is_none() {
                return Err(format!(
                    "A {} here comes in {}.",
                    part.name().to_lowercase(),
                    fuse::ratings_said(part)
                ));
            }
            Ok(())
        }
        ElementKind::Transformer { ratio } => {
            use crate::solver::device::transformer::{FEWEST, MOST};
            within(
                ratio,
                FEWEST,
                MOST,
                &format!("A transformer here is wound from {FEWEST} : 1 to {MOST} : 1."),
            )
        }
        ElementKind::Pir { hold, .. } => within(
            hold,
            crate::solver::device::pir::SHORTEST_HOLD,
            crate::solver::device::pir::LONGEST_HOLD,
            "Its delay knob turns from 5 s to 200 s.",
        ),
        ElementKind::Resistor { ohms } => positive(
            ohms,
            "A resistance has to be more than 0 Ω. To join two points with nothing between \
             them, draw a wire.",
        ),
        ElementKind::Potentiometer { ohms, wiper } => {
            positive(ohms, "A potentiometer's track has to be more than 0 Ω.")?;
            within(wiper, 0.0, 1.0, "A knob turns from 0 to 1.")
        }
        ElementKind::Capacitor {
            farads,
            initial_volts,
        }
        | ElementKind::Electrolytic {
            farads,
            initial_volts,
            ..
        } => {
            positive(farads, "A capacitance has to be more than 0 F.")?;
            finite(initial_volts)
        }
        ElementKind::Inductor {
            henries,
            initial_amps,
        } => {
            positive(
                henries,
                "An inductance has to be more than 0 H. To join two points with nothing \
                 between them, draw a wire.",
            )?;
            finite(initial_amps)
        }
        ElementKind::DcSource { volts, series_ohms } => {
            finite(volts)?;
            within(
                series_ohms,
                0.0,
                f64::MAX,
                "An internal resistance cannot be below 0 Ω: 0 is an ideal battery.",
            )
        }
        ElementKind::SignalGenerator { volts, hertz, .. } => {
            finite(volts)?;
            positive(hertz, "A frequency has to be more than 0 Hz.")
        }
        ElementKind::Ldr { light }
        | ElementKind::Phototransistor { light }
        | ElementKind::SolarCell { sun: light } => {
            within(light, 0.0, 1.0, "The light dial turns from 0 to 1.")
        }
        ElementKind::Thermistor { celsius } => {
            use crate::solver::device::sensor::{COLDEST, HOTTEST};
            within(
                celsius,
                COLDEST,
                HOTTEST,
                &format!("A thermistor here reads from {COLDEST} °C to {HOTTEST} °C."),
            )
        }
        ElementKind::Tmp36 { celsius } => {
            use crate::solver::device::tmp36::{COLDEST, HOTTEST};
            within(
                celsius,
                COLDEST,
                HOTTEST,
                &format!("A TMP36 reads from {COLDEST} °C to {HOTTEST} °C."),
            )
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::REGISTRY;

    #[test]
    fn every_part_straight_out_of_the_palette_passes() {
        for component in REGISTRY {
            assert_eq!(check(&(component.blank)()), Ok(()), "{}", component.kind);
        }
    }

    #[test]
    fn nothing_and_less_than_nothing_are_refused_where_they_cannot_be() {
        for ohms in [0.0, -100.0, f64::NAN, f64::INFINITY] {
            assert!(check(&ElementKind::Resistor { ohms }).is_err(), "{ohms} Ω");
        }
        assert!(
            check(&ElementKind::Inductor {
                henries: 0.0,
                initial_amps: 0.0
            })
            .is_err()
        );
        assert!(
            check(&ElementKind::Capacitor {
                farads: -1e-6,
                initial_volts: 0.0
            })
            .is_err()
        );
        assert!(
            check(&ElementKind::Potentiometer {
                ohms: 10e3,
                wiper: 7.0
            })
            .is_err()
        );
        assert!(
            check(&ElementKind::DcSource {
                volts: 9.0,
                series_ohms: -1.0
            })
            .is_err()
        );
        assert!(check(&ElementKind::Thermistor { celsius: -400.0 }).is_err());
    }

    #[test]
    fn a_battery_the_other_way_round_is_still_a_battery() {
        assert_eq!(
            check(&ElementKind::DcSource {
                volts: -9.0,
                series_ohms: 0.0
            }),
            Ok(())
        );
    }

    #[test]
    fn a_zero_ohm_resistor_is_told_where_the_wire_is() {
        let said = check(&ElementKind::Resistor { ohms: 0.0 }).unwrap_err();
        assert!(said.contains("draw a wire"), "{said}");
    }
}
