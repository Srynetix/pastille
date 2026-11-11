// SPDX-License-Identifier: MIT OR Apache-2.0

//! Per-kind words and numbers: names, labels, values, units, part numbers.

use super::*;

pub(super) fn parse_kind(kind: &str) -> Result<ElementKind, CircuitError> {
    // Only what the app can draw. A kind the registry knows but nothing can
    // render would be a component that exists and cannot be seen (ADR 0008).
    crate::registry::definition(kind)
        .filter(|component| component.drawable)
        .map(|component| (component.blank)())
        .ok_or_else(|| CircuitError::UnknownKind(kind.to_string()))
}

pub(super) fn kind_name(kind: &ElementKind) -> &'static str {
    kind.name()
}

/// A strip's key for the view: the same for every hole one clip joins.
pub(super) fn strip_key(strip: breadboard::Strip) -> String {
    use breadboard::{Rail, Side, Strip};
    match strip {
        Strip::Row {
            row,
            side: Side::Upper,
        } => format!("{row}u"),
        Strip::Row {
            row,
            side: Side::Lower,
        } => format!("{row}l"),
        Strip::Rail(Rail::TopPlus) => "top+".to_string(),
        Strip::Rail(Rail::TopMinus) => "top-".to_string(),
        Strip::Rail(Rail::BottomMinus) => "bottom-".to_string(),
        Strip::Rail(Rail::BottomPlus) => "bottom+".to_string(),
    }
}

/// Strips as a sentence names them: "Row 12 (a–e) and the top + rail".
pub(super) fn strips_named(strips: &[breadboard::Strip]) -> String {
    use breadboard::{Side, Strip};
    let names: Vec<String> = strips
        .iter()
        .map(|strip| match *strip {
            Strip::Row {
                row,
                side: Side::Upper,
            } => format!("row {row} (a–e)"),
            Strip::Row {
                row,
                side: Side::Lower,
            } => format!("row {row} (f–j)"),
            Strip::Rail(rail) => format!("the {}", rail.name()),
        })
        .collect();
    let sentence = match names.split_last() {
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
        None => String::new(),
    };
    let mut chars = sentence.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

/// The part number to buy a part as, and its datasheet: the catalogue's for
/// an LED or a transistor (B-10), the registry's basis for anything else that
/// is a part in particular (C-112), nothing for an ideal one.
pub(super) fn part_number(kind: &ElementKind) -> (Option<String>, Option<String>) {
    let bought = match *kind {
        ElementKind::Led { colour } => crate::catalogue::leds(colour)
            .next()
            .map(|part| (part.number, part.datasheet)),
        ElementKind::Bjt { .. } | ElementKind::Mosfet { .. } => crate::catalogue::transistors(kind)
            .next()
            .map(|part| (part.number, part.datasheet)),
        _ => None,
    };
    if let Some((number, datasheet)) = bought {
        return (Some(number.to_string()), Some(datasheet.to_string()));
    }
    crate::registry::definition(kind.name())
        .and_then(|component| (component.basis)(kind))
        .map_or((None, None), |basis| {
            (
                Some(basis.part.to_string()),
                basis.datasheet.map(str::to_string),
            )
        })
}

/// What to write beside the symbol.
///
/// Engineering notation, because 220 Ω and 4.7 kΩ are how the values are spoken,
/// written on the part and sold. Parsing the other direction is M-18.
pub(super) fn label_for(kind: &ElementKind) -> String {
    match *kind {
        ElementKind::DcSource { volts, .. } => format!("{} V", trim(volts)),
        // Which battery it is; how full and how long, the run's (C-121).
        ElementKind::Cell { part, .. } => part.name().to_string(),
        // Both numbers, because either one changes what the circuit does; the
        // shape is drawn inside the symbol.
        //
        // Plain hertz below a kilohertz. A signal slow enough to watch is
        // what the live canvas can show, and "500 mHz" is a worse way to say
        // half a hertz to a beginner — and one capital away from a megahertz.
        ElementKind::SignalGenerator { volts, hertz, .. } => {
            let rate = if hertz >= 1e3 {
                value::format(hertz, "Hz")
            } else {
                format!("{} Hz", trim(hertz))
            };
            format!("±{} V {rate}", trim(volts))
        }
        ElementKind::Resistor { ohms } => {
            if ohms >= 1e6 {
                format!("{} MΩ", trim(ohms / 1e6))
            } else if ohms >= 1e3 {
                format!("{} kΩ", trim(ohms / 1e3))
            } else {
                format!("{} Ω", trim(ohms))
            }
        }
        // Kept now that the colour can be chosen (C-46): an LED that is not lit
        // draws no glow, so the word beside it is the only thing that says
        // which colour it is — and an unlit LED is most of them, most of the
        // time.
        ElementKind::Led { colour } => colour.name().to_string(),
        ElementKind::Capacitor { farads, .. } => value::format(farads, "F"),
        // Its capacitance and its rating, as both are printed on the can.
        ElementKind::Electrolytic { farads, rated, .. } => {
            format!("{} {}", value::format(farads, "F"), rated.name())
        }
        // The track end to end, which is what the part is sold as. Where the
        // wiper sits is drawn rather than written (C-20): the arrow moves.
        ElementKind::Potentiometer { ohms, .. } => value::format(ohms, "Ω"),
        // What the world is doing to it, which is what its dial says; its
        // resistance follows, and a meter reads that.
        ElementKind::Ldr { light } | ElementKind::Phototransistor { light } => {
            let lux = crate::solver::device::sensor::lux(light);
            format!("{} lux", trim(if lux >= 10.0 { lux.round() } else { lux }))
        }
        ElementKind::Thermistor { celsius } | ElementKind::Tmp36 { celsius } => {
            format!("{} °C", trim(celsius.round()))
        }
        // How much sun is on it, which is what its dial says (C-117).
        ElementKind::SolarCell { sun } => format!("{} % sun", trim((sun * 100.0).round())),
        ElementKind::Inductor { henries, .. } => value::format(henries, "H"),
        // Its turns, primary to secondary, as a transformer is sold.
        ElementKind::Transformer { ratio } => format!("{} : 1", trim(ratio)),
        ElementKind::Bridge => "DB107".to_string(),
        ElementKind::Lm3914 => "LM3914".to_string(),
        // Its rating, and the word that matters most once it has gone.
        ElementKind::Fuse { amps, blown, .. } => {
            let rating = value::format(amps, "A");
            if blown {
                format!("{rating} blown")
            } else {
                rating
            }
        }
        ElementKind::Bargraph => "10 LEDs".to_string(),
        ElementKind::Bjt { polarity } => match polarity {
            crate::solver::device::Polarity::Npn => "NPN".to_string(),
            crate::solver::device::Polarity::Pnp => "PNP".to_string(),
            // Its part number: drawn as an NPN, which is what it is outside.
            crate::solver::device::Polarity::Darlington => "TIP120".to_string(),
        },
        // Its part number, which is also its voltage to anyone who knows the
        // family and is what is printed on the part to anyone who does not.
        ElementKind::Regulator { part } => part.name().to_string(),
        ElementKind::Adjustable => crate::solver::device::RegulatorPart::Lm317
            .name()
            .to_string(),
        // The one part it can be, so the label is what is printed on it.
        ElementKind::OpAmp => crate::solver::device::opamp::PART.to_string(),
        ElementKind::Comparator => crate::solver::device::comparator::PART.to_string(),
        ElementKind::Timer => crate::solver::device::timer::PART.to_string(),
        // Its part number: the channel is drawn, and which part is the rest.
        ElementKind::Mosfet { channel } => channel.part_number().to_string(),
        ElementKind::Relay { part } => part.part_number().to_string(),
        ElementKind::SevenSegment { common } => common.part_number().to_string(),
        // Which end its dice share, which nothing on the symbol can show and
        // which decides whether a pin lights it high or low (C-84).
        ElementKind::RgbLed { common } => common.name().to_string(),
        ElementKind::LcdDigit => crate::solver::device::lcd::PART.to_string(),
        ElementKind::Decoder => crate::solver::device::decoder::PART.to_string(),
        ElementKind::Servo => crate::solver::device::servo::PART.to_string(),
        ElementKind::Counter => crate::solver::device::counter::PART.to_string(),
        ElementKind::MotorDriver => crate::solver::device::driver::PART.to_string(),
        ElementKind::Optocoupler => crate::solver::device::opto::PART.to_string(),
        ElementKind::ShiftRegister => crate::solver::device::shift::PART.to_string(),
        // Its part number: what each gate does is drawn.
        ElementKind::Gate { logic } => logic.part_number().to_string(),
        ElementKind::Inverter => crate::solver::device::gate::INVERTER_PART.to_string(),
        ElementKind::Schmitt => crate::solver::device::gate::SCHMITT_PART.to_string(),
        ElementKind::Decade => crate::solver::device::decade::PART.to_string(),
        ElementKind::FlipFlop => crate::solver::device::flipflop::PART.to_string(),
        ElementKind::Hall { .. } => crate::solver::device::magnet::PART.to_string(),
        ElementKind::Uno => "Arduino Uno".to_string(),
        // Its delay, which is what its knob is set to.
        ElementKind::Pir { hold, .. } => value::format(hold, "s"),
        // Its magnet's distance is on its dial, as an LDR's light is.
        ElementKind::Reed { .. } => String::new(),
        // What a kit calls them; nothing about either is set.
        ElementKind::Buzzer | ElementKind::Motor | ElementKind::PassiveBuzzer => String::new(),
        // Nothing to write: the symbol is drawn shut or open, which says it
        // better than a word beside it would and in every language.
        ElementKind::Switch { .. }
        | ElementKind::PushButton { .. }
        | ElementKind::TiltSwitch { .. } => String::new(),
        // Nor here. A diode has no property to name — the bar on the symbol
        // says which way round it is, which is the only thing to know.
        ElementKind::Diode | ElementKind::Schottky => String::new(),
        // Its voltage, which is the one thing about it anybody asks.
        ElementKind::Zener { part } => part.name().to_string(),
        // A probe's label is its reading, and a reading belongs to the solution
        // rather than to the element: it moves sixty times a second and this is
        // rebuilt only when the circuit does. The app writes it there (C-21).
        ElementKind::Voltmeter | ElementKind::Ammeter => String::new(),
        ElementKind::Ground => String::new(),
    }
}

/// The editable magnitude of a component, if it has one.
pub(super) fn value_of(kind: &ElementKind) -> Option<f64> {
    match *kind {
        ElementKind::DcSource { volts, .. } | ElementKind::SignalGenerator { volts, .. } => {
            Some(volts)
        }
        ElementKind::Resistor { ohms } => Some(ohms),
        ElementKind::Capacitor { farads, .. } | ElementKind::Electrolytic { farads, .. } => {
            Some(farads)
        }
        ElementKind::Inductor { henries, .. } => Some(henries),
        ElementKind::Potentiometer { ohms, .. } => Some(ohms),
        ElementKind::Pir { hold, .. } => Some(hold),
        ElementKind::Transformer { ratio } => Some(ratio),
        ElementKind::Fuse { amps, .. } => Some(amps),
        // A contact is shut or open, and a probe reads rather than is set:
        // neither is a number to type.
        ElementKind::Switch { .. }
        | ElementKind::PushButton { .. }
        | ElementKind::TiltSwitch { .. }
        | ElementKind::Voltmeter
        | ElementKind::Ammeter
        | ElementKind::Led { .. }
        | ElementKind::Diode
        | ElementKind::Schottky
        | ElementKind::Bridge
        | ElementKind::Lm3914
        | ElementKind::Bargraph
        // Its charge is a dial; what it is, a choice (C-121).
        | ElementKind::Cell { .. }
        | ElementKind::Zener { .. }
        | ElementKind::Ldr { .. }
        | ElementKind::Phototransistor { .. }
        | ElementKind::SolarCell { .. }
        | ElementKind::Thermistor { .. }
        | ElementKind::Bjt { .. }
        | ElementKind::Mosfet { .. }
        | ElementKind::Regulator { .. }
        | ElementKind::Adjustable
        | ElementKind::OpAmp
        | ElementKind::Tmp36 { .. }
        | ElementKind::Comparator
        | ElementKind::Timer
        | ElementKind::Relay { .. }
        | ElementKind::Buzzer
        | ElementKind::PassiveBuzzer
        | ElementKind::Motor
        | ElementKind::SevenSegment { .. }
        | ElementKind::RgbLed { .. }
        | ElementKind::LcdDigit
        | ElementKind::Decoder
        | ElementKind::Servo
        | ElementKind::Counter
        | ElementKind::MotorDriver
        | ElementKind::Optocoupler
        | ElementKind::ShiftRegister
        | ElementKind::Gate { .. }
        | ElementKind::Inverter
        | ElementKind::Schmitt
        | ElementKind::Decade
        | ElementKind::FlipFlop
        | ElementKind::Hall { .. }
        | ElementKind::Reed { .. }
        | ElementKind::Uno
        | ElementKind::Ground => None,
    }
}

pub(super) fn unit_of(kind: &ElementKind) -> &'static str {
    crate::registry::definition(kind.name())
        .map(|component| component.unit)
        .unwrap_or("")
}

/// Drops a trailing `.0` so a 9 V battery is not labelled "9.0 V".
pub(super) fn trim(value: f64) -> String {
    let text = format!("{value:.2}");
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}
