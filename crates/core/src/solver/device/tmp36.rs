// SPDX-License-Identifier: MIT OR Apache-2.0

//! A TMP36: a temperature sensor whose output is a voltage (C-87).
//!
//! The first sensor here that puts out a voltage rather than being a
//! resistance (the LDR and the thermistor are, C-65), and the one every
//! Arduino kit's first analogue reading uses. Behavioural under ADR 0010:
//! the saturating source of the regulator and the op-amp with no gain, its
//! target a fixed height above GND, its rails the supply.
//!
//! From Analog Devices' TMP35/TMP36/TMP37 sheet, Rev. F, for the TMP36:
//! 10 mV/°C and 750 mV at 25 °C, so 0.5 V + 10 mV per degree, from −40 °C to
//! +125 °C; a supply of 2.7 V to 5.5 V drawing at most 50 µA unloaded (the
//! sheet gives no typical); 50 µA of load, 250 µA into a short; load
//! regulation of 6 m°C/µA typical, which at 10 mV/°C is 60 Ω of output
//! resistance. Without its supply the source sits at its low rail: it puts
//! out nothing, and the review says why.
//!
//! **The temperature it reads is its own dial** (Denis, 2026-10-01), as the
//! thermistor's is: a beginner turns it and watches Vout move 10 mV a degree.
//! The room's temperature (C-74) stays the circuit's, for the parts that warm.
//!
//! What it ignores: its ±2 °C accuracy and ±0.5 °C non-linearity; its own
//! self-heating; the SHUTDOWN pin of the 8-pin parts (the TO-92 a kit has
//! has none); how close to its supply the output can swing, which the sheet
//! does not give and nothing here gets near — 2 V out is guaranteed on 2.7 V.

use super::saturating::{HIGH_RAIL, INVERTING, LOW_RAIL, NON_INVERTING, OUTPUT, SaturatingSource};

/// Pin order: the order the registry names them in.
pub const SUPPLY_PIN: usize = 0;
pub const OUTPUT_PIN: usize = 1;
pub const GROUND_PIN: usize = 2;

/// Which of its pins each terminal of the saturating source sits on: no
/// inputs to compare, so both on GND; the supply the high rail and GND the
/// low, so the battery pays for what it draws.
pub const WIRING: [usize; 5] = {
    let mut wiring = [0; 5];
    wiring[NON_INVERTING] = GROUND_PIN;
    wiring[INVERTING] = GROUND_PIN;
    wiring[OUTPUT] = OUTPUT_PIN;
    wiring[HIGH_RAIL] = SUPPLY_PIN;
    wiring[LOW_RAIL] = GROUND_PIN;
    wiring
};

/// The part, for the label and the Inspector.
pub const PART: &str = "TMP36";

/// Its sheet.
pub const DATASHEET: &str =
    "https://www.analog.com/media/en/technical-documentation/data-sheets/TMP35_36_37.pdf";

/// Volts out at 0 °C, and per degree.
pub const OFFSET_VOLTS: f64 = 0.5;
pub const VOLTS_PER_DEGREE: f64 = 0.01;

/// The supply it runs on, in volts.
pub const LEAST_SUPPLY: f64 = 2.7;
pub const MOST_SUPPLY: f64 = 5.5;

/// Its specified range, which is also the dial's, in °C.
pub const COLDEST: f64 = -40.0;
pub const HOTTEST: f64 = 125.0;

/// Supply current, the sheet's maximum unloaded.
const QUIESCENT_AMPS: f64 = 50e-6;

/// 6 m°C/µA at 10 mV/°C.
const OUTPUT_OHMS: f64 = 60.0;

/// Short-circuit current.
const SOURCE_LIMIT: f64 = 250e-6;

/// What it can pull down, which the sheet does not give: as small as the
/// regulator's, for the same reason — a corner at exactly zero is where an
/// unloaded output sits.
const SINK_LIMIT: f64 = 1e-6;

/// The volts it puts out at a temperature, powered.
pub fn output_volts(celsius: f64) -> f64 {
    OFFSET_VOLTS + VOLTS_PER_DEGREE * celsius
}

/// The temperature a dial position stands for, in °C.
pub fn celsius(dial: f64) -> f64 {
    COLDEST + (HOTTEST - COLDEST) * dial.clamp(0.0, 1.0)
}

/// The dial position for a temperature.
pub fn dial_for(celsius: f64) -> f64 {
    ((celsius - COLDEST) / (HOTTEST - COLDEST)).clamp(0.0, 1.0)
}

/// The output stage, at the temperature its dial reads.
pub fn model(celsius: f64) -> SaturatingSource {
    SaturatingSource {
        gain: 0.0,
        offset: output_volts(celsius),
        reference: 0.0,
        headroom_high: 0.0,
        headroom_low: 0.0,
        output_ohms: OUTPUT_OHMS,
        source_limit: SOURCE_LIMIT,
        sink_limit: SINK_LIMIT,
        quiescent: QUIESCENT_AMPS,
        input_ohms: f64::INFINITY,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_reads_its_sheet_s_transfer_function() {
        // 750 mV at 25 °C; 0.5 V + 10 mV a degree across its range.
        assert!((output_volts(25.0) - 0.75).abs() < 1e-12);
        assert!((output_volts(-40.0) - 0.1).abs() < 1e-12);
        assert!((output_volts(125.0) - 1.75).abs() < 1e-12);
    }

    #[test]
    fn its_dial_covers_its_specified_range() {
        assert_eq!(celsius(0.0), COLDEST);
        assert_eq!(celsius(1.0), HOTTEST);
        for degrees in [-40.0, 0.0, 25.0, 100.0] {
            assert!((celsius(dial_for(degrees)) - degrees).abs() < 1e-9);
        }
    }
}
