// SPDX-License-Identifier: MIT OR Apache-2.0

//! Pins, and the electrical model they carry.
//!
//! In v1 every pin is [`PinDiscipline::Electrical`] and almost every field
//! below is `None`. That is the point: this is seam #3 of the design doc
//! (7.1.3), and it exists now because retrofitting it is the expensive path.
//!
//! Phase 3 attaches an MCU through it — a digital output drives a real net
//! through its output impedance and slew rate, and an analog input comes back
//! through a comparator with hysteresis. Phase 4 needs the same shape at the
//! co-simulation boundary. MAME's netlist reached the same conclusion from the
//! other direction: it inserts proxy devices wherever a logic net meets an
//! analog one, rather than special-casing inside the solver (see the prior-art study).

use super::ElementId;

/// What kind of thing a pin speaks.
///
/// The variant decides which parts of [`ElectricalModel`] mean anything, and
/// which boundary component is needed when two disciplines meet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PinDiscipline {
    /// A node in the analog solver. Everything in v1.
    #[default]
    Electrical,
    /// A digital level, driven and sensed through thresholds. Phase 3.
    Logic,
    /// A bus or protocol endpoint, exchanged as messages rather than voltages.
    /// Phase 4 — I²C, SPI and the like, where modelling every edge is waste.
    Transactional,
}

/// Output levels as fractions of the supply, never as absolute volts.
///
/// A part's datasheet gives Voh relative to Vdd for exactly this reason: the
/// same logic family at 5 V and at 3.3 V has the same numbers here.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LogicDrive {
    /// High-level output, as a fraction of Vdd. 0.9 is a typical CMOS figure.
    pub voh_fraction: f64,
    /// Low-level output, as a fraction of Vdd.
    pub vol_fraction: f64,
}

/// Input switching thresholds, again as fractions of Vdd.
///
/// The gap between them is the hysteresis that makes a bounded phase error at
/// the analog/digital boundary invisible at the logic level — the
/// simplification phase 4 depends on (design doc 8, phase 4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InputThresholds {
    /// Below this fraction of Vdd, the input reads low.
    pub vil_fraction: f64,
    /// Above this fraction of Vdd, the input reads high.
    pub vih_fraction: f64,
}

impl InputThresholds {
    /// Width of the band where the input holds its previous value.
    pub fn hysteresis_fraction(&self) -> f64 {
        self.vih_fraction - self.vil_fraction
    }
}

/// How a pin behaves electrically.
///
/// Every field is optional because an ideal pin — all of v1 — declares none of
/// them. `None` means "not modelled", never "zero": a zero output impedance is
/// a real and different claim.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ElectricalModel {
    /// Series resistance the pin drives through, in ohms.
    pub output_impedance: Option<f64>,
    /// How fast the pin can move, in volts per second.
    pub slew_rate: Option<f64>,
    /// Supply this pin's levels are referenced to, in volts.
    pub vdd: Option<f64>,
    pub drive: Option<LogicDrive>,
    pub thresholds: Option<InputThresholds>,
}

impl ElectricalModel {
    /// Absolute high-level output voltage, if both a supply and a drive are known.
    pub fn voh(&self) -> Option<f64> {
        Some(self.vdd? * self.drive?.voh_fraction)
    }

    /// Absolute low-level output voltage, if both a supply and a drive are known.
    pub fn vol(&self) -> Option<f64> {
        Some(self.vdd? * self.drive?.vol_fraction)
    }
}

/// One connection point of an element.
#[derive(Debug, Clone, PartialEq)]
pub struct Pin {
    pub name: &'static str,
    pub discipline: PinDiscipline,
    pub electrical: ElectricalModel,
    /// The net this pin sits on, or `None` while it is unconnected.
    pub net: Option<super::NetId>,
}

impl Pin {
    /// An ideal analog pin: connected to nothing, modelling nothing. Every pin
    /// in v1 starts here.
    pub(super) fn electrical(name: &'static str) -> Self {
        Pin {
            name,
            discipline: PinDiscipline::Electrical,
            electrical: ElectricalModel::default(),
            net: None,
        }
    }
}

/// Identifies one pin of one element. Pins have no independent existence, so
/// they have no id of their own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinRef {
    pub element: ElementId,
    pub index: usize,
}

impl PinRef {
    pub fn new(element: ElementId, index: usize) -> Self {
        PinRef { element, index }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_v1_pin_models_nothing() {
        let pin = Pin::electrical("a");

        assert_eq!(pin.discipline, PinDiscipline::Electrical);
        assert_eq!(pin.electrical, ElectricalModel::default());
        assert_eq!(pin.electrical.voh(), None);
        assert!(pin.net.is_none());
    }

    #[test]
    fn the_shape_can_describe_an_mcu_output_pin() {
        // What phase 3 will need to express: an AVR pin at 5 V, driving through
        // its output impedance, with CMOS levels. Nothing consumes this yet —
        // the test exists to prove the shape is sufficient before anything
        // depends on it.
        let model = ElectricalModel {
            output_impedance: Some(25.0),
            slew_rate: Some(50e6),
            vdd: Some(5.0),
            drive: Some(LogicDrive {
                voh_fraction: 0.9,
                vol_fraction: 0.1,
            }),
            thresholds: Some(InputThresholds {
                vil_fraction: 0.3,
                vih_fraction: 0.6,
            }),
        };

        assert_eq!(model.voh(), Some(4.5));
        assert_eq!(model.vol(), Some(0.5));
        assert_eq!(model.thresholds.unwrap().hysteresis_fraction(), 0.3);
    }

    #[test]
    fn levels_scale_with_the_supply_not_with_the_family() {
        // The same logic family at 3.3 V: identical fractions, different volts.
        let drive = LogicDrive {
            voh_fraction: 0.9,
            vol_fraction: 0.1,
        };
        let at_5v = ElectricalModel {
            vdd: Some(5.0),
            drive: Some(drive),
            ..ElectricalModel::default()
        };
        let at_3v3 = ElectricalModel {
            vdd: Some(3.3),
            ..at_5v
        };

        assert_eq!(at_5v.voh(), Some(4.5));
        assert_eq!(at_3v3.voh(), Some(0.9 * 3.3));
    }

    #[test]
    fn an_unsupplied_pin_has_no_absolute_levels() {
        let model = ElectricalModel {
            drive: Some(LogicDrive {
                voh_fraction: 0.9,
                vol_fraction: 0.1,
            }),
            ..ElectricalModel::default()
        };

        assert_eq!(model.voh(), None);
    }
}
