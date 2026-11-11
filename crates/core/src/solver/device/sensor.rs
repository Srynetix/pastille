// SPDX-License-Identifier: MIT OR Apache-2.0

//! The two sensors in every starter kit: a light-dependent resistor and a
//! thermistor (C-65).
//!
//! Each is a resistor whose value follows something outside the circuit — the
//! light falling on it, the temperature around it — and that something is set
//! by a dial, as a potentiometer's wiper is, from 0 to 1. What the dial means is
//! here; the app says it in words.
//!
//! What they honour: an LDR's resistance falling with light as a power law, a
//! thermistor's with temperature as the B-parameter equation. What they ignore:
//! an LDR's slowness (it takes a fraction of a second to follow the light, and
//! longer to recover from dark), and a thermistor's self-heating.

/// Resistance of a GL5528 at 10 lux, the datasheet's reference, in ohms.
const LDR_AT_10_LUX: f64 = 10e3;

/// How steeply it falls with light: the datasheet's γ.
const LDR_GAMMA: f64 = 0.7;

/// Its resistance in the dark, which it never exceeds.
const LDR_DARK: f64 = 1e6;

/// The light a dial position stands for, in lux: 0.1 at the bottom — a room
/// at night — to 10 000 at the top, daylight, on a log scale because that is
/// how both eyes and photocells take light.
pub fn lux(dial: f64) -> f64 {
    10f64.powf(5.0 * dial.clamp(0.0, 1.0) - 1.0)
}

/// An LDR's resistance, in ohms, at a dial position.
pub fn ldr_ohms(dial: f64) -> f64 {
    (LDR_AT_10_LUX * (lux(dial) / 10.0).powf(-LDR_GAMMA)).min(LDR_DARK)
}

/// A 10 kΩ NTC thermistor's resistance at 25 °C, and its B constant.
const THERMISTOR_AT_25: f64 = 10e3;
const THERMISTOR_B: f64 = 3950.0;

/// The dial's range, in °C: a freezer to boiling water.
pub const COLDEST: f64 = -20.0;
pub const HOTTEST: f64 = 100.0;

/// The temperature a dial position stands for, in °C.
pub fn celsius(dial: f64) -> f64 {
    COLDEST + (HOTTEST - COLDEST) * dial.clamp(0.0, 1.0)
}

/// The dial position for a temperature.
pub fn dial_for(celsius: f64) -> f64 {
    ((celsius - COLDEST) / (HOTTEST - COLDEST)).clamp(0.0, 1.0)
}

/// A thermistor's resistance, in ohms, at a temperature in °C.
pub fn thermistor_ohms(celsius: f64) -> f64 {
    let kelvin = celsius + 273.15;
    THERMISTOR_AT_25 * (THERMISTOR_B * (1.0 / kelvin - 1.0 / 298.15)).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_ldr_reads_its_datasheet_at_ten_lux_and_falls_with_light() {
        let ten_lux = (1.0 + 1.0) / 5.0; // 10^(5·0.4 − 1) = 10
        assert!((lux(ten_lux) - 10.0).abs() < 1e-9);
        assert!((ldr_ohms(ten_lux) - 10e3).abs() < 1e-6);
        assert!(ldr_ohms(0.9) < ldr_ohms(0.1), "brighter is lower");
        assert!(ldr_ohms(0.0) <= LDR_DARK);
    }

    #[test]
    fn a_thermistor_is_ten_kilohms_at_room_temperature_and_falls_when_warm() {
        assert!((thermistor_ohms(25.0) - 10e3).abs() < 1e-6);
        // The B equation's own figures at 0 °C and 50 °C, worked by hand.
        assert!((thermistor_ohms(0.0) - 33_620.0).abs() / 33_620.0 < 0.01);
        assert!((thermistor_ohms(50.0) - 3_588.0).abs() / 3_588.0 < 0.01);
        assert!((celsius(dial_for(37.0)) - 37.0).abs() < 1e-9);
    }
}
