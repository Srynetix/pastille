// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

/// A 9 V battery, a 470 Ω resistor and a red LED, in a loop.
fn lit() -> (Circuit, String, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 160.0, 0.0).unwrap();
    circuit.try_set_value(&resistor, "470").unwrap();
    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
    circuit.try_connect(&led, "cathode", &battery, "-").unwrap();
    (circuit, resistor, led)
}

fn wired(circuit: &Circuit, id: &str) -> Vec<&'static str> {
    let element = circuit.netlist.element(decode(id).unwrap()).unwrap();
    element
        .pins
        .iter()
        .filter(|pin| pin.net.is_some())
        .map(|pin| pin.name)
        .collect()
}

#[test]
fn a_resistor_becomes_a_pot_in_place_with_its_wires_and_its_value() {
    let (mut circuit, resistor, _) = lit();
    let replaced = circuit.try_replace(&resistor, "Potentiometer").unwrap();
    assert_eq!(
        replaced.kept,
        [("a".into(), "a".into()), ("b".into(), "b".into())]
    );
    assert!(replaced.dropped.is_empty());
    let element = circuit.netlist.element(decode(&resistor).unwrap()).unwrap();
    assert!(matches!(element.kind, ElementKind::Potentiometer { ohms, .. } if ohms == 470.0));
    assert_eq!(wired(&circuit, &resistor), ["a", "b"]);
    // Still lit: the circuit runs on the new part.
    circuit.advance_by(1.0 / 60.0).unwrap();
    assert!(circuit.solution().failure.is_none());
}

#[test]
fn a_part_with_nothing_in_common_lets_its_wires_go_and_names_them() {
    let (mut circuit, resistor, led) = lit();
    let replaced = circuit.try_replace(&resistor, "Timer").unwrap();
    assert!(replaced.kept.is_empty());
    assert_eq!(replaced.dropped, ["a", "b"]);
    assert!(wired(&circuit, &resistor).is_empty());
    // The battery's + held only the resistor: that wire is gone. The
    // LED's anode likewise.
    assert_eq!(wired(&circuit, &led), ["cathode"]);
}

#[test]
fn a_replacement_is_one_step_of_undo() {
    let (mut circuit, resistor, _) = lit();
    circuit.try_replace(&resistor, "Led").unwrap();
    assert!(circuit.undo());
    let element = circuit.netlist.element(decode(&resistor).unwrap()).unwrap();
    assert!(matches!(element.kind, ElementKind::Resistor { ohms } if ohms == 470.0));
    assert_eq!(wired(&circuit, &resistor), ["a", "b"]);
}

#[test]
fn a_transistor_keeps_its_wires_by_what_each_pin_does() {
    let mut circuit = Circuit::new();
    let bjt = circuit.try_add("Bjt", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
    circuit.try_connect(&resistor, "b", &bjt, "b").unwrap();
    let replaced = circuit.try_replace(&bjt, "Mosfet").unwrap();
    assert_eq!(replaced.kept, [("b".into(), "g".into())]);
    assert_eq!(wired(&circuit, &bjt), ["g"]);
}
