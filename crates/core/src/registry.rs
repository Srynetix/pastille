// SPDX-License-Identifier: MIT OR Apache-2.0

//! What is true about each kind of component.
//!
//! One entry per kind, and the only place the core learns that a resistor has
//! pins `a` and `b`, is measured in ohms, and is 220 Ω until somebody says
//! otherwise. Everything electrical, and everything a saved file or an export
//! has to agree about.
//!
//! What a component *looks like* and what it is *called* are not here: they are
//! the app's, in `symbols.ts` and `palette.ts` respectively. ADR 0008 says why,
//! and which tests hold the three together.

use wasm_bindgen::prelude::wasm_bindgen;

use crate::netlist::{ElementKind, LedColour};
use crate::solver::device::comparator;
use crate::solver::device::digit::Common;
use crate::solver::device::electrolytic::Rated;
use crate::solver::device::{
    Channel, OpAmp, Polarity, RATED_CURRENT, RegulatorPart, RelayPart, Waveform, ZENER_WATTS,
    ZenerPart,
};

/// What a component is built to withstand.
///
/// Recorded now and consumed later: B-03's rule is "current or power rating
/// exceeded", and it can only be written once there is somewhere to read the
/// rating from. `None` means the part has no meaningful limit at this level of
/// modelling rather than that it has none at all.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rating {
    /// Largest current it is meant to carry, in amps.
    pub current: Option<f64>,
    /// Largest power it is meant to dissipate, in watts.
    pub power: Option<f64>,
    /// Largest voltage it is meant to be given, in volts. For a regulator,
    /// its input above its ground pin.
    pub voltage: Option<f64>,
    /// How it sheds heat, for a part whose limit is its temperature rather
    /// than a power it can be given a number for (C-54).
    pub thermal: Option<Thermal>,
}

/// How a package gets rid of its heat, with nothing bolted to it.
///
/// A regulator's limit is not a power: it is the temperature its die reaches,
/// which is the power it dissipates times how badly its package sheds it, on
/// top of the room. B-06's rule is that sum, and these are its two numbers.
///
/// With the third, how long it takes to get there, it is also everything a
/// run needs to carry the part's temperature (C-74): a part that has one is a
/// part that warms, and a part with no sourced figure has none rather than an
/// invented one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Thermal {
    /// Kelvin of die temperature per watt dissipated, in still air.
    pub junction_to_ambient: f64,
    /// Hottest the die may run, in °C.
    pub max_junction_celsius: f64,
    /// Seconds to cover 63 % of the way to where it will settle.
    pub time_constant: f64,
}

/// A quarter-watt carbon film resistor, as its datasheet derates it.
///
/// Full power up to 70 °C around it, nothing at 155 °C: so its body climbs
/// 85 K for a quarter of a watt, 340 K/W, and 155 °C is as hot as it may get
/// (Yageo's CFR-25, and the same curve in every assortment box's datasheet).
/// Nobody publishes its time constant; it is taken from what it weighs, a
/// tenth of a gram of ceramic and coating at about 1 J/K per gram, times its
/// 340 K/W: some 30 s. An estimate, and "tens of seconds" is what a finger on
/// one finds.
const QUARTER_WATT_RESISTOR: Thermal = Thermal {
    junction_to_ambient: 340.0,
    max_junction_celsius: 155.0,
    time_constant: 30.0,
};

/// A 1N4148 in its glass DO-35: 350 K/W with 4 mm of lead, and a junction
/// good to 175 °C (Vishay). It weighs 105 mg; at about 0.75 J/K per gram of
/// glass and lead that is some 0.08 J/K, and times 350 K/W about 30 s.
const SMALL_SIGNAL_DIODE: Thermal = Thermal {
    junction_to_ambient: 350.0,
    max_junction_celsius: 175.0,
    time_constant: 30.0,
};

/// A 1N5819 in its DO-41: 80 K/W on a board and 125 °C (onsemi). The same
/// package as a 1N47xx, some 0.25 J/K, so about 20 s.
const SCHOTTKY_RECTIFIER: Thermal = Thermal {
    junction_to_ambient: 80.0,
    max_junction_celsius: 125.0,
    time_constant: 20.0,
};

/// A DB107 in its DB-1 package: 40 K/W on a board and 150 °C (Diodes Inc).
/// A gram of epoxy and four dice, some 1 J/K, so about 40 s.
const BRIDGE_RECTIFIER: Thermal = Thermal {
    junction_to_ambient: 40.0,
    max_junction_celsius: 150.0,
    time_constant: 40.0,
};

/// A 1N47xx in its DO-41: 110 K/W and 175 °C (Vishay). 310 mg of epoxy and
/// copper lead, some 0.25 J/K, so about 30 s.
const ONE_WATT_ZENER: Thermal = Thermal {
    junction_to_ambient: 110.0,
    max_junction_celsius: 175.0,
    time_constant: 30.0,
};

/// A TO-92 weighs about 0.2 g, which nobody's datasheet here says: a typical
/// figure for the package, some 0.16 J/K, and each part's own θja makes the
/// time constant. The 2N7000's 312.5 K/W and 150 °C are onsemi's; the BS250's
/// 150 K/W and 150 °C are Philips's, the sheet its 0.83 W comes from.
const TO92_2N7000: Thermal = Thermal {
    junction_to_ambient: 312.5,
    max_junction_celsius: 150.0,
    time_constant: 50.0,
};
const TO92_BS250: Thermal = Thermal {
    junction_to_ambient: 150.0,
    max_junction_celsius: 150.0,
    time_constant: 25.0,
};

/// A TO-220 with nothing bolted to it (C-112): the IRLZ44N's 62 K/W and
/// 175 °C are Infineon's, the TIP120's 62.5 K/W and 150 °C onsemi's. Neither
/// sheet gives a time constant; a TO-220 weighs some 2 g, mostly its copper
/// tab, about 0.8 J/K, which times 62 K/W is some 50 s.
const TO220_IRLZ44N: Thermal = Thermal {
    junction_to_ambient: 62.0,
    max_junction_celsius: 175.0,
    time_constant: 50.0,
};
const TO220_TIP120: Thermal = Thermal {
    junction_to_ambient: 62.5,
    max_junction_celsius: 150.0,
    time_constant: 50.0,
};

/// The real part a component is modelled on, and where its figures come from.
///
/// A fact about the model rather than about the drawing: every number in the
/// entry above it is read off this part's datasheet, so the part is the
/// claim a reader can check (ADR 0008 keeps what it is *called* in the app;
/// what it *is* is here).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Basis {
    /// The part number, and the maker whose sheet was used when there is one.
    pub part: &'static str,
    /// The datasheet, when there is one worth linking.
    pub datasheet: Option<&'static str>,
    /// Whether the model is only close to it — tuned to its kind of figures
    /// rather than to its sheet line by line.
    pub approximate: bool,
}

impl Basis {
    const fn sheet(part: &'static str, datasheet: &'static str) -> Option<Basis> {
        Some(Basis {
            part,
            datasheet: Some(datasheet),
            approximate: false,
        })
    }
}

impl Rating {
    const NONE: Rating = Rating {
        current: None,
        power: None,
        voltage: None,
        thermal: None,
    };
}

/// Everything the core knows about one kind of component.
#[derive(Debug, Clone, Copy)]
pub struct Component {
    /// The name that crosses the boundary: "DcSource", "Resistor", …
    pub kind: &'static str,
    /// Pin names, in pin order. Polarity is part of the name because it is part
    /// of the electrical meaning.
    pub pins: &'static [&'static str],
    /// Unit the component's own number is in, empty for one that has none.
    ///
    /// Normally the value a field edits — ohms, farads. For a probe there is
    /// nothing to edit and the unit is what it *reads* in (C-21), which is the
    /// same fact about the same component and belongs in the same place.
    pub unit: &'static str,
    /// A fresh one, with the value a beginner would reach for first.
    pub blank: fn() -> ElementKind,
    /// The alternatives this kind can be set to, empty for one with none.
    ///
    /// An LED's colours, a transistor's polarities: a closed set rather than a
    /// number, and the only place that set is written (C-46). The app is handed
    /// it rather than keeping a copy, and a test holds the two together the way
    /// the pin names have been held since M-17.
    pub choices: fn() -> Vec<&'static str>,
    /// What this one is built to withstand.
    ///
    /// Asked of the element rather than of the kind, because a choice can be
    /// a different part: a 7805 and an LM1117 are both regulators and have
    /// neither the same current limit nor the same package (C-54). For every
    /// other kind the answer is the same whatever it is set to.
    pub rating: fn(&ElementKind) -> Rating,
    /// The real part this one is modelled on, or nothing for an ideal one —
    /// a battery, a wire, a meter — that is not any part in particular.
    pub basis: fn(&ElementKind) -> Option<Basis>,
    /// Whether v1's palette offers it.
    pub offered: bool,
    /// Whether the app can draw one that exists.
    ///
    /// Two facts, not one. A ground is drawable and not offered — the element
    /// remains for circuits that contain one and the palette does not ask the
    /// question (C-34). A capacitor is currently neither, and C-18 and C-28 are
    /// what flip it.
    pub drawable: bool,
}

/// Every kind the core knows, in the order the palette offers them.
pub const REGISTRY: &[Component] = &[
    Component {
        kind: "DcSource",
        pins: &["+", "-"],
        unit: "V",
        blank: || ElementKind::DcSource {
            volts: 9.0,
            // Ideal by default: every textbook circuit assumes it, and a
            // beginner asked to pick an internal resistance would have no way
            // to answer.
            series_ohms: 0.0,
        },
        choices: Vec::new,
        rating: |_| Rating::NONE,
        basis: |_| None,
        offered: true,
        drawable: true,
    },
    Component {
        kind: "SignalGenerator",
        // A battery's two ends, because it is wired like one and a beginner
        // who has placed a battery knows which end is which.
        pins: &["+", "-"],
        unit: "V",
        blank: || ElementKind::SignalGenerator {
            // A square wave at one hertz, five volts either side: an LED on
            // it blinks once a second, and the scope's ten seconds show ten
            // cycles (ADR 0011).
            waveform: Waveform::Square,
            volts: 5.0,
            hertz: 1.0,
        },
        choices: || Waveform::ALL.iter().map(Waveform::name).collect(),
        rating: |_| Rating::NONE,
        basis: |_| None,
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Cell",
        // A battery's two ends.
        pins: &["+", "-"],
        unit: "",
        // A fresh 9 V, the one every kit has.
        blank: || ElementKind::Cell {
            part: crate::solver::device::cell::CellPart::NineVolt,
            charge: 1.0,
        },
        choices: || {
            crate::solver::device::cell::CellPart::ALL
                .iter()
                .map(crate::solver::device::cell::CellPart::name)
                .collect()
        },
        rating: |_| Rating::NONE,
        basis: |kind| match *kind {
            ElementKind::Cell { part, .. } => {
                let sheet = part.sheet();
                Basis::sheet(sheet.part, sheet.datasheet)
            }
            _ => None,
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "SolarCell",
        // A battery's two ends: it is a source, wired as one.
        pins: &["+", "-"],
        unit: "",
        // Full sun, so placing one shows its sheet's open-circuit voltage.
        blank: || ElementKind::SolarCell { sun: 1.0 },
        choices: Vec::new,
        // The sheet gives none: a panel cannot give more than its own
        // short-circuit current, and nothing here drives one backwards.
        rating: |_| Rating::NONE,
        basis: |_| {
            Basis::sheet(
                crate::solver::device::solar::PART,
                crate::solver::device::solar::DATASHEET,
            )
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Led",
        pins: &["anode", "cathode"],
        unit: "",
        blank: || ElementKind::Led {
            colour: LedColour::Red,
        },
        choices: || LedColour::ALL.iter().map(LedColour::name).collect(),
        rating: |_| Rating {
            current: Some(RATED_CURRENT),
            power: None,
            ..Rating::NONE
        },
        basis: |_| {
            Some(Basis {
                part: "5 mm indicator LED",
                datasheet: None,
                approximate: true,
            })
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "RgbLed",
        // The dice in the order the name spells them, then the pin they
        // share (C-84).
        pins: &["R", "G", "B", "com"],
        unit: "",
        blank: || ElementKind::RgbLed {
            common: Common::Cathode,
        },
        choices: || Common::ALL.iter().map(Common::name).collect(),
        // Each die is an indicator LED, rated as one.
        rating: |_| Rating {
            current: Some(RATED_CURRENT),
            ..Rating::NONE
        },
        // No part number anybody would recognise, and its dice are the single
        // LEDs' — whose green is not the InGaN one a real RGB part has.
        basis: |_| {
            Some(Basis {
                part: crate::solver::device::rgb::PART,
                datasheet: None,
                approximate: true,
            })
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "SevenSegment",
        // The segments in the order they are lettered, the point, then the
        // pin they share (C-75).
        pins: &["a", "b", "c", "d", "e", "f", "g", "dp", "com"],
        unit: "",
        blank: || ElementKind::SevenSegment {
            common: Common::Cathode,
        },
        choices: || Common::ALL.iter().map(Common::name).collect(),
        // Each segment is an indicator LED, rated as one.
        rating: |kind| match *kind {
            ElementKind::SevenSegment { .. } => Rating {
                current: Some(RATED_CURRENT),
                ..Rating::NONE
            },
            _ => Rating::NONE,
        },
        basis: |kind| match *kind {
            ElementKind::SevenSegment { common } => Some(Basis {
                part: common.part_number(),
                datasheet: None,
                approximate: false,
            }),
            _ => None,
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "LcdDigit",
        // The LED digit's pins, in its order: a digit is lettered the same
        // whatever it is made of (C-76). COM is the backplane.
        pins: &["a", "b", "c", "d", "e", "f", "g", "dp", "com"],
        unit: "",
        blank: || ElementKind::LcdDigit,
        choices: Vec::new,
        // Glass has no current or power to exceed; what it has is a DC limit,
        // and that is a finding rather than a rating.
        rating: |_| Rating::NONE,
        // Lumex's sheet gives the pinout and the 5 V AC drive, and nothing
        // optical: the threshold, the response and the DC limit are typical
        // TN figures, so the model is only close to it.
        basis: |_| {
            Some(Basis {
                part: crate::solver::device::lcd::PART,
                datasheet: Some(
                    "https://datasheet.octopart.com/LCD-S101D22TR-Lumex-datasheet-14187.pdf",
                ),
                approximate: true,
            })
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Bargraph",
        // Ten anodes, then the ten cathodes in the same order, as the
        // package's two rows of pins run.
        pins: &[
            "a1", "a2", "a3", "a4", "a5", "a6", "a7", "a8", "a9", "a10", "k1", "k2", "k3", "k4",
            "k5", "k6", "k7", "k8", "k9", "k10",
        ],
        unit: "",
        blank: || ElementKind::Bargraph,
        choices: Vec::new,
        // Each bar is an indicator LED, rated as one.
        rating: |_| Rating {
            current: Some(RATED_CURRENT),
            ..Rating::NONE
        },
        // Kingbright's DC-10EWA, red, 2.0 V typical at 20 mA by its
        // distributors' listings; drawn with the red LED's own model (M-11),
        // which drops 1.8 V, so close rather than its sheet's.
        basis: |_| {
            Some(Basis {
                part: "Kingbright DC-10EWA",
                datasheet: None,
                approximate: true,
            })
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Diode",
        // The LED's names, because they are the same two ends of the same
        // junction and a beginner who has met one has met the other.
        pins: &["anode", "cathode"],
        unit: "",
        blank: || ElementKind::Diode,
        choices: Vec::new,
        rating: |_| Rating {
            // A small-signal switching part, roughly a 1N4148: 200 mA is what
            // its datasheet allows continuously, and B-03 is what will say so
            // when a circuit asks for more.
            current: Some(0.2),
            // Vishay's 440 mW, with the leads held at 45 °C.
            power: Some(0.44),
            thermal: Some(SMALL_SIGNAL_DIODE),
            ..Rating::NONE
        },
        basis: |_| {
            Basis::sheet(
                "1N4148 (Vishay)",
                "https://www.vishay.com/docs/81857/1n4148.pdf",
            )
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Schottky",
        pins: &["anode", "cathode"],
        unit: "",
        blank: || ElementKind::Schottky,
        choices: Vec::new,
        rating: |_| Rating {
            // onsemi's 1 A and 40 V; 80 K/W on a board, from 25 °C to its
            // 125 °C, 1.25 W.
            current: Some(1.0),
            voltage: Some(40.0),
            power: Some(1.25),
            thermal: Some(SCHOTTKY_RECTIFIER),
        },
        basis: |_| {
            Basis::sheet(
                "1N5819 (onsemi)",
                "https://www.onsemi.com/download/data-sheet/pdf/1n5817-d.pdf",
            )
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Bridge",
        // The two AC pins, then the DC ones, as the package marks them.
        pins: &["ac1", "ac2", "+", "-"],
        unit: "",
        blank: || ElementKind::Bridge,
        choices: Vec::new,
        rating: |_| Rating {
            // Diodes Inc's DB101–DB107 (DS21211 Rev. 5): 1 A average, 1000 V
            // for the DB107, 40 K/W on a board and 150 °C, so 3.1 W.
            current: Some(1.0),
            voltage: Some(1000.0),
            power: Some(3.1),
            thermal: Some(BRIDGE_RECTIFIER),
        },
        basis: |_| {
            Basis::sheet(
                "DB107 (Diodes Inc)",
                "https://www.diodes.com/assets/Datasheets/ds21211.pdf",
            )
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Zener",
        // A diode's two ends, because it is one: the bar is the cathode, and
        // it is the cathode that goes to the higher voltage when it is used
        // for what it is for.
        pins: &["anode", "cathode"],
        unit: "",
        blank: || ElementKind::Zener {
            part: ZenerPart::V5_1,
        },
        choices: || ZenerPart::ALL.iter().map(ZenerPart::name).collect(),
        rating: |kind| match *kind {
            ElementKind::Zener { part } => Rating {
                // A one-watt part: the current that dissipates its watt at
                // its own voltage.
                current: Some(ZENER_WATTS / part.volts()),
                power: Some(ZENER_WATTS),
                thermal: Some(ONE_WATT_ZENER),
                ..Rating::NONE
            },
            _ => Rating::NONE,
        },
        basis: |kind| match *kind {
            ElementKind::Zener { part } => Some(Basis {
                part: part.part_number(),
                datasheet: Some("https://www.vishay.com/docs/85816/1n4728a.pdf"),
                approximate: false,
            }),
            _ => None,
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Resistor",
        pins: &["a", "b"],
        unit: "Ω",
        // 470 Ω: on the 9 V battery a fresh one comes with, a red LED behind it
        // carries some 15 mA, inside its 20 mA, and it is the value B-02 tells
        // an LED with no resistor to get. 220 Ω, the old default, gave the
        // first circuit anybody builds 33 mA, and B-03 a warning on it.
        blank: || ElementKind::Resistor { ohms: 470.0 },
        choices: Vec::new,
        rating: |_| Rating {
            current: None,
            // A quarter-watt part: what a beginner's assortment box contains,
            // and what B-03 should measure against until C-16's successor lets
            // the value be chosen.
            power: Some(0.25),
            thermal: Some(QUARTER_WATT_RESISTOR),
            ..Rating::NONE
        },
        basis: |_| {
            Basis::sheet(
                "Yageo CFR-25, quarter-watt carbon film",
                "https://www.yageogroup.com/content/Resource%20Library/Datasheet/YAGEO-CFR_DATASHEET.pdf",
            )
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Potentiometer",
        // The wiper between the two ends, because that is the order the track
        // runs in and a pin list is read as a picture of the part.
        pins: &["a", "wiper", "b"],
        unit: "Ω",
        blank: || ElementKind::Potentiometer {
            // 10 kΩ at half turn: the value in every beginner's kit, and the
            // position where a divider does the least surprising thing.
            ohms: 10_000.0,
            wiper: 0.5,
        },
        choices: Vec::new,
        rating: |_| Rating {
            current: None,
            // A quarter-watt track, the same assumption the resistor makes.
            power: Some(0.25),
            ..Rating::NONE
        },
        basis: |_| None,
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Ldr",
        pins: &["a", "b"],
        // Nothing to type: its resistance is the light's, set by its dial.
        unit: "",
        blank: || ElementKind::Ldr {
            // Half way: a lit room, a few hundred lux.
            light: 0.5,
        },
        choices: Vec::new,
        rating: |_| Rating {
            // A GL5528 is a 100 mW part.
            power: Some(0.1),
            ..Rating::NONE
        },
        basis: |_| {
            Basis::sheet(
                "GL5528",
                "https://www.handsontec.com/dataspecs/sensor/GL55-LDR.pdf",
            )
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Phototransistor",
        // Collector, then emitter: the long leg first, as the LED's anode is.
        pins: &["c", "e"],
        unit: "",
        blank: || ElementKind::Phototransistor { light: 0.5 },
        choices: Vec::new,
        // 20 mA and 6 V collector to emitter, from its sheet.
        rating: |_| Rating {
            current: Some(0.02),
            voltage: Some(6.0),
            ..Rating::NONE
        },
        basis: |_| {
            Basis::sheet(
                crate::solver::device::photo::PART,
                crate::solver::device::photo::DATASHEET,
            )
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Thermistor",
        pins: &["a", "b"],
        unit: "",
        blank: || ElementKind::Thermistor { celsius: 25.0 },
        choices: Vec::new,
        rating: |_| Rating {
            power: Some(0.5),
            ..Rating::NONE
        },
        basis: |_| {
            Some(Basis {
                part: "10 kΩ NTC thermistor, B = 3950",
                datasheet: None,
                approximate: false,
            })
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Capacitor",
        pins: &["a", "b"],
        unit: "F",
        blank: || ElementKind::Capacitor {
            // 100 nF: the decoupling capacitor in every beginner's kit.
            farads: 100e-9,
            initial_volts: 0.0,
        },
        choices: Vec::new,
        rating: |_| Rating::NONE,
        basis: |_| None,
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Electrolytic",
        // Polarised: its + is the longer leg, and the stripe on the can marks
        // the − (C-86).
        pins: &["+", "-"],
        unit: "F",
        // 100 µF at 25 V: the one a kit has most of, at a rating no kit
        // battery reaches.
        blank: || ElementKind::Electrolytic {
            farads: 100e-6,
            initial_volts: 0.0,
            rated: Rated::V25,
        },
        choices: || Rated::ALL.iter().map(Rated::name).collect(),
        rating: |kind| match *kind {
            ElementKind::Electrolytic { rated, .. } => Rating {
                voltage: Some(rated.volts()),
                ..Rating::NONE
            },
            _ => Rating::NONE,
        },
        // The sheet's leakage is the limit, and the model takes it; nothing
        // else about the part is past what any capacitor is.
        basis: |kind| match *kind {
            ElementKind::Electrolytic { rated, .. } => Some(Basis {
                part: rated.part(),
                datasheet: Some(crate::solver::device::electrolytic::DATASHEET),
                approximate: false,
            }),
            _ => None,
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Inductor",
        pins: &["a", "b"],
        unit: "H",
        blank: || ElementKind::Inductor {
            henries: 1e-3,
            initial_amps: 0.0,
        },
        choices: Vec::new,
        rating: |_| Rating::NONE,
        basis: |_| None,
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Transformer",
        // The primary's two ends, then the secondary's: each winding's dotted
        // end first, so the two sines are in step.
        pins: &["p1", "p2", "s1", "s2"],
        unit: "",
        // 2 : 1, so a sine is halved: the example's 24 V to 12 V.
        blank: || ElementKind::Transformer { ratio: 2.0 },
        choices: Vec::new,
        // No mains here, so nothing a small one would be rated for is reached.
        rating: |_| Rating::NONE,
        basis: |_| {
            Some(Basis {
                part: "a 2 VA PCB transformer, typical",
                datasheet: None,
                approximate: true,
            })
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Switch",
        pins: &["a", "b"],
        // No editable value: a contact is open or shut, and that is not a
        // number to type. The inspector and the canvas both read `contact()`.
        unit: "",
        blank: || ElementKind::Switch { closed: false },
        // Whatever a real switch is rated for, an ideal one carries it: there
        // is no resistance to dissipate anything. B-03 will want a rating when
        // the parts catalogue of B-10 gives it one.
        choices: Vec::new,
        rating: |_| Rating::NONE,
        basis: |_| None,
        offered: true,
        drawable: true,
    },
    Component {
        kind: "PushButton",
        pins: &["a", "b"],
        unit: "",
        blank: || ElementKind::PushButton {
            closed: false,
            bounces: false,
        },
        choices: || crate::netlist::BUTTON_CHOICES.to_vec(),
        rating: |_| Rating::NONE,
        basis: |_| None,
        offered: true,
        drawable: true,
    },
    Component {
        kind: "TiltSwitch",
        pins: &["a", "b"],
        unit: "",
        // Upright, which is how it comes out of the bag and how it is shut.
        blank: || ElementKind::TiltSwitch { closed: true },
        choices: Vec::new,
        rating: |_| Rating::NONE,
        // An ideal contact, as the switch is: nothing about it is the part's
        // but the name and which way up it is shut.
        basis: |_| {
            Some(Basis {
                part: "SW-520D",
                datasheet: None,
                approximate: true,
            })
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Reed",
        pins: &["a", "b"],
        unit: "",
        // The magnet as far away as the dial goes, so it starts open.
        blank: || ElementKind::Reed { magnet: 0.0 },
        choices: Vec::new,
        rating: |_| Rating::NONE,
        // No one part: a glass reed, and the figure it closes at is an
        // estimate (`magnet`).
        basis: |_| None,
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Relay",
        // The coil's two ends, then the contact's three: the common, the one
        // it is pulled to, the one it rests on.
        pins: &["coil+", "coil-", "com", "no", "nc"],
        unit: "",
        blank: || ElementKind::Relay {
            part: RelayPart::V9,
        },
        choices: || RelayPart::ALL.iter().map(RelayPart::name).collect(),
        rating: |kind| match *kind {
            // An SRD's contact carries 10 A; its coil is rated at its voltage.
            ElementKind::Relay { part } => Rating {
                current: Some(10.0),
                voltage: Some(part.volts() * 1.2),
                ..Rating::NONE
            },
            _ => Rating::NONE,
        },
        basis: |kind| match *kind {
            ElementKind::Relay { part } => Some(Basis {
                part: part.part_number(),
                datasheet: Some("https://www.handsontec.com/dataspecs/relay/SRD-05VDC-SL-C.pdf"),
                approximate: false,
            }),
            _ => None,
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Fuse",
        pins: &["a", "b"],
        unit: "A",
        // Half an amp, fast-blow: the 217 series' 500 mA.
        blank: || ElementKind::Fuse {
            amps: 0.5,
            part: crate::solver::device::fuse::FusePart::Fast,
            blown: false,
        },
        choices: || {
            crate::solver::device::fuse::FusePart::ALL
                .iter()
                .map(crate::solver::device::fuse::FusePart::name)
                .collect()
        },
        // What it is rated for is what it opens past; a fuse over its
        // rating is doing its job, not being misused.
        rating: |_| Rating::NONE,
        basis: |kind| match *kind {
            ElementKind::Fuse { part, .. } => Some(match part {
                crate::solver::device::fuse::FusePart::Fast => Basis {
                    part: "Littelfuse 217 series",
                    datasheet: Some("https://www.farnell.com/datasheets/3097901.pdf"),
                    approximate: false,
                },
                crate::solver::device::fuse::FusePart::Polyfuse => Basis {
                    part: "Bourns MF-R series",
                    datasheet: Some("https://www.bourns.com/docs/Product-Datasheets/mfr.pdf"),
                    approximate: false,
                },
            }),
            _ => None,
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Buzzer",
        // Polarised, as an active buzzer is: its + leg is the longer one.
        pins: &["+", "-"],
        unit: "",
        blank: || ElementKind::Buzzer,
        choices: Vec::new,
        rating: |_| Rating {
            // 30 mA, 8 V: a 5 V buzzer on a 9 V battery is past both, and B-03
            // is what will say so.
            current: Some(0.03),
            voltage: Some(8.0),
            ..Rating::NONE
        },
        basis: |_| {
            Some(Basis {
                part: "5 V active buzzer",
                datasheet: None,
                approximate: true,
            })
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "PassiveBuzzer",
        // Marked + and −, as a kit's is, though it sounds either way round:
        // it is a coil, and the note is in the swing, not the sign (C-85).
        pins: &["+", "-"],
        unit: "",
        blank: || ElementKind::PassiveBuzzer,
        choices: Vec::new,
        rating: |_| Rating::NONE,
        // Sold without a part number anybody would recognise; the coil's
        // 16 Ω is what kits give, and its inductance and resonance are not
        // modelled.
        basis: |_| {
            Some(Basis {
                part: crate::solver::device::pitch::PART,
                datasheet: None,
                approximate: true,
            })
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Motor",
        pins: &["a", "b"],
        unit: "",
        blank: || ElementKind::Motor,
        choices: Vec::new,
        rating: |_| Rating {
            // A 130 motor stalls at a couple of amps, and is rated for 6 V.
            current: Some(2.0),
            voltage: Some(6.0),
            ..Rating::NONE
        },
        basis: |_| {
            Some(Basis {
                part: "130-size hobby motor",
                datasheet: None,
                approximate: true,
            })
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Servo",
        // The three wires, in the order a servo's plug has them: signal,
        // then the supply's +, then its −.
        pins: &["SIG", "+5V", "GND"],
        unit: "",
        blank: || ElementKind::Servo,
        choices: Vec::new,
        // 4.8 V to 6 V, its sheet's range.
        rating: |_| Rating {
            voltage: Some(6.0),
            ..Rating::NONE
        },
        // Its speed and travel are its sheet's; its current is not on it.
        basis: |_| {
            Some(Basis {
                part: crate::solver::device::servo::PART,
                datasheet: None,
                approximate: true,
            })
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Bjt",
        pins: &["b", "c", "e"],
        unit: "",
        blank: || ElementKind::Bjt {
            polarity: Polarity::Npn,
        },
        choices: || Polarity::ALL.iter().map(Polarity::name).collect(),
        rating: |kind| match *kind {
            // 5 A and 60 V; 2 W in free air, onsemi's.
            ElementKind::Bjt {
                polarity: Polarity::Darlington,
            } => Rating {
                current: Some(5.0),
                voltage: Some(60.0),
                power: Some(2.0),
                thermal: Some(TO220_TIP120),
            },
            _ => Rating::NONE,
        },
        basis: |kind| match *kind {
            ElementKind::Bjt { polarity } => Some(Basis {
                part: polarity.part_number(),
                datasheet: Some(polarity.datasheet()),
                approximate: polarity != Polarity::Darlington,
            }),
            _ => None,
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Mosfet",
        // Gate, drain, source: the order a transistor's base, collector and
        // emitter are in, because they are the same three jobs.
        pins: &["g", "d", "s"],
        unit: "",
        blank: || ElementKind::Mosfet {
            channel: Channel::N,
        },
        choices: || Channel::ALL.iter().map(Channel::name).collect(),
        rating: |kind| match *kind {
            // A 2N7000 carries 200 mA and stands 60 V; a BS250, 180 mA and 45 V.
            ElementKind::Mosfet { channel } => match channel {
                Channel::N => Rating {
                    current: Some(0.2),
                    voltage: Some(60.0),
                    power: Some(0.4),
                    thermal: Some(TO92_2N7000),
                },
                Channel::P => Rating {
                    current: Some(0.18),
                    voltage: Some(45.0),
                    power: Some(0.83),
                    thermal: Some(TO92_BS250),
                },
                // 47 A on a heatsink; in free air, what 62 K/W lets it shed
                // from 25 °C to 175 °C, 2.4 W, is the limit that bites.
                Channel::Power => Rating {
                    current: Some(47.0),
                    voltage: Some(55.0),
                    power: Some(2.4),
                    thermal: Some(TO220_IRLZ44N),
                },
            },
            _ => Rating::NONE,
        },
        basis: |kind| match *kind {
            ElementKind::Mosfet { channel } => Some(Basis {
                part: channel.part_number(),
                datasheet: Some(match channel {
                    Channel::N => "https://www.onsemi.com/download/data-sheet/pdf/nds7002a-d.pdf",
                    Channel::P => {
                        "https://web.ece.ucsb.edu/~ilan/Classes/ECE2A_F2010/Labs/Datasheets/BS250.pdf"
                    }
                    Channel::Power => {
                        "https://www.infineon.com/dgdl/irlz44npbf.pdf?fileId=5546d462533600a40153567217c32725"
                    }
                }),
                approximate: false,
            }),
            _ => None,
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Regulator",
        // In, ground, out: the order the three legs of a TO-220 78xx run in,
        // left to right with the writing towards you.
        pins: &["in", "gnd", "out"],
        // Nothing to type. Its output voltage is which part it is, and a part
        // is chosen rather than entered — an "adjustable 7805" set to 6.1 V is
        // a part nobody can buy (ADR 0010, term 3).
        unit: "",
        blank: || ElementKind::Regulator {
            part: RegulatorPart::L7805,
        },
        choices: || RegulatorPart::ALL.iter().map(RegulatorPart::name).collect(),
        rating: |kind| match *kind {
            ElementKind::Regulator { part } => {
                let sheet = part.datasheet();
                Rating {
                    current: Some(sheet.current_limit),
                    power: None,
                    voltage: Some(sheet.max_input_volts),
                    thermal: Some(Thermal {
                        junction_to_ambient: sheet.junction_to_ambient,
                        max_junction_celsius: sheet.max_junction_celsius,
                        time_constant: sheet.thermal_seconds,
                    }),
                }
            }
            _ => Rating::NONE,
        },
        basis: |kind| match *kind {
            ElementKind::Regulator { part } => Some(Basis {
                part: part.name(),
                datasheet: Some(match part {
                    RegulatorPart::L7805 | RegulatorPart::L7812 => {
                        "https://www.st.com/resource/en/datasheet/l78.pdf"
                    }
                    RegulatorPart::Lm1117 => "https://www.ti.com/lit/ds/symlink/lm1117.pdf",
                    RegulatorPart::Lm317 => "https://www.ti.com/lit/ds/symlink/lm317.pdf",
                }),
                approximate: false,
            }),
            _ => None,
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Adjustable",
        // In, adjust, out: the regulator's terminals in the regulator's order,
        // so its model is wired the same way. On the part itself the legs run
        // ADJ, OUT, IN, which the breadboard (B-12) will have to map.
        pins: &["in", "adj", "out"],
        unit: "",
        blank: || ElementKind::Adjustable,
        choices: Vec::new,
        rating: |_| {
            let sheet = RegulatorPart::Lm317.datasheet();
            Rating {
                current: Some(sheet.current_limit),
                power: None,
                voltage: Some(sheet.max_input_volts),
                thermal: Some(Thermal {
                    junction_to_ambient: sheet.junction_to_ambient,
                    max_junction_celsius: sheet.max_junction_celsius,
                    time_constant: sheet.thermal_seconds,
                }),
            }
        },
        basis: |_| Basis::sheet("LM317", "https://www.ti.com/lit/ds/symlink/lm317.pdf"),
        offered: true,
        drawable: true,
    },
    Component {
        kind: "OpAmp",
        // Its two inputs, its output, and the two supply pins it draws that
        // output from. The supply is drawn and named like any other pin
        // because it is wired like any other: an op-amp with no V+ is the
        // circuit that works on paper and not on the breadboard (ADR 0010).
        pins: &["in+", "in-", "out", "V+", "V-"],
        unit: "",
        blank: || ElementKind::OpAmp,
        choices: Vec::new,
        rating: |_| Rating {
            current: Some(OpAmp.model().source_limit),
            voltage: Some(crate::solver::device::opamp::MAX_SUPPLY_VOLTS),
            ..Rating::NONE
        },
        basis: |_| Basis::sheet("LM358", "https://www.ti.com/lit/ds/symlink/lm358.pdf"),
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Comparator",
        // The op-amp's pins, in the op-amp's order: it is drawn as the same
        // triangle and wired the same way, and a beginner who has met one has
        // met the other.
        pins: &["in+", "in-", "out", "V+", "V-"],
        unit: "",
        blank: || ElementKind::Comparator,
        choices: Vec::new,
        rating: |_| Rating {
            current: Some(comparator::SINK_AMPS),
            voltage: Some(comparator::MAX_SUPPLY_VOLTS),
            ..Rating::NONE
        },
        basis: |_| Basis::sheet("LM393", "https://www.ti.com/lit/ds/symlink/lm393.pdf"),
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Tmp36",
        // As printed beside its legs, flat face towards you: +Vs, Vout, GND.
        pins: &["+Vs", "Vout", "GND"],
        unit: "",
        blank: || ElementKind::Tmp36 { celsius: 25.0 },
        choices: Vec::new,
        // Its supply's absolute maximum, 7 V: a 9 V battery is past it, and
        // the review says so long before B-03 will.
        rating: |_| Rating {
            voltage: Some(7.0),
            ..Rating::NONE
        },
        basis: |_| {
            Some(Basis {
                part: crate::solver::device::tmp36::PART,
                datasheet: Some(crate::solver::device::tmp36::DATASHEET),
                approximate: false,
            })
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Hall",
        pins: &crate::solver::device::magnet::PINS,
        unit: "",
        blank: || ElementKind::Hall { magnet: 0.0 },
        choices: Vec::new,
        rating: |_| Rating {
            current: Some(0.025),
            voltage: Some(crate::solver::device::magnet::MOST_SUPPLY),
            ..Rating::NONE
        },
        basis: |_| {
            Basis::sheet(
                "A3144 (Allegro)",
                "https://www.allegromicro.com/-/media/files/datasheets/a3141-2-3-4-datasheet.ashx",
            )
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Pir",
        pins: &crate::solver::device::pir::PINS,
        // Its delay, which is what its trimmer sets and what a beginner turns.
        unit: "s",
        blank: || ElementKind::Pir {
            moving: false,
            hold: crate::solver::device::pir::SHORTEST_HOLD,
            retriggers: true,
        },
        choices: || crate::netlist::PIR_CHOICES.to_vec(),
        rating: |_| Rating {
            voltage: Some(crate::solver::device::pir::MOST_SUPPLY),
            ..Rating::NONE
        },
        basis: |_| {
            Basis::sheet(
                "HC-SR501 (MPJA)",
                "https://courses.ece.cornell.edu/ece5990/ECE5725_Spring2020_Projects/May_15_Demo/Smart%20Lock/W_pp445_yl656/datasheet/31227sc.pdf",
            )
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Timer",
        // The six of its eight a circuit wires (ADR 0012): supply, the two
        // thresholds, the discharge switch and the output. RESET is tied to
        // VCC and CONTROL left free, as nearly every beginner circuit has them.
        pins: &["VCC", "GND", "TRIG", "THR", "DIS", "OUT"],
        unit: "",
        blank: || ElementKind::Timer,
        choices: Vec::new,
        rating: |_| Rating {
            // An NE555's output sources or sinks 200 mA, off up to 16 V.
            current: Some(0.2),
            voltage: Some(16.0),
            ..Rating::NONE
        },
        basis: |_| Basis::sheet("NE555", "https://www.ti.com/lit/ds/symlink/ne555.pdf"),
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Decoder",
        // The four bits, the three controls, the seven segments, the supply.
        pins: &crate::solver::device::decoder::PINS,
        unit: "",
        blank: || ElementKind::Decoder,
        choices: Vec::new,
        rating: |_| Rating {
            // A CD4511B runs from 3 to 18 V and sources 25 mA per output.
            current: Some(0.025),
            voltage: Some(18.0),
            ..Rating::NONE
        },
        basis: |_| Basis::sheet("CD4511B", "https://www.ti.com/lit/ds/symlink/cd4511b.pdf"),
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Counter",
        // Clock, inhibit, reset, display enable, carry out; the seven
        // segments; the supply.
        pins: &crate::solver::device::counter::PINS,
        unit: "",
        blank: || ElementKind::Counter,
        choices: Vec::new,
        rating: |_| Rating {
            // A CD4026B runs from 3 to 18 V.
            voltage: Some(18.0),
            ..Rating::NONE
        },
        basis: |_| Basis::sheet("CD4026B", "https://www.ti.com/lit/ds/symlink/cd4026b.pdf"),
        offered: true,
        drawable: true,
    },
    Component {
        kind: "MotorDriver",
        // The sheet's order round the package, its four GND pins one.
        pins: &crate::solver::device::driver::PINS,
        unit: "",
        blank: || ElementKind::MotorDriver,
        choices: Vec::new,
        rating: |_| Rating {
            // An L293D drives 600 mA per output, from a motor supply of up
            // to 36 V.
            current: Some(0.6),
            voltage: Some(36.0),
            ..Rating::NONE
        },
        basis: |_| Basis::sheet("L293D", "https://www.ti.com/lit/ds/symlink/l293.pdf"),
        offered: true,
        drawable: true,
    },
    Component {
        kind: "ShiftRegister",
        // The sheet's order round the package.
        pins: &crate::solver::device::shift::PINS,
        unit: "",
        blank: || ElementKind::ShiftRegister,
        choices: Vec::new,
        rating: |_| Rating {
            // An SN74HC595 runs from 2 to 6 V, and an output takes 35 mA at
            // most.
            current: Some(0.035),
            voltage: Some(6.0),
            ..Rating::NONE
        },
        basis: |_| {
            Basis::sheet(
                "SN74HC595",
                "https://www.ti.com/lit/ds/symlink/sn74hc595.pdf",
            )
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Optocoupler",
        // The LED's anode and cathode, then the transistor's collector and
        // emitter: the two sides of the gap.
        pins: &crate::solver::device::opto::PINS,
        unit: "",
        blank: || ElementKind::Optocoupler,
        choices: Vec::new,
        rating: |_| Rating {
            // 50 mA either side, and 30 V across its transistor.
            current: Some(0.05),
            voltage: Some(30.0),
            ..Rating::NONE
        },
        basis: |_| {
            Basis::sheet(
                crate::solver::device::opto::PART,
                crate::solver::device::opto::DATASHEET,
            )
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Lm3914",
        // The sheet's order round the package.
        pins: &crate::solver::device::lm3914::PINS,
        unit: "",
        blank: || ElementKind::Lm3914,
        choices: Vec::new,
        rating: |_| Rating {
            // 25 V and 1.365 W in its PDIP.
            voltage: Some(crate::solver::device::lm3914::MOST_SUPPLY),
            power: Some(1.365),
            ..Rating::NONE
        },
        basis: |_| {
            Basis::sheet(
                "LM3914 (TI)",
                "https://www.ti.com/lit/ds/symlink/lm3914.pdf",
            )
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Gate",
        // The sheet's order round the package, which the four quads share.
        pins: &crate::solver::device::gate::QUAD_PINS,
        unit: "",
        blank: || ElementKind::Gate {
            logic: crate::solver::device::gate::Logic::Nand,
        },
        choices: || {
            crate::solver::device::gate::Logic::ALL
                .iter()
                .map(crate::solver::device::gate::Logic::name)
                .collect()
        },
        rating: |_| Rating {
            // The HC family runs from 2 to 6 V, and an output takes 25 mA at
            // most.
            current: Some(0.025),
            voltage: Some(crate::solver::device::gate::MOST_SUPPLY),
            ..Rating::NONE
        },
        basis: |kind| match *kind {
            ElementKind::Gate { logic } => Basis::sheet(logic.part_number(), logic.datasheet()),
            _ => None,
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Inverter",
        pins: &crate::solver::device::gate::HEX_PINS,
        unit: "",
        blank: || ElementKind::Inverter,
        choices: Vec::new,
        rating: |_| Rating {
            current: Some(0.025),
            voltage: Some(crate::solver::device::gate::MOST_SUPPLY),
            ..Rating::NONE
        },
        basis: |_| {
            Basis::sheet(
                crate::solver::device::gate::INVERTER_PART,
                crate::solver::device::gate::INVERTER_DATASHEET,
            )
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Schmitt",
        // The 74HC04's pins, in the same places.
        pins: &crate::solver::device::gate::HEX_PINS,
        unit: "",
        blank: || ElementKind::Schmitt,
        choices: Vec::new,
        rating: |_| Rating {
            current: Some(0.025),
            voltage: Some(crate::solver::device::gate::MOST_SUPPLY),
            ..Rating::NONE
        },
        basis: |_| {
            Basis::sheet(
                crate::solver::device::gate::SCHMITT_PART,
                crate::solver::device::gate::SCHMITT_DATASHEET,
            )
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Decade",
        // The sheet's order round the package.
        pins: &crate::solver::device::decade::PINS,
        unit: "",
        blank: || ElementKind::Decade,
        choices: Vec::new,
        rating: |_| Rating {
            voltage: Some(crate::solver::device::decade::MOST_SUPPLY),
            ..Rating::NONE
        },
        basis: |_| Basis::sheet("CD4017B", "https://www.ti.com/lit/ds/symlink/cd4017b.pdf"),
        offered: true,
        drawable: true,
    },
    Component {
        kind: "FlipFlop",
        // The sheet's order round the package.
        pins: &crate::solver::device::flipflop::PINS,
        unit: "",
        blank: || ElementKind::FlipFlop,
        choices: Vec::new,
        rating: |_| Rating {
            voltage: Some(crate::solver::device::flipflop::MOST_SUPPLY),
            ..Rating::NONE
        },
        basis: |_| Basis::sheet("CD4013B", "https://www.ti.com/lit/ds/symlink/cd4013b.pdf"),
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Uno",
        pins: &crate::solver::device::uno::PINS,
        unit: "",
        blank: || ElementKind::Uno,
        choices: Vec::new,
        rating: |_| Rating {
            voltage: Some(5.5),
            ..Rating::NONE
        },
        basis: |_| {
            Basis::sheet(
                "Arduino Uno R3 (ATmega328P)",
                "https://docs.arduino.cc/hardware/uno-rev3/",
            )
        },
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Voltmeter",
        // `+` and `-`, because a voltage has a direction and a meter put on
        // backwards should read a negative number rather than quietly agree.
        pins: &["+", "-"],
        unit: "V",
        blank: || ElementKind::Voltmeter,
        // An ideal meter carries nothing and dissipates nothing, so there is
        // nothing for B-03 to measure against.
        choices: Vec::new,
        rating: |_| Rating::NONE,
        basis: |_| None,
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Ammeter",
        pins: &["+", "-"],
        unit: "A",
        blank: || ElementKind::Ammeter,
        choices: Vec::new,
        rating: |_| Rating::NONE,
        basis: |_| None,
        offered: true,
        drawable: true,
    },
    Component {
        kind: "Ground",
        pins: &["gnd"],
        unit: "",
        blank: || ElementKind::Ground,
        choices: Vec::new,
        rating: |_| Rating::NONE,
        basis: |_| None,
        offered: false,
        drawable: true,
    },
];

/// Every kind's name, as a TypeScript type the views' `kind` fields carry.
///
/// Written out rather than generated, because a custom section has to be a
/// literal; `the_typescript_kinds_are_the_registry_s` holds it to `REGISTRY`,
/// so a kind added there and not here fails a test rather than a type check
/// nobody runs. With it, the app compares kinds against names the compiler
/// knows instead of against any string at all.
// Read through the section below, which the lint does not count as a use.
#[allow(dead_code, reason = "read by the custom section and by a test")]
const COMPONENT_KIND: &str = r#"
/** Every kind of component the core knows, by the name it crosses the boundary under (ADR 0008). */
export type ComponentKind =
    | "DcSource"
    | "SignalGenerator"
    | "Cell"
    | "SolarCell"
    | "Led"
    | "RgbLed"
    | "SevenSegment"
    | "LcdDigit"
    | "Bargraph"
    | "Diode"
    | "Schottky"
    | "Bridge"
    | "Zener"
    | "Resistor"
    | "Potentiometer"
    | "Ldr"
    | "Phototransistor"
    | "Thermistor"
    | "Capacitor"
    | "Electrolytic"
    | "Inductor"
    | "Transformer"
    | "Switch"
    | "PushButton"
    | "TiltSwitch"
    | "Reed"
    | "Relay"
    | "Fuse"
    | "Buzzer"
    | "PassiveBuzzer"
    | "Motor"
    | "Servo"
    | "Bjt"
    | "Mosfet"
    | "Regulator"
    | "Adjustable"
    | "OpAmp"
    | "Comparator"
    | "Tmp36"
    | "Hall"
    | "Pir"
    | "Timer"
    | "Decoder"
    | "Counter"
    | "MotorDriver"
    | "ShiftRegister"
    | "Optocoupler"
    | "Lm3914"
    | "Gate"
    | "Inverter"
    | "Schmitt"
    | "Decade"
    | "FlipFlop"
    | "Uno"
    | "Voltmeter"
    | "Ammeter"
    | "Ground";
"#;

#[wasm_bindgen(typescript_custom_section)]
const COMPONENT_KIND_SECTION: &str = COMPONENT_KIND;

/// What the core knows about a kind, by name.
pub fn definition(kind: &str) -> Option<&'static Component> {
    REGISTRY.iter().find(|component| component.kind == kind)
}

/// What this element is built to withstand, from its kind's entry.
pub fn rating_of(kind: &ElementKind) -> Rating {
    definition(kind.name()).map_or(Rating::NONE, |component| (component.rating)(kind))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_typescript_kinds_are_the_registry_s() {
        let declared: Vec<&str> = COMPONENT_KIND.split('"').skip(1).step_by(2).collect();
        let registered: Vec<&str> = REGISTRY.iter().map(|component| component.kind).collect();
        assert_eq!(declared, registered);
    }

    #[test]
    fn every_kind_appears_once() {
        for component in REGISTRY {
            let found = REGISTRY
                .iter()
                .filter(|other| other.kind == component.kind)
                .count();
            assert_eq!(
                found, 1,
                "{} is in the registry {found} times",
                component.kind
            );
        }
    }

    #[test]
    fn a_blank_component_is_of_the_kind_it_is_filed_under() {
        // The one way the table could lie to itself: an entry whose constructor
        // builds something else.
        for component in REGISTRY {
            assert_eq!(
                (component.blank)().name(),
                component.kind,
                "{} builds the wrong kind",
                component.kind
            );
        }
    }

    #[test]
    fn a_blank_component_has_the_pins_the_entry_promises() {
        for component in REGISTRY {
            assert_eq!(
                (component.blank)().pin_names(),
                component.pins,
                "{}",
                component.kind
            );
        }
    }

    #[test]
    fn anything_offered_can_be_drawn() {
        // Offering a component the app cannot draw is a crash waiting for
        // somebody to click it.
        for component in REGISTRY.iter().filter(|entry| entry.offered) {
            assert!(
                component.drawable,
                "{} is offered but not drawable",
                component.kind
            );
        }
    }

    #[test]
    fn v1_offers_what_section_5_lists() {
        // The same guard C-34 put on the palette, now in the place the palette
        // gets its list from.
        //
        // The order is the list's own claim: the palette's categories, one
        // after another — power, lights and diodes, resistors and capacitors,
        // switches, transistors and chips, meters — and within each, the part
        // somebody meets first. The app groups by category and holds its flat
        // order to this one (C-59, ADR 0008).
        let offered: Vec<&str> = REGISTRY
            .iter()
            .filter(|entry| entry.offered)
            .map(|entry| entry.kind)
            .collect();

        assert_eq!(
            offered,
            [
                "DcSource",
                "SignalGenerator",
                "Cell",
                "SolarCell",
                "Led",
                "RgbLed",
                "SevenSegment",
                "LcdDigit",
                "Bargraph",
                "Diode",
                "Schottky",
                "Bridge",
                "Zener",
                "Resistor",
                "Potentiometer",
                "Ldr",
                "Phototransistor",
                "Thermistor",
                "Capacitor",
                "Electrolytic",
                "Inductor",
                "Transformer",
                "Switch",
                "PushButton",
                "TiltSwitch",
                "Reed",
                "Relay",
                "Fuse",
                "Buzzer",
                "PassiveBuzzer",
                "Motor",
                "Servo",
                "Bjt",
                "Mosfet",
                "Regulator",
                "Adjustable",
                "OpAmp",
                "Comparator",
                "Tmp36",
                "Hall",
                "Pir",
                "Timer",
                "Decoder",
                "Counter",
                "MotorDriver",
                "ShiftRegister",
                "Optocoupler",
                "Lm3914",
                "Gate",
                "Inverter",
                "Schmitt",
                "Decade",
                "FlipFlop",
                "Uno",
                "Voltmeter",
                "Ammeter"
            ]
        );
    }

    #[test]
    fn a_choice_a_kind_offers_is_a_choice_it_accepts() {
        // The list and the setter are two halves of one closed set (C-46). A
        // name offered and refused would be a chooser with a dead option in it.
        for component in REGISTRY {
            let blank = (component.blank)();
            for choice in (component.choices)() {
                let set = blank
                    .with_choice(choice)
                    .unwrap_or_else(|| panic!("{} offers {choice} and refuses it", component.kind));
                assert_eq!(set.choice(), Some(choice), "{}", component.kind);
                // And it is the same component afterwards, not another one.
                assert_eq!(set.name(), component.kind);
            }
        }
    }

    #[test]
    fn a_kind_with_something_to_choose_says_what_it_is_set_to() {
        // The other direction: anything that reports a current choice has to
        // offer it, or the chooser would open with nothing selected.
        for component in REGISTRY {
            let blank = (component.blank)();
            match blank.choice() {
                Some(current) => assert!(
                    (component.choices)().contains(&current),
                    "{} starts as {current}, which it does not offer",
                    component.kind
                ),
                None => assert!(
                    (component.choices)().is_empty(),
                    "{} offers choices and reports none of them",
                    component.kind
                ),
            }
        }
    }

    #[test]
    fn a_choice_it_has_never_heard_of_is_refused() {
        let led = (definition("Led").unwrap().blank)();

        assert!(led.with_choice("Purple").is_none());
        assert!(led.with_choice("red").is_none(), "names are exact");
        // And a kind with nothing to choose refuses everything.
        assert!(
            (definition("Resistor").unwrap().blank)()
                .with_choice("Red")
                .is_none()
        );
    }

    #[test]
    fn a_regulator_is_rated_as_the_part_it_is_set_to() {
        // One kind, three parts, three sets of limits: the rating follows the
        // choice rather than the kind (C-54).
        let blank = (definition("Regulator").unwrap().blank)();
        let seventy_eight = rating_of(&blank);
        let ldo = rating_of(&blank.with_choice("LM1117-3.3").unwrap());

        assert_eq!(seventy_eight.current, Some(1.5));
        assert_eq!(ldo.current, Some(0.8));
        assert!(ldo.voltage < seventy_eight.voltage, "an LDO takes less in");
        assert!(seventy_eight.thermal.is_some() && ldo.thermal.is_some());
    }

    #[test]
    fn a_quarter_watt_resistor_at_its_rating_is_as_hot_as_its_derating_allows() {
        // The two figures are one curve read two ways: a quarter of a watt
        // at 70 °C around it is 155 °C in it.
        let rating = rating_of(&ElementKind::Resistor { ohms: 220.0 });
        let thermal = rating.thermal.unwrap();
        let hottest = 70.0 + rating.power.unwrap() * thermal.junction_to_ambient;
        assert!((hottest - thermal.max_junction_celsius).abs() < 1e-9);
    }

    #[test]
    fn every_part_that_warms_says_whose_figures_it_uses() {
        // A temperature is only as good as the θja behind it, so a part with
        // one names the sheet it came from (C-74).
        for component in REGISTRY {
            for choice in std::iter::once(None).chain((component.choices)().into_iter().map(Some)) {
                let kind = match choice {
                    Some(choice) => (component.blank)().with_choice(choice).unwrap(),
                    None => (component.blank)(),
                };
                if rating_of(&kind).thermal.is_some() {
                    let basis = (component.basis)(&kind);
                    assert!(
                        basis.is_some_and(|basis| basis.datasheet.is_some()),
                        "{} {choice:?} warms with no datasheet named",
                        component.kind
                    );
                }
            }
        }
    }

    #[test]
    fn a_datasheet_is_a_link_that_can_be_followed() {
        for component in REGISTRY {
            let kind = (component.blank)();
            if let Some(Basis {
                datasheet: Some(link),
                ..
            }) = (component.basis)(&kind)
            {
                assert!(link.starts_with("https://"), "{}: {link}", component.kind);
            }
        }
    }

    #[test]
    fn the_part_named_is_the_part_chosen() {
        let zener = (definition("Zener").unwrap().blank)();
        let named =
            |kind: &ElementKind| (definition(kind.name()).unwrap().basis)(kind).unwrap().part;
        assert_eq!(named(&zener), "1N4733A");
        assert_eq!(
            named(&zener.with_choice(ZenerPart::V9_1.name()).unwrap()),
            "1N4739A"
        );
        // And an ideal part names none.
        assert!(
            (definition("DcSource").unwrap().basis)(&(definition("DcSource").unwrap().blank)())
                .is_none()
        );
    }

    #[test]
    fn an_unknown_kind_is_not_invented() {
        assert!(definition("Thyristor").is_none());
    }
}
