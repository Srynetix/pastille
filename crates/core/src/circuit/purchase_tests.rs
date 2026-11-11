// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

#[test]
fn a_resistor_steps_through_e12_either_side_of_its_value() {
    // C-136: from a value in the series, and from one between two.
    let (circuit, resistor) = led_behind("4.7k");
    let steps = circuit.try_steps(&resistor).unwrap();
    let values = |side: &[Step]| side.iter().map(|s| s.value.clone()).collect::<Vec<_>>();
    assert_eq!(values(&steps.below), ["3.9k", "3.3k"]);
    assert_eq!(values(&steps.above), ["5.6k", "6.8k"]);
    assert_eq!(steps.above[0].label, "5.6 kΩ");
    let (circuit, resistor) = led_behind("217");
    let steps = circuit.try_steps(&resistor).unwrap();
    assert_eq!(values(&steps.below), ["180", "150"]);
    assert_eq!(values(&steps.above), ["220", "270"]);
}

#[test]
fn a_capacitor_steps_and_a_part_with_no_series_does_not() {
    let mut circuit = Circuit::new();
    let capacitor = circuit.try_add("Capacitor", 0.0, 0.0).unwrap();
    circuit.try_set_value(&capacitor, "100n").unwrap();
    let steps = circuit.try_steps(&capacitor).unwrap();
    // E6, as it is bought (B-09): 68 n, 100 n, 150 n.
    assert_eq!(steps.above[0].label, "150 nF");
    assert_eq!(steps.below[0].label, "68 nF");
    let battery = circuit.try_add("DcSource", 80.0, 0.0).unwrap();
    assert_eq!(circuit.try_steps(&battery).unwrap(), Steps::default());
}

/// A 9 V battery, a resistor of `ohms` and a red LED.
fn led_behind(ohms: &str) -> (Circuit, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 160.0, 0.0).unwrap();
    circuit.try_set_value(&resistor, ohms).unwrap();
    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
    circuit.try_connect(&led, "cathode", &battery, "-").unwrap();
    circuit.advance_by(1.0 / 60.0).unwrap();
    (circuit, resistor)
}

#[test]
fn a_computed_resistor_is_told_what_to_buy_and_what_flows_through_it() {
    // The design doc's 217 Ω (§6.2): buy 220 Ω, 1.4 % high.
    let (circuit, resistor) = led_behind("217");
    let purchase = circuit.try_purchase(&resistor).unwrap().unwrap();
    assert_eq!(purchase.value, "220");
    assert_eq!(purchase.label, "220 Ω");
    assert_eq!(purchase.series, "E12");
    assert!((purchase.error_percent - 1.38).abs() < 0.01, "{purchase:?}");
    // (9 V less the LED) through 220 Ω.
    let amps = purchase.amps.unwrap();
    assert!((0.030..0.034).contains(&amps), "{amps} A");
    // 217 Ω is already past the LED's rating, so the rounding adds nothing.
    assert!(purchase.findings.is_empty(), "{:?}", purchase.findings);
}

#[test]
fn the_rounding_itself_can_raise_a_warning() {
    // 350 Ω keeps a red LED inside its rating; the nearest E12, 330 Ω,
    // does not, and that is the rounding's to say.
    let (circuit, resistor) = led_behind("350");
    assert!(
        circuit.solution().findings.is_empty(),
        "{:?}",
        circuit.solution().findings
    );
    let purchase = circuit.try_purchase(&resistor).unwrap().unwrap();
    assert_eq!(purchase.value, "330");
    let rules: Vec<_> = purchase
        .findings
        .iter()
        .map(|finding| finding.rule.as_str())
        .collect();
    assert_eq!(rules, ["over-rated"], "{:?}", purchase.findings);
    // And the live circuit is untouched.
    assert!(circuit.solution().findings.is_empty());
}

#[test]
fn a_value_that_can_be_bought_is_nothing_to_buy() {
    let (circuit, resistor) = led_behind("470");
    assert_eq!(circuit.try_purchase(&resistor).unwrap(), None);
}

#[test]
fn a_capacitor_is_bought_in_e6_and_given_no_current() {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let capacitor = circuit.try_add("Capacitor", 80.0, 0.0).unwrap();
    circuit.try_set_value(&capacitor, "4.5u").unwrap();
    circuit.try_connect(&battery, "+", &capacitor, "a").unwrap();
    circuit.try_connect(&capacitor, "b", &battery, "-").unwrap();
    let purchase = circuit.try_purchase(&capacitor).unwrap().unwrap();
    assert_eq!(purchase.label, "4.7 µF");
    assert_eq!(purchase.series, "E6");
    assert_eq!(purchase.amps, None);
}

#[test]
fn a_part_with_no_value_to_buy_is_nothing() {
    let mut circuit = Circuit::new();
    let led = circuit.try_add("Led", 0.0, 0.0).unwrap();
    assert_eq!(circuit.try_purchase(&led).unwrap(), None);
}
