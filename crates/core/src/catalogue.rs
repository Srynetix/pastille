// SPDX-License-Identifier: MIT OR Apache-2.0

//! Real parts, by the number printed on them (B-10, design doc §6.2).
//!
//! The model is "a red LED", "an NPN"; what somebody buys is a WP7113ID or a
//! BC547B. This is the short list of each that a beginner's kit holds — enough
//! that every LED colour and every transistor the app models has at least one
//! part to buy — and what each one's datasheet says, so a bill of materials
//! (B-11) can name a part and a reader can check the claim.
//!
//! **Hand-curated, every figure read off the maker's own sheet** (2026-10-02),
//! the sheet linked beside it; nothing filled from memory or averaged across
//! makers. Where a sheet does not give a figure it is `None` here, rather than
//! one worked out: the 2N7000's and the IRLZ44N's power is given only with
//! the case held at 25 °C, which is not how either sits on a breadboard.
//! Whether this grows a contribution format is L-05's question.
//!
//! What is not here: a part the app has no model for. A TIP125 is a TIP120's
//! PNP mirror, and the transistor's Darlington is NPN only.

use crate::netlist::{ElementKind, LedColour};
use crate::solver::device::{Channel, Polarity};

/// A 5 mm indicator LED.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LedPart {
    pub number: &'static str,
    pub maker: &'static str,
    pub colour: LedColour,
    /// Forward voltage, typical and maximum, at [`LedPart::at_amps`].
    pub forward_volts: f64,
    pub forward_volts_max: f64,
    pub at_amps: f64,
    /// The most DC forward current it is rated for (absolute maximum).
    pub max_amps: f64,
    pub package: &'static str,
    pub datasheet: &'static str,
}

/// What a transistor is, as the app models it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Model {
    Bjt(Polarity),
    Mosfet(Channel),
}

/// A DC current gain and the collector current the sheet gives it at.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gain {
    pub min: f64,
    pub max: Option<f64>,
    pub at_amps: f64,
}

/// A MOSFET's resistance switched on, and the gate voltage it is given at.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OnResistance {
    pub ohms: f64,
    pub at_gate_volts: f64,
}

/// A transistor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransistorPart {
    pub number: &'static str,
    pub maker: &'static str,
    pub model: Model,
    /// The most continuous collector or drain current, in amps.
    pub max_amps: f64,
    /// The most collector-emitter or drain-source voltage, in volts.
    pub max_volts: f64,
    /// A bipolar transistor's gain.
    pub gain: Option<Gain>,
    /// A MOSFET's gate threshold, least and most, in volts.
    pub threshold: Option<(f64, f64)>,
    /// A MOSFET's resistance switched on, at the lowest gate voltage given.
    pub on_resistance: Option<OnResistance>,
    /// The most it may dissipate in free air at 25 °C, when the sheet says.
    pub power: Option<f64>,
    /// Its package, when the sheet's text names it.
    pub package: Option<&'static str>,
    pub datasheet: &'static str,
}

/// The LEDs, one a colour: Kingbright's WP7113 series, the 5 mm lamp in kits.
pub const LEDS: &[LedPart] = &[
    LedPart {
        number: "WP7113ID",
        maker: "Kingbright",
        colour: LedColour::Red,
        forward_volts: 1.9,
        forward_volts_max: 2.3,
        at_amps: 10e-3,
        max_amps: 30e-3,
        package: "T-1¾ (5 mm)",
        datasheet: "https://www.kingbrightusa.com/images/catalog/SPEC/WP7113ID.pdf",
    },
    LedPart {
        number: "WP7113GD",
        maker: "Kingbright",
        colour: LedColour::Green,
        forward_volts: 2.0,
        forward_volts_max: 2.4,
        at_amps: 10e-3,
        max_amps: 25e-3,
        package: "T-1¾ (5 mm)",
        datasheet: "https://www.kingbrightusa.com/images/catalog/SPEC/WP7113GD.pdf",
    },
    LedPart {
        number: "WP7113QBC/D",
        maker: "Kingbright",
        colour: LedColour::Blue,
        forward_volts: 3.3,
        forward_volts_max: 4.0,
        at_amps: 20e-3,
        max_amps: 30e-3,
        package: "T-1¾ (5 mm)",
        datasheet: "https://www.kingbrightusa.com/images/catalog/SPEC/WP7113QBC-D.pdf",
    },
    LedPart {
        number: "WP7113QWC/D",
        maker: "Kingbright",
        colour: LedColour::White,
        forward_volts: 3.3,
        forward_volts_max: 4.0,
        at_amps: 20e-3,
        max_amps: 30e-3,
        package: "T-1¾ (5 mm)",
        datasheet: "https://www.kingbrightusa.com/images/catalog/SPEC/WP7113QWC-D.pdf",
    },
];

/// The transistors, the part each model is based on first.
pub const TRANSISTORS: &[TransistorPart] = &[
    TransistorPart {
        number: "2N3904",
        maker: "onsemi",
        model: Model::Bjt(Polarity::Npn),
        max_amps: 0.2,
        max_volts: 40.0,
        gain: Some(Gain {
            min: 100.0,
            max: Some(300.0),
            at_amps: 10e-3,
        }),
        threshold: None,
        on_resistance: None,
        power: Some(0.625),
        package: Some("TO-92"),
        datasheet: "https://www.onsemi.com/download/data-sheet/pdf/2n3903-d.pdf",
    },
    TransistorPart {
        number: "BC547B",
        maker: "onsemi",
        model: Model::Bjt(Polarity::Npn),
        max_amps: 0.1,
        max_volts: 45.0,
        // The sheet has no line at 10 mA; 2 mA is the nearest.
        gain: Some(Gain {
            min: 200.0,
            max: Some(450.0),
            at_amps: 2e-3,
        }),
        threshold: None,
        on_resistance: None,
        power: Some(0.625),
        package: Some("TO-92"),
        datasheet: "https://www.onsemi.com/download/data-sheet/pdf/bc546-d.pdf",
    },
    TransistorPart {
        number: "PN2222A",
        maker: "onsemi",
        model: Model::Bjt(Polarity::Npn),
        max_amps: 1.0,
        max_volts: 40.0,
        gain: Some(Gain {
            min: 100.0,
            max: Some(300.0),
            at_amps: 0.15,
        }),
        threshold: None,
        on_resistance: None,
        power: Some(0.625),
        package: Some("TO-92"),
        datasheet: "https://www.onsemi.com/download/data-sheet/pdf/pn2222a-d.pdf",
    },
    TransistorPart {
        number: "2N3906",
        maker: "onsemi",
        model: Model::Bjt(Polarity::Pnp),
        max_amps: 0.2,
        max_volts: 40.0,
        gain: Some(Gain {
            min: 100.0,
            max: Some(300.0),
            at_amps: 10e-3,
        }),
        threshold: None,
        on_resistance: None,
        power: Some(0.625),
        package: Some("TO-92"),
        datasheet: "https://www.onsemi.com/download/data-sheet/pdf/2n3906-d.pdf",
    },
    TransistorPart {
        number: "BC557B",
        maker: "onsemi",
        model: Model::Bjt(Polarity::Pnp),
        max_amps: 0.1,
        max_volts: 45.0,
        gain: Some(Gain {
            min: 180.0,
            max: Some(460.0),
            at_amps: 2e-3,
        }),
        threshold: None,
        on_resistance: None,
        power: Some(0.625),
        package: Some("TO-92"),
        datasheet: "https://www.onsemi.com/download/data-sheet/pdf/bc556b-d.pdf",
    },
    TransistorPart {
        number: "TIP120",
        maker: "onsemi",
        model: Model::Bjt(Polarity::Darlington),
        max_amps: 5.0,
        max_volts: 60.0,
        gain: Some(Gain {
            min: 1000.0,
            max: None,
            at_amps: 0.5,
        }),
        threshold: None,
        on_resistance: None,
        power: Some(2.0),
        package: Some("TO-220"),
        datasheet: "https://www.onsemi.com/download/data-sheet/pdf/tip120-d.pdf",
    },
    TransistorPart {
        number: "2N7000",
        maker: "onsemi",
        model: Model::Mosfet(Channel::N),
        max_amps: 0.2,
        max_volts: 60.0,
        gain: None,
        threshold: Some((0.8, 3.0)),
        on_resistance: Some(OnResistance {
            ohms: 6.0,
            at_gate_volts: 4.5,
        }),
        // Given only with the case at 25 °C.
        power: None,
        package: Some("TO-92"),
        datasheet: "https://www.onsemi.com/download/data-sheet/pdf/2n7000-d.pdf",
    },
    TransistorPart {
        number: "BS250P",
        maker: "Diodes Incorporated",
        model: Model::Mosfet(Channel::P),
        max_amps: 0.23,
        max_volts: 45.0,
        gain: None,
        threshold: Some((1.0, 3.5)),
        on_resistance: Some(OnResistance {
            ohms: 14.0,
            at_gate_volts: 10.0,
        }),
        power: Some(0.7),
        // Not in the sheet's text.
        package: None,
        datasheet: "https://www.diodes.com/assets/Datasheets/BS250P.pdf",
    },
    TransistorPart {
        number: "IRLZ44N",
        maker: "Infineon",
        model: Model::Mosfet(Channel::Power),
        max_amps: 47.0,
        max_volts: 55.0,
        gain: None,
        threshold: Some((1.0, 2.0)),
        on_resistance: Some(OnResistance {
            ohms: 0.025,
            at_gate_volts: 5.0,
        }),
        // Given only with the case at 25 °C.
        power: None,
        package: Some("TO-220"),
        datasheet: "https://www.infineon.com/dgdl/irlz44npbf.pdf?fileId=5546d462533600a40153567217c32725",
    },
];

/// The LEDs that can be bought for a colour, the first being the one to name.
pub fn leds(colour: LedColour) -> impl Iterator<Item = &'static LedPart> {
    LEDS.iter().filter(move |part| part.colour == colour)
}

/// The transistors that can be bought for a placed transistor, the first
/// being the one its model is based on.
pub fn transistors(kind: &ElementKind) -> impl Iterator<Item = &'static TransistorPart> {
    let model = match *kind {
        ElementKind::Bjt { polarity } => Some(Model::Bjt(polarity)),
        ElementKind::Mosfet { channel } => Some(Model::Mosfet(channel)),
        _ => None,
    };
    TRANSISTORS
        .iter()
        .filter(move |part| Some(part.model) == model)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_colour_the_app_models_can_be_bought() {
        for &colour in LedColour::ALL {
            assert!(leds(colour).next().is_some(), "{colour:?}");
        }
    }

    #[test]
    fn every_transistor_the_app_models_can_be_bought_as_the_part_it_is_based_on() {
        // The first part offered for each model is the one the registry says
        // the model is based on (C-112), so the bill names what was simulated.
        let kinds = Polarity::ALL
            .iter()
            .map(|&polarity| ElementKind::Bjt { polarity })
            .chain(
                Channel::ALL
                    .iter()
                    .map(|&channel| ElementKind::Mosfet { channel }),
            );
        for kind in kinds {
            let first = transistors(&kind)
                .next()
                .unwrap_or_else(|| panic!("{kind:?}"));
            let basis = crate::registry::definition(kind.name())
                .and_then(|component| (component.basis)(&kind))
                .unwrap();
            assert!(
                basis.part.starts_with(first.number) || first.number.starts_with(basis.part),
                "{kind:?}: {} against {}",
                first.number,
                basis.part
            );
        }
    }

    #[test]
    fn its_figures_hang_together() {
        for part in LEDS {
            assert!(
                part.forward_volts < part.forward_volts_max,
                "{}",
                part.number
            );
            assert!(part.at_amps <= part.max_amps, "{}", part.number);
            assert!(part.datasheet.starts_with("https://"), "{}", part.number);
        }
        for part in TRANSISTORS {
            let bipolar = matches!(part.model, Model::Bjt(_));
            assert_eq!(part.gain.is_some(), bipolar, "{}", part.number);
            assert_eq!(part.threshold.is_some(), !bipolar, "{}", part.number);
            if let Some(gain) = part.gain {
                assert!(gain.max.is_none_or(|max| max > gain.min), "{}", part.number);
                assert!(gain.at_amps < part.max_amps, "{}", part.number);
            }
            if let Some((least, most)) = part.threshold {
                assert!(least < most, "{}", part.number);
            }
            assert!(part.datasheet.starts_with("https://"), "{}", part.number);
        }
    }

    #[test]
    fn a_kit_has_more_than_one_small_transistor_of_each_sign() {
        let npn = transistors(&ElementKind::Bjt {
            polarity: Polarity::Npn,
        })
        .map(|part| part.number)
        .collect::<Vec<_>>();
        assert_eq!(npn, ["2N3904", "BC547B", "PN2222A"]);
        assert_eq!(
            transistors(&ElementKind::Bjt {
                polarity: Polarity::Pnp
            })
            .count(),
            2
        );
    }
}
