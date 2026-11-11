// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

/// A 9 V battery and two red LEDs, each through its own resistor.
fn two_leds(first: &str, second: &str) -> Circuit {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    for (row, ohms) in [first, second].into_iter().enumerate() {
        let y = 80.0 * row as f64;
        let resistor = circuit.try_add("Resistor", 80.0, y).unwrap();
        let led = circuit.try_add("Led", 160.0, y).unwrap();
        circuit.try_set_value(&resistor, ohms).unwrap();
        circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
        circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
        circuit.try_connect(&led, "cathode", &battery, "-").unwrap();
    }
    let meter = circuit.try_add("Voltmeter", 240.0, 0.0).unwrap();
    circuit.try_connect(&battery, "+", &meter, "+").unwrap();
    circuit.try_connect(&meter, "-", &battery, "-").unwrap();
    circuit.advance_by(1.0 / 60.0).unwrap();
    circuit
}

#[test]
fn parts_alike_are_one_line_and_the_bench_is_not_on_it() {
    let lines = two_leds("470", "470").try_bill().unwrap();
    let summary: Vec<_> = lines
        .iter()
        .map(|line| (line.kind.as_str(), line.value.as_str(), line.quantity))
        .collect();
    assert_eq!(
        summary,
        [
            ("DcSource", "9 V", 1),
            ("Resistor", "470 Ω", 2),
            ("Led", "Red", 2)
        ]
    );
    let led = &lines[2];
    assert_eq!(led.part.as_deref(), Some("WP7113ID"));
    assert!(led.datasheet.as_deref().unwrap().contains("kingbright"));
    assert!(lines.iter().all(|line| line.notes.is_empty()), "{lines:?}");
}

#[test]
fn a_computed_value_is_bought_as_one_that_is_sold_and_says_so() {
    // 217 Ω and 220 Ω are both bought as 220 Ω: one line of two.
    let lines = two_leds("217", "220").try_bill().unwrap();
    let resistors = lines.iter().find(|line| line.kind == "Resistor").unwrap();
    assert_eq!(resistors.value, "220 Ω");
    assert_eq!(resistors.quantity, 2);
    assert!(
        resistors
            .notes
            .contains(&"Set to 217 Ω; 220 Ω is the nearest E12, 1.4 % more.".to_string()),
        "{:?}",
        resistors.notes
    );
}

#[test]
fn what_the_linter_says_goes_on_the_line() {
    // 220 Ω runs a red LED past its 20 mA (B-03).
    let lines = two_leds("220", "470").try_bill().unwrap();
    let led = lines.iter().find(|line| line.kind == "Led").unwrap();
    assert_eq!(led.quantity, 2);
    assert_eq!(led.notes.len(), 1, "{:?}", led.notes);
    assert!(led.notes[0].contains("33 mA"), "{:?}", led.notes);
}
