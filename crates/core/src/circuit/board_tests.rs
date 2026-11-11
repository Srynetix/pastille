// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

/// The milestone, built the way the app builds it.
fn milestone() -> (Circuit, String, String, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 160.0, 0.0).unwrap();
    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
    circuit.try_connect(&led, "cathode", &battery, "-").unwrap();
    (circuit, battery, resistor, led)
}

fn legs(view: &BoardView, id: &str) -> Vec<(f64, f64)> {
    view.parts
        .iter()
        .find(|part| part.element_id == id)
        .unwrap()
        .legs
        .iter()
        .map(|leg| (leg.x, leg.y))
        .collect()
}

#[test]
fn the_board_shows_every_part_and_the_battery_cannot_be_moved() {
    let (circuit, battery, resistor, led) = milestone();
    let view = circuit.try_breadboard().unwrap();
    assert_eq!(view.size, "half");
    assert_eq!(view.holes.len(), 400);
    assert_eq!(view.parts.len(), 3);
    let of = |id: &str| {
        view.parts
            .iter()
            .find(|part| part.element_id == id)
            .unwrap()
    };
    assert!(!of(&battery).movable);
    assert!(of(&resistor).movable && of(&led).movable);
    assert!(view.problems.is_empty(), "{:?}", view.problems);
}

#[test]
fn moving_a_part_changes_the_board_and_not_the_schematic() {
    let (mut circuit, _, resistor, _) = milestone();
    circuit.advance_by(1.0 / 60.0).unwrap();
    let schematic = (circuit.views(), circuit.to_document().nets);
    let current = circuit.solution().element_currents;
    let before = legs(&circuit.try_breadboard().unwrap(), &resistor);

    circuit.try_move_on_board(&resistor, 6, 0).unwrap();
    let view = circuit.try_breadboard().unwrap();
    let after = legs(&view, &resistor);
    // A leg in a row moves by the six; one in a rail stays in its rail.
    let in_row = |y: f64| (3.0..=14.0).contains(&y);
    let followed = before.iter().zip(&after).all(|(b, a)| {
        if in_row(b.1) {
            a.0 == b.0 + 6.0 && a.1 == b.1
        } else {
            a.1 == b.1
        }
    });
    assert!(followed, "{before:?} → {after:?}");
    assert!(
        view.parts
            .iter()
            .any(|part| part.element_id == resistor && part.moved)
    );
    // The jumpers follow; nothing is shorted or left unjoined.
    assert!(view.problems.is_empty(), "{:?}", view.problems);
    // And the schematic, and what it does, are as they were.
    circuit.advance_by(1.0 / 60.0).unwrap();
    assert_eq!((circuit.views(), circuit.to_document().nets), schematic);
    assert_eq!(circuit.solution().element_currents, current);
}

#[test]
fn a_move_into_another_wires_row_is_a_short_on_the_board_only() {
    // + → R1 → R2 → LED → −: R1 and R2 meet on a wire of their own, in a
    // row; the LED's anode is on another.
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let first = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
    let second = circuit.try_add("Resistor", 160.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 240.0, 0.0).unwrap();
    circuit.try_connect(&battery, "+", &first, "a").unwrap();
    circuit.try_connect(&first, "b", &second, "a").unwrap();
    circuit.try_connect(&second, "b", &led, "anode").unwrap();
    circuit.try_connect(&led, "cathode", &battery, "-").unwrap();
    // Moved parts keep their holes and the rest are placed around them,
    // so a board-only short is two moved parts meeting: R1 moved first.
    circuit.try_move_on_board(&first, 1, 0).unwrap();
    let view = circuit.try_breadboard().unwrap();
    assert!(view.problems.is_empty(), "{:?}", view.problems);
    let leg = |id: &str, pin: &str| {
        let leg = view
            .parts
            .iter()
            .find(|part| part.element_id == id)
            .unwrap()
            .legs
            .iter()
            .find(|leg| leg.pin == pin)
            .unwrap();
        (leg.x, leg.y)
    };
    // The anode, into the row R1 and R2 meet in, a hole along from R1's.
    let (mid_x, mid_y) = leg(&first, "b");
    let (anode_x, anode_y) = leg(&led, "anode");
    let moved = [1, -1, 2, -2].into_iter().find_map(|along| {
        circuit
            .try_move_on_board(
                &led,
                (mid_x - anode_x) as i32,
                (mid_y - anode_y) as i32 + along,
            )
            .ok()
    });
    assert!(moved.is_some(), "no free hole in that row");
    let view = circuit.try_breadboard().unwrap();
    assert!(
        view.problems
            .iter()
            .any(|problem| problem.message.contains("shorted")),
        "{:?}",
        view.problems
    );
    // One undo, and it is back where the placer had it.
    assert!(circuit.undo());
    assert!(circuit.try_breadboard().unwrap().problems.is_empty());
}

#[test]
fn a_moved_part_stays_put_when_the_schematic_changes_and_is_saved() {
    let (mut circuit, battery, resistor, _) = milestone();
    circuit.try_move_on_board(&resistor, 10, 0).unwrap();
    let moved = legs(&circuit.try_breadboard().unwrap(), &resistor);
    // A second LED and resistor, wired on the schematic.
    let more = circuit.try_add("Resistor", 80.0, 80.0).unwrap();
    let led = circuit.try_add("Led", 160.0, 80.0).unwrap();
    circuit.try_connect(&battery, "+", &more, "a").unwrap();
    circuit.try_connect(&more, "b", &led, "anode").unwrap();
    circuit.try_connect(&led, "cathode", &battery, "-").unwrap();
    let view = circuit.try_breadboard().unwrap();
    assert_eq!(legs(&view, &resistor), moved);
    assert!(view.problems.is_empty(), "{:?}", view.problems);
    // Saved and opened again, it is still there.
    let text = serde_json::to_string(&circuit.to_document()).unwrap();
    let mut opened = Circuit::new();
    opened.try_open(&text).unwrap();
    let first = opened
        .netlist()
        .elements()
        .find(|(_, element)| matches!(element.kind, ElementKind::Resistor { .. }))
        .map(|(id, _)| encode(id))
        .unwrap();
    assert_eq!(legs(&opened.try_breadboard().unwrap(), &first), moved);
}

#[test]
fn some_moves_are_refused_and_change_nothing() {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let chip = circuit.try_add("Timer", 80.0, 0.0).unwrap();
    circuit.try_connect(&battery, "+", &chip, "VCC").unwrap();
    circuit.try_connect(&chip, "GND", &battery, "-").unwrap();
    let before = circuit.try_breadboard().unwrap();
    for (id, dx, dy) in [(&battery, 1, 0), (&chip, 0, 1), (&chip, -40, 0)] {
        assert!(circuit.try_move_on_board(id, dx, dy).is_err(), "{dx} {dy}");
    }
    assert_eq!(circuit.try_breadboard().unwrap(), before);
    // Along the channel, it slides.
    circuit.try_move_on_board(&chip, 3, 0).unwrap();
    assert!(circuit.try_breadboard().unwrap().problems.is_empty());
}
