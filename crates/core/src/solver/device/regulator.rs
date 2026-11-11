// SPDX-License-Identifier: MIT OR Apache-2.0

//! The linear regulator: a fixed voltage out, whatever the battery does.
//!
//! The first part built on the saturating source of C-53, and modelled from
//! its datasheet rather than its die (ADR 0010). What it honours: its output
//! voltage; its dropout, below which it follows its input down; the quiescent
//! current its ground pin carries; the current limit that lets it survive a
//! shorted output. What it ignores: line and load regulation beyond what ten
//! milliohms of output resistance amounts to, ripple rejection, thermal
//! shutdown and start-up — nothing here has a past.
//!
//! Each choice is one real part a person can buy (ADR 0010, term 3), and the
//! name it crosses the boundary under is its part number: a beginner holding
//! the part reads the same thing on its case.

use crate::solver::MAX_TERMINALS;

use super::saturating::{HIGH_RAIL, INVERTING, LOW_RAIL, NON_INVERTING, OUTPUT, SaturatingSource};

/// Pin order: the order the registry names them in.
pub const INPUT_PIN: usize = 0;
pub const GROUND_PIN: usize = 1;
pub const OUTPUT_PIN: usize = 2;

/// Which of the regulator's pins each terminal of the saturating source sits
/// on, in terminal order.
///
/// A regulator has no inputs to compare — its target is its output voltage
/// above its ground pin — so both inputs sit on the ground pin, where with a
/// gain of zero they change nothing. The high rail is the input, which is what
/// makes the battery pay for the load; the low rail is the ground pin, which
/// is what the quiescent current returns through.
pub const WIRING: [usize; 5] = {
    let mut wiring = [0; 5];
    wiring[NON_INVERTING] = GROUND_PIN;
    wiring[INVERTING] = GROUND_PIN;
    wiring[OUTPUT] = OUTPUT_PIN;
    wiring[HIGH_RAIL] = INPUT_PIN;
    wiring[LOW_RAIL] = GROUND_PIN;
    wiring
};

/// Which part it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RegulatorPart {
    /// 5 V from the 78xx family, in TO-220: what an Arduino's barrel jack
    /// feeds, and the regulator in every beginner's kit.
    L7805,
    /// The same family at 12 V, which a 9 V battery cannot feed — the choice
    /// exists partly to be the one that teaches dropout.
    L7812,
    /// 3.3 V at low dropout, in SOT-223: what sits in front of an ESP32.
    Lm1117,
    /// The adjustable one (C-113): 1.25 V held between its output and its
    /// ADJ pin, so two resistors set what it gives. Not one of the fixed
    /// regulator's choices — its middle pin is not a ground — but the
    /// [`ElementKind::Adjustable`](crate::netlist::ElementKind::Adjustable)
    /// component, on this same model.
    Lm317,
}

/// A regulator's datasheet, as far as ADR 0010 honours it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Datasheet {
    /// Volts out, above the ground pin.
    pub output_volts: f64,
    /// How far above the output the input has to be for the output to hold.
    pub dropout_volts: f64,
    /// Current through the ground pin, in amps.
    pub quiescent_amps: f64,
    /// Most it will deliver, into anything, a short included.
    pub current_limit: f64,
    /// Most the input may be above the ground pin.
    pub max_input_volts: f64,
    /// Thermal resistance from the die to still air with no heatsink, in
    /// kelvin per watt. The package's figure, and what B-06 divides by.
    pub junction_to_ambient: f64,
    /// Hottest the die may run, in °C.
    pub max_junction_celsius: f64,
    /// How long the package takes to cover 63 % of the way to the
    /// temperature it will settle at, in seconds (C-74).
    pub thermal_seconds: f64,
}

impl RegulatorPart {
    /// Every part, in the order a chooser should offer them.
    pub const ALL: &'static [RegulatorPart] = &[
        RegulatorPart::L7805,
        RegulatorPart::Lm1117,
        RegulatorPart::L7812,
    ];

    /// The name this part crosses the boundary under: its part number.
    pub fn name(&self) -> &'static str {
        match self {
            RegulatorPart::L7805 => "7805",
            RegulatorPart::L7812 => "7812",
            RegulatorPart::Lm1117 => "LM1117-3.3",
            RegulatorPart::Lm317 => "LM317",
        }
    }

    /// The part of that name, or nothing if it is not one.
    pub fn named(name: &str) -> Option<RegulatorPart> {
        RegulatorPart::ALL
            .iter()
            .copied()
            .find(|part| part.name() == name)
    }

    /// Typical figures from the part's datasheet, rounded to what a beginner's
    /// circuit can tell apart.
    pub fn datasheet(&self) -> Datasheet {
        // The 78xx family shares everything but its output voltage: the same
        // two volts of dropout, the same few milliamps in the ground pin, the
        // same 1.5 A and the same TO-220 — 50 K/W in still air and a die good
        // to 125 °C. So a 7805 with nothing bolted to it is warm by half a watt
        // and in trouble past two, which is the arithmetic B-06 will do.
        //
        // No datasheet gives a package's time constant, so it is worked out
        // from what the package weighs (C-74): a TO-220 is about a gram of
        // copper tab and a gram of epoxy, some 1.3 J/K, and 50 K/W times that
        // is about 65 s. An estimate, and a minute or so is what a hand on one
        // feels.
        let seventy_eight = |output_volts| Datasheet {
            output_volts,
            dropout_volts: 2.0,
            quiescent_amps: 5e-3,
            current_limit: 1.5,
            max_input_volts: 35.0,
            junction_to_ambient: 50.0,
            max_junction_celsius: 125.0,
            thermal_seconds: 65.0,
        };
        match self {
            RegulatorPart::L7805 => seventy_eight(5.0),
            RegulatorPart::L7812 => seventy_eight(12.0),
            // Low dropout is the reason it exists: 1.2 V at full load, against
            // the 78xx's 2. A smaller package with a worse way of shedding heat,
            // and 15 V the most it may be fed.
            RegulatorPart::Lm1117 => Datasheet {
                output_volts: 3.3,
                dropout_volts: 1.2,
                quiescent_amps: 5e-3,
                current_limit: 0.8,
                max_input_volts: 15.0,
                junction_to_ambient: 60.0,
                max_junction_celsius: 125.0,
                // A SOT-223 weighs a tenth of a gram, and its 60 K/W is the
                // board's copper as much as its own: counting a square inch of
                // it and the board beneath, some 0.4 J/K, so about 25 s.
                thermal_seconds: 25.0,
            },
            // From TI's LM317 sheet (SLVS044Z): 1.25 V between output and
            // ADJ; 50 µA out of the ADJ pin, which is this model's quiescent
            // current, since it leaves by the pin a 78xx's leaves by; 1.5 A;
            // 40 V across it at most. Its dropout is not a figure the sheet
            // states — it promises its figures from 3 V across it — so the
            // 78xx's 2 V is taken. Its TO-220's 23.5 K/W is on a JEDEC test
            // board; standing in a breadboard it sheds as the 7805's does.
            // Ignored: the 3.5 to 10 mA it needs drawn to hold its output,
            // which R1 = 240 Ω is there to draw, and which a load here never
            // has to — the model holds without it.
            RegulatorPart::Lm317 => Datasheet {
                output_volts: 1.25,
                dropout_volts: 2.0,
                quiescent_amps: 50e-6,
                current_limit: 1.5,
                max_input_volts: 40.0,
                junction_to_ambient: 50.0,
                max_junction_celsius: 125.0,
                thermal_seconds: 65.0,
            },
        }
    }
}

/// A linear regulator, as the solver sees it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Regulator {
    pub part: RegulatorPart,
}

/// Resistance behind the output, in ohms.
///
/// Ten milliohms: a 7805's load regulation is tens of millivolts across its
/// whole range, and this is that, rounded down to what a reading shows.
const OUTPUT_OHMS: f64 = 0.01;

/// What the output can take back, in amps.
///
/// A regulator's output stage only pushes, but it is not an open circuit the
/// other way: its own feedback divider hangs off the output and draws from
/// it. A milliamp stands for that, and it keeps an output with no load from
/// sitting on a corner at exactly zero current (C-53).
const SINK_AMPS: f64 = 1e-3;

impl Regulator {
    pub fn new(part: RegulatorPart) -> Self {
        Regulator { part }
    }

    /// The saturating source that behaves like this part, wired by [`WIRING`].
    pub fn model(&self) -> SaturatingSource {
        let sheet = self.part.datasheet();
        SaturatingSource {
            // Nothing to amplify: the target is a fixed height above the
            // ground pin, and the dropout is what the high rail takes off it.
            gain: 0.0,
            offset: sheet.output_volts,
            reference: 0.0,
            headroom_high: sheet.dropout_volts,
            headroom_low: 0.0,
            output_ohms: OUTPUT_OHMS,
            source_limit: sheet.current_limit,
            sink_limit: SINK_AMPS,
            quiescent: sheet.quiescent_amps,
            input_ohms: f64::INFINITY,
        }
    }

    /// Watts it turns into heat with its pins at these potentials, in pin
    /// order (C-74).
    ///
    /// What goes in at every pin less what comes out, which for a regulator
    /// is the drop from input to output times the load, plus its quiescent
    /// current times its input: the whole reason a 7805 fed from 12 V runs
    /// hot and one fed from 7 V does not.
    pub fn watts(&self, pins: [f64; 3]) -> f64 {
        let mut potentials = [0.0; MAX_TERMINALS];
        for (terminal, &pin) in WIRING.iter().enumerate() {
            potentials[terminal] = pins[pin];
        }
        let into = self.model().currents(&potentials);
        WIRING
            .iter()
            .enumerate()
            .map(|(terminal, &pin)| pins[pin] * into[terminal])
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_part_is_found_by_its_own_name() {
        for part in RegulatorPart::ALL {
            assert_eq!(RegulatorPart::named(part.name()), Some(*part));
        }
        assert_eq!(RegulatorPart::named("LM317"), None);
    }

    #[test]
    fn the_wiring_puts_every_terminal_on_a_pin() {
        // Three pins, five terminals: every terminal lands on one of the
        // three, and the output is the only one on the output pin.
        assert!(WIRING.iter().all(|&pin| pin <= OUTPUT_PIN));
        assert_eq!(WIRING[OUTPUT], OUTPUT_PIN);
        assert_eq!(
            WIRING.iter().filter(|&&pin| pin == OUTPUT_PIN).count(),
            1,
            "only the output drives the output pin"
        );
    }

    #[test]
    fn a_low_dropout_part_drops_out_later() {
        let (seventy_eight, ldo) = (
            RegulatorPart::L7805.datasheet(),
            RegulatorPart::Lm1117.datasheet(),
        );
        assert!(ldo.dropout_volts < seventy_eight.dropout_volts);
    }
}
