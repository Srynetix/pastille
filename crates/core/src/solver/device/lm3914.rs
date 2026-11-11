// SPDX-License-Identifier: MIT OR Apache-2.0

//! An LM3914: a voltage shown as a bar of up to ten LEDs (C-119).
//!
//! Inside, a buffer on SIG feeds ten comparators, each against a tap of a
//! ten-resistor divider strung between RLO and RHI: the k-th turns its LED on
//! once SIG passes RLO + k/10 of the way to RHI. In bar mode (MODE tied to
//! V+) every LED whose step has been passed is on; in dot mode (MODE left
//! open) only the highest. Each output is a current sink into V−, set by the
//! chip's 1.25 V reference: about ten times the current drawn out of REF OUT
//! flows through each lit LED, so the resistor on the reference sets both
//! the scale and the brightness.
//!
//! Behavioural, under ADR 0010, as the CD4017 is (C-110): which LEDs are on,
//! and how hard, is read after each step from where the step left the pins,
//! and drives the next. Ten comparators of C-56's kind would cost a frame
//! for what is a staircase.
//!
//! **From TI's sheet, SNVS761B (rev. March 2013)**, at 25 °C: 3 to 25 V; the
//! reference 1.25 V nominal between REF OUT and REF ADJ (1.2 to 1.34 V, 1.28
//! typical), its load regulation 0.4 % from 0.1 to 4 mA, and 75 µA out of
//! REF ADJ; 10 mA in each lit LED for 1 mA from the reference; an output
//! saturating at 0.15 V for 2 mA; the divider 12 kΩ end to end; 2.4 mA
//! standing at 5 V; bar mode within 20 mV of V+, dot mode 200 mV below it or
//! open; an input bias of 25 nA. What it ignores: the comparators' 3 mV of
//! offset; dot mode's carry between chained chips, and LED 10's switch-off
//! that goes with it; the LEDs' 30 mA ceiling, which is a rating rather than
//! a limit the part enforces — held here at 30 mA all the same.

use crate::solver::{Charge, Companion, Nonlinear};

/// Pins, in the package's order: 1 to 18.
pub const LED1: usize = 0;
pub const V_MINUS: usize = 1;
pub const V_PLUS: usize = 2;
pub const RLO: usize = 3;
pub const SIG: usize = 4;
pub const RHI: usize = 5;
pub const REF_OUT: usize = 6;
pub const REF_ADJ: usize = 7;
pub const MODE: usize = 8;

/// The pins' names, in that order: LED 10 at pin 10, and down to LED 2 at 18.
pub const PINS: [&str; 18] = [
    "L1", "V-", "V+", "RLO", "SIG", "RHI", "REFOUT", "REFADJ", "MODE", "L10", "L9", "L8", "L7",
    "L6", "L5", "L4", "L3", "L2",
];

/// The pin of LED `k`, 1 to 10.
pub const fn led_pin(k: usize) -> usize {
    if k == 1 { LED1 } else { 19 - k }
}

/// The reference's voltage, nominal, and its resistance: 0.4 % of it over
/// 3.9 mA of load.
pub const REFERENCE_VOLTS: f64 = 1.25;
pub const REFERENCE_OHMS: f64 = 0.004 * REFERENCE_VOLTS / 3.9e-3;

/// What leaves REF ADJ, in amps.
pub const ADJUST_AMPS: f64 = 75e-6;

/// Each lit LED's current, per amp drawn out of REF OUT.
pub const GAIN: f64 = 10.0;

/// The most an output is let sink, in amps.
pub const MOST_LED_AMPS: f64 = 30e-3;

/// The divider, RHI to RLO, in ohms.
pub const DIVIDER_OHMS: f64 = 12e3;

/// V+ to V− when every output is off: 2.4 mA at 5 V.
pub const STANDING_OHMS: f64 = 5.0 / 2.4e-3;

/// Below this between V+ and V−, nothing works.
pub const LEAST_SUPPLY: f64 = 3.0;

/// Bar mode while MODE is within this of V+; the sheet's comparator sits at
/// 100 mV.
pub const BAR_WITHIN: f64 = 0.1;

/// What an input and an off output leak, in siemens: the sheet's 25 nA of
/// bias at a volt or so, and its 0.1 µA of output leakage.
pub const LEAK: f64 = 1e-8;

/// The most it may have across it, in volts.
pub const MOST_SUPPLY: f64 = 25.0;

/// What a step leaves it doing.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct State {
    /// Bit k − 1 set for LED k lit.
    pub lit: u16,
    /// What each lit output sinks, in amps.
    pub amps: f64,
    /// Whether V+ is far enough above V− to run it.
    pub powered: bool,
}

impl State {
    pub fn encode(self) -> Charge {
        Charge {
            value: f64::from(self.lit),
            slope: None,
            latched: self.powered,
            held: [self.amps, 0.0],
        }
    }

    pub fn decode(carried: Charge) -> State {
        State {
            lit: carried.value as u16,
            amps: carried.held[0],
            powered: carried.latched,
        }
    }

    /// Whether LED `k`, 1 to 10, is lit.
    pub fn is_lit(self, k: usize) -> bool {
        self.lit & (1 << (k - 1)) != 0
    }
}

/// The step at which LED `k` lights, 1 to 10, between `low` and `high`.
pub fn threshold(low: f64, high: f64, k: usize) -> f64 {
    low + (high - low) * k as f64 / 10.0
}

/// What the chip does next, from its pins' potentials — `None` for a pin on
/// no net.
pub fn read(volts: &[Option<f64>; 18]) -> State {
    let (Some(plus), Some(minus)) = (volts[V_PLUS], volts[V_MINUS]) else {
        return State::default();
    };
    if plus - minus < LEAST_SUPPLY {
        return State::default();
    }
    let at = |pin: usize| volts[pin].unwrap_or(minus);
    // What REF OUT is giving, from V+: as far as it has sagged below its
    // 1.25 V over REF ADJ, over its resistance.
    let reference = (REFERENCE_VOLTS - (at(REF_OUT) - at(REF_ADJ))) / REFERENCE_OHMS;
    let amps = (GAIN * reference).clamp(0.0, MOST_LED_AMPS);
    let signal = at(SIG);
    let passed = (1..=10)
        .filter(|&k| signal > threshold(at(RLO), at(RHI), k))
        .fold(0u16, |lit, k| lit | 1 << (k - 1));
    let bar = volts[MODE].is_some_and(|mode| mode >= plus - BAR_WITHIN);
    let lit = if bar || passed == 0 {
        passed
    } else {
        // Dot: only the highest step passed.
        1 << (15 - passed.leading_zeros())
    };
    State {
        lit,
        amps,
        powered: true,
    }
}

/// How sharply an output comes out of saturation: 95 % of its current by
/// three of these, which is the sheet's 0.15 V at 2 mA.
const KNEE: f64 = 0.05;

/// One output, as Newton sees it: from its pin to V−, sinking `amps` once it
/// is a tenth of a volt or so clear of V−, and less as it saturates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sink {
    pub amps: f64,
}

impl Sink {
    /// Into the pin, with `volts` from it to V−, and the slope.
    fn current_and_slope(&self, volts: f64) -> (f64, f64) {
        let (on, slope) = if volts >= 0.0 {
            let fall = (-volts / KNEE).exp();
            (self.amps * (1.0 - fall), self.amps * fall / KNEE)
        } else {
            // Below V− it would be a short; carried on as the slope at zero
            // so the curve and its slope are continuous.
            (self.amps * volts / KNEE, self.amps / KNEE)
        };
        (on + LEAK * volts, slope + LEAK)
    }

    pub fn current(&self, volts: f64) -> f64 {
        self.current_and_slope(volts).0
    }
}

impl Nonlinear for Sink {
    fn linearise(&self, voltage_across: f64, _previous_point: f64) -> Companion {
        // Bounded in slope and current both, so a raw step is safe.
        let (current, slope) = self.current_and_slope(voltage_across);
        Companion {
            conductance: slope,
            current: current - slope * voltage_across,
            point: voltage_across,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A chip on 5 V, the divider across the reference (RLO at 0 V, RHI at
    /// 1.25 V) with `reference` amps drawn from it, SIG at `signal` and MODE
    /// on V+ for `bar`.
    fn pins(signal: f64, reference: f64, bar: bool) -> [Option<f64>; 18] {
        let mut volts = [None; 18];
        volts[V_MINUS] = Some(0.0);
        volts[V_PLUS] = Some(5.0);
        volts[RLO] = Some(0.0);
        volts[REF_ADJ] = Some(0.0);
        let held = REFERENCE_VOLTS - reference * REFERENCE_OHMS;
        volts[REF_OUT] = Some(held);
        volts[RHI] = Some(held);
        volts[SIG] = Some(signal);
        if bar {
            volts[MODE] = Some(5.0);
        }
        volts
    }

    #[test]
    fn each_step_is_a_tenth_of_the_divider() {
        // 1.25 V across it: a step every 125 mV.
        for k in 1..=10 {
            let step = threshold(0.0, 1.25, k);
            assert!((step - 0.125 * k as f64).abs() < 1e-12);
            let below = read(&pins(step - 0.01, 1e-3, true));
            let above = read(&pins(step + 0.01, 1e-3, true));
            assert_eq!(below.lit.count_ones() as usize, k - 1, "LED {k}");
            assert_eq!(above.lit.count_ones() as usize, k, "LED {k}");
        }
    }

    #[test]
    fn dot_mode_lights_only_the_highest() {
        let bar = read(&pins(0.7, 1e-3, true));
        let dot = read(&pins(0.7, 1e-3, false));
        assert_eq!(bar.lit, 0b1_1111);
        assert_eq!(dot.lit, 0b1_0000);
        assert!(dot.is_lit(5) && !dot.is_lit(4));
        assert_eq!(read(&pins(0.05, 1e-3, false)).lit, 0);
    }

    #[test]
    fn its_leds_carry_ten_times_what_the_reference_gives() {
        for reference in [0.2e-3, 1e-3, 2e-3] {
            let state = read(&pins(1.0, reference, true));
            assert!((state.amps / (GAIN * reference) - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn unpowered_or_underpowered_it_does_nothing() {
        let mut volts = pins(1.0, 1e-3, true);
        volts[V_PLUS] = Some(2.5);
        assert_eq!(read(&volts), State::default());
        volts[V_PLUS] = None;
        assert_eq!(read(&volts), State::default());
    }

    #[test]
    fn an_output_saturates_where_its_sheet_says() {
        let sink = Sink { amps: 2e-3 };
        assert!(sink.current(0.15) > 0.94 * 2e-3);
        assert!(sink.current(0.02) < 0.5 * 2e-3);
        assert!((sink.current(5.0) - 2e-3).abs() < 1e-7);
        let off = Sink { amps: 0.0 };
        assert!(off.current(5.0) < 1e-7);
    }

    #[test]
    fn the_pins_are_the_package_s() {
        assert_eq!(PINS[led_pin(1)], "L1");
        assert_eq!(PINS[led_pin(10)], "L10");
        assert_eq!(PINS[led_pin(2)], "L2");
        assert_eq!(led_pin(2), 17);
    }
}
