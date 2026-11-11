// SPDX-License-Identifier: MIT OR Apache-2.0

//! The LED: a pn junction with a forward drop that depends on its colour, plus
//! the curve that turns its current into something to draw.

use crate::netlist::LedColour;
use crate::solver::{Companion, Nonlinear};

use super::{Diode, THERMAL_VOLTAGE};

/// Current an indicator LED is specified at. Datasheet forward voltages are
/// quoted here, and it is the point the brightness curve treats as "full".
pub const RATED_CURRENT: f64 = 20e-3;

/// Emission coefficient for an LED.
///
/// Far from the ~1 of a small-signal silicon diode: a wide-bandgap junction has
/// a much shallower current/voltage slope, which is why an LED's drop moves so
/// little between a tenth of its rated current and all of it.
const EMISSION_COEFFICIENT: f64 = 2.0;

/// Exponent relating current to apparent brightness.
///
/// Luminous flux is very nearly proportional to forward current. Perceived
/// lightness is not: it follows roughly a power law, and 1/2.2 is the same
/// exponent sRGB encoding uses. Raising the current ratio to this power gives a
/// number that can be used directly as a display intensity, which is what M-20
/// wants.
const PERCEPTUAL_EXPONENT: f64 = 1.0 / 2.2;

/// An LED.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Led {
    pub colour: LedColour,
    junction: Diode,
}

impl Led {
    pub fn new(colour: LedColour) -> Self {
        Led {
            colour,
            junction: Diode {
                saturation_current: saturation_current_for(forward_voltage(colour)),
                emission_coefficient: EMISSION_COEFFICIENT,
            },
        }
    }

    /// Current through the LED at a given forward voltage.
    pub fn forward_current(&self, voltage_across: f64) -> f64 {
        self.junction.current(voltage_across)
    }

    /// Datasheet forward drop at [`RATED_CURRENT`].
    pub fn nominal_forward_voltage(&self) -> f64 {
        forward_voltage(self.colour)
    }

    /// Apparent brightness, from 0 (dark) to 1 (as bright as it usefully gets).
    ///
    /// Flux is taken as proportional to forward current, and apparent lightness
    /// as flux to the power 1/2.2. The ratio is clamped at the rated current:
    /// past that an LED does not get usefully brighter, it gets hot, and saying
    /// so is the linter's job (B-03) rather than the renderer's.
    pub fn brightness(&self, forward_current: f64) -> f64 {
        if forward_current <= 0.0 {
            return 0.0;
        }
        let ratio = (forward_current / RATED_CURRENT).min(1.0);
        ratio.powf(PERCEPTUAL_EXPONENT)
    }
}

impl Nonlinear for Led {
    fn linearise(&self, voltage_across: f64, previous_point: f64) -> Companion {
        self.junction.linearise(voltage_across, previous_point)
    }
}

/// Typical forward drop at [`RATED_CURRENT`], by colour.
///
/// These are ordinary indicator-LED figures rather than any particular part.
/// Real part numbers, with their own spreads, arrive with the catalogue in B-10.
fn forward_voltage(colour: LedColour) -> f64 {
    match colour {
        LedColour::Red => 1.8,
        LedColour::Green => 2.1,
        // Blue and white are both InGaN junctions and sit at the same drop.
        LedColour::Blue | LedColour::White => 3.2,
    }
}

/// Saturation current that puts the junction at `forward_voltage` volts when it
/// carries [`RATED_CURRENT`].
///
/// Is is what the Shockley equation actually takes; a forward voltage is what a
/// datasheet gives and what a person recognises. This inverts one into the other
/// so the colour table can be written in the units it is published in.
fn saturation_current_for(forward_voltage: f64) -> f64 {
    let scale = EMISSION_COEFFICIENT * THERMAL_VOLTAGE;
    RATED_CURRENT / ((forward_voltage / scale).exp() - 1.0)
}

#[cfg(test)]
mod tests {
    use crate::netlist::{ElementId, ElementKind, LedColour, Netlist};
    use crate::solver::operating_point;

    use super::*;

    /// The milestone circuit: a battery, a series resistor and an LED.
    fn milestone_circuit(volts: f64, ohms: f64, colour: LedColour) -> (Netlist, ElementId) {
        let mut netlist = Netlist::new();
        let battery = netlist.add_element(ElementKind::DcSource {
            volts,
            series_ohms: 0.0,
        });
        let resistor = netlist.add_element(ElementKind::Resistor { ohms });
        let led = netlist.add_element(ElementKind::Led { colour });

        let top = netlist.add_net();
        let middle = netlist.add_net();
        let bottom = netlist.add_net();

        netlist.connect_named(battery, "+", top).unwrap();
        netlist.connect_named(resistor, "a", top).unwrap();
        netlist.connect_named(resistor, "b", middle).unwrap();
        netlist.connect_named(led, "anode", middle).unwrap();
        netlist.connect_named(led, "cathode", bottom).unwrap();
        netlist.connect_named(battery, "-", bottom).unwrap();

        (netlist, led)
    }

    #[test]
    fn a_colour_sits_at_its_datasheet_drop_when_carrying_its_rated_current() {
        // The point of deriving Is from a forward voltage: feed the LED its
        // rated current and the drop comes back out as published.
        for colour in [
            LedColour::Red,
            LedColour::Green,
            LedColour::Blue,
            LedColour::White,
        ] {
            let led = Led::new(colour);
            let nominal = led.nominal_forward_voltage();

            assert!(
                (led.forward_current(nominal) - RATED_CURRENT).abs() < 1e-12,
                "{colour:?} draws {} at {nominal} V",
                led.forward_current(nominal)
            );
        }
    }

    #[test]
    fn colours_are_ordered_by_forward_drop() {
        // Red lowest, blue and white highest — a red LED lights from two cells
        // where a blue one will not.
        let red = Led::new(LedColour::Red).nominal_forward_voltage();
        let green = Led::new(LedColour::Green).nominal_forward_voltage();
        let blue = Led::new(LedColour::Blue).nominal_forward_voltage();
        let white = Led::new(LedColour::White).nominal_forward_voltage();

        assert!(red < green && green < blue);
        assert_eq!(blue, white);
    }

    #[test]
    fn the_milestone_circuit_solves_end_to_end() {
        // 9 V, 220 Ω, red LED. Reference found by bisecting
        // (9 − v)/220 = Is·(exp(v/2Vt) − 1) in 50-digit arithmetic.
        const EXPECTED_V: f64 = 1.825_281_138_046_504;
        const EXPECTED_I: f64 = 3.261_235_846_342_498e-2;

        let (netlist, led) = milestone_circuit(9.0, 220.0, LedColour::Red);
        let point = operating_point(&netlist).unwrap();

        let v = point.voltage_across(led).unwrap();
        assert!((v - EXPECTED_V).abs() < 1e-9, "v = {v}");
        assert!((Led::new(LedColour::Red).forward_current(v) - EXPECTED_I).abs() < 1e-9);
    }

    #[test]
    fn raising_the_resistor_dims_the_led() {
        // The behaviour M-20 renders: the same circuit, a bigger resistor, less
        // current, less light. All three currents computed by bisection.
        let red = Led::new(LedColour::Red);
        let brightness_with = |ohms: f64| {
            let (netlist, led) = milestone_circuit(9.0, ohms, LedColour::Red);
            let point = operating_point(&netlist).unwrap();
            let current = red.forward_current(point.voltage_across(led).unwrap());
            (current, red.brightness(current))
        };

        let (i_220, b_220) = brightness_with(220.0);
        let (i_470, b_470) = brightness_with(470.0);
        let (i_1k, b_1k) = brightness_with(1_000.0);

        assert!((i_470 - 1.534_827_142_501_364e-2).abs() < 1e-9);
        assert!((i_1k - 7.252_448_185_030_159e-3).abs() < 1e-9);

        assert!(i_220 > i_470 && i_470 > i_1k);
        assert!(b_470 > b_1k);
        // 220 Ω already drives it past its rating, so brightness is pinned.
        assert_eq!(b_220, 1.0);
    }

    #[test]
    fn brightness_is_perceptual_rather_than_proportional() {
        let led = Led::new(LedColour::Red);

        // Half the rated current is much more than half as bright to the eye.
        let half = led.brightness(RATED_CURRENT / 2.0);
        assert!(half > 0.7, "half current reads as {half}");
        assert!(half < 1.0);

        // A tenth of the current is still clearly visible, which is what makes
        // an LED useful as an indicator.
        assert!(led.brightness(RATED_CURRENT / 10.0) > 0.3);
    }

    #[test]
    fn brightness_is_bounded_at_both_ends() {
        let led = Led::new(LedColour::Red);

        assert_eq!(led.brightness(0.0), 0.0);
        assert_eq!(led.brightness(-1e-3), 0.0, "reverse current is not light");
        assert_eq!(led.brightness(RATED_CURRENT), 1.0);
        // Past the rating it does not get brighter — it gets hot. Saying so is
        // the linter's job (B-03), not the renderer's.
        assert_eq!(led.brightness(RATED_CURRENT * 10.0), 1.0);
    }

    #[test]
    fn an_led_below_its_forward_voltage_stays_dark() {
        // A single cell cannot light a blue LED.
        let (netlist, led) = milestone_circuit(1.5, 220.0, LedColour::Blue);
        let point = operating_point(&netlist).unwrap();

        let blue = Led::new(LedColour::Blue);
        let current = blue.forward_current(point.voltage_across(led).unwrap());

        assert!(current < 1e-6, "current = {current:e}");
        assert!(blue.brightness(current) < 0.1);
    }

    #[test]
    fn a_reversed_led_blocks() {
        // Anode and cathode swapped: the junction blocks and nothing lights.
        let mut netlist = Netlist::new();
        let battery = netlist.add_element(ElementKind::DcSource {
            volts: 9.0,
            series_ohms: 0.0,
        });
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: 220.0 });
        let led = netlist.add_element(ElementKind::Led {
            colour: LedColour::Red,
        });

        let top = netlist.add_net();
        let middle = netlist.add_net();
        let bottom = netlist.add_net();
        netlist.connect_named(battery, "+", top).unwrap();
        netlist.connect_named(resistor, "a", top).unwrap();
        netlist.connect_named(resistor, "b", middle).unwrap();
        netlist.connect_named(led, "cathode", middle).unwrap();
        netlist.connect_named(led, "anode", bottom).unwrap();
        netlist.connect_named(battery, "-", bottom).unwrap();

        let point = operating_point(&netlist).unwrap();
        let red = Led::new(LedColour::Red);
        let current = red.forward_current(point.voltage_across(led).unwrap());

        assert!(current.abs() < 1e-15, "current = {current:e}");
        assert_eq!(red.brightness(current), 0.0);
    }
}
