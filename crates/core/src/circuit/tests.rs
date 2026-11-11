// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

#[test]
fn adds_elements_where_they_were_dropped() {
    let mut circuit = Circuit::new();

    let id = circuit.try_add("Resistor", 40.0, -20.0).unwrap();

    assert_eq!(circuit.size(), 1);
    let element = circuit.netlist().element(decode(&id).unwrap()).unwrap();
    assert_eq!(element.kind, ElementKind::Resistor { ohms: 470.0 });
    assert_eq!(
        circuit.placement_of(decode(&id).unwrap()),
        Some(Placement {
            x: 40.0,
            y: -20.0,
            turns: 0
        })
    );
}

#[test]
fn refuses_a_kind_it_does_not_know() {
    let mut circuit = Circuit::new();

    assert!(circuit.try_add("Flux capacitor", 0.0, 0.0).is_err());
    assert_eq!(circuit.size(), 0);
}

#[test]
fn removing_forgets_the_position_too() {
    let mut circuit = Circuit::new();
    let id = circuit.try_add("Led", 0.0, 0.0).unwrap();

    circuit.try_remove(&id).unwrap();

    assert_eq!(circuit.size(), 0);
    assert_eq!(circuit.placement_of(decode(&id).unwrap()), None);
}

#[test]
fn a_handle_does_not_outlive_its_element() {
    let mut circuit = Circuit::new();
    let id = circuit.try_add("Resistor", 0.0, 0.0).unwrap();
    circuit.try_remove(&id).unwrap();

    // The generational key pays off across the boundary as well: a stale
    // handle from the UI is an error, not a hit on whatever took the slot.
    assert!(circuit.try_remove(&id).is_err());
    circuit.try_add("Led", 10.0, 10.0).unwrap();
    assert!(circuit.try_remove(&id).is_err());
}

#[test]
fn rejects_a_handle_that_was_never_one() {
    let mut circuit = Circuit::new();

    assert_eq!(
        circuit.try_remove("banana"),
        Err(CircuitError::BadHandle("banana".to_string()))
    );
}

#[test]
fn labels_values_the_way_they_are_spoken() {
    let mut circuit = Circuit::new();
    circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    circuit.try_add("Resistor", 0.0, 0.0).unwrap();

    let labels: Vec<String> = circuit
        .netlist()
        .elements()
        .map(|(_, element)| label_for(&element.kind))
        .collect();

    // "9 V", not "9.00 V"; "470 Ω", not "470.00 Ω".
    assert_eq!(labels, vec!["9 V".to_string(), "470 Ω".to_string()]);
}

#[test]
fn scales_resistor_labels_to_the_unit_people_use() {
    assert_eq!(label_for(&ElementKind::Resistor { ohms: 470.0 }), "470 Ω");
    assert_eq!(
        label_for(&ElementKind::Resistor { ohms: 4_700.0 }),
        "4.7 kΩ"
    );
    assert_eq!(
        label_for(&ElementKind::Resistor { ohms: 1_000_000.0 }),
        "1 MΩ"
    );
}

#[test]
fn wiring_two_loose_pins_makes_one_net() {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 40.0, 0.0).unwrap();

    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();

    assert_eq!(circuit.net_count(), 1);
}

#[test]
fn wiring_onto_an_existing_net_joins_it_rather_than_starting_another() {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let first = circuit.try_add("Resistor", 40.0, 0.0).unwrap();
    let second = circuit.try_add("Resistor", 80.0, 0.0).unwrap();

    let net = circuit.try_connect(&battery, "+", &first, "a").unwrap();
    let same = circuit.try_connect(&battery, "+", &second, "a").unwrap();

    assert_eq!(net, same);
    assert_eq!(circuit.net_count(), 1);
}

#[test]
fn wiring_two_nets_together_merges_them() {
    // Two half-built circuits meeting. Two nets that merely touch would
    // solve as two separate circuits, which is the bug this prevents.
    let mut circuit = Circuit::new();
    let a = circuit.try_add("Resistor", 0.0, 0.0).unwrap();
    let b = circuit.try_add("Resistor", 40.0, 0.0).unwrap();
    let c = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
    let d = circuit.try_add("Resistor", 120.0, 0.0).unwrap();

    circuit.try_connect(&a, "b", &b, "a").unwrap();
    circuit.try_connect(&c, "b", &d, "a").unwrap();
    assert_eq!(circuit.net_count(), 2);

    // Both pins are already on nets, so this is a merge rather than a new
    // net — wiring two loose pins of the same elements would not be.
    circuit.try_connect(&b, "a", &c, "b").unwrap();

    assert_eq!(circuit.net_count(), 1);
    let pins = &circuit.netlist().nets().next().unwrap().1.pins().len();
    assert_eq!(*pins, 4);
}

/// A battery and a resistor with one wire between them.
fn wired_pair() -> (Circuit, String, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
    let net = circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    (circuit, battery, net)
}

fn bend(circuit: &Circuit, net: &str) -> Vec<PointView> {
    circuit.waypoints_of(decode_net(net).unwrap()).to_vec()
}

const CORNER: PointView = PointView { x: 40.0, y: -60.0 };

#[test]
fn a_wire_starts_with_no_bends_in_it() {
    let (circuit, _, net) = wired_pair();

    assert!(bend(&circuit, &net).is_empty());
}

#[test]
fn a_wire_remembers_where_it_was_bent() {
    let (mut circuit, _, net) = wired_pair();

    circuit.try_set_waypoints(&net, vec![CORNER]).unwrap();

    assert_eq!(bend(&circuit, &net), vec![CORNER]);
}

#[test]
fn straightening_a_wire_leaves_nothing_behind() {
    // An empty list means "straighten it". Storing one would leave an entry
    // that says nothing, and C-04 would save it.
    let (mut circuit, _, net) = wired_pair();
    circuit.try_set_waypoints(&net, vec![CORNER]).unwrap();

    circuit.try_set_waypoints(&net, Vec::new()).unwrap();

    assert!(circuit.wires.is_empty());
}

#[test]
fn bending_a_wire_can_be_undone() {
    let (mut circuit, _, net) = wired_pair();
    circuit.try_set_waypoints(&net, vec![CORNER]).unwrap();

    assert!(circuit.undo());

    assert!(bend(&circuit, &net).is_empty());
}

#[test]
fn a_bend_survives_the_components_moving() {
    // The whole asymmetry of ADR 0007: the interior is the user's, the two
    // runs to the pins are the app's.
    let (mut circuit, battery, net) = wired_pair();
    circuit.try_set_waypoints(&net, vec![CORNER]).unwrap();

    circuit.try_move(&battery, 0.0, 200.0).unwrap();

    assert_eq!(bend(&circuit, &net), vec![CORNER]);
}

#[test]
fn a_bend_survives_a_component_turning() {
    let (mut circuit, battery, net) = wired_pair();
    circuit.try_set_waypoints(&net, vec![CORNER]).unwrap();

    circuit.try_rotate(&battery).unwrap();

    assert_eq!(bend(&circuit, &net), vec![CORNER]);
}

#[test]
fn removing_a_wire_forgets_where_it_ran() {
    let (mut circuit, _, net) = wired_pair();
    circuit.try_set_waypoints(&net, vec![CORNER]).unwrap();

    circuit.try_remove_net(&net).unwrap();

    assert!(circuit.wires.is_empty());
}

#[test]
fn deleting_a_component_forgets_where_its_wires_ran() {
    let (mut circuit, battery, net) = wired_pair();
    circuit.try_set_waypoints(&net, vec![CORNER]).unwrap();

    circuit.try_remove(&battery).unwrap();

    assert!(
        circuit.wires.is_empty(),
        "a route with no wire left to draw"
    );
}

#[test]
fn bending_a_wire_that_is_not_there_is_refused() {
    let mut circuit = Circuit::new();

    assert!(
        circuit
            .try_set_waypoints("not a handle", Vec::new())
            .is_err()
    );
}

#[test]
fn bending_a_wire_does_not_disturb_the_circuit() {
    // Where a wire runs is the drawing, not the circuit.
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
    let net = circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &battery, "-").unwrap();
    circuit.advance(0.016).unwrap();
    let before = circuit.solution();

    circuit.try_set_waypoints(&net, vec![CORNER]).unwrap();
    circuit.advance(0.016).unwrap();

    let after = circuit.solution();
    assert!(after.failure.is_none());
    assert!(after.time > before.time, "the clock restarted");
}

#[test]
fn a_component_turns_a_quarter_at_a_time() {
    let mut circuit = Circuit::new();
    let resistor = circuit.try_add("Resistor", 40.0, 0.0).unwrap();

    assert_eq!(circuit.try_rotate(&resistor).unwrap(), 1);
    assert_eq!(circuit.try_rotate(&resistor).unwrap(), 2);
    assert_eq!(circuit.try_rotate(&resistor).unwrap(), 3);
}

#[test]
fn four_turns_bring_it_back() {
    // Which is the whole reason for counting quarters rather than storing
    // an angle: there is no way to end up at 359°.
    let mut circuit = Circuit::new();
    let resistor = circuit.try_add("Resistor", 40.0, 0.0).unwrap();

    for _ in 0..4 {
        circuit.try_rotate(&resistor).unwrap();
    }

    assert_eq!(
        circuit
            .placement_of(decode(&resistor).unwrap())
            .unwrap()
            .turns,
        0
    );
}

#[test]
fn turning_leaves_the_component_where_it_was() {
    let mut circuit = Circuit::new();
    let resistor = circuit.try_add("Resistor", 40.0, -20.0).unwrap();

    circuit.try_rotate(&resistor).unwrap();

    let at = circuit.placement_of(decode(&resistor).unwrap()).unwrap();
    assert_eq!((at.x, at.y), (40.0, -20.0));
}

#[test]
fn moving_leaves_the_component_facing_where_it_was() {
    // A drag moves a component; it does not straighten it.
    let mut circuit = Circuit::new();
    let resistor = circuit.try_add("Resistor", 40.0, 0.0).unwrap();
    circuit.try_rotate(&resistor).unwrap();

    circuit.try_move(&resistor, 200.0, 160.0).unwrap();

    assert_eq!(
        circuit
            .placement_of(decode(&resistor).unwrap())
            .unwrap()
            .turns,
        1
    );
}

#[test]
fn turning_can_be_undone() {
    let mut circuit = Circuit::new();
    let resistor = circuit.try_add("Resistor", 40.0, 0.0).unwrap();
    circuit.try_rotate(&resistor).unwrap();

    assert!(circuit.undo());

    assert_eq!(
        circuit
            .placement_of(decode(&resistor).unwrap())
            .unwrap()
            .turns,
        0
    );
}

#[test]
fn turning_does_not_disturb_the_circuit() {
    // Which way a component faces changes the drawing, not the solve. A
    // working circuit keeps working, and keeps its simulated time.
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 40.0, 0.0).unwrap();
    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &battery, "-").unwrap();
    circuit.advance(0.016).unwrap();
    let before = circuit.solution();

    circuit.try_rotate(&resistor).unwrap();
    circuit.advance(0.016).unwrap();

    let after = circuit.solution();
    assert!(after.failure.is_none());
    assert!(after.time > before.time, "the clock did not restart");
    assert_eq!(after.element_currents.len(), before.element_currents.len());
}

#[test]
fn turning_something_that_is_not_there_is_refused() {
    let mut circuit = Circuit::new();

    assert!(circuit.try_rotate("not a handle").is_err());
}

#[test]
fn an_open_circuit_solves_to_nothing_happening() {
    // Cutting one wire of a working circuit is the fastest way to reach
    // this (C-41), and the answer has to be "nothing flows", not an error
    // about a matrix.
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 40.0, 0.0).unwrap();
    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    let net = circuit.try_connect(&resistor, "b", &battery, "-").unwrap();
    circuit.advance(0.016).unwrap();
    assert!(circuit.solution().failure.is_none());

    circuit.try_remove_net(&net).unwrap();
    circuit.advance(0.016).unwrap();

    let state = circuit.solution();
    assert!(state.failure.is_none(), "{:?}", state.failure);
    assert!(
        state
            .element_currents
            .iter()
            .all(|current| current.amps.abs() < 1e-12),
        "an open circuit carries no current"
    );
}

#[test]
fn removing_a_wire_leaves_what_it_joined() {
    // The whole point of C-41: until now the only way to be rid of a wire
    // was to delete a component that was perfectly fine.
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 40.0, 0.0).unwrap();
    let net = circuit.try_connect(&battery, "+", &resistor, "a").unwrap();

    circuit.try_remove_net(&net).unwrap();

    assert_eq!(circuit.net_count(), 0);
    assert_eq!(circuit.size(), 2);
}

#[test]
fn removing_a_wire_frees_its_pins_to_be_wired_again() {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let first = circuit.try_add("Resistor", 40.0, 0.0).unwrap();
    let second = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
    let net = circuit.try_connect(&battery, "+", &first, "a").unwrap();

    circuit.try_remove_net(&net).unwrap();
    circuit.try_connect(&battery, "+", &second, "a").unwrap();

    assert_eq!(circuit.net_count(), 1);
}

#[test]
fn removing_a_wire_takes_every_pin_on_it() {
    // A net joining three pins is one wire, not three: the netlist never
    // knew which drag joined which pair, so the net is the unit a user can
    // delete. Undo is what makes that safe to get wrong.
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let first = circuit.try_add("Resistor", 40.0, 0.0).unwrap();
    let second = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
    circuit.try_connect(&battery, "+", &first, "a").unwrap();
    let net = circuit.try_connect(&battery, "+", &second, "a").unwrap();

    circuit.try_remove_net(&net).unwrap();

    assert_eq!(circuit.net_count(), 0);
    assert_eq!(circuit.size(), 3);
}

#[test]
fn removing_a_wire_can_be_undone() {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 40.0, 0.0).unwrap();
    let net = circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit.try_remove_net(&net).unwrap();

    assert!(circuit.undo());

    assert_eq!(circuit.net_count(), 1);
}

#[test]
fn removing_a_wire_that_is_not_there_is_refused() {
    let mut circuit = Circuit::new();

    assert!(circuit.try_remove_net("not a handle").is_err());
}

#[test]
fn a_pin_cannot_be_wired_to_itself() {
    let mut circuit = Circuit::new();
    let resistor = circuit.try_add("Resistor", 0.0, 0.0).unwrap();

    assert_eq!(
        circuit.try_connect(&resistor, "a", &resistor, "a"),
        Err(CircuitError::SamePin)
    );
    assert_eq!(circuit.net_count(), 0);
}

#[test]
fn both_ends_of_one_element_can_share_a_net() {
    // Electrically it is a short, not a mistake the editor should refuse.
    // The linter is what says something about it (C-05).
    let mut circuit = Circuit::new();
    let resistor = circuit.try_add("Resistor", 0.0, 0.0).unwrap();

    assert!(circuit.try_connect(&resistor, "a", &resistor, "b").is_ok());
    assert_eq!(circuit.net_count(), 1);
}

#[test]
fn wiring_a_pin_that_does_not_exist_is_refused() {
    let mut circuit = Circuit::new();
    let led = circuit.try_add("Led", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 40.0, 0.0).unwrap();

    assert!(circuit.try_connect(&led, "gate", &resistor, "a").is_err());
    assert_eq!(circuit.net_count(), 0);
}

#[test]
fn the_milestone_circuit_can_be_wired_end_to_end() {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 40.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 80.0, 0.0).unwrap();

    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
    circuit.try_connect(&led, "cathode", &battery, "-").unwrap();

    assert_eq!(circuit.net_count(), 3);
    // And it solves: the loop is closed and every pin is on a net.
    let point = crate::solver::operating_point(circuit.netlist()).unwrap();
    assert!(point.node_voltages().iter().any(|volts| *volts > 1.5));
}

#[test]
fn a_value_edit_changes_the_component() {
    let mut circuit = Circuit::new();
    let resistor = circuit.try_add("Resistor", 0.0, 0.0).unwrap();

    circuit.try_set_value(&resistor, "4k7").unwrap();

    let element = circuit
        .netlist()
        .element(decode(&resistor).unwrap())
        .unwrap();
    assert_eq!(element.kind, ElementKind::Resistor { ohms: 4_700.0 });
}

#[test]
fn a_rejected_edit_leaves_the_circuit_exactly_as_it_was() {
    // The acceptance: invalid input does not break the running simulation,
    // and the only way to promise that is to not apply it at all.
    let mut circuit = Circuit::new();
    let resistor = circuit.try_add("Resistor", 0.0, 0.0).unwrap();

    for bad in ["", "abc", "4..7k", "220 bananas"] {
        assert!(circuit.try_set_value(&resistor, bad).is_err(), "{bad:?}");
        let element = circuit
            .netlist()
            .element(decode(&resistor).unwrap())
            .unwrap();
        assert_eq!(
            element.kind,
            ElementKind::Resistor { ohms: 470.0 },
            "{bad:?}"
        );
    }
}

#[test]
fn a_component_without_a_numeric_value_says_so() {
    let mut circuit = Circuit::new();
    let ground = circuit.try_add("Ground", 0.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 0.0, 0.0).unwrap();

    assert_eq!(
        circuit.try_set_value(&ground, "5"),
        Err(CircuitError::NotEditable("Ground".to_string()))
    );
    assert_eq!(
        circuit.try_set_value(&led, "5"),
        Err(CircuitError::NotEditable("Led".to_string()))
    );
}

#[test]
fn the_value_offered_for_editing_reads_back_as_itself() {
    // What the inspector shows has to be something the parser accepts, or
    // pressing Enter without typing would reject the existing value.
    let mut circuit = Circuit::new();
    let resistor = circuit.try_add("Resistor", 0.0, 0.0).unwrap();

    for written in ["4k7", "1M", "0.47", "220"] {
        circuit.try_set_value(&resistor, written).unwrap();
        let shown = value_of(
            &circuit
                .netlist()
                .element(decode(&resistor).unwrap())
                .unwrap()
                .kind,
        )
        .map(crate::value::format_editable)
        .unwrap();

        assert!(
            circuit.try_set_value(&resistor, &shown).is_ok(),
            "{written} showed as {shown:?}, which does not parse"
        );
    }
}

#[test]
fn editing_a_value_does_not_disturb_the_wiring() {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 40.0, 0.0).unwrap();
    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();

    circuit.try_set_value(&resistor, "1k").unwrap();

    assert_eq!(circuit.net_count(), 1);
    assert_eq!(circuit.netlist().nets().next().unwrap().1.pins().len(), 2);
}

#[test]
fn a_changed_value_changes_the_answer() {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 40.0, 0.0).unwrap();
    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &battery, "-").unwrap();

    let current_with = |circuit: &Circuit| {
        let point = crate::solver::operating_point(circuit.netlist()).unwrap();
        point.source_current(decode(&battery).unwrap()).unwrap()
    };

    let before = current_with(&circuit);
    circuit.try_set_value(&resistor, "1k").unwrap();
    let after = current_with(&circuit);

    // 470 Ω to 1 kΩ: the current should fall by roughly the same ratio.
    assert!((before / after - 1000.0 / 470.0).abs() < 1e-9);
}

/// The milestone circuit, wired.
fn milestone() -> (Circuit, String, String, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 40.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 80.0, 0.0).unwrap();

    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
    circuit.try_connect(&led, "cathode", &battery, "-").unwrap();

    (circuit, battery, resistor, led)
}

fn led_current(circuit: &Circuit, led: &str) -> f64 {
    let state = circuit.solution();
    state
        .element_currents
        .iter()
        .find(|current| current.element_id == led)
        .map(|current| current.amps)
        .unwrap_or_else(|| panic!("no current for {led}"))
}

#[test]
fn the_circuit_solves_without_being_told_to_start() {
    // No run button: the first advance finds the circuit already there and
    // solves it.
    let (mut circuit, _, _, led) = milestone();

    circuit.advance(0.016).unwrap();

    assert!((led_current(&circuit, &led) - 1.534_827_142_501_358_5e-2).abs() < 1e-9);
}

#[test]
fn a_value_edit_is_reflected_without_an_explicit_run() {
    // The acceptance. Change the resistor, advance a frame, the LED dims.
    let (mut circuit, _, resistor, led) = milestone();
    circuit.advance(0.016).unwrap();
    let bright = led_current(&circuit, &led);

    circuit.try_set_value(&resistor, "2k2").unwrap();
    circuit.advance(0.016).unwrap();

    let dim = led_current(&circuit, &led);
    assert!(dim < bright / 4.0, "{dim} vs {bright}");
    assert!((dim - 3.314_966_730_637_971_5e-3).abs() < 1e-9, "{dim}");
}

#[test]
fn an_edit_between_frames_never_leaves_a_half_solved_circuit() {
    // Editing while solving: every advance sees a whole circuit, because an
    // edit only lands between them.
    let (mut circuit, _, resistor, led) = milestone();

    for ohms in ["220", "470", "1k", "2k2", "330"] {
        circuit.try_set_value(&resistor, ohms).unwrap();
        circuit.advance(0.016).unwrap();
        let current = led_current(&circuit, &led);
        assert!(current > 0.0 && current.is_finite(), "{ohms}: {current}");
    }
}

#[test]
fn a_half_built_circuit_reports_instead_of_stopping() {
    // A single component on the canvas is not a circuit. The loop keeps
    // running and says why, rather than throwing.
    let mut circuit = Circuit::new();
    circuit.try_add("Resistor", 0.0, 0.0).unwrap();

    circuit.advance(0.016).unwrap();

    let state = circuit.solution();
    assert!(state.failure.is_some());
    assert!(state.net_voltages.is_empty());

    // And it recovers on its own once the circuit makes sense.
    let (mut whole, _, _, led) = milestone();
    whole.advance(0.016).unwrap();
    assert!(led_current(&whole, &led) > 0.0);
}

#[test]
fn a_circuit_being_built_starts_solving_the_moment_it_can() {
    // How the app is actually used: parts appear one at a time and the
    // wires come last. The loop runs throughout, failing quietly until the
    // circuit is a circuit, and then simply working — no restart, no
    // button.
    let mut circuit = Circuit::new();

    let resistor = circuit.try_add("Resistor", 40.0, 0.0).unwrap();
    circuit.advance(0.016).unwrap();
    assert!(
        circuit.solution().failure.is_some(),
        "a lone resistor is not a circuit"
    );

    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    circuit.advance(0.016).unwrap();
    assert!(
        circuit.solution().failure.is_some(),
        "still nothing is wired"
    );

    // An open circuit is a circuit. Wired at one end only, nothing
    // conducts and every current is zero — which is the true answer, and
    // the one the solver used to refuse to give: the source got a branch
    // unknown that nothing stamped, and the user was shown "matrix is
    // singular at column 0" for having wired one thing so far.
    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit.advance(0.016).unwrap();
    let half = circuit.solution();
    assert!(half.failure.is_none(), "{:?}", half.failure);
    assert!(
        half.element_currents
            .iter()
            .all(|current| current.amps.abs() < 1e-12),
        "nothing conducts through an open circuit"
    );

    circuit.try_connect(&resistor, "b", &battery, "-").unwrap();
    circuit.advance(0.016).unwrap();

    let state = circuit.solution();
    assert!(state.failure.is_none(), "{:?}", state.failure);
    let current = state
        .element_currents
        .iter()
        .find(|current| current.element_id == battery)
        .unwrap();
    // Negative: the current leaves by the `+` terminal rather than entering
    // it, which is what a source does.
    assert!(
        (current.amps + 9.0 / 470.0).abs() < 1e-9,
        "{}",
        current.amps
    );
}

#[test]
fn a_dangling_lead_does_not_stop_the_circuit_solving() {
    // A half-placed component is normal while building. The node it leaves
    // floating settles at the potential it is tied to rather than breaking
    // the solve — flagging it is the linter's job (C-05), not the solver's.
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 40.0, 0.0).unwrap();
    let spare = circuit.try_add("Resistor", 80.0, 0.0).unwrap();

    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &battery, "-").unwrap();
    circuit.try_connect(&spare, "a", &resistor, "a").unwrap();

    circuit.advance(0.016).unwrap();

    assert!(circuit.solution().failure.is_none());
}

#[test]
fn an_edit_does_not_rewind_the_clock() {
    // The engine is rebuilt on every edit and starts counting again; the
    // circuit's own clock does not. Invisible in DC, and wrong the moment
    // C-09 makes time mean something.
    let (mut circuit, _, resistor, _) = milestone();
    for _ in 0..10 {
        circuit.advance(0.016).unwrap();
    }
    let before = circuit.solution().time;

    circuit.try_set_value(&resistor, "1k").unwrap();
    circuit.advance(0.016).unwrap();

    assert!(circuit.solution().time > before, "the clock went backwards");
}

#[test]
fn simulated_time_advances_with_the_frames() {
    let (mut circuit, _, _, _) = milestone();

    for _ in 0..10 {
        circuit.advance(0.016).unwrap();
    }

    // 160 ms of wall time at the default rate.
    assert!((circuit.solution().time - 0.16).abs() < 1e-3);
}

/// A battery straight across a resistor: 9 V over 220 Ω, some 0.37 W in
/// a quarter-watt part.
fn overloaded() -> (Circuit, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
    // 220 Ω: 0.37 W, past a quarter watt.
    circuit.try_set_value(&resistor, "220").unwrap();
    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &battery, "-").unwrap();
    (circuit, resistor)
}

fn heat_of(circuit: &Circuit, element: &str) -> ElementHeat {
    circuit
        .solution()
        .heat
        .into_iter()
        .find(|heat| heat.element_id == element)
        .expect("a resistor has a temperature")
}

/// Runs the live loop for `seconds`, a quarter of a second at a time.
fn warm_for(circuit: &mut Circuit, seconds: f64) {
    for _ in 0..(seconds / 0.25).round() as usize {
        circuit.advance(0.25).unwrap();
    }
}

#[test]
fn a_resistor_warms_along_its_packages_curve() {
    // C-74, against the closed form: after one time constant, 63 % of the
    // way from the room to room + P·θja.
    let (mut circuit, resistor) = overloaded();
    let thermal = crate::registry::rating_of(&ElementKind::Resistor { ohms: 220.0 })
        .thermal
        .unwrap();
    warm_for(&mut circuit, thermal.time_constant);

    let heat = heat_of(&circuit, &resistor);
    let watts = 81.0 / 220.0;
    assert!((heat.watts - watts).abs() < 1e-6, "{} W", heat.watts);
    let target = 25.0 + watts * thermal.junction_to_ambient;
    assert!((heat.settles_at - target).abs() < 1e-6);
    let expected = 25.0 + (target - 25.0) * (1.0 - (-1.0f64).exp());
    assert!(
        (heat.celsius - expected).abs() < 0.01 * expected,
        "{} °C after one time constant, expected {expected} °C",
        heat.celsius
    );
    // And it knows what it is rated for: 0.37 W is past a quarter watt.
    assert_eq!(heat.rated_watts, Some(0.25));
    assert_eq!(heat.max_celsius, 155.0);
}

#[test]
fn a_part_starts_at_the_rooms_temperature() {
    let (mut circuit, resistor) = overloaded();
    circuit.try_set_room("40").unwrap();
    circuit.advance(0.001).unwrap();

    let heat = heat_of(&circuit, &resistor);
    assert!((heat.celsius - 40.0).abs() < 0.1, "{} °C", heat.celsius);
}

#[test]
fn a_part_keeps_its_heat_through_an_edit() {
    // As a capacitor keeps its charge: a resistor does not cool because
    // something else on the canvas changed.
    let (mut circuit, resistor) = overloaded();
    warm_for(&mut circuit, 10.0);
    let before = heat_of(&circuit, &resistor).celsius;

    circuit.try_add("Led", 200.0, 0.0).unwrap();
    circuit.advance(0.016).unwrap();

    let after = heat_of(&circuit, &resistor).celsius;
    assert!(before > 60.0, "{before} °C after ten seconds");
    assert!(
        (after - before).abs() < 1.0,
        "{before} °C became {after} °C"
    );
}

#[test]
fn a_part_that_is_not_packaged_has_no_temperature() {
    let (mut circuit, _, _, led) = milestone();
    circuit.advance(0.016).unwrap();

    let solution = circuit.solution();
    assert!(solution.heat.iter().all(|heat| heat.element_id != led));
    assert_eq!(solution.heat.len(), 1, "the resistor, and only it");
}

#[test]
fn the_room_is_an_edit_like_any_other() {
    let mut circuit = Circuit::new();
    assert_eq!(circuit.room_celsius(), 25.0);

    circuit.try_set_room("31.5 °C").unwrap();
    assert_eq!(circuit.room_celsius(), 31.5);

    assert!(circuit.undo());
    assert_eq!(circuit.room_celsius(), 25.0);
}

#[test]
fn a_room_no_circuit_is_built_in_is_refused() {
    let mut circuit = Circuit::new();

    assert!(matches!(
        circuit.try_set_room("300"),
        Err(CircuitError::RoomOutOfRange(_))
    ));
    assert!(matches!(
        circuit.try_set_room("warm"),
        Err(CircuitError::Value(_))
    ));
    assert_eq!(circuit.room_celsius(), 25.0);
}

/// A battery, a resistor and a diode in a loop, the diode either way round.
fn rectifier(forwards: bool) -> (Circuit, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
    let diode = circuit.try_add("Diode", 160.0, 0.0).unwrap();

    let (first, second) = if forwards {
        ("anode", "cathode")
    } else {
        ("cathode", "anode")
    };
    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &diode, first).unwrap();
    circuit.try_connect(&diode, second, &battery, "-").unwrap();

    (circuit, diode)
}

fn current_through(circuit: &Circuit, element: &str) -> f64 {
    circuit
        .solution()
        .element_currents
        .iter()
        .find(|current| current.element_id == element)
        .map(|current| current.amps)
        .expect("a diode reports its current")
}

#[test]
fn a_diode_passes_current_one_way_and_not_the_other() {
    // The whole of what a diode is, and the only thing C-50 had to make
    // placeable: the model has been in the crate since M-10 and until now
    // could only be reached through an LED.
    let (mut forwards, diode) = rectifier(true);
    forwards.advance(0.016).unwrap();
    let (mut backwards, reversed) = rectifier(false);
    backwards.advance(0.016).unwrap();

    // 9 V through 470 Ω, less the drop across the junction: some
    // eighteen milliamps forwards.
    let passing = current_through(&forwards, &diode);
    assert!(passing > 0.015 && passing < 0.02, "{passing} A forwards");

    // Backwards it is the saturation current, which is picoamps — ten
    // orders of magnitude down, and zero for anyone looking at it.
    let blocked = current_through(&backwards, &reversed);
    assert!(blocked.abs() < 1e-9, "{blocked} A backwards");
}

#[test]
fn a_diode_drops_about_what_silicon_drops() {
    // Not a specification, a sanity check: a small-signal silicon junction
    // sits near 0.7 V at tens of milliamps, and a model that said 0.2 V or
    // 2 V would be teaching something untrue about the part.
    let (mut circuit, diode) = rectifier(true);
    circuit.advance(0.016).unwrap();
    let solution = circuit.solution();

    let volts_at = |net: NetId| {
        let id = encode_net(net);
        solution
            .net_voltages
            .iter()
            .find(|voltage| voltage.net_id == id)
            .map(|voltage| voltage.volts)
            .unwrap_or(0.0)
    };
    let element = circuit.netlist().element(decode(&diode).unwrap()).unwrap();
    let across = volts_at(element.pins[0].net.unwrap()) - volts_at(element.pins[1].net.unwrap());

    assert!((0.6..0.9).contains(&across), "{across} V across it");
}

/// A battery feeding a regulator, and the regulator feeding a resistor.
fn regulated(volts: &str) -> (Circuit, String, String, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let regulator = circuit.try_add("Regulator", 80.0, 0.0).unwrap();
    let load = circuit.try_add("Resistor", 160.0, 0.0).unwrap();
    circuit.try_set_value(&battery, volts).unwrap();

    circuit
        .try_connect(&battery, "+", &regulator, "in")
        .unwrap();
    circuit.try_connect(&regulator, "out", &load, "a").unwrap();
    circuit.try_connect(&load, "b", &regulator, "gnd").unwrap();
    circuit
        .try_connect(&regulator, "gnd", &battery, "-")
        .unwrap();

    (circuit, battery, regulator, load)
}

/// Volts across an element's first two pins, from the live solution.
fn across_pins(circuit: &Circuit, element: &str) -> f64 {
    let solution = circuit.solution();
    let volts_at = |net: NetId| {
        let id = encode_net(net);
        solution
            .net_voltages
            .iter()
            .find(|voltage| voltage.net_id == id)
            .map(|voltage| voltage.volts)
            .unwrap_or(0.0)
    };
    let element = circuit.netlist().element(decode(element).unwrap()).unwrap();
    volts_at(element.pins[0].net.unwrap()) - volts_at(element.pins[1].net.unwrap())
}

#[test]
fn a_regulator_holds_five_volts_off_a_nine_volt_battery() {
    // The whole of what a 7805 is for (C-54). Within a millivolt: the
    // load draws 23 mA, and ten milliohms of output resistance is a
    // quarter of a millivolt of that.
    let (mut circuit, _, _, load) = regulated("9");
    circuit.advance(0.016).unwrap();

    assert!(circuit.solution().failure.is_none());
    let out = across_pins(&circuit, &load);
    assert!((out - 5.0).abs() < 1e-3, "{out} V out");
}

#[test]
fn a_regulator_does_not_care_how_much_it_is_fed() {
    // Twelve volts in or seven, the same five out: that is the difference
    // between a regulator and a resistor divider.
    for volts in ["7", "12", "24"] {
        let (mut circuit, _, _, load) = regulated(volts);
        circuit.advance(0.016).unwrap();
        let out = across_pins(&circuit, &load);
        assert!((out - 5.0).abs() < 1e-3, "{out} V out from {volts} V");
    }
}

#[test]
fn fed_too_little_a_regulator_follows_its_input_down() {
    // 6 V in and two volts of dropout: the most a 7805 can make is 4 V,
    // which is what it makes. It does not invent the fifth.
    let (mut circuit, _, _, load) = regulated("6");
    circuit.advance(0.016).unwrap();
    let out = across_pins(&circuit, &load);
    assert!((out - 4.0).abs() < 1e-2, "{out} V out from 6 V");
}

#[test]
fn the_battery_pays_for_the_load_and_the_ground_pin_for_the_rest() {
    // What comes out of the battery is what the load takes plus what the
    // regulator burns to run itself, a few milliamps down its ground pin.
    let (mut circuit, battery, _, load) = regulated("9");
    circuit.advance(0.016).unwrap();

    // Every element's current is reported into its first pin, so a battery
    // that is delivering reads negative.
    let drawn = -current_through(&circuit, &battery);
    let delivered = current_through(&circuit, &load);
    let quiescent = drawn - delivered;
    assert!(
        (4e-3..6e-3).contains(&quiescent),
        "{drawn} A drawn, {delivered} A delivered, {quiescent} A to ground"
    );
}

#[test]
fn a_regulator_is_labelled_with_the_part_it_is() {
    let (mut circuit, _, regulator, _) = regulated("9");
    let label = |circuit: &Circuit| {
        circuit
            .views()
            .into_iter()
            .find(|view| view.id == regulator)
            .map(|view| view.label)
            .unwrap()
    };
    assert_eq!(label(&circuit), "7805");

    // Chosen as another part, it is that part — label, voltage and all.
    circuit.try_set_choice(&regulator, "LM1117-3.3").unwrap();
    assert_eq!(label(&circuit), "LM1117-3.3");
}

/// A battery and an op-amp wired as a follower of a divider's midpoint,
/// with its supply pins joined only if asked.
fn follower(powered: bool) -> (Circuit, String, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let top = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
    let bottom = circuit.try_add("Resistor", 80.0, 80.0).unwrap();
    let amp = circuit.try_add("OpAmp", 160.0, 40.0).unwrap();
    let load = circuit.try_add("Resistor", 240.0, 40.0).unwrap();

    circuit.try_connect(&battery, "+", &top, "a").unwrap();
    circuit.try_connect(&top, "b", &bottom, "a").unwrap();
    circuit.try_connect(&bottom, "b", &battery, "-").unwrap();
    circuit.try_connect(&top, "b", &amp, "in+").unwrap();
    circuit.try_connect(&amp, "out", &amp, "in-").unwrap();
    circuit.try_connect(&amp, "out", &load, "a").unwrap();
    circuit.try_connect(&load, "b", &battery, "-").unwrap();
    if powered {
        circuit.try_connect(&amp, "V+", &battery, "+").unwrap();
        circuit.try_connect(&amp, "V-", &battery, "-").unwrap();
    }
    (circuit, amp, load)
}

#[test]
fn an_op_amp_follows_half_the_battery_and_says_nothing() {
    let (mut circuit, _, load) = follower(true);
    circuit.advance(0.016).unwrap();

    let solution = circuit.solution();
    assert!(solution.failure.is_none());
    assert!(solution.findings.is_empty(), "{:?}", solution.findings);
    let out = across_pins(&circuit, &load);
    assert!((out - 4.5).abs() < 1e-3, "{out} V out");
}

#[test]
fn an_op_amp_with_no_supply_says_so_rather_than_amplifying() {
    // It must not solve as though it were powered: with nothing on V+ and
    // V− the output drives nothing, and the one line that says why is the
    // finding pointing at the part.
    let (mut circuit, amp, load) = follower(false);
    circuit.advance(0.016).unwrap();

    let out = across_pins(&circuit, &load);
    assert!(out.abs() < 0.1, "an unpowered op-amp put {out} V out");
    let findings = circuit.solution().findings;
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].elements, vec![amp]);
    assert!(findings[0].message.contains("no power"));
    // Said by the engine's rule, by name, with no fix to offer (B-01).
    assert_eq!(findings[0].rule, "unpowered");
    assert_eq!(findings[0].severity, "warning");
    assert_eq!(findings[0].fix, None);
}

#[test]
fn an_op_amp_is_labelled_with_the_part_it_is() {
    let (circuit, amp, _) = follower(true);
    let view = circuit
        .views()
        .into_iter()
        .find(|view| view.id == amp)
        .unwrap();
    assert_eq!(view.label, "LM358");
}

/// A threshold detector: a knob against a divider, the comparator pulling
/// an LED's cathode down while the knob is below it (C-56).
///
/// The divider is 10k over 12k, so the threshold is 4.91 V rather than
/// half the battery. Half would be exactly where the knob sits at half a
/// turn, and with both inputs at the same voltage a comparator is in its
/// linear region — a real one too — so a sweep in hundredths would stop
/// on the one position a real knob never lands on: the linear region is
/// 45 µV wide, five millionths of a turn.
fn threshold() -> (Circuit, String, String, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let knob = circuit.try_add("Potentiometer", 80.0, 0.0).unwrap();
    let top = circuit.try_add("Resistor", 80.0, 80.0).unwrap();
    let bottom = circuit.try_add("Resistor", 80.0, 160.0).unwrap();
    let comparator = circuit.try_add("Comparator", 160.0, 80.0).unwrap();
    let pull_up = circuit.try_add("Resistor", 240.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 240.0, 80.0).unwrap();
    circuit.try_set_value(&top, "10k").unwrap();
    circuit.try_set_value(&bottom, "12k").unwrap();
    circuit.try_set_value(&pull_up, "1k").unwrap();

    // The knob across the battery, its wiper on the non-inverting input.
    circuit.try_connect(&battery, "+", &knob, "a").unwrap();
    circuit.try_connect(&knob, "b", &battery, "-").unwrap();
    circuit
        .try_connect(&knob, "wiper", &comparator, "in+")
        .unwrap();
    // The threshold on the inverting input.
    circuit.try_connect(&battery, "+", &top, "a").unwrap();
    circuit.try_connect(&top, "b", &bottom, "a").unwrap();
    circuit.try_connect(&bottom, "b", &battery, "-").unwrap();
    circuit.try_connect(&top, "b", &comparator, "in-").unwrap();
    // The LED from the battery through its resistor into the output,
    // which is what pulls it down.
    circuit.try_connect(&battery, "+", &pull_up, "a").unwrap();
    circuit.try_connect(&pull_up, "b", &led, "anode").unwrap();
    circuit
        .try_connect(&led, "cathode", &comparator, "out")
        .unwrap();
    // And its supply.
    circuit
        .try_connect(&comparator, "V+", &battery, "+")
        .unwrap();
    circuit
        .try_connect(&comparator, "V-", &battery, "-")
        .unwrap();

    (circuit, knob, comparator, led)
}

#[test]
fn a_comparator_switches_an_led_cleanly_as_a_knob_is_turned() {
    // C-56's acceptance. The whole knob, a hundredth of a turn at a time,
    // and at every position: a solve that did not fail, and an LED that is
    // either properly on or properly off — never somewhere between — with
    // exactly one change from one to the other.
    let (mut circuit, knob, _, led) = threshold();
    let mut states = Vec::new();
    for step in 0..=100 {
        let wiper = f64::from(step) / 100.0;
        circuit.try_set_wiper(&knob, wiper).unwrap();
        circuit.advance(0.016).unwrap();

        let solution = circuit.solution();
        assert!(
            solution.failure.is_none(),
            "failed at {wiper}: {:?}",
            solution.failure
        );
        let amps = current_through(&circuit, &led);
        let lit = amps > 5e-3;
        let dark = amps < 1e-5;
        assert!(
            lit || dark,
            "{amps} A through the LED at {wiper}: neither on nor off"
        );
        states.push((wiper, lit));
    }

    let changes: Vec<f64> = states
        .windows(2)
        .filter(|pair| pair[0].1 != pair[1].1)
        .map(|pair| pair[1].0)
        .collect();
    assert_eq!(changes.len(), 1, "switched at {changes:?}");
    // 4.91 V of 9 is 0.55 of the way along the knob, from whichever end
    // its wiper counts from.
    assert!(
        (0.44..=0.46).contains(&changes[0]) || (0.54..=0.56).contains(&changes[0]),
        "switched at {}",
        changes[0]
    );
}

#[test]
fn a_comparator_pulls_down_to_its_saturation_voltage() {
    // Sinking the LED's few milliamps, an LM393's output sits a few tenths
    // of a volt above V− — its datasheet's quarter volt at 4 mA — not at
    // V− itself, and not at the LED's forward drop.
    let (mut circuit, knob, comparator, _) = threshold();
    // Wherever the wiper's 0 is, one end of the knob is below the
    // threshold; find the end that lights the LED.
    circuit.try_set_wiper(&knob, 0.0).unwrap();
    circuit.advance(0.016).unwrap();
    let out = |circuit: &Circuit| {
        let element = circuit
            .netlist()
            .element(decode(&comparator).unwrap())
            .unwrap();
        let net = element.pins[2].net.unwrap();
        let id = encode_net(net);
        circuit
            .solution()
            .net_voltages
            .iter()
            .find(|voltage| voltage.net_id == id)
            .map(|voltage| voltage.volts)
            .unwrap()
    };
    let at_zero = out(&circuit);
    circuit.try_set_wiper(&knob, 1.0).unwrap();
    circuit.advance(0.016).unwrap();
    let at_one = out(&circuit);

    let pulled_down = at_zero.min(at_one);
    let let_go = at_zero.max(at_one);
    assert!(
        (0.1..0.5).contains(&pulled_down),
        "pulled down to {pulled_down} V"
    );
    // Let go, only the microamp it leaks flows through the LED, so its
    // cathode sits at the battery — within the millivolt of rounding the
    // rails carry (C-53), and never below the half its pull-down reached.
    assert!((4.5..=9.001).contains(&let_go), "let go at {let_go} V");
}

#[test]
fn an_led_on_a_signal_is_not_called_backwards_for_its_off_half() {
    // A square wave reverses every LED on it for half of each second. The
    // reversed-LED warning is about a battery wired the wrong way, and on
    // an alternating source there is no wrong way to have wired it.
    let mut circuit = Circuit::new();
    let generator = circuit.try_add("SignalGenerator", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 160.0, 0.0).unwrap();
    circuit
        .try_connect(&generator, "+", &resistor, "a")
        .unwrap();
    circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
    circuit
        .try_connect(&led, "cathode", &generator, "-")
        .unwrap();

    // Into the low half, where the LED is reversed — a frame at a time,
    // as the live loop does, because one long call is not the same thing:
    // the scheduler decides what a span of wall time is worth (C-13).
    for _ in 0..45 {
        circuit.advance(1.0 / 60.0).unwrap();
    }
    let solution = circuit.solution();
    assert!(solution.failure.is_none());
    assert!(
        current_through(&circuit, &led).abs() < 1e-5,
        "dark in the low half"
    );
    assert!(solution.findings.is_empty(), "{:?}", solution.findings);
}

#[test]
fn a_slow_signal_is_labelled_in_plain_hertz() {
    let mut circuit = Circuit::new();
    let generator = circuit.try_add("SignalGenerator", 0.0, 0.0).unwrap();
    let label = |circuit: &Circuit| {
        circuit
            .views()
            .into_iter()
            .find(|view| view.id == generator)
            .map(|view| view.label)
            .unwrap()
    };
    assert_eq!(label(&circuit), "±5 V 1 Hz");
    circuit.try_set_frequency(&generator, "0.5").unwrap();
    assert_eq!(label(&circuit), "±5 V 0.5 Hz", "not 500 mHz");
    circuit.try_set_frequency(&generator, "2k").unwrap();
    assert_eq!(label(&circuit), "±5 V 2 kHz");
    // And nothing that is not a frequency is taken as one.
    assert!(circuit.try_set_frequency(&generator, "0").is_err());
    assert!(circuit.try_set_frequency(&generator, "-1").is_err());
}

/// A 9 V relay switched by a push-button under its coil, a voltmeter
/// across the button, and — if asked — a diode across the coil pointing
/// back at the supply. Returns the circuit, the relay, the button and the
/// meter.
fn switched_coil(flyback: bool) -> (Circuit, String, String, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let relay = circuit.try_add("Relay", 80.0, 0.0).unwrap();
    let button = circuit.try_add("PushButton", 160.0, 0.0).unwrap();
    let meter = circuit.try_add("Voltmeter", 240.0, 0.0).unwrap();
    circuit.try_connect(&battery, "+", &relay, "coil+").unwrap();
    circuit.try_connect(&relay, "coil-", &button, "a").unwrap();
    circuit.try_connect(&button, "b", &battery, "-").unwrap();
    circuit.try_connect(&meter, "+", &button, "a").unwrap();
    circuit.try_connect(&meter, "-", &button, "b").unwrap();
    if flyback {
        let diode = circuit.try_add("Diode", 80.0, 80.0).unwrap();
        circuit
            .try_connect(&diode, "cathode", &relay, "coil+")
            .unwrap();
        circuit
            .try_connect(&diode, "anode", &relay, "coil-")
            .unwrap();
    }
    circuit.try_set_closed(&button, true).unwrap();
    (circuit, relay, button, meter)
}

/// The highest the meter read over the last frame, from every step the
/// run took rather than only the one it ended on.
fn peak(circuit: &mut Circuit, meter: &str) -> f64 {
    let written = circuit.plot_trace(meter, 1.0 / 60.0, 1).unwrap();
    assert_eq!(written, 2);
    let traces = circuit.simulation.as_ref().unwrap().engine().traces();
    traces.plotted()[1]
}

#[test]
fn a_coil_switched_off_kicks_and_a_diode_catches_it() {
    // Held for a tenth of a second, the coil carries its 45 mA and the
    // contact is thrown. Let go, and the coil's current has nowhere to go
    // but into the open button: the voltage across it jumps far past the
    // battery. With a diode across the coil the current goes round through
    // the diode instead, and the button never sees more than the battery
    // and a diode's drop (C-67).
    for flyback in [false, true] {
        let (mut circuit, relay, button, meter) = switched_coil(flyback);
        for _ in 0..6 {
            circuit.advance(1.0 / 60.0).unwrap();
        }
        assert!(circuit.solution().energised.contains(&relay));
        assert!(
            peak(&mut circuit, &meter) < 0.1,
            "shut, the button reads nothing"
        );

        circuit.try_set_closed(&button, false).unwrap();
        circuit.advance(1.0 / 60.0).unwrap();
        let solution = circuit.solution();
        assert!(solution.failure.is_none(), "{:?}", solution.failure);
        let kick = peak(&mut circuit, &meter);
        if flyback {
            assert!(
                kick < 9.0 + 1.0,
                "a diode's drop above the battery at most: {kick} V"
            );
        } else {
            // The kick itself is 36 V; the steps of a frame see most of it.
            assert!(kick > 2.0 * 9.0, "far past the battery: {kick} V");
            assert!(
                kick < 9.0 + 27.0 + 0.1,
                "no higher than the part sets: {kick} V"
            );
        }
        assert!(
            !solution.energised.contains(&relay),
            "let go within the frame either way"
        );
        // And it settles: the button reads the battery once the coil has
        // given up what it held.
        circuit.advance(1.0 / 60.0).unwrap();
        let settled = peak(&mut circuit, &meter);
        assert!((settled - 9.0).abs() < 0.05, "{settled} V");
    }
}

#[test]
fn a_coil_takes_a_moment_to_pull_in() {
    // Its current climbs with a time constant of 2 ms towards 45 mA, and
    // the contact is thrown at 75 % of that, after 2 · ln 4 ≈ 2.8 ms.
    // Asked for less than that, it has not moved; asked for a frame, it
    // has.
    let (mut circuit, relay, button, _) = switched_coil(false);
    // As long as a finger's press for the one the setup made, so the
    // release lets go at once (C-123); then a frame released.
    circuit.advance(0.05).unwrap();
    circuit.try_set_closed(&button, false).unwrap();
    circuit.advance(1.0 / 60.0).unwrap();
    circuit.try_set_closed(&button, true).unwrap();
    circuit.advance(1e-3).unwrap();
    assert!(!circuit.solution().energised.contains(&relay), "not yet");
    circuit.advance(1.0 / 60.0).unwrap();
    assert!(circuit.solution().energised.contains(&relay), "thrown");
}

/// A CD4511 on 9 V driving a digit through 330 Ω a segment, its four
/// inputs and its latch enable each a switch up to the + with a 10 kΩ
/// pull-down, and lamp test and blanking wired high, out of the way.
/// Returns the circuit, the latch's switch, the four inputs' (A first) and
/// the digit.
fn decoding() -> (Circuit, String, Vec<String>, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let chip = circuit.try_add("Decoder", 200.0, 0.0).unwrap();
    let digit = circuit.try_add("SevenSegment", 400.0, 0.0).unwrap();
    circuit.try_connect(&chip, "VDD", &battery, "+").unwrap();
    circuit.try_connect(&chip, "VSS", &battery, "-").unwrap();
    circuit.try_connect(&chip, "LT", &battery, "+").unwrap();
    circuit.try_connect(&chip, "BI", &battery, "+").unwrap();
    circuit.try_connect(&digit, "com", &battery, "-").unwrap();
    for segment in ["a", "b", "c", "d", "e", "f", "g"] {
        let resistor = circuit.try_add("Resistor", 300.0, 0.0).unwrap();
        circuit.try_set_value(&resistor, "330").unwrap();
        circuit.try_connect(&chip, segment, &resistor, "a").unwrap();
        circuit
            .try_connect(&resistor, "b", &digit, segment)
            .unwrap();
    }
    let switched = |circuit: &mut Circuit, input: &str| {
        let switch = circuit.try_add("Switch", 100.0, 0.0).unwrap();
        let pull = circuit.try_add("Resistor", 100.0, 80.0).unwrap();
        circuit.try_set_value(&pull, "10k").unwrap();
        circuit.try_connect(&battery, "+", &switch, "a").unwrap();
        circuit.try_connect(&switch, "b", &chip, input).unwrap();
        circuit.try_connect(&chip, input, &pull, "a").unwrap();
        circuit.try_connect(&pull, "b", &battery, "-").unwrap();
        switch
    };
    let latch = switched(&mut circuit, "LE");
    let switches = ["A", "B", "C", "D"]
        .iter()
        .map(|input| switched(&mut circuit, input))
        .collect();
    (circuit, latch, switches, digit)
}

/// Which segments of a digit are lit, as letters.
fn lit(circuit: &Circuit, digit: &str) -> String {
    glow(circuit, digit)
        .iter()
        .zip(["a", "b", "c", "d", "e", "f", "g", "."])
        .filter(|(brightness, _)| **brightness > 0.1)
        .map(|(_, letter)| letter)
        .collect()
}

/// Sets the four switches to a number and runs a frame.
fn show(circuit: &mut Circuit, switches: &[String], number: u8) {
    for (bit, switch) in switches.iter().enumerate() {
        circuit
            .try_set_closed(switch, number & (1 << bit) != 0)
            .unwrap();
    }
    circuit.advance(1.0 / 60.0).unwrap();
    assert!(
        circuit.solution().failure.is_none(),
        "{:?}",
        circuit.solution().failure
    );
}

#[test]
fn a_decoder_draws_the_number_its_switches_spell() {
    // The CD4511's own font, tail-less 6 and 9 included, and nothing past
    // nine (C-77).
    let (mut circuit, _, switches, digit) = decoding();
    let expected = [
        "abcdef", "bc", "abdeg", "abcdg", "bcfg", "acdfg", "cdefg", "abc", "abcdefg", "abcfg",
    ];
    for (number, segments) in expected.iter().enumerate() {
        show(&mut circuit, &switches, number as u8);
        assert_eq!(lit(&circuit, &digit), *segments, "{number}");
    }
    show(&mut circuit, &switches, 12);
    assert_eq!(lit(&circuit, &digit), "", "blank past nine");
}

#[test]
fn a_decoder_drives_a_segment_as_a_battery_through_its_resistor_would() {
    // A lit output is a base–emitter drop under VDD: 8.3 V into 330 Ω and
    // a red LED, about 19 mA — a segment well lit, and the resistor still
    // what keeps it alive.
    let (mut circuit, _, switches, digit) = decoding();
    show(&mut circuit, &switches, 8);
    let brightness = glow(&circuit, &digit)[0];
    assert!(brightness > 0.9, "{brightness}");
}

#[test]
fn a_decoder_holds_its_number_while_latched() {
    let (mut circuit, latch, switches, digit) = decoding();
    show(&mut circuit, &switches, 7);
    // Latch enable up: whatever the switches do now, it shows 7 — and
    // down again, it follows them.
    circuit.try_set_closed(&latch, true).unwrap();
    show(&mut circuit, &switches, 2);
    assert_eq!(lit(&circuit, &digit), "abc", "still 7");
    circuit.try_set_closed(&latch, false).unwrap();
    show(&mut circuit, &switches, 2);
    assert_eq!(lit(&circuit, &digit), "abdeg", "a 2 once let go");
}

#[test]
fn a_decoder_with_no_power_says_so() {
    let (mut circuit, _, _, _) = decoding();
    let chip = circuit
        .views()
        .into_iter()
        .find(|view| view.kind == "Decoder")
        .unwrap()
        .id;
    let vdd = circuit
        .netlist
        .element(decode(&chip).unwrap())
        .and_then(|element| element.pins[crate::solver::device::decoder::VDD].net)
        .unwrap();
    circuit.try_remove_net(&encode_net(vdd)).unwrap();
    circuit.advance(1.0 / 60.0).unwrap();
    let findings = circuit.solution().findings;
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("CD4511's VDD is not wired")),
        "{findings:?}"
    );
}

/// A CD4026 on 9 V driving a digit through 330 Ω a segment, its display
/// enabled and its reset and inhibit held low. Returns the counter and
/// the digit; its clock is the caller's to wire.
fn counting(circuit: &mut Circuit, battery: &str) -> (String, String) {
    let counter = circuit.try_add("Counter", 200.0, 0.0).unwrap();
    let digit = circuit.try_add("SevenSegment", 400.0, 0.0).unwrap();
    circuit.try_connect(&counter, "VDD", battery, "+").unwrap();
    circuit.try_connect(&counter, "VSS", battery, "-").unwrap();
    circuit.try_connect(&counter, "DEI", battery, "+").unwrap();
    circuit.try_connect(&counter, "RST", battery, "-").unwrap();
    circuit.try_connect(&counter, "INH", battery, "-").unwrap();
    circuit.try_connect(&digit, "com", battery, "-").unwrap();
    for segment in ["a", "b", "c", "d", "e", "f", "g"] {
        let resistor = circuit.try_add("Resistor", 300.0, 0.0).unwrap();
        circuit.try_set_value(&resistor, "330").unwrap();
        circuit
            .try_connect(&counter, segment, &resistor, "a")
            .unwrap();
        circuit
            .try_connect(&resistor, "b", &digit, segment)
            .unwrap();
    }
    (counter, digit)
}

/// A ±5 V square at 50 Hz against the battery's −: high past half the
/// supply, low below it, fifty rising edges a second.
fn clocked(circuit: &mut Circuit, battery: &str, into: &str) {
    let clock = circuit.try_add("SignalGenerator", 100.0, 0.0).unwrap();
    circuit.try_set_frequency(&clock, "50").unwrap();
    circuit.try_connect(&clock, "+", into, "CLK").unwrap();
    circuit.try_connect(&clock, "-", battery, "-").unwrap();
}

fn run_for(circuit: &mut Circuit, seconds: f64) {
    for _ in 0..(seconds * 60.0).round() as usize {
        circuit.advance(1.0 / 60.0).unwrap();
    }
    assert!(
        circuit.solution().failure.is_none(),
        "{:?}",
        circuit.solution().failure
    );
}

#[test]
fn a_counter_s_segment_carries_what_its_sheet_s_output_gives() {
    // At nought on 5 V, segment a high through 220 Ω into a red LED:
    // behind the sheet's 780 Ω about 3.2 mA, its own curve's 3.3 near
    // enough (C-122).
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    circuit.try_set_value(&battery, "5").unwrap();
    let counter = circuit.try_add("Counter", 200.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 300.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 400.0, 0.0).unwrap();
    circuit.try_set_value(&resistor, "220").unwrap();
    circuit.try_connect(&counter, "VDD", &battery, "+").unwrap();
    circuit.try_connect(&counter, "VSS", &battery, "-").unwrap();
    circuit.try_connect(&counter, "DEI", &battery, "+").unwrap();
    circuit.try_connect(&counter, "RST", &battery, "-").unwrap();
    circuit.try_connect(&counter, "INH", &battery, "-").unwrap();
    circuit.try_connect(&counter, "CLK", &battery, "-").unwrap();
    circuit.try_connect(&counter, "a", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
    circuit.try_connect(&led, "cathode", &battery, "-").unwrap();
    run_for(&mut circuit, 0.1);
    let amps = volts_on(&circuit, &resistor) / 220.0;
    assert!((amps - 3.2e-3).abs() < 0.3e-3, "{amps} A");
}

/// A CD4026 on 5 V clocked by a push-button to VDD, 10 kΩ holding CLK
/// down, driving a digit. Returns the circuit, the button and the digit.
fn counted_by_hand(bounces: bool) -> (Circuit, String, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    circuit.try_set_value(&battery, "5").unwrap();
    let (counter, digit) = counting(&mut circuit, &battery);
    let button = circuit.try_add("PushButton", 100.0, 0.0).unwrap();
    if bounces {
        circuit.try_set_choice(&button, "Bouncy").unwrap();
    }
    let down = circuit.try_add("Resistor", 100.0, 80.0).unwrap();
    circuit.try_set_value(&down, "10k").unwrap();
    circuit.try_connect(&button, "a", &battery, "+").unwrap();
    circuit.try_connect(&button, "b", &counter, "CLK").unwrap();
    circuit.try_connect(&down, "a", &counter, "CLK").unwrap();
    circuit.try_connect(&down, "b", &battery, "-").unwrap();
    run_for(&mut circuit, 0.1);
    (circuit, button, digit)
}

#[test]
fn on_9_v_a_counter_s_segment_is_lit_as_its_sheet_s_curves_say() {
    // The same on 9 V: behind the 311 Ω the sheet's curves give there —
    // four fifths of the way from 5 V's 780 to 10 V's 270, in
    // conductance — about 13 mA (C-125).
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    circuit.try_set_value(&battery, "9").unwrap();
    let counter = circuit.try_add("Counter", 200.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 300.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 400.0, 0.0).unwrap();
    circuit.try_set_value(&resistor, "220").unwrap();
    circuit.try_connect(&counter, "VDD", &battery, "+").unwrap();
    circuit.try_connect(&counter, "VSS", &battery, "-").unwrap();
    circuit.try_connect(&counter, "DEI", &battery, "+").unwrap();
    circuit.try_connect(&counter, "RST", &battery, "-").unwrap();
    circuit.try_connect(&counter, "INH", &battery, "-").unwrap();
    circuit.try_connect(&counter, "CLK", &battery, "-").unwrap();
    circuit.try_connect(&counter, "a", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
    circuit.try_connect(&led, "cathode", &battery, "-").unwrap();
    run_for(&mut circuit, 0.1);
    let amps = volts_on(&circuit, &resistor) / 220.0;
    assert!((amps - 13.4e-3).abs() < 0.6e-3, "{amps} A");
}

#[test]
fn a_tap_on_a_push_button_too_quick_for_a_frame_still_counts() {
    // Pressed and let go with no frame between, as a trackpad's tap is:
    // the release waits until the press has lasted a finger's shortest,
    // 50 ms, so steps see it and the CD4026 counts one (C-123).
    let (mut circuit, button, digit) = counted_by_hand(false);
    assert_eq!(lit(&circuit, &digit), "abcdef", "nought");
    circuit.try_set_closed(&button, true).unwrap();
    circuit.try_set_closed(&button, false).unwrap();
    run_for(&mut circuit, 1.0 / 60.0);
    assert_eq!(lit(&circuit, &digit), "bc", "one");
    let id = decode(&button).unwrap();
    let contact = |circuit: &Circuit| circuit.netlist.element(id).unwrap().kind.contact();
    assert_eq!(contact(&circuit), Some(true), "still held");
    run_for(&mut circuit, 3.0 / 60.0);
    assert_eq!(contact(&circuit), Some(false), "let go");
    assert_eq!(lit(&circuit, &digit), "bc", "and one only");
    // And a press held over frames, a moment later, is let go when it is
    // let go.
    run_for(&mut circuit, 0.1);
    circuit.try_set_closed(&button, true).unwrap();
    run_for(&mut circuit, 0.2);
    circuit.try_set_closed(&button, false).unwrap();
    assert_eq!(
        circuit.netlist.element(id).unwrap().kind.contact(),
        Some(false)
    );
    run_for(&mut circuit, 0.1);
    assert_eq!(lit(&circuit, &digit), "abdeg", "two");
}

#[test]
fn a_tap_on_a_bouncing_button_still_counts_its_bounces() {
    // The press's chatter runs from the press, and the release held back
    // a frame chatters from when it lets go: a tap on a bouncy button is
    // still several counts, as C-109's are, not none.
    let (mut circuit, button, digit) = counted_by_hand(true);
    circuit.try_set_closed(&button, true).unwrap();
    circuit.try_set_closed(&button, false).unwrap();
    run_for(&mut circuit, 0.5);
    assert_ne!(lit(&circuit, &digit), "abcdef", "it counted");
}

#[test]
fn a_counter_counts_the_edges_it_is_clocked_by() {
    // Fifty edges a second, a frame a sixtieth: after four frames three
    // have risen (at 0.02, 0.04 and 0.06 — at 0 the square starts high,
    // which is a level and not an edge), and after thirteen ten, which is
    // round to nought (C-78). No frame ends on an edge.
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let (counter, digit) = counting(&mut circuit, &battery);
    clocked(&mut circuit, &battery, &counter);
    run_for(&mut circuit, 4.0 / 60.0);
    assert_eq!(lit(&circuit, &digit), "abcdg", "three");
    run_for(&mut circuit, 9.0 / 60.0);
    assert_eq!(lit(&circuit, &digit), "abcdef", "round to nought");
}

#[test]
fn two_counters_chained_count_the_tens() {
    // The first's carry clocks the second, which goes up once for every
    // ten the first counts: twelve edges, after 15 frames, read 1 and 2.
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let (units, ones) = counting(&mut circuit, &battery);
    let (tens, tenths) = counting(&mut circuit, &battery);
    clocked(&mut circuit, &battery, &units);
    circuit.try_connect(&units, "CO", &tens, "CLK").unwrap();
    run_for(&mut circuit, 15.0 / 60.0);
    assert_eq!(lit(&circuit, &tenths), "bc", "one ten");
    assert_eq!(lit(&circuit, &ones), "abdeg", "and two");
}

#[test]
fn two_counters_chained_count_to_42_on_42_pulses() {
    // The acceptance's own count. At 50 Hz the edges come at 20 ms, 40 ms
    // and so on — the first sample, high, is a level and not an edge —
    // so 51 frames, 0.85 s, hold the 42nd and not the 43rd.
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let (units, ones) = counting(&mut circuit, &battery);
    let (tens, tenths) = counting(&mut circuit, &battery);
    clocked(&mut circuit, &battery, &units);
    circuit.try_connect(&units, "CO", &tens, "CLK").unwrap();
    run_for(&mut circuit, 51.0 / 60.0);
    assert_eq!(lit(&circuit, &tenths), "bcfg", "four tens");
    assert_eq!(lit(&circuit, &ones), "abdeg", "and two");
}

/// A CD4026 on 5 V clocked by a push-button up to the + with a 10 kΩ
/// pull-down, set `Ideal` or `Bouncy`; with `debounced`, through 10 kΩ
/// into 1 µF with 100 kΩ across it and two of a 74HC14's inverters in
/// a row. Returns the circuit, the button and the digit.
fn pressed_into_a_counter(choice: &str, debounced: bool) -> (Circuit, String, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    circuit.try_set_value(&battery, "5").unwrap();
    let (counter, digit) = counting(&mut circuit, &battery);
    let button = circuit.try_add("PushButton", 100.0, 0.0).unwrap();
    circuit.try_set_choice(&button, choice).unwrap();
    circuit.try_connect(&battery, "+", &button, "a").unwrap();
    let resistor = |circuit: &mut Circuit, ohms: &str| {
        let id = circuit.try_add("Resistor", 100.0, 80.0).unwrap();
        circuit.try_set_value(&id, ohms).unwrap();
        id
    };
    if debounced {
        let series = resistor(&mut circuit, "10k");
        let across = resistor(&mut circuit, "100k");
        let capacitor = circuit.try_add("Capacitor", 100.0, 120.0).unwrap();
        circuit.try_set_value(&capacitor, "1u").unwrap();
        let schmitt = circuit.try_add("Schmitt", 150.0, 0.0).unwrap();
        circuit.try_connect(&schmitt, "VCC", &battery, "+").unwrap();
        circuit.try_connect(&schmitt, "GND", &battery, "-").unwrap();
        circuit.try_connect(&button, "b", &series, "a").unwrap();
        circuit.try_connect(&series, "b", &capacitor, "a").unwrap();
        circuit.try_connect(&capacitor, "b", &battery, "-").unwrap();
        circuit.try_connect(&across, "a", &capacitor, "a").unwrap();
        circuit.try_connect(&across, "b", &battery, "-").unwrap();
        circuit
            .try_connect(&capacitor, "a", &schmitt, "1A")
            .unwrap();
        circuit.try_connect(&schmitt, "1Y", &schmitt, "2A").unwrap();
        circuit
            .try_connect(&schmitt, "2Y", &counter, "CLK")
            .unwrap();
    } else {
        let pull = resistor(&mut circuit, "10k");
        circuit.try_connect(&button, "b", &pull, "a").unwrap();
        circuit.try_connect(&pull, "b", &battery, "-").unwrap();
        circuit.try_connect(&button, "b", &counter, "CLK").unwrap();
    }
    run_for(&mut circuit, 0.2);
    (circuit, button, digit)
}

fn gate_rising() -> f64 {
    crate::solver::device::gate::RISING
}

fn gate_falling() -> f64 {
    crate::solver::device::gate::FALLING
}

/// One press, held for a fifth of a second, then let go for half of one.
fn press_once(circuit: &mut Circuit, button: &str) {
    circuit.try_set_closed(button, true).unwrap();
    run_for(circuit, 0.2);
    circuit.try_set_closed(button, false).unwrap();
    run_for(circuit, 0.5);
}

#[test]
fn a_bouncy_button_counts_five_where_an_ideal_one_counts_one() {
    // Pressed, it makes three times; let go, its chatter makes twice
    // more before it stays open (C-109). An ideal one, once.
    let (mut circuit, button, digit) = pressed_into_a_counter("Ideal", false);
    press_once(&mut circuit, &button);
    assert_eq!(lit(&circuit, &digit), "bc", "one");
    let (mut circuit, button, digit) = pressed_into_a_counter("Bouncy", false);
    circuit.try_set_closed(&button, true).unwrap();
    run_for(&mut circuit, 0.2);
    assert_eq!(lit(&circuit, &digit), "abcdg", "three on the press");
    circuit.try_set_closed(&button, false).unwrap();
    run_for(&mut circuit, 0.2);
    assert_eq!(lit(&circuit, &digit), "acdfg", "five once let go");
}

#[test]
fn a_schmitt_trigger_after_an_rc_counts_a_bouncy_button_once() {
    // The capacitor rides through the chatter, and the 74HC14 switches
    // once on its slow climb and once on its slow fall: one a press.
    let (mut circuit, button, digit) = pressed_into_a_counter("Bouncy", true);
    assert_eq!(lit(&circuit, &digit), "abcdef", "nought to begin with");
    press_once(&mut circuit, &button);
    assert_eq!(lit(&circuit, &digit), "bc", "one");
    press_once(&mut circuit, &button);
    assert_eq!(lit(&circuit, &digit), "abdeg", "two");
}

#[test]
fn one_schmitt_inverter_a_resistor_and_a_capacitor_oscillate() {
    // 1 MΩ from 1Y back to 1A, 1 µF from 1A to −: the capacitor charges
    // to the upper threshold and discharges to the lower, so at 5 V the
    // period is RC·(ln((5 − 1.78)/(5 − 2.78)) + ln(2.78/1.78)) = 0.82 s.
    // The LED is on 10 kΩ, so its quarter of a milliamp leaves the output
    // at 5 V; on 330 Ω its 8 mA would drop 0.4 V and slow it to 0.88 s.
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    circuit.try_set_value(&battery, "5").unwrap();
    let schmitt = circuit.try_add("Schmitt", 200.0, 0.0).unwrap();
    let feedback = circuit.try_add("Resistor", 200.0, 80.0).unwrap();
    let capacitor = circuit.try_add("Capacitor", 100.0, 80.0).unwrap();
    let resistor = circuit.try_add("Resistor", 300.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 400.0, 0.0).unwrap();
    circuit.try_set_value(&feedback, "1M").unwrap();
    circuit.try_set_value(&capacitor, "1u").unwrap();
    circuit.try_set_value(&resistor, "10k").unwrap();
    circuit.try_connect(&schmitt, "VCC", &battery, "+").unwrap();
    circuit.try_connect(&schmitt, "GND", &battery, "-").unwrap();
    circuit.try_connect(&schmitt, "1Y", &feedback, "a").unwrap();
    circuit.try_connect(&feedback, "b", &schmitt, "1A").unwrap();
    circuit
        .try_connect(&capacitor, "a", &schmitt, "1A")
        .unwrap();
    circuit.try_connect(&capacitor, "b", &battery, "-").unwrap();
    circuit.try_connect(&schmitt, "1Y", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
    circuit.try_connect(&led, "cathode", &battery, "-").unwrap();
    let mut was = true;
    let mut rises = Vec::new();
    for frame in 1..=600 {
        circuit.advance(1.0 / 60.0).unwrap();
        let lit = circuit
            .solution()
            .element_currents
            .iter()
            .find(|current| current.element_id == led)
            .is_some_and(|current| current.amps > 1e-4);
        if lit && !was {
            rises.push(f64::from(frame) / 60.0);
        }
        was = lit;
    }
    let period = (rises[rises.len() - 1] - rises[0]) / (rises.len() - 1) as f64;
    let expected = ((5.0 - 5.0 * gate_falling()) / (5.0 - 5.0 * gate_rising())).ln()
        + (gate_rising() / gate_falling()).ln();
    assert!(
        (period - expected).abs() < 0.03,
        "{period} s, not {expected} s"
    );
}

/// A CD4017 on 9 V clocked by a switch up to the + with a 10 kΩ
/// pull-down, inhibit to −, and a red LED through 330 Ω on each of Q0 to
/// Q9; its reset to −, or to `reset_from` when given. Returns the
/// circuit, the switch and the LEDs, Q0's first.
fn running_light(reset_from: Option<&str>) -> (Circuit, String, Vec<String>) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let chip = circuit.try_add("Decade", 200.0, 0.0).unwrap();
    circuit.try_connect(&chip, "VDD", &battery, "+").unwrap();
    circuit.try_connect(&chip, "VSS", &battery, "-").unwrap();
    circuit.try_connect(&chip, "INH", &battery, "-").unwrap();
    match reset_from {
        Some(output) => circuit.try_connect(&chip, output, &chip, "RST"),
        None => circuit.try_connect(&chip, "RST", &battery, "-"),
    }
    .unwrap();
    let switch = circuit.try_add("Switch", 100.0, 0.0).unwrap();
    let pull = circuit.try_add("Resistor", 100.0, 80.0).unwrap();
    circuit.try_set_value(&pull, "10k").unwrap();
    circuit.try_connect(&battery, "+", &switch, "a").unwrap();
    circuit.try_connect(&switch, "b", &pull, "a").unwrap();
    circuit.try_connect(&pull, "b", &battery, "-").unwrap();
    circuit.try_connect(&switch, "b", &chip, "CLK").unwrap();
    let leds = (0..10)
        .map(|count| {
            let resistor = circuit.try_add("Resistor", 300.0, 0.0).unwrap();
            let led = circuit.try_add("Led", 400.0, 0.0).unwrap();
            circuit.try_set_value(&resistor, "330").unwrap();
            circuit
                .try_connect(&chip, &format!("Q{count}"), &resistor, "a")
                .unwrap();
            circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
            circuit.try_connect(&led, "cathode", &battery, "-").unwrap();
            led
        })
        .collect();
    run_for(&mut circuit, 0.1);
    (circuit, switch, leds)
}

/// Which of `leds` are lit, by index.
fn which_lit(circuit: &Circuit, leds: &[String]) -> Vec<usize> {
    let solution = circuit.solution();
    leds.iter()
        .enumerate()
        .filter(|(_, led)| {
            solution
                .element_currents
                .iter()
                .any(|current| &current.element_id == *led && current.amps > 1e-3)
        })
        .map(|(index, _)| index)
        .collect()
}

fn taps(circuit: &mut Circuit, switch: &str, count: usize) {
    for _ in 0..count {
        circuit.try_set_closed(switch, true).unwrap();
        run_for(circuit, 0.05);
        circuit.try_set_closed(switch, false).unwrap();
        run_for(circuit, 0.05);
    }
}

#[test]
fn a_decade_counter_lights_one_led_at_a_time_and_goes_round() {
    // Q0 to begin with; three edges, Q3 alone; ten, Q0 again (C-110).
    let (mut circuit, switch, leds) = running_light(None);
    assert_eq!(which_lit(&circuit, &leds), [0]);
    taps(&mut circuit, &switch, 3);
    assert_eq!(which_lit(&circuit, &leds), [3]);
    taps(&mut circuit, &switch, 7);
    assert_eq!(which_lit(&circuit, &leds), [0], "round after Q9");
    assert!(warnings(&circuit).is_empty(), "{:?}", warnings(&circuit));
}

#[test]
fn a_decade_counter_drives_an_led_as_the_sheet_says() {
    // At nought on 5 V, Q0 high through 330 Ω into a red LED: behind the
    // sheet's 780 Ω about 2.9 mA, its own curve's 3 or so near enough;
    // 400 Ω would give 4.3 (C-124).
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    circuit.try_set_value(&battery, "5").unwrap();
    let chip = circuit.try_add("Decade", 200.0, 0.0).unwrap();
    for (pin, rail) in [
        ("VDD", "+"),
        ("VSS", "-"),
        ("INH", "-"),
        ("RST", "-"),
        ("CLK", "-"),
    ] {
        circuit.try_connect(&chip, pin, &battery, rail).unwrap();
    }
    let led = lit_by(&mut circuit, &battery, &chip, "Q0", "330");
    run_for(&mut circuit, 0.1);
    let amps = led_current(&circuit, &led);
    assert!((amps - 2.9e-3).abs() < 0.3e-3, "{amps} A");
}

#[test]
fn on_9_v_a_decade_counter_drives_its_led_harder_than_on_5_v() {
    // 9 V behind the sheet's 311 Ω there — four fifths of the way from
    // 5 V's 780 to 10 V's 270, in conductance — then 330 Ω and a red LED:
    // about 11 mA, against 2.9 on 5 V (C-124).
    let (circuit, _, leds) = running_light(None);
    let amps = led_current(&circuit, &leds[0]);
    assert!((amps - 11.1e-3).abs() < 0.5e-3, "{amps} A");
}

#[test]
fn a_flip_flop_drives_an_led_as_the_sheet_says() {
    // /Q1 high on 5 V through 330 Ω into a red LED: the CD4013's sheet
    // gives the CD4017's 3.2 mA at 2.5 V, so the same 2.9 mA (C-124).
    let (mut circuit, battery, chip, _) = toggling("5", "330");
    let led = lit_by(&mut circuit, &battery, &chip, "/Q1", "330");
    run_for(&mut circuit, 0.1);
    let amps = led_current(&circuit, &led);
    assert!((amps - 2.9e-3).abs() < 0.3e-3, "{amps} A");
}

#[test]
fn q4_wired_back_to_reset_counts_four_and_starts_again() {
    // Q4 rises and resets the count at once: Q0 to Q3, round.
    let (mut circuit, switch, leds) = running_light(Some("Q4"));
    taps(&mut circuit, &switch, 3);
    assert_eq!(which_lit(&circuit, &leds), [3]);
    taps(&mut circuit, &switch, 1);
    assert_eq!(which_lit(&circuit, &leds), [0], "four is nought");
    taps(&mut circuit, &switch, 2);
    assert_eq!(which_lit(&circuit, &leds), [2]);
}

/// A CD4013 on `volts` with its first flip-flop wired to toggle — /Q1 on
/// D1, SET1 and RST1 to − — and a red LED through `ohms` on Q1. Returns
/// the circuit, the battery, the chip and the LED; the clock is the
/// caller's.
fn toggling(volts: &str, ohms: &str) -> (Circuit, String, String, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    circuit.try_set_value(&battery, volts).unwrap();
    let chip = circuit.try_add("FlipFlop", 200.0, 0.0).unwrap();
    circuit.try_connect(&chip, "VDD", &battery, "+").unwrap();
    circuit.try_connect(&chip, "VSS", &battery, "-").unwrap();
    circuit.try_connect(&chip, "/Q1", &chip, "D1").unwrap();
    circuit.try_connect(&chip, "SET1", &battery, "-").unwrap();
    circuit.try_connect(&chip, "RST1", &battery, "-").unwrap();
    let led = lit_by(&mut circuit, &battery, &chip, "Q1", ohms);
    (circuit, battery, chip, led)
}

/// A red LED through `ohms` from `pin` of `chip` to the battery's −.
fn lit_by(circuit: &mut Circuit, battery: &str, chip: &str, pin: &str, ohms: &str) -> String {
    let resistor = circuit.try_add("Resistor", 300.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 400.0, 0.0).unwrap();
    circuit.try_set_value(&resistor, ohms).unwrap();
    circuit.try_connect(chip, pin, &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
    circuit.try_connect(&led, "cathode", battery, "-").unwrap();
    led
}

fn glowing(circuit: &Circuit, led: &str) -> bool {
    circuit
        .solution()
        .element_currents
        .iter()
        .any(|current| current.element_id == led && current.amps > 1e-4)
}

#[test]
fn a_flip_flop_with_its_not_q_on_d_toggles_on_every_edge() {
    // One switch on CLK1: on, off, on (C-111).
    let (mut circuit, battery, chip, led) = toggling("9", "330");
    let switch = circuit.try_add("Switch", 100.0, 0.0).unwrap();
    let pull = circuit.try_add("Resistor", 100.0, 80.0).unwrap();
    circuit.try_set_value(&pull, "10k").unwrap();
    circuit.try_connect(&battery, "+", &switch, "a").unwrap();
    circuit.try_connect(&switch, "b", &pull, "a").unwrap();
    circuit.try_connect(&pull, "b", &battery, "-").unwrap();
    circuit.try_connect(&switch, "b", &chip, "CLK1").unwrap();
    run_for(&mut circuit, 0.1);
    let mut seen = vec![glowing(&circuit, &led)];
    for _ in 0..3 {
        taps(&mut circuit, &switch, 1);
        seen.push(glowing(&circuit, &led));
    }
    assert_eq!(seen, [false, true, false, true]);
    assert!(warnings(&circuit).is_empty(), "{:?}", warnings(&circuit));
}

#[test]
fn two_flip_flops_in_a_row_halve_a_frequency_and_halve_it_again() {
    // A 10 Hz square on CLK1, Q1 on CLK2, both toggling: in two seconds
    // twenty rising edges in, ten out of Q1, five out of Q2.
    let (mut circuit, battery, chip, first) = toggling("9", "10k");
    circuit.try_connect(&chip, "Q1", &chip, "CLK2").unwrap();
    circuit.try_connect(&chip, "/Q2", &chip, "D2").unwrap();
    circuit.try_connect(&chip, "SET2", &battery, "-").unwrap();
    circuit.try_connect(&chip, "RST2", &battery, "-").unwrap();
    let second = lit_by(&mut circuit, &battery, &chip, "Q2", "10k");
    let clock = circuit.try_add("SignalGenerator", 100.0, 0.0).unwrap();
    circuit.try_set_value(&clock, "9").unwrap();
    circuit.try_set_frequency(&clock, "10").unwrap();
    circuit.try_connect(&clock, "+", &chip, "CLK1").unwrap();
    circuit.try_connect(&clock, "-", &battery, "-").unwrap();
    let mut was = [false, false];
    let mut rises = [0, 0];
    for _ in 0..120 {
        circuit.advance(1.0 / 60.0).unwrap();
        for (index, led) in [&first, &second].into_iter().enumerate() {
            let now = glowing(&circuit, led);
            rises[index] += usize::from(now && !was[index]);
            was[index] = now;
        }
    }
    assert!((9..=11).contains(&rises[0]), "Q1 rose {} times", rises[0]);
    assert!((4..=6).contains(&rises[1]), "Q2 rose {} times", rises[1]);
}

/// A motor on 6 V switched by a transistor whose gate or base is driven
/// from a 5 V battery, through 1 kΩ for a base. Returns the circuit and
/// the transistor, after a frame.
fn motor_switched_by(kind: &str, choice: &str) -> (Circuit, String) {
    let mut circuit = Circuit::new();
    let supply = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    circuit.try_set_value(&supply, "6").unwrap();
    let logic = circuit.try_add("DcSource", 0.0, 100.0).unwrap();
    circuit.try_set_value(&logic, "5").unwrap();
    let switch = circuit.try_add(kind, 200.0, 0.0).unwrap();
    circuit.try_set_choice(&switch, choice).unwrap();
    let motor = circuit.try_add("Motor", 200.0, -80.0).unwrap();
    let (control, out, common) = if kind == "Mosfet" {
        ("g", "d", "s")
    } else {
        ("b", "c", "e")
    };
    circuit.try_connect(&supply, "+", &motor, "a").unwrap();
    circuit.try_connect(&motor, "b", &switch, out).unwrap();
    circuit.try_connect(&switch, common, &supply, "-").unwrap();
    circuit.try_connect(&logic, "-", &supply, "-").unwrap();
    if kind == "Mosfet" {
        circuit.try_connect(&logic, "+", &switch, control).unwrap();
    } else {
        let base = circuit.try_add("Resistor", 100.0, 0.0).unwrap();
        circuit.try_set_value(&base, "1k").unwrap();
        circuit.try_connect(&logic, "+", &base, "a").unwrap();
        circuit.try_connect(&base, "b", &switch, control).unwrap();
    }
    run_for(&mut circuit, 1.0 / 60.0);
    (circuit, switch)
}

#[test]
fn a_2n7000_on_a_motor_cooks_where_an_irlz44n_stays_cool() {
    // A 2N7000 at 5 V on its gate tops out near 0.3 A and drops a volt and
    // a half doing it: half a watt into 312 K/W, past its 150 °C. An
    // IRLZ44N carries the motor's 0.4 A through 25 mΩ: milliwatts (C-112).
    let (small, small_id) = motor_switched_by("Mosfet", "N-channel");
    let cooked = heat_of(&small, &small_id);
    assert!(cooked.settles_at > cooked.max_celsius, "{cooked:?}");
    let (power, power_id) = motor_switched_by("Mosfet", "Power N-channel");
    let cool = heat_of(&power, &power_id);
    assert!(cool.watts < 0.01 && cool.settles_at < 30.0, "{cool:?}");
}

#[test]
fn a_tip120_runs_the_motor_warm_not_hot() {
    // Saturated at 0.7 V or so for 0.4 A: a third of a watt into 62.5
    // K/W, settling some 20 K above the room.
    let (circuit, id) = motor_switched_by("Bjt", "Darlington");
    let heat = heat_of(&circuit, &id);
    assert!((0.2..0.45).contains(&heat.watts), "{heat:?}");
    assert!(heat.settles_at < heat.max_celsius, "{heat:?}");
}

/// An LM317 on a battery of `volts`, `upper` between OUT and ADJ and
/// `lower` from ADJ to the battery's −, and a voltmeter on OUT. Returns
/// the circuit, the battery and the meter.
fn adjusted(volts: &str, upper: &str, lower: &str) -> (Circuit, String, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    circuit.try_set_value(&battery, volts).unwrap();
    let regulator = circuit.try_add("Adjustable", 100.0, 0.0).unwrap();
    let top = circuit.try_add("Resistor", 200.0, 0.0).unwrap();
    let bottom = circuit.try_add("Resistor", 200.0, 80.0).unwrap();
    let meter = circuit.try_add("Voltmeter", 300.0, 0.0).unwrap();
    circuit.try_set_value(&top, upper).unwrap();
    circuit.try_set_value(&bottom, lower).unwrap();
    circuit
        .try_connect(&battery, "+", &regulator, "in")
        .unwrap();
    circuit.try_connect(&regulator, "out", &top, "a").unwrap();
    circuit.try_connect(&top, "b", &regulator, "adj").unwrap();
    circuit
        .try_connect(&regulator, "adj", &bottom, "a")
        .unwrap();
    circuit.try_connect(&bottom, "b", &battery, "-").unwrap();
    circuit.try_connect(&meter, "+", &regulator, "out").unwrap();
    circuit.try_connect(&meter, "-", &battery, "-").unwrap();
    run_for(&mut circuit, 0.1);
    (circuit, battery, meter)
}

#[test]
fn an_lm317_gives_what_its_two_resistors_ask_for() {
    // 1.25 × (1 + 720 / 240) + 50 µA × 720 Ω = 5.036 V (C-113).
    let (circuit, _, meter) = adjusted("12", "240", "720");
    let volts = meter_reads(&circuit, &meter);
    assert!((volts - 5.036).abs() < 0.02, "{volts} V");
    assert!(warnings(&circuit).is_empty(), "{:?}", warnings(&circuit));
}

#[test]
fn an_lm317_short_of_headroom_follows_its_input_and_says_so() {
    // Set for 5 V on 6 V: under its 2 V of dropout, so about 4 V, and
    // a note saying why.
    let (circuit, _, meter) = adjusted("6", "240", "720");
    let volts = meter_reads(&circuit, &meter);
    assert!((3.8..4.2).contains(&volts), "{volts} V");
    let notes: Vec<String> = circuit
        .solution()
        .findings
        .into_iter()
        .map(|finding| finding.message)
        .collect();
    assert!(
        notes
            .iter()
            .any(|note| note.starts_with("This LM317 needs 2 V more in than out")),
        "{notes:?}"
    );
}

#[test]
fn an_lm317_with_one_resistor_holds_an_led_at_a_current() {
    // 62 Ω between OUT and ADJ, the LED from ADJ to −: 1.25 / 62 + 50 µA,
    // 20.2 mA, on 9 V and on 12 V alike.
    let mut at = Vec::new();
    for volts in ["9", "12"] {
        let mut circuit = Circuit::new();
        let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
        circuit.try_set_value(&battery, volts).unwrap();
        let regulator = circuit.try_add("Adjustable", 100.0, 0.0).unwrap();
        let sense = circuit.try_add("Resistor", 200.0, 0.0).unwrap();
        let led = circuit.try_add("Led", 300.0, 0.0).unwrap();
        circuit.try_set_value(&sense, "62").unwrap();
        circuit
            .try_connect(&battery, "+", &regulator, "in")
            .unwrap();
        circuit.try_connect(&regulator, "out", &sense, "a").unwrap();
        circuit.try_connect(&sense, "b", &regulator, "adj").unwrap();
        circuit
            .try_connect(&regulator, "adj", &led, "anode")
            .unwrap();
        circuit.try_connect(&led, "cathode", &battery, "-").unwrap();
        run_for(&mut circuit, 0.1);
        at.push(led_current(&circuit, &led));
    }
    for amps in &at {
        assert!((amps - 20.2e-3).abs() < 0.2e-3, "{amps} A");
    }
    assert!((at[0] - at[1]).abs() < 0.05e-3, "{at:?}");
}

/// A 9 V battery, a `kind` diode in series and a 100 Ω load, the battery
/// in `backwards` or not. Returns the volts across the load.
fn protected(kind: &str, backwards: bool) -> f64 {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let diode = circuit.try_add(kind, 100.0, 0.0).unwrap();
    let load = circuit.try_add("Resistor", 200.0, 0.0).unwrap();
    let meter = circuit.try_add("Voltmeter", 200.0, 80.0).unwrap();
    circuit.try_set_value(&load, "100").unwrap();
    let (high, low) = if backwards { ("-", "+") } else { ("+", "-") };
    circuit
        .try_connect(&battery, high, &diode, "anode")
        .unwrap();
    circuit.try_connect(&diode, "cathode", &load, "a").unwrap();
    circuit.try_connect(&load, "b", &battery, low).unwrap();
    circuit.try_connect(&meter, "+", &load, "a").unwrap();
    circuit.try_connect(&meter, "-", &load, "b").unwrap();
    run_for(&mut circuit, 0.1);
    meter_reads(&circuit, &meter)
}

#[test]
fn a_schottky_in_series_leaves_the_load_more_than_a_1n4148_does() {
    // About 83 mA: the 1N4148 drops 0.77 V of it, the 1N5819 0.33 V
    // (C-114). Backwards, both block: a Schottky's leak is 14 mV here.
    let silicon = protected("Diode", false);
    let schottky = protected("Schottky", false);
    assert!((8.15..8.3).contains(&silicon), "{silicon} V");
    assert!((8.6..8.75).contains(&schottky), "{schottky} V");
    assert!(protected("Diode", true).abs() < 1e-6);
    assert!(protected("Schottky", true).abs() < 0.02);
}

/// A magnet `mm` away, as a dial.
fn magnet_at(mm: f64) -> f64 {
    crate::solver::device::magnet::dial_for(mm)
}

#[test]
fn a_reed_switch_lights_an_led_while_the_magnet_is_near() {
    // 9 V, the reed, 330 Ω and an LED: dark with the magnet 30 mm off,
    // lit at 10 mm, dark again at 25 (C-115).
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let reed = circuit.try_add("Reed", 100.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 200.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 300.0, 0.0).unwrap();
    circuit.try_set_value(&resistor, "330").unwrap();
    circuit.try_connect(&battery, "+", &reed, "a").unwrap();
    circuit.try_connect(&reed, "b", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
    circuit.try_connect(&led, "cathode", &battery, "-").unwrap();
    let mut lit_at = |mm: f64| {
        circuit.try_set_wiper(&reed, magnet_at(mm)).unwrap();
        run_for(&mut circuit, 0.05);
        led_current(&circuit, &led) > 1e-3
    };
    assert_eq!(
        [lit_at(30.0), lit_at(10.0), lit_at(25.0)],
        [false, true, false]
    );
}

/// An A3144 on `volts`, a red LED through 330 Ω from the + to its OUT.
fn hall_sensing(volts: &str) -> (Circuit, String, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    circuit.try_set_value(&battery, volts).unwrap();
    let hall = circuit.try_add("Hall", 100.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 200.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 300.0, 0.0).unwrap();
    circuit.try_set_value(&resistor, "330").unwrap();
    circuit.try_connect(&hall, "VCC", &battery, "+").unwrap();
    circuit.try_connect(&hall, "GND", &battery, "-").unwrap();
    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
    circuit.try_connect(&led, "cathode", &hall, "OUT").unwrap();
    run_for(&mut circuit, 0.05);
    (circuit, hall, led)
}

#[test]
fn a_hall_sensor_switches_on_closer_than_it_lets_go() {
    // On at 10 mm; at 11.5 mm still on; off at 13 mm; at 11.5 mm still
    // off — between its two points it keeps what it was (C-115).
    let (mut circuit, hall, led) = hall_sensing("5");
    let mut lit_at = |mm: f64| {
        circuit.try_set_wiper(&hall, magnet_at(mm)).unwrap();
        run_for(&mut circuit, 0.05);
        led_current(&circuit, &led) > 1e-3
    };
    assert_eq!(
        [
            lit_at(30.0),
            lit_at(10.0),
            lit_at(11.5),
            lit_at(13.0),
            lit_at(11.5)
        ],
        [false, true, true, false, false]
    );
    assert!(warnings(&circuit).is_empty(), "{:?}", warnings(&circuit));
}

#[test]
fn a_hall_sensor_with_nothing_to_pull_its_output_up_is_told_so() {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    circuit.try_set_value(&battery, "5").unwrap();
    let hall = circuit.try_add("Hall", 100.0, 0.0).unwrap();
    let meter = circuit.try_add("Voltmeter", 200.0, 0.0).unwrap();
    circuit.try_connect(&hall, "VCC", &battery, "+").unwrap();
    circuit.try_connect(&hall, "GND", &battery, "-").unwrap();
    circuit.try_connect(&meter, "+", &hall, "OUT").unwrap();
    circuit.try_connect(&meter, "-", &battery, "-").unwrap();
    run_for(&mut circuit, 0.05);
    assert_eq!(
        warnings(&circuit),
        [
            "This A3144 can only pull OUT down — give OUT a resistor up to +, or it floats when \
              the magnet is away."
        ]
    );
}

#[test]
fn a_hall_sensor_on_3_3_volts_does_nothing_and_says_why() {
    let (mut circuit, hall, led) = hall_sensing("3.3");
    circuit.try_set_wiper(&hall, magnet_at(5.0)).unwrap();
    run_for(&mut circuit, 0.05);
    assert!(led_current(&circuit, &led) < 1e-6);
    assert_eq!(
        warnings(&circuit),
        ["This A3144 needs at least 4.5 V across VCC and GND."]
    );
}

#[test]
fn a_pir_lights_a_lamp_through_a_mosfet_after_it_settles_and_for_its_delay() {
    // 5 V, the module's OUT on a 2N7000's gate, an LED and 330 Ω from the
    // + to the drain (C-116).
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    circuit.try_set_value(&battery, "5").unwrap();
    let pir = circuit.try_add("Pir", 100.0, 0.0).unwrap();
    let fet = circuit.try_add("Mosfet", 200.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 200.0, -80.0).unwrap();
    let led = circuit.try_add("Led", 200.0, -40.0).unwrap();
    circuit.try_set_value(&resistor, "330").unwrap();
    circuit.try_connect(&pir, "VCC", &battery, "+").unwrap();
    circuit.try_connect(&pir, "GND", &battery, "-").unwrap();
    circuit.try_connect(&pir, "OUT", &fet, "g").unwrap();
    circuit.try_connect(&fet, "s", &battery, "-").unwrap();
    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
    circuit.try_connect(&led, "cathode", &fet, "d").unwrap();
    let lit = |circuit: &Circuit| led_current(circuit, &led) > 1e-3;

    circuit.try_set_closed(&pir, true).unwrap();
    run_for(&mut circuit, 30.0);
    assert!(!lit(&circuit), "still settling");
    let notes: Vec<String> = circuit
        .solution()
        .findings
        .into_iter()
        .map(|finding| finding.message)
        .collect();
    assert!(
        notes
            .iter()
            .any(|note| note.starts_with("This PIR is still settling")),
        "{notes:?}"
    );

    run_for(&mut circuit, 31.0);
    assert!(lit(&circuit), "seen once settled");
    circuit.try_set_closed(&pir, false).unwrap();
    run_for(&mut circuit, 4.5);
    assert!(lit(&circuit), "held for its 5 s");
    run_for(&mut circuit, 1.0);
    assert!(!lit(&circuit), "and off after");
    assert!(warnings(&circuit).is_empty(), "{:?}", warnings(&circuit));
}

#[test]
fn a_tap_on_a_pir_too_quick_for_a_frame_is_still_someone_passing() {
    // Pressed and let go with no frame between, as a trackpad's tap is:
    // the release waits a frame, the module sees the pass, and the LED is
    // lit for the delay (C-116).
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    circuit.try_set_value(&battery, "5").unwrap();
    let pir = circuit.try_add("Pir", 100.0, 0.0).unwrap();
    let fet = circuit.try_add("Mosfet", 200.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 200.0, -40.0).unwrap();
    let resistor = circuit.try_add("Resistor", 200.0, -80.0).unwrap();
    circuit.try_set_value(&resistor, "330").unwrap();
    circuit.try_connect(&pir, "VCC", &battery, "+").unwrap();
    circuit.try_connect(&pir, "GND", &battery, "-").unwrap();
    circuit.try_connect(&pir, "OUT", &fet, "g").unwrap();
    circuit.try_connect(&fet, "s", &battery, "-").unwrap();
    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
    circuit.try_connect(&led, "cathode", &fet, "d").unwrap();
    run_for(&mut circuit, 61.0);
    circuit.try_set_closed(&pir, true).unwrap();
    circuit.try_set_closed(&pir, false).unwrap();
    run_for(&mut circuit, 0.5);
    assert!(led_current(&circuit, &led) > 1e-3, "seen");
    let id = decode(&pir).unwrap();
    assert_eq!(
        circuit.netlist.element(id).unwrap().kind.moving(),
        Some(false),
        "let go"
    );
    run_for(&mut circuit, 5.5);
    assert!(
        led_current(&circuit, &led) < 1e-6,
        "and off after its delay"
    );
}

#[test]
fn a_pirs_delay_is_typed_and_held_to_its_knob() {
    let mut circuit = Circuit::new();
    let pir = circuit.try_add("Pir", 0.0, 0.0).unwrap();
    circuit.try_set_value(&pir, "30").unwrap();
    assert!(circuit.try_set_value(&pir, "2").is_err(), "under 5 s");
    assert!(circuit.try_set_value(&pir, "300").is_err(), "over 200 s");
    let id = decode(&pir).unwrap();
    assert!(matches!(
        circuit.netlist.element(id).unwrap().kind,
        ElementKind::Pir { hold, .. } if hold == 30.0
    ));
}

/// A 9 V battery lighting one segment of a digit through 330 Ω, the other
/// way round for a common-anode one. Returns the circuit and the digit.
fn one_segment(common: &str) -> (Circuit, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
    let digit = circuit.try_add("SevenSegment", 160.0, 0.0).unwrap();
    circuit.try_set_choice(&digit, common).unwrap();
    circuit.try_set_value(&resistor, "470").unwrap();
    let (to_segment, to_common) = if common == "Common cathode" {
        ("+", "-")
    } else {
        ("-", "+")
    };
    circuit
        .try_connect(&battery, to_segment, &resistor, "a")
        .unwrap();
    circuit.try_connect(&resistor, "b", &digit, "a").unwrap();
    circuit
        .try_connect(&digit, "com", &battery, to_common)
        .unwrap();
    (circuit, digit)
}

fn glow(circuit: &Circuit, digit: &str) -> Vec<f64> {
    circuit
        .solution()
        .digits
        .into_iter()
        .find(|entry| entry.element_id == digit)
        .expect("a digit reports its segments")
        .segments
}

#[test]
fn a_digit_lights_the_segment_that_is_driven_and_no_other() {
    // One segment through 470 Ω off 9 V is a red LED through 470 Ω off
    // 9 V: the same current, so the same brightness (C-75).
    let mut led = Circuit::new();
    let battery = led.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = led.try_add("Resistor", 80.0, 0.0).unwrap();
    let red = led.try_add("Led", 160.0, 0.0).unwrap();
    led.try_set_value(&resistor, "470").unwrap();
    led.try_connect(&battery, "+", &resistor, "a").unwrap();
    led.try_connect(&resistor, "b", &red, "anode").unwrap();
    led.try_connect(&red, "cathode", &battery, "-").unwrap();
    led.advance(1.0 / 60.0).unwrap();
    let expected = led
        .solution()
        .element_currents
        .into_iter()
        .find(|entry| entry.element_id == red)
        .and_then(|entry| entry.brightness)
        .unwrap();

    for common in ["Common cathode", "Common anode"] {
        let (mut circuit, digit) = one_segment(common);
        circuit.advance(1.0 / 60.0).unwrap();
        assert!(circuit.solution().failure.is_none());
        let segments = glow(&circuit, &digit);
        assert_eq!(segments.len(), 8);
        assert!(
            (segments[0] - expected).abs() < 1e-9,
            "{common}: segment a at {} against an LED's {expected}",
            segments[0]
        );
        assert!(
            segments[1..].iter().all(|&dark| dark == 0.0),
            "{common}: {segments:?}"
        );
    }
}

#[test]
fn a_digit_wired_the_wrong_way_round_stays_dark() {
    // A common-cathode digit with its common on the + is every LED reversed.
    let (mut circuit, digit) = one_segment("Common cathode");
    circuit.try_set_choice(&digit, "Common anode").unwrap();
    circuit.advance(1.0 / 60.0).unwrap();
    assert!(glow(&circuit, &digit)[0] < 1e-3);
}

#[test]
fn a_digit_gives_back_at_its_common_what_its_segments_take() {
    // What enters at the lit segment leaves at the common, so the wires
    // either side of the digit carry the same current (C-73).
    let (mut circuit, digit) = one_segment("Common cathode");
    circuit.advance(1.0 / 60.0).unwrap();
    let solution = circuit.solution();
    let into = |pin: &str| {
        solution
            .pin_currents
            .iter()
            .find(|entry| entry.pin.element_id == digit && entry.pin.pin == pin)
            .map(|entry| entry.amps)
            .unwrap()
    };
    assert!(into("a") < -0.01, "the segment takes current from its wire");
    assert!((into("a") + into("com")).abs() < 1e-12);
}

#[test]
fn a_digit_half_wired_is_no_trouble() {
    // A segment wired and its common not, beside a circuit that works:
    // the digit is dark and nothing else minds, the way a relay with a
    // free contact is still a relay (C-64). Being built is its normal
    // state.
    let (mut circuit, digit) = one_segment("Common cathode");
    let common = circuit
        .netlist
        .element(decode(&digit).unwrap())
        .and_then(|element| element.pins[crate::solver::device::digit::COMMON].net)
        .unwrap();
    circuit.try_remove_net(&encode_net(common)).unwrap();
    circuit.advance(1.0 / 60.0).unwrap();

    assert!(
        circuit.solution().failure.is_none(),
        "{:?}",
        circuit.solution().failure
    );
    assert!(glow(&circuit, &digit).iter().all(|&dark| dark == 0.0));
}

/// A battery charging a capacitor of `kind` through 1 kΩ, wired with
/// the battery's positive end to the capacitor's first pin — or to its
/// second, `backwards`. Returns the circuit, the battery and the capacitor.
fn charging(kind: &str, volts: &str, backwards: bool) -> (Circuit, String, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
    let capacitor = circuit.try_add(kind, 160.0, 0.0).unwrap();
    circuit.try_set_value(&battery, volts).unwrap();
    circuit.try_set_value(&resistor, "1k").unwrap();
    circuit.try_set_value(&capacitor, "100u").unwrap();
    let (one, other) = if kind == "Electrolytic" {
        ("+", "-")
    } else {
        ("a", "b")
    };
    let (feed, back) = if backwards {
        (other, one)
    } else {
        (one, other)
    };
    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit
        .try_connect(&resistor, "b", &capacitor, feed)
        .unwrap();
    circuit
        .try_connect(&capacitor, back, &battery, "-")
        .unwrap();
    (circuit, battery, capacitor)
}

fn warnings(circuit: &Circuit) -> Vec<String> {
    circuit
        .solution()
        .findings
        .into_iter()
        .map(|finding| finding.message)
        .collect()
}

fn volts_on(circuit: &Circuit, id: &str) -> f64 {
    let element = circuit.netlist.element(decode(id).unwrap()).unwrap();
    let solution = circuit.solution();
    let at = |pin: usize| {
        let net = encode_net(element.pins[pin].net.unwrap());
        solution
            .net_voltages
            .iter()
            .find(|entry| entry.net_id == net)
            .map(|entry| entry.volts)
            .unwrap_or(0.0)
    };
    at(0) - at(1)
}

#[test]
fn an_electrolytic_charges_as_a_capacitor_does() {
    // 9 V through 1 kΩ into 100 µF: τ = 0.1 s. Its leak is 1 MΩ, a
    // thousandth of the resistor's conductance, so the curve is the
    // capacitor's to within a part in a thousand (C-86).
    let (mut plain, _, capacitor) = charging("Capacitor", "9", false);
    let (mut electrolytic, _, polarised) = charging("Electrolytic", "9", false);
    for frames in [3, 6, 12] {
        for _ in 0..frames {
            plain.advance(1.0 / 60.0).unwrap();
            electrolytic.advance(1.0 / 60.0).unwrap();
        }
        let (one, two) = (
            volts_on(&plain, &capacitor),
            volts_on(&electrolytic, &polarised),
        );
        assert!((one - two).abs() < 9.0 / 1000.0, "{one} against {two}");
        // And below it, never above: the leak only ever takes charge away.
        assert!(two <= one, "{one} against {two}");
    }
    assert!(
        warnings(&electrolytic).is_empty(),
        "{:?}",
        warnings(&electrolytic)
    );
}

#[test]
fn an_electrolytic_wired_backwards_says_so_and_turned_round_stops() {
    let (mut circuit, _, capacitor) = charging("Electrolytic", "9", true);
    run_for(&mut circuit, 0.5);
    assert!(
        volts_on(&circuit, &capacitor) < -1.0,
        "it charged backwards"
    );
    assert_eq!(
        warnings(&circuit),
        ["This capacitor is the wrong way round — its + must face the higher voltage."]
    );
    // And it is still a capacitor: nothing broke.
    let (mut right, _, _) = charging("Electrolytic", "9", false);
    run_for(&mut right, 0.5);
    assert!(warnings(&right).is_empty());
}

#[test]
fn a_volt_backwards_is_not_wired_backwards() {
    // Under the volt it tolerates, as the rule of thumb has it.
    let (mut circuit, _, _) = charging("Electrolytic", "0.8", true);
    run_for(&mut circuit, 0.5);
    assert!(warnings(&circuit).is_empty(), "{:?}", warnings(&circuit));
}

#[test]
fn an_electrolytic_past_its_rating_says_so_and_a_higher_one_does_not() {
    let (mut circuit, _, capacitor) = charging("Electrolytic", "20", false);
    circuit.try_set_choice(&capacitor, "16 V").unwrap();
    run_for(&mut circuit, 1.0);
    assert_eq!(
        warnings(&circuit),
        ["This capacitor has more than its 16 V across it — choose one rated higher."]
    );
    circuit.try_set_choice(&capacitor, "25 V").unwrap();
    run_for(&mut circuit, 1.0);
    assert!(warnings(&circuit).is_empty(), "{:?}", warnings(&circuit));
}

#[test]
fn an_electrolytic_on_ac_is_backwards_every_half_cycle_and_says_so_steadily() {
    // A ±5 V square at 10 Hz straight across it: reversed half of every
    // cycle, which is a real fault for an electrolytic — and said on every
    // frame, not on and off with the wave.
    let mut circuit = Circuit::new();
    let generator = circuit.try_add("SignalGenerator", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
    let capacitor = circuit.try_add("Electrolytic", 160.0, 0.0).unwrap();
    circuit.try_set_frequency(&generator, "10").unwrap();
    circuit.try_set_value(&capacitor, "1u").unwrap();
    circuit
        .try_connect(&generator, "+", &resistor, "a")
        .unwrap();
    circuit
        .try_connect(&resistor, "b", &capacitor, "+")
        .unwrap();
    circuit
        .try_connect(&capacitor, "-", &generator, "-")
        .unwrap();
    run_for(&mut circuit, 0.2);
    for _ in 0..60 {
        circuit.advance(1.0 / 60.0).unwrap();
        assert_eq!(warnings(&circuit).len(), 1, "{:?}", warnings(&circuit));
    }
}

#[test]
fn a_charged_electrolytic_left_alone_leaks_away() {
    // 10 V on 100 µF with nothing but a voltmeter across it: its own leak,
    // 1 MΩ, empties it with τ = 100 s, so 10 s on it holds 10·e^(−0.1).
    let mut circuit = Circuit::new();
    let capacitor = circuit.try_add("Electrolytic", 0.0, 0.0).unwrap();
    let meter = circuit.try_add("Voltmeter", 80.0, 0.0).unwrap();
    circuit.try_set_initial_condition(&capacitor, "10").unwrap();
    circuit.try_connect(&capacitor, "+", &meter, "+").unwrap();
    circuit.try_connect(&capacitor, "-", &meter, "-").unwrap();
    run_for(&mut circuit, 10.0);
    let held = volts_on(&circuit, &capacitor);
    let expected = 10.0 * (-0.1f64).exp();
    assert!(
        (held - expected).abs() < 0.02,
        "{held} V against {expected}"
    );
}

/// A TMP36 on a battery of `volts`, its output into 100 kΩ, and a
/// voltmeter on that output. Returns the circuit, the sensor and the
/// meter.
fn sensing(volts: &str) -> (Circuit, String, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let sensor = circuit.try_add("Tmp36", 120.0, 0.0).unwrap();
    let load = circuit.try_add("Resistor", 240.0, 0.0).unwrap();
    let meter = circuit.try_add("Voltmeter", 320.0, 0.0).unwrap();
    circuit.try_set_value(&battery, volts).unwrap();
    circuit.try_set_value(&load, "100k").unwrap();
    circuit.try_connect(&sensor, "+Vs", &battery, "+").unwrap();
    circuit.try_connect(&sensor, "GND", &battery, "-").unwrap();
    circuit.try_connect(&sensor, "Vout", &load, "a").unwrap();
    circuit.try_connect(&load, "b", &battery, "-").unwrap();
    circuit.try_connect(&meter, "+", &sensor, "Vout").unwrap();
    circuit.try_connect(&meter, "-", &battery, "-").unwrap();
    (circuit, sensor, meter)
}

/// What a voltmeter reads, + over −.
fn meter_reads(circuit: &Circuit, meter: &str) -> f64 {
    volts_on(circuit, meter)
}

#[test]
fn a_tmp36_puts_out_half_a_volt_and_ten_millivolts_a_degree() {
    // 750 mV at 25 °C on 5 V, as its sheet says, and along its whole
    // range; into 100 kΩ, which its 60 Ω barely notices (C-87).
    let (mut circuit, sensor, meter) = sensing("5");
    circuit.advance(1.0 / 60.0).unwrap();
    assert!(
        (meter_reads(&circuit, &meter) - 0.75).abs() < 1e-3,
        "{}",
        meter_reads(&circuit, &meter)
    );
    for degrees in [-40.0, 0.0, 60.0, 125.0] {
        let dial = crate::solver::device::tmp36::dial_for(degrees);
        circuit.try_set_wiper(&sensor, dial).unwrap();
        circuit.advance(1.0 / 60.0).unwrap();
        let read = meter_reads(&circuit, &meter);
        let expected = 0.5 + 0.01 * degrees;
        // Within 2 mV: its 60 Ω drops a millivolt into 100 kΩ at the top.
        assert!((read - expected).abs() < 2e-3, "{degrees} °C read {read} V");
    }
    assert!(warnings(&circuit).is_empty(), "{:?}", warnings(&circuit));
}

#[test]
fn a_tmp36_draws_its_supply_current_and_no_more() {
    let (mut circuit, _, _) = sensing("5");
    circuit.advance(1.0 / 60.0).unwrap();
    let battery = circuit.solution().element_currents[0].amps.abs();
    // 50 µA for itself, and 7.5 µA into the 100 kΩ. The saturating
    // source softens its own draw near zero supply, shared with the
    // regulator, so at 5 V it is 49 µA rather than the full 50.
    assert!((55e-6..58e-6).contains(&battery), "{battery} A");
}

#[test]
fn a_tmp36_says_when_its_supply_is_wrong() {
    for (volts, said) in [
        (
            "9",
            "This TMP36 has more than the 5.5 V it runs on — give it 5 V, not the battery.",
        ),
        ("2", "This TMP36 needs at least 2.7 V across +Vs and GND."),
    ] {
        let (mut circuit, _, _) = sensing(volts);
        circuit.advance(1.0 / 60.0).unwrap();
        assert_eq!(warnings(&circuit), [said], "{volts} V");
    }
    for volts in ["3.3", "5"] {
        let (mut circuit, _, _) = sensing(volts);
        circuit.advance(1.0 / 60.0).unwrap();
        assert!(
            warnings(&circuit).is_empty(),
            "{volts} V: {:?}",
            warnings(&circuit)
        );
    }
}

#[test]
fn a_tmp36_without_its_supply_puts_out_nothing_and_says_so() {
    // Vout and GND wired, +Vs left in the air.
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let sensor = circuit.try_add("Tmp36", 120.0, 0.0).unwrap();
    let meter = circuit.try_add("Voltmeter", 320.0, 0.0).unwrap();
    let load = circuit.try_add("Resistor", 240.0, 0.0).unwrap();
    circuit.try_set_value(&load, "100k").unwrap();
    circuit.try_connect(&sensor, "GND", &battery, "-").unwrap();
    circuit.try_connect(&sensor, "Vout", &load, "a").unwrap();
    circuit.try_connect(&load, "b", &battery, "-").unwrap();
    circuit.try_connect(&battery, "+", &meter, "+").unwrap();
    circuit.try_connect(&meter, "-", &sensor, "Vout").unwrap();
    circuit.advance(1.0 / 60.0).unwrap();
    assert!(
        circuit.solution().failure.is_none(),
        "{:?}",
        circuit.solution().failure
    );
    let out = 9.0 - meter_reads(&circuit, &meter);
    assert!(out.abs() < 1e-3, "{out} V out of an unpowered TMP36");
    assert_eq!(
        warnings(&circuit),
        ["This TMP36 has no power — wire +Vs and GND to the battery."]
    );
}

/// A part between 5 V and a resistor of `ohms` to the battery's −, the
/// part's first pin to the +. Returns the circuit, the part, the resistor.
fn above_a_resistor(kind: &str, ohms: &str) -> (Circuit, String, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let part = circuit.try_add(kind, 120.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 240.0, 0.0).unwrap();
    circuit.try_set_value(&battery, "5").unwrap();
    circuit.try_set_value(&resistor, ohms).unwrap();
    let (first, second) = if kind == "Phototransistor" {
        ("c", "e")
    } else {
        ("a", "b")
    };
    circuit.try_connect(&battery, "+", &part, first).unwrap();
    circuit.try_connect(&part, second, &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &battery, "-").unwrap();
    (circuit, part, resistor)
}

/// The volts across the resistor below a part with its dial at `dial`.
fn below(circuit: &mut Circuit, part: &str, resistor: &str, dial: f64) -> f64 {
    circuit.try_set_wiper(part, dial).unwrap();
    circuit.advance(1.0 / 60.0).unwrap();
    assert!(
        circuit.solution().failure.is_none(),
        "{:?}",
        circuit.solution().failure
    );
    volts_on(circuit, resistor)
}

#[test]
fn a_phototransistor_s_current_follows_the_light() {
    // 2 µA a lux into 1 kΩ: its current rises with every step of the dial,
    // and in the dark is only its dark current (C-89).
    let (mut circuit, photo, resistor) = above_a_resistor("Phototransistor", "1k");
    let mut last = 0.0;
    for step in 0..=6 {
        let volts = below(&mut circuit, &photo, &resistor, step as f64 / 10.0);
        assert!(volts > last, "dial {step}: {volts} V after {last}");
        last = volts;
    }
    // 1 lux at dial 0.2: 2 µA, so 2 mV across 1 kΩ.
    let one_lux = below(&mut circuit, &photo, &resistor, 0.2);
    assert!((one_lux - 2e-3).abs() < 1e-4, "{one_lux} V at 1 lx");
    let dark = below(&mut circuit, &photo, &resistor, 0.0);
    assert!(dark < 1e-3 * 1e-3 * 1000.0, "{dark} V in the dark");
}

#[test]
fn a_phototransistor_in_bright_light_saturates() {
    // 10 000 lux would be 20 mA; 1 kΩ on 5 V allows 5 mA. So it is
    // pulled down to a transistor's saturation, not to a resistor's
    // share: under a few tenths of a volt across it.
    let (mut circuit, photo, resistor) = above_a_resistor("Phototransistor", "1k");
    let across_resistor = below(&mut circuit, &photo, &resistor, 1.0);
    let across_photo = 5.0 - across_resistor;
    assert!(across_photo < 0.3, "{across_photo} V across it");
    assert!(across_photo > 0.0);
}

#[test]
fn a_phototransistor_swings_further_than_an_ldr_s_divider() {
    // The same light, 1 lx to 1 000 lx, over 10 kΩ on 5 V: the LDR's
    // divider moves from under a volt to near the top, the
    // phototransistor from nearly nothing to nearly all of it.
    let swing = |kind: &str| {
        let (mut circuit, part, resistor) = above_a_resistor(kind, "10k");
        let dark = below(&mut circuit, &part, &resistor, 0.2);
        let bright = below(&mut circuit, &part, &resistor, 0.8);
        (dark, bright, bright - dark)
    };
    let ldr = swing("Ldr");
    let photo = swing("Phototransistor");
    assert!(
        photo.2 > ldr.2,
        "phototransistor {photo:?} against LDR {ldr:?}"
    );
    assert!(photo.0 < 0.05 && ldr.0 > 0.5, "{photo:?} {ldr:?}");
}

/// A battery of `part` with `load` across it — a part's kind, its first
/// two pins to the battery's + and −. Returns the circuit, the battery
/// and the load.
fn on_battery(part: &str, load: &str) -> (Circuit, String, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("Cell", 0.0, 0.0).unwrap();
    let thing = circuit.try_add(load, 120.0, 0.0).unwrap();
    circuit.try_set_choice(&battery, part).unwrap();
    let pins = crate::registry::definition(load).unwrap().pins;
    circuit.try_connect(&battery, "+", &thing, pins[0]).unwrap();
    circuit.try_connect(&thing, pins[1], &battery, "-").unwrap();
    (circuit, battery, thing)
}

fn charge_of(circuit: &Circuit, battery: &str) -> ElementCharge {
    circuit
        .solution()
        .batteries
        .into_iter()
        .find(|entry| entry.element_id == battery)
        .expect("a battery reports its charge")
}

#[test]
fn a_battery_sags_under_load_by_its_internal_resistance() {
    // A motor's 15 Ω: a fresh 9 V, 9.12 V behind 9.6 Ω, gives it 5.6 V;
    // four AAs, 6.04 V behind 0.9 Ω, 5.7 V; and on 10 kΩ both near
    // their whole.
    for (part, under, open) in [("9 V", 5.55, 9.11), ("4 × AA", 5.70, 6.04)] {
        let (mut circuit, _, motor) = on_battery(part, "Motor");
        run_for(&mut circuit, 0.1);
        let volts = volts_on(&circuit, &motor);
        assert!((volts - under).abs() < 0.05, "{part}: {volts} V on a motor");
        let (mut circuit, _, resistor) = on_battery(part, "Resistor");
        circuit.try_set_value(&resistor, "10k").unwrap();
        run_for(&mut circuit, 0.1);
        assert!(
            (volts_on(&circuit, &resistor) - open).abs() < 0.02,
            "{part} at 10 kΩ"
        );
    }
}

#[test]
fn a_battery_gives_up_the_charge_its_current_draws_and_says_how_long_is_left() {
    // 9 V into 100 Ω for a minute: about 83 mA, so 83 mA·min of a
    // capacity of some 380 mAh at that rate; and its readout is the rest
    // over the rate.
    let (mut circuit, battery, resistor) = on_battery("9 V", "Resistor");
    circuit.try_set_value(&resistor, "100").unwrap();
    run_for(&mut circuit, 0.1);
    let before = charge_of(&circuit, &battery);
    let amps = volts_on(&circuit, &resistor) / 100.0;
    run_for(&mut circuit, 60.0);
    let after = charge_of(&circuit, &battery);
    let sheet = crate::solver::device::cell::CellPart::NineVolt.sheet();
    let expected = amps * 60.0 / 3600.0 / (sheet.capacity(amps) * 1e-3);
    let drawn = before.left - after.left;
    assert!(
        (drawn / expected - 1.0).abs() < 0.02,
        "{drawn} of {expected}"
    );
    let hours = after.hours.unwrap();
    assert!(
        (hours - after.left * sheet.capacity(amps) * 1e-3 / amps).abs() < 0.05,
        "{hours} h"
    );
    // Only a meter on it: nothing drawn, and no readout of hours.
    let (mut circuit, alone, _) = on_battery("9 V", "Voltmeter");
    run_for(&mut circuit, 1.0);
    let idle = charge_of(&circuit, &alone);
    assert_eq!(idle.hours, None);
    assert_eq!(idle.left, 1.0);
}

#[test]
fn a_battery_turned_down_on_its_dial_shows_its_knee_and_then_says_it_is_flat() {
    let (mut circuit, battery, resistor) = on_battery("AA", "Resistor");
    circuit.try_set_value(&resistor, "1k").unwrap();
    let at = |circuit: &mut Circuit, left: f64| {
        circuit.try_set_wiper(&battery, left).unwrap();
        run_for(circuit, 0.1);
        volts_on(circuit, &resistor)
    };
    let full = at(&mut circuit, 1.0);
    let half = at(&mut circuit, 0.5);
    let nearly = at(&mut circuit, 0.1);
    assert!((full - 1.51).abs() < 0.01 && half < full && nearly < half);
    assert!(
        full - half < half - nearly,
        "{full} {half} {nearly}: the knee"
    );
    assert!(!warnings(&circuit).iter().any(|w| w.contains("flat")));
    let flat = at(&mut circuit, 0.0);
    assert!(flat < 0.85, "{flat} V");
    assert!(
        warnings(&circuit).iter().any(|w| w.contains("flat")),
        "{:?}",
        warnings(&circuit)
    );
    // Turning the dial back up is a new battery as full as it says.
    assert!((at(&mut circuit, 1.0) - full).abs() < 1e-6);
}

#[test]
fn a_nine_volt_battery_runs_a_motor_for_a_fraction_of_four_aas_time() {
    // At a motor's 370 mA or so, a 9 V has under an hour in it; four
    // AAs, several.
    let hours = |part: &str| {
        let (mut circuit, battery, _) = on_battery(part, "Motor");
        run_for(&mut circuit, 0.5);
        charge_of(&circuit, &battery).hours.unwrap()
    };
    let (nine, four) = (hours("9 V"), hours("4 × AA"));
    assert!(nine < 1.0 && four > 3.0, "{nine} h against {four} h");
}

/// A battery of `volts`, a fuse of `part` rated `amps`, and a load of
/// `ohms`, with a switch across the load that shorts it when shut.
/// Returns the circuit, the fuse, the load and the switch.
fn fused(volts: &str, part: &str, amps: &str, ohms: &str) -> (Circuit, String, String, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let fuse = circuit.try_add("Fuse", 100.0, 0.0).unwrap();
    let load = circuit.try_add("Resistor", 200.0, 0.0).unwrap();
    let short = circuit.try_add("Switch", 200.0, 100.0).unwrap();
    circuit.try_set_value(&battery, volts).unwrap();
    circuit.try_set_choice(&fuse, part).unwrap();
    circuit.try_set_value(&fuse, amps).unwrap();
    circuit.try_set_value(&load, ohms).unwrap();
    circuit.try_connect(&battery, "+", &fuse, "a").unwrap();
    circuit.try_connect(&fuse, "b", &load, "a").unwrap();
    circuit.try_connect(&load, "b", &battery, "-").unwrap();
    circuit.try_connect(&short, "a", &load, "a").unwrap();
    circuit.try_connect(&short, "b", &load, "b").unwrap();
    (circuit, fuse, load, short)
}

fn is_blown(circuit: &Circuit, fuse: &str) -> bool {
    matches!(
        circuit.netlist.element(decode(fuse).unwrap()).unwrap().kind,
        ElementKind::Fuse { blown: true, .. }
    )
}

#[test]
fn a_fuse_below_its_rating_never_blows() {
    // 5 V into 12 Ω through a 500 mA fast-blow: 0.41 A, for ten minutes.
    let (mut circuit, fuse, load, _) = fused("5", "Fast-blow", "500m", "12");
    for _ in 0..60 {
        run_for(&mut circuit, 10.0);
    }
    assert!(!is_blown(&circuit, &fuse));
    assert!((volts_on(&circuit, &load) / 12.0 - 0.41).abs() < 0.01);
}

#[test]
fn a_fuse_at_twice_its_rating_blows_within_its_sheet_s_time_and_stays_blown() {
    // 1 A through a 500 mA fast-blow: the sheet's 210 % is within 30
    // minutes; the model's closed form, under half a second.
    let (mut circuit, fuse, load, _) = fused("5", "Fast-blow", "500m", "4.8");
    let mut seconds = 0.0;
    while !is_blown(&circuit, &fuse) {
        run_for(&mut circuit, 1.0 / 60.0);
        seconds += 1.0 / 60.0;
        assert!(seconds < 1800.0, "never blew");
    }
    assert!(seconds > 0.2 && seconds < 1.0, "{seconds} s");
    run_for(&mut circuit, 1.0);
    assert!(volts_on(&circuit, &load).abs() < 1e-6, "nothing flows");
    assert!(
        warnings(&circuit)
            .iter()
            .any(|warning| warning.contains("blown"))
    );
    let view = circuit
        .elements_for_test()
        .into_iter()
        .find(|view| view.id == fuse)
        .unwrap();
    assert_eq!(view.label, "500 mA blown");
    assert_eq!(view.blown, Some(true));
    // Taking the cause away does not mend it: a blown fuse is blown.
    circuit.try_set_value(&load, "100").unwrap_or(());
    run_for(&mut circuit, 5.0);
    assert!(is_blown(&circuit, &fuse));
}

#[test]
fn a_short_blows_its_fuse_for_good_until_it_is_replaced() {
    // A LED-sized load, then the switch across it shut: the battery's
    // whole voltage across the fuse's 0.21 Ω, and it goes in a frame.
    let (mut circuit, fuse, load, short) = fused("9", "Fast-blow", "500m", "470");
    run_for(&mut circuit, 1.0);
    assert!(!is_blown(&circuit, &fuse));
    circuit.try_set_closed(&short, true).unwrap();
    run_for(&mut circuit, 0.1);
    assert!(is_blown(&circuit, &fuse));
    circuit.try_set_closed(&short, false).unwrap();
    run_for(&mut circuit, 1.0);
    assert!(is_blown(&circuit, &fuse), "still blown with the short gone");
    assert!(volts_on(&circuit, &load).abs() < 1e-6);
    // Replaced: whole, cold, and the load runs again.
    circuit.try_replace_fuse(&fuse).unwrap();
    run_for(&mut circuit, 1.0);
    assert!(!is_blown(&circuit, &fuse));
    assert!((volts_on(&circuit, &load) - 9.0).abs() < 0.01);
    // And a replacement is a step of undo.
    assert!(circuit.undo());
    assert!(is_blown(&circuit, &fuse));
}

#[test]
fn a_polyfuse_trips_on_a_short_and_conducts_again_once_it_is_gone() {
    // An MF-R050 on 9 V: shorted, it trips within its sheet's time, and
    // holds itself hot on its 0.75 W; the short taken away, the load
    // cannot keep it hot, and it cools and conducts again.
    let (mut circuit, fuse, load, short) = fused("9", "Polyfuse", "500m", "470");
    run_for(&mut circuit, 1.0);
    circuit.try_set_closed(&short, true).unwrap();
    let tripped = |circuit: &Circuit| {
        circuit
            .solution()
            .findings
            .iter()
            .any(|finding| finding.message.contains("tripped"))
    };
    let mut seconds = 0.0;
    while !tripped(&circuit) {
        run_for(&mut circuit, 1.0 / 60.0);
        seconds += 1.0 / 60.0;
        assert!(seconds < 4.0, "not tripped in the sheet's 4 s");
    }
    run_for(&mut circuit, 20.0);
    assert!(tripped(&circuit), "a short holds it tripped");
    assert!(!is_blown(&circuit, &fuse), "a polyfuse is never blown");
    circuit.try_set_closed(&short, false).unwrap();
    let mut seconds = 0.0;
    while tripped(&circuit) {
        run_for(&mut circuit, 0.5);
        seconds += 0.5;
        assert!(seconds < 120.0, "never reset");
    }
    run_for(&mut circuit, 0.5);
    assert!(
        (volts_on(&circuit, &load) - 9.0).abs() < 0.05,
        "conducting again"
    );
}

#[test]
fn a_polyfuse_on_a_heavy_load_stays_tripped_until_the_power_goes() {
    // 10 Ω on 9 V is 0.85 A, under its 1 A trip, so it carries it. Once a
    // short has tripped it, the 10 Ω alone can keep it hot — the supply
    // can still give it its tripped watts — so it holds tripped with the
    // short gone, as a real one does, and only the power going resets it.
    let (mut circuit, _, load, short) = fused("9", "Polyfuse", "500m", "10");
    run_for(&mut circuit, 2.0);
    assert!((volts_on(&circuit, &load) / 10.0 - 0.85).abs() < 0.02);
    circuit.try_set_closed(&short, true).unwrap();
    run_for(&mut circuit, 5.0);
    circuit.try_set_closed(&short, false).unwrap();
    run_for(&mut circuit, 30.0);
    assert!(
        volts_on(&circuit, &load) < 1.5,
        "{} V: still tripped",
        volts_on(&circuit, &load)
    );
    let battery = circuit
        .netlist
        .elements()
        .next()
        .map(|(id, _)| encode(id))
        .unwrap();
    circuit.try_set_value(&battery, "0").unwrap();
    run_for(&mut circuit, 30.0);
    circuit.try_set_value(&battery, "9").unwrap();
    run_for(&mut circuit, 0.5);
    assert!(
        (volts_on(&circuit, &load) / 10.0 - 0.85).abs() < 0.02,
        "reset"
    );
}

#[test]
fn a_fuse_comes_only_in_its_ratings_and_saves_blown() {
    let (mut circuit, fuse, _, short) = fused("9", "Fast-blow", "500m", "470");
    let refused = circuit.try_set_value(&fuse, "600m").unwrap_err();
    assert!(refused.to_string().contains("500 mA"), "{refused}");
    circuit.try_set_closed(&short, true).unwrap();
    run_for(&mut circuit, 0.1);
    let saved = serde_json::to_string(&circuit.to_document()).unwrap();
    let document: crate::document::Document = serde_json::from_str(&saved).unwrap();
    let opened = Circuit::from_document(&document).unwrap();
    let id = opened
        .netlist
        .elements()
        .nth(1)
        .map(|(id, _)| encode(id))
        .unwrap();
    assert!(is_blown(&opened, &id));
}

/// An LM3914 on 5 V, R1 of `r1` on its reference with the divider across
/// it (RLO at 0 V, RHI at REF OUT), a bar of ten LEDs from 5 V into its
/// outputs — LED 1's anode through 10 Ω, to read its current — and
/// `signal` volts on SIG from a source of its own, in bar mode or dot.
/// Returns the circuit, the signal source, the bar and the 10 Ω.
fn level_meter(r1: &str, signal: &str, bar: bool) -> (Circuit, String, String, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let input = circuit.try_add("DcSource", 0.0, 200.0).unwrap();
    let chip = circuit.try_add("Lm3914", 200.0, 0.0).unwrap();
    let leds = circuit.try_add("Bargraph", 400.0, 0.0).unwrap();
    let program = circuit.try_add("Resistor", 200.0, 200.0).unwrap();
    let sense = circuit.try_add("Resistor", 400.0, 200.0).unwrap();
    circuit.try_set_value(&battery, "5").unwrap();
    circuit.try_set_value(&input, signal).unwrap();
    circuit.try_set_value(&program, r1).unwrap();
    circuit.try_set_value(&sense, "10").unwrap();
    let wire = |circuit: &mut Circuit, a: &str, p: &str, b: &str, q: &str| {
        circuit.try_connect(a, p, b, q).unwrap();
    };
    wire(&mut circuit, &battery, "+", &chip, "V+");
    wire(&mut circuit, &battery, "-", &chip, "V-");
    wire(&mut circuit, &chip, "V-", &chip, "RLO");
    wire(&mut circuit, &chip, "V-", &chip, "REFADJ");
    wire(&mut circuit, &chip, "REFOUT", &chip, "RHI");
    wire(&mut circuit, &chip, "REFOUT", &program, "a");
    wire(&mut circuit, &program, "b", &chip, "V-");
    wire(&mut circuit, &input, "+", &chip, "SIG");
    wire(&mut circuit, &input, "-", &chip, "V-");
    if bar {
        wire(&mut circuit, &chip, "MODE", &chip, "V+");
    }
    wire(&mut circuit, &battery, "+", &sense, "a");
    wire(&mut circuit, &sense, "b", &leds, "a1");
    for k in 1..=10 {
        if k > 1 {
            wire(&mut circuit, &battery, "+", &leds, &format!("a{k}"));
        }
        wire(
            &mut circuit,
            &leds,
            &format!("k{k}"),
            &chip,
            &format!("L{k}"),
        );
    }
    run_for(&mut circuit, 0.1);
    (circuit, input, leds, sense)
}

/// Which of a bar's LEDs are lit, 1 to 10.
fn lit_bars(circuit: &Circuit, bar: &str) -> Vec<usize> {
    glow(circuit, bar)
        .iter()
        .enumerate()
        .filter(|&(_, &glow)| glow > 0.05)
        .map(|(index, _)| index + 1)
        .collect()
}

#[test]
fn an_lm3914_lights_a_bar_a_step_at_a_time() {
    // 1.25 V across its divider: a step every 125 mV. Half a step short
    // of each one, it has lit one fewer.
    for k in 1..=10 {
        let below = format!("{}", (k as f64 - 0.5) * 0.125);
        let above = format!("{}", (k as f64 + 0.3) * 0.125);
        let (circuit, _, leds, _) = level_meter("1.2k", &below, true);
        assert_eq!(
            lit_bars(&circuit, &leds),
            (1..k).collect::<Vec<_>>(),
            "{below} V"
        );
        let (circuit, _, leds, _) = level_meter("1.2k", &above, true);
        assert_eq!(
            lit_bars(&circuit, &leds),
            (1..=k).collect::<Vec<_>>(),
            "{above} V"
        );
    }
    // Nothing at nothing, everything past the top.
    let (circuit, _, leds, _) = level_meter("1.2k", "0", true);
    assert!(lit_bars(&circuit, &leds).is_empty());
    let (circuit, _, leds, _) = level_meter("1.2k", "3", true);
    assert_eq!(lit_bars(&circuit, &leds).len(), 10);
}

#[test]
fn an_lm3914_in_dot_mode_lights_only_its_highest_step() {
    let (circuit, _, leds, _) = level_meter("1.2k", "0.7", false);
    assert_eq!(lit_bars(&circuit, &leds), vec![5]);
    let (circuit, _, leds, _) = level_meter("1.2k", "0.7", true);
    assert_eq!(lit_bars(&circuit, &leds), vec![1, 2, 3, 4, 5]);
}

#[test]
fn an_lm3914_s_led_current_is_set_by_its_reference_resistor() {
    // About ten times what leaves REF OUT: 1.25 V into R1, and into the
    // 12 kΩ divider beside it. 1.2 kΩ is some 11.5 mA; twice R1, about
    // half — the brightness is the resistor's to set.
    let amps = |r1: &str, ohms: f64| {
        let (circuit, _, _, sense) = level_meter(r1, "3", true);
        let amps = volts_on(&circuit, &sense) / 10.0;
        let expected = 10.0 * 1.25 * (1.0 / ohms + 1.0 / 12e3);
        assert!(
            (amps / expected - 1.0).abs() < 0.03,
            "{r1}: {amps} A, {expected} A expected"
        );
        amps
    };
    let low = amps("1.2k", 1200.0);
    let high = amps("2.4k", 2400.0);
    assert!(high < 0.6 * low);
}

#[test]
fn an_lm3914_on_too_little_supply_says_so_and_lights_nothing() {
    let (mut circuit, _, leds, _) = level_meter("1.2k", "1", true);
    let battery = circuit
        .netlist
        .elements()
        .next()
        .map(|(id, _)| encode(id))
        .unwrap();
    circuit.try_set_value(&battery, "2").unwrap();
    run_for(&mut circuit, 0.1);
    assert!(lit_bars(&circuit, &leds).is_empty());
    assert!(
        warnings(&circuit)
            .iter()
            .any(|warning| warning.contains("LM3914")),
        "{:?}",
        warnings(&circuit)
    );
}

/// A 50 Hz sine of `peak` volts into a 2 : 1 transformer's primary.
/// Returns the circuit and the transformer, its secondary free.
fn transformer_on_a_sine(peak: &str) -> (Circuit, String) {
    let mut circuit = Circuit::new();
    let generator = circuit.try_add("SignalGenerator", 0.0, 0.0).unwrap();
    let transformer = circuit.try_add("Transformer", 120.0, 0.0).unwrap();
    circuit.try_set_choice(&generator, "Sine").unwrap();
    circuit.try_set_value(&generator, peak).unwrap();
    circuit.try_set_frequency(&generator, "50").unwrap();
    circuit
        .try_connect(&generator, "+", &transformer, "p1")
        .unwrap();
    circuit
        .try_connect(&transformer, "p2", &generator, "-")
        .unwrap();
    (circuit, transformer)
}

/// The lowest and highest volts across `id` over `seconds`, read every
/// half millisecond — a frame is too coarse for 50 Hz.
fn swing_over(circuit: &mut Circuit, id: &str, seconds: f64) -> (f64, f64) {
    let (mut low, mut high) = (f64::MAX, f64::MIN);
    for _ in 0..(seconds * 2000.0).round() as usize {
        circuit.advance(0.5e-3).unwrap();
        assert!(
            circuit.solution().failure.is_none(),
            "{:?}",
            circuit.solution().failure
        );
        let volts = volts_on(circuit, id);
        low = low.min(volts);
        high = high.max(volts);
    }
    (low, high)
}

#[test]
fn a_transformer_steps_a_sine_down_by_its_ratio() {
    // 24 V peak on the primary: 12 V peak across 1 kΩ on a 2 : 1's
    // secondary, both ways — and 10 : 1 makes it 2.4 V.
    for (ratio, expected) in [("2", 12.0), ("10", 2.4)] {
        let (mut circuit, transformer) = transformer_on_a_sine("24");
        circuit.try_set_value(&transformer, ratio).unwrap();
        let load = circuit.try_add("Resistor", 240.0, 0.0).unwrap();
        circuit.try_set_value(&load, "1k").unwrap();
        circuit.try_connect(&transformer, "s1", &load, "a").unwrap();
        circuit.try_connect(&load, "b", &transformer, "s2").unwrap();
        run_for(&mut circuit, 0.5);
        let (low, high) = swing_over(&mut circuit, &load, 0.1);
        for peak in [high, -low] {
            assert!(
                (peak / expected - 1.0).abs() < 0.03,
                "{ratio}: {low}..{high} V"
            );
        }
    }
}

#[test]
fn a_transformer_s_sides_are_two_circuits() {
    // Nothing conducts from one winding to the other, so the secondary
    // has a reference of its own (C-92's rule); and a ratio outside what
    // one is wound to is refused.
    let (mut circuit, transformer) = transformer_on_a_sine("24");
    let load = circuit.try_add("Resistor", 240.0, 0.0).unwrap();
    circuit.try_connect(&transformer, "s1", &load, "a").unwrap();
    circuit.try_connect(&load, "b", &transformer, "s2").unwrap();
    assert_eq!(circuit.netlist.circuits().len(), 2);
    assert_eq!(circuit.netlist.compile().unwrap().references().len(), 2);
    assert!(circuit.try_set_value(&transformer, "1000").is_err());
    assert!(circuit.try_set_value(&transformer, "0.01").is_err());
}

#[test]
fn a_transformer_gives_nothing_from_a_battery() {
    // 5 V steady on the primary: a kick on the secondary while the
    // primary's current builds — 10 H over 10 Ω, a second or so — then
    // only its copper's current, and nothing across the secondary.
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let transformer = circuit.try_add("Transformer", 120.0, 0.0).unwrap();
    let load = circuit.try_add("Resistor", 240.0, 0.0).unwrap();
    circuit.try_set_value(&battery, "5").unwrap();
    circuit
        .try_connect(&battery, "+", &transformer, "p1")
        .unwrap();
    circuit
        .try_connect(&transformer, "p2", &battery, "-")
        .unwrap();
    circuit.try_connect(&transformer, "s1", &load, "a").unwrap();
    circuit.try_connect(&load, "b", &transformer, "s2").unwrap();
    run_for(&mut circuit, 0.2);
    assert!(volts_on(&circuit, &load).abs() > 0.1, "a kick at switch-on");
    run_for(&mut circuit, 10.0);
    assert!(volts_on(&circuit, &load).abs() < 1e-3);
}

/// The old way to 5 V: 24 V peak at 50 Hz, the 2 : 1 transformer, a
/// bridge, a reservoir of `farads`, then `load_ohms` — through a 7805 if
/// `regulated`. Returns the circuit, the capacitor and the load.
fn power_supply(farads: &str, load_ohms: &str, regulated: bool) -> (Circuit, String, String) {
    let (mut circuit, transformer) = transformer_on_a_sine("24");
    let bridge = circuit.try_add("Bridge", 200.0, 0.0).unwrap();
    let capacitor = circuit.try_add("Electrolytic", 280.0, 0.0).unwrap();
    let load = circuit.try_add("Resistor", 400.0, 0.0).unwrap();
    circuit.try_set_value(&capacitor, farads).unwrap();
    circuit.try_set_value(&load, load_ohms).unwrap();
    circuit
        .try_connect(&transformer, "s1", &bridge, "ac1")
        .unwrap();
    circuit
        .try_connect(&transformer, "s2", &bridge, "ac2")
        .unwrap();
    circuit.try_connect(&bridge, "+", &capacitor, "+").unwrap();
    circuit.try_connect(&bridge, "-", &capacitor, "-").unwrap();
    if regulated {
        let regulator = circuit.try_add("Regulator", 340.0, 0.0).unwrap();
        circuit
            .try_connect(&capacitor, "+", &regulator, "in")
            .unwrap();
        circuit
            .try_connect(&regulator, "gnd", &capacitor, "-")
            .unwrap();
        circuit.try_connect(&regulator, "out", &load, "a").unwrap();
    } else {
        circuit.try_connect(&capacitor, "+", &load, "a").unwrap();
    }
    circuit.try_connect(&load, "b", &capacitor, "-").unwrap();
    (circuit, capacitor, load)
}

#[test]
fn a_bridge_turns_a_sine_into_pulses_both_ways_up() {
    // No capacitor to speak of: every half cycle comes out on top, at
    // the secondary's 12 V less two diodes — and never below nothing.
    let (mut circuit, _, load) = power_supply("1u", "1k", false);
    run_for(&mut circuit, 0.5);
    let (low, high) = swing_over(&mut circuit, &load, 0.1);
    assert!(low > -0.05, "{low} V at the bottom");
    assert!(high > 10.2 && high < 11.0, "{high} V at the top");
}

#[test]
fn a_reservoir_s_ripple_is_its_current_over_twice_the_mains_and_its_size() {
    // About 10 mA into 1 kΩ, the reservoir refilled 100 times a second:
    // at most I / (2f·C), which is the reservoir discharging for the
    // whole half cycle — 0.1 V on 1 000 µF. A real transformer's leakage
    // and copper stretch each charging pulse to a few milliseconds, so it
    // discharges for less, and the ripple is some three quarters of that.
    // Ten times less capacitance, close to ten times the ripple.
    let ripple = |farads: &str, microfarads: f64| {
        let (mut circuit, capacitor, _) = power_supply(farads, "1k", false);
        run_for(&mut circuit, 1.5);
        let (low, high) = swing_over(&mut circuit, &capacitor, 0.1);
        let amps = (low + high) / 2.0 / 1000.0;
        let bound = amps / (100.0 * microfarads * 1e-6);
        let ripple = high - low;
        assert!(
            ripple < bound && ripple > 0.6 * bound,
            "{farads}: {ripple} V of ripple, {bound} V at most"
        );
        ripple
    };
    let large = ripple("1000u", 1000.0);
    let small = ripple("100u", 100.0);
    assert!(small > 6.0 * large, "{small} V against {large} V");
}

#[test]
fn a_7805_after_the_bridge_holds_a_steady_five_volts() {
    // The reservoir's ripple goes into the 7805 and does not come out:
    // 25 mA into 200 Ω at 5 V, steady.
    let (mut circuit, capacitor, load) = power_supply("1000u", "200", true);
    run_for(&mut circuit, 1.5);
    let (low, high) = swing_over(&mut circuit, &capacitor, 0.1);
    assert!(high - low > 0.1, "{low}..{high} V on the reservoir");
    let (low, high) = swing_over(&mut circuit, &load, 0.1);
    assert!((low - 5.0).abs() < 0.1, "{low}..{high} V out");
    assert!(high - low < 0.02, "{low}..{high} V out");
}

/// A solar cell in `sun` with a resistor of `ohms` across it. Returns
/// the volts across the resistor.
fn panel_on(ohms: &str, sun: f64) -> f64 {
    let mut circuit = Circuit::new();
    let panel = circuit.try_add("SolarCell", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 120.0, 0.0).unwrap();
    circuit.try_set_value(&resistor, ohms).unwrap();
    circuit.try_set_wiper(&panel, sun).unwrap();
    circuit.try_connect(&panel, "+", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &panel, "-").unwrap();
    run_for(&mut circuit, 1.0 / 60.0);
    volts_on(&circuit, &resistor)
}

#[test]
fn a_solar_cell_gives_its_sheet_s_ends_in_full_sun() {
    // 1 Ω is a short to it: 105 mA is 0.105 V. 10 MΩ is open: 4.6 V.
    let short = panel_on("1", 1.0);
    assert!((short - 0.105).abs() < 1e-3, "{short} V on 1 Ω");
    let open = panel_on("10M", 1.0);
    assert!((open - 4.6).abs() < 0.01, "{open} V open");
}

#[test]
fn a_solar_cell_s_current_follows_the_light_and_its_voltage_barely() {
    for sun in [0.1, 0.3, 0.5, 0.9] {
        let short = panel_on("1", sun);
        assert!((short - 0.105 * sun).abs() < 1e-4, "{sun}: {short} V");
    }
    // A tenth of the sun still gives 4.1 V to nothing.
    let open = panel_on("10M", 0.1);
    assert!((open - 4.15).abs() < 0.02, "{open} V open at a tenth");
}

#[test]
fn a_load_finds_its_point_on_a_solar_cell_s_curve() {
    // 40 Ω is the sheet's best load, 4.0 V / 100 mA: there it gives
    // 0.4 W, and more or less resistance gives less.
    let watts = |ohms: f64| {
        let volts = panel_on(&ohms.to_string(), 1.0);
        volts * volts / ohms
    };
    let best = panel_on("40", 1.0);
    assert!((best - 4.0).abs() < 0.05, "{best} V on 40 Ω");
    assert!((watts(40.0) - 0.4).abs() < 0.01);
    assert!(watts(20.0) < watts(40.0) && watts(80.0) < watts(40.0));
    // Half the sun on the same 40 Ω: a current source's half, near enough,
    // because the load is now on the flat of its curve.
    let half = panel_on("40", 0.5);
    assert!((half - 2.1).abs() < 0.05, "{half} V at half sun");
}

/// A solar cell in `sun` charging 1 000 µF, through a 1N4148 or straight.
/// Returns the circuit, the panel and the capacitor.
fn sun_into_a_capacitor(through_a_diode: bool) -> (Circuit, String, String) {
    let mut circuit = Circuit::new();
    let panel = circuit.try_add("SolarCell", 0.0, 0.0).unwrap();
    let capacitor = circuit.try_add("Electrolytic", 240.0, 0.0).unwrap();
    circuit.try_set_value(&capacitor, "1000u").unwrap();
    if through_a_diode {
        let diode = circuit.try_add("Diode", 120.0, 0.0).unwrap();
        circuit.try_connect(&panel, "+", &diode, "anode").unwrap();
        circuit
            .try_connect(&diode, "cathode", &capacitor, "+")
            .unwrap();
    } else {
        circuit.try_connect(&panel, "+", &capacitor, "+").unwrap();
    }
    circuit.try_connect(&capacitor, "-", &panel, "-").unwrap();
    (circuit, panel, capacitor)
}

#[test]
fn a_diode_keeps_a_solar_cell_from_emptying_its_capacitor_at_night() {
    // By day both charge it; at night the panel is a diode across it, and
    // only the one with a diode of its own in the way keeps its charge.
    let night = |through_a_diode: bool| {
        let (mut circuit, panel, capacitor) = sun_into_a_capacitor(through_a_diode);
        run_for(&mut circuit, 2.0);
        let day = volts_on(&circuit, &capacitor);
        circuit.try_set_wiper(&panel, 0.0).unwrap();
        run_for(&mut circuit, 5.0);
        (day, volts_on(&circuit, &capacitor))
    };
    let (day, kept) = night(true);
    assert!(day > 3.8 && day < 4.6, "{day} V by day, through the diode");
    assert!(kept > 0.93 * day, "{kept} V kept of {day}");
    let (day, lost) = night(false);
    assert!(day > 4.5, "{day} V by day, straight");
    assert!(lost < day - 0.5, "{lost} V left of {day}");
}

/// A servo on 5 V, its signal pulled down by 10 kΩ and lifted by a
/// switch to the +: a contact the test flips a millisecond at a time,
/// which no finger could on a push-button (C-123). Returns the circuit,
/// the switch, the servo and the battery.
fn servoed(supplied: bool) -> (Circuit, String, String, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let button = circuit.try_add("Switch", 80.0, 0.0).unwrap();
    let pull = circuit.try_add("Resistor", 80.0, 80.0).unwrap();
    let servo = circuit.try_add("Servo", 200.0, 0.0).unwrap();
    circuit.try_set_value(&battery, "5").unwrap();
    circuit.try_set_value(&pull, "10k").unwrap();
    circuit.try_connect(&battery, "+", &button, "a").unwrap();
    circuit.try_connect(&button, "b", &servo, "SIG").unwrap();
    circuit.try_connect(&servo, "SIG", &pull, "a").unwrap();
    circuit.try_connect(&pull, "b", &battery, "-").unwrap();
    circuit.try_connect(&servo, "GND", &battery, "-").unwrap();
    if supplied {
        circuit.try_connect(&servo, "+5V", &battery, "+").unwrap();
    }
    (circuit, button, servo, battery)
}

/// One millisecond, the scheduler's step, with the button `held` or not.
fn tick(circuit: &mut Circuit, button: &str, held: bool) {
    circuit.try_set_closed(button, held).unwrap();
    circuit.advance(0.0).unwrap();
    circuit.simulation.as_mut().unwrap().run(1).unwrap();
}

/// `periods` of 20 ms, each high for `high` milliseconds.
fn pulse(circuit: &mut Circuit, button: &str, high: usize, periods: usize) {
    for _ in 0..periods {
        for step in 0..20 {
            tick(circuit, button, step < high);
        }
    }
}

fn horn(circuit: &Circuit, servo: &str) -> f64 {
    circuit
        .solution()
        .angles
        .into_iter()
        .find(|entry| entry.element_id == servo)
        .expect("a servo reports its angle")
        .degrees
}

#[test]
fn a_servo_turns_to_the_angle_its_pulses_ask_for() {
    // 1 ms is one end, 2 ms the other; a fresh one sits in the middle
    // (C-90).
    let (mut circuit, button, servo, _) = servoed(true);
    circuit.advance(0.0).unwrap();
    assert_eq!(horn(&circuit, &servo), 90.0);
    pulse(&mut circuit, &button, 1, 20);
    assert!(horn(&circuit, &servo) < 1.0, "{}", horn(&circuit, &servo));
    pulse(&mut circuit, &button, 2, 20);
    assert!(horn(&circuit, &servo) > 179.0, "{}", horn(&circuit, &servo));
    assert!(circuit.solution().failure.is_none());
    assert!(warnings(&circuit).is_empty(), "{:?}", warnings(&circuit));
}

#[test]
fn a_servo_turns_at_its_speed_and_holds_when_the_pulses_stop() {
    // From the middle to one end is 90° at 600° a second, 0.15 s: after
    // five periods, 0.1 s, it is on its way and not there.
    let (mut circuit, button, servo, _) = servoed(true);
    pulse(&mut circuit, &button, 2, 5);
    let partway = horn(&circuit, &servo);
    assert!(partway > 120.0 && partway < 175.0, "{partway}");
    pulse(&mut circuit, &button, 2, 10);
    let there = horn(&circuit, &servo);
    assert!(there > 179.0, "{there}");
    // No more pulses: it stays.
    pulse(&mut circuit, &button, 0, 25);
    assert_eq!(horn(&circuit, &servo), there);
}

#[test]
fn a_servo_draws_more_while_it_turns() {
    let (mut circuit, button, _, battery) = servoed(true);
    let drawn = |circuit: &Circuit| {
        circuit
            .solution()
            .element_currents
            .into_iter()
            .find(|entry| entry.element_id == battery)
            .unwrap()
            .amps
            .abs()
    };
    pulse(&mut circuit, &button, 2, 3);
    let turning = drawn(&circuit);
    pulse(&mut circuit, &button, 2, 20);
    pulse(&mut circuit, &button, 0, 2);
    let holding = drawn(&circuit);
    // 200 mA turning, 10 mA holding still, at 5 V.
    assert!((turning - 0.2).abs() < 0.01, "{turning} A turning");
    assert!((holding - 0.01).abs() < 0.001, "{holding} A holding");
}

#[test]
fn a_servo_without_its_supply_does_not_move_and_says_so() {
    let (mut circuit, button, servo, _) = servoed(false);
    pulse(&mut circuit, &button, 2, 20);
    assert_eq!(horn(&circuit, &servo), 90.0);
    assert_eq!(
        warnings(&circuit),
        ["This servo's +5V is not wired — wire it to the battery's +."]
    );
}

/// An L293D with a motor across 1Y and 2Y, its logic on a 5 V battery and
/// the motor on a 6 V one of its own, the two sharing −. 1A and 2A are
/// each a switch up to the logic's + with a 10 kΩ pull-down, and 1,2EN is
/// wired to the logic's + if `enabled`. `logic` and `motor` say whether
/// VCC1 and VCC2 are wired at all. Returns the circuit, the two switches,
/// the motor and the chip.
fn bridged(logic: bool, motor: bool, enabled: bool) -> (Circuit, [String; 2], String, String) {
    let mut circuit = Circuit::new();
    let five = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let six = circuit.try_add("DcSource", 0.0, 200.0).unwrap();
    let chip = circuit.try_add("MotorDriver", 200.0, 0.0).unwrap();
    let spinner = circuit.try_add("Motor", 400.0, 0.0).unwrap();
    circuit.try_set_value(&five, "5").unwrap();
    circuit.try_set_value(&six, "6").unwrap();
    circuit.try_connect(&six, "-", &five, "-").unwrap();
    circuit.try_connect(&chip, "GND", &five, "-").unwrap();
    if logic {
        circuit.try_connect(&chip, "VCC1", &five, "+").unwrap();
    }
    if motor {
        circuit.try_connect(&chip, "VCC2", &six, "+").unwrap();
    }
    if enabled {
        circuit.try_connect(&chip, "1,2EN", &five, "+").unwrap();
    }
    circuit.try_connect(&chip, "1Y", &spinner, "a").unwrap();
    circuit.try_connect(&chip, "2Y", &spinner, "b").unwrap();
    let switched = |circuit: &mut Circuit, input: &str| {
        let switch = circuit.try_add("Switch", 100.0, 0.0).unwrap();
        let pull = circuit.try_add("Resistor", 100.0, 80.0).unwrap();
        circuit.try_set_value(&pull, "10k").unwrap();
        circuit.try_connect(&five, "+", &switch, "a").unwrap();
        circuit.try_connect(&switch, "b", &chip, input).unwrap();
        circuit.try_connect(&chip, input, &pull, "a").unwrap();
        circuit.try_connect(&pull, "b", &five, "-").unwrap();
        switch
    };
    let switches = [switched(&mut circuit, "1A"), switched(&mut circuit, "2A")];
    (circuit, switches, spinner, chip)
}

/// Sets 1A and 2A and runs a frame. Returns whether the motor turns, and
/// whether backwards.
fn steer(circuit: &mut Circuit, switches: &[String; 2], one: bool, two: bool) -> (bool, bool) {
    circuit.try_set_closed(&switches[0], one).unwrap();
    circuit.try_set_closed(&switches[1], two).unwrap();
    circuit.advance(1.0 / 60.0).unwrap();
    let solution = circuit.solution();
    assert!(solution.failure.is_none(), "{:?}", solution.failure);
    let motor = circuit
        .views()
        .into_iter()
        .find(|view| view.kind == "Motor")
        .unwrap()
        .id;
    (
        solution.energised.contains(&motor),
        solution.reversed.contains(&motor),
    )
}

/// The potential of one of an element's pins.
fn pin_volts(circuit: &Circuit, id: &str, pin: &str) -> f64 {
    let element = circuit.netlist.element(decode(id).unwrap()).unwrap();
    let net = encode_net(element.pins[element.pin_index(pin).unwrap()].net.unwrap());
    circuit
        .solution()
        .net_voltages
        .into_iter()
        .find(|entry| entry.net_id == net)
        .map(|entry| entry.volts)
        .unwrap()
}

#[test]
fn a_driver_turns_its_motor_the_way_its_inputs_say() {
    // Forwards, backwards, and stopped with both ends low or both high
    // (C-91).
    let (mut circuit, switches, _, _) = bridged(true, true, true);
    assert_eq!(steer(&mut circuit, &switches, true, false), (true, false));
    assert_eq!(steer(&mut circuit, &switches, false, true), (true, true));
    assert_eq!(steer(&mut circuit, &switches, false, false), (false, false));
    assert_eq!(steer(&mut circuit, &switches, true, true), (false, false));
    assert!(warnings(&circuit).is_empty(), "{:?}", warnings(&circuit));
}

#[test]
fn a_driver_drops_what_its_sheet_says_across_its_outputs() {
    // 6 V less the high output's volt, across its 0.67 Ω, the motor's
    // 15 Ω and the low output's 2 Ω: 283 mA, and the motor sees 4.2 V of
    // the 6 — the volt and a half the bridge keeps is the price of it.
    use crate::solver::device::{driver, load};
    let (mut circuit, switches, spinner, chip) = bridged(true, true, true);
    steer(&mut circuit, &switches, true, false);
    let amps =
        (6.0 - driver::HIGH_DROP) / (driver::HIGH_OHMS + load::MOTOR_OHMS + driver::LOW_OHMS);
    let high = pin_volts(&circuit, &chip, "1Y");
    let low = pin_volts(&circuit, &chip, "2Y");
    assert!(
        (6.0 - high - (driver::HIGH_DROP + amps * driver::HIGH_OHMS)).abs() < 0.01,
        "{high}"
    );
    assert!((low - amps * driver::LOW_OHMS).abs() < 0.01, "{low}");
    let across = volts_on(&circuit, &spinner);
    assert!((across - 4.24).abs() < 0.02, "{across} V across the motor");
}

#[test]
fn a_driver_disabled_lets_its_motor_go() {
    let (mut circuit, switches, _, _) = bridged(true, true, false);
    assert_eq!(steer(&mut circuit, &switches, true, false), (false, false));
}

#[test]
fn a_driver_without_its_logic_supply_turns_nothing_and_says_so() {
    let (mut circuit, switches, _, _) = bridged(false, true, true);
    assert_eq!(steer(&mut circuit, &switches, true, false), (false, false));
    assert_eq!(
        warnings(&circuit),
        ["This L293D's VCC1 is not wired — wire it to the battery's +."]
    );
}

#[test]
fn a_driver_without_its_motor_supply_turns_nothing_and_says_so() {
    let (mut circuit, switches, _, _) = bridged(true, false, true);
    assert_eq!(steer(&mut circuit, &switches, true, false), (false, false));
    assert_eq!(
        warnings(&circuit),
        ["This L293D has nothing on VCC2 for its motors — wire it to the battery’s +."]
    );
}

#[test]
fn a_driver_clamps_an_output_pushed_past_its_supply() {
    // Its outputs let go, and 12 V through 100 Ω pushed onto 1Y: the
    // clamp diode up to VCC2 holds it a diode drop above the 6 V, which is
    // what it is there to do when a motor's coil kicks back.
    let (mut circuit, _, spinner, chip) = bridged(true, true, false);
    circuit.try_remove(&spinner).unwrap();
    let twelve = circuit.try_add("DcSource", 0.0, 400.0).unwrap();
    let resistor = circuit.try_add("Resistor", 200.0, 400.0).unwrap();
    circuit.try_set_value(&twelve, "12").unwrap();
    circuit.try_set_value(&resistor, "100").unwrap();
    circuit.try_connect(&twelve, "-", &chip, "GND").unwrap();
    circuit.try_connect(&twelve, "+", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &chip, "1Y").unwrap();
    circuit.advance(1.0 / 60.0).unwrap();
    let clamped = pin_volts(&circuit, &chip, "1Y") - 6.0;
    assert!((0.5..1.0).contains(&clamped), "{clamped} V over VCC2");
}

/// A 4N35 between two circuits: a 5 V battery lighting its LED through
/// `input` ohms and a switch, and a battery of `volts` driving a red LED
/// and 1 kΩ through its transistor. Nothing joins the two unless
/// `joined`, which wires the first battery's + to the second's −.
/// Returns the circuit, the switch, and the second circuit's resistor.
fn isolated(input: &str, volts: &str, joined: bool) -> (Circuit, String, String) {
    let mut circuit = Circuit::new();
    let five = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let switch = circuit.try_add("Switch", 80.0, 0.0).unwrap();
    let limit = circuit.try_add("Resistor", 160.0, 0.0).unwrap();
    let opto = circuit.try_add("Optocoupler", 240.0, 0.0).unwrap();
    circuit.try_set_value(&five, "5").unwrap();
    circuit.try_set_value(&limit, input).unwrap();
    circuit.try_connect(&five, "+", &switch, "a").unwrap();
    circuit.try_connect(&switch, "b", &limit, "a").unwrap();
    circuit.try_connect(&limit, "b", &opto, "A").unwrap();
    circuit.try_connect(&opto, "K", &five, "-").unwrap();

    let supply = circuit.try_add("DcSource", 400.0, 0.0).unwrap();
    let load = circuit.try_add("Resistor", 480.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 560.0, 0.0).unwrap();
    circuit.try_set_value(&supply, volts).unwrap();
    circuit.try_set_value(&load, "1k").unwrap();
    circuit.try_connect(&supply, "+", &load, "a").unwrap();
    circuit.try_connect(&load, "b", &led, "anode").unwrap();
    circuit.try_connect(&led, "cathode", &opto, "C").unwrap();
    circuit.try_connect(&opto, "E", &supply, "-").unwrap();
    if joined {
        circuit.try_connect(&five, "+", &supply, "-").unwrap();
    }
    (circuit, switch, load)
}

/// The current through the second circuit's 1 kΩ, with the switch set.
fn coupled_amps(circuit: &mut Circuit, switch: &str, closed: bool, load: &str) -> f64 {
    circuit.try_set_closed(switch, closed).unwrap();
    circuit.advance(1.0 / 60.0).unwrap();
    let solution = circuit.solution();
    assert!(solution.failure.is_none(), "{:?}", solution.failure);
    volts_on(circuit, load) / 1_000.0
}

#[test]
fn an_optocoupler_lets_a_5_v_circuit_switch_a_9_v_one() {
    // 11 mA through its LED could pass 11 mA; the 9 V side's 1 kΩ and
    // red LED take about 7, so the transistor is the switch, fully on
    // (C-92). Open, only its leak.
    let (mut circuit, switch, load) = isolated("330", "9", false);
    let off = coupled_amps(&mut circuit, &switch, false, &load);
    let on = coupled_amps(&mut circuit, &switch, true, &load);
    assert!(off < 1e-6, "{off} A dark");
    assert!((on - 7e-3).abs() < 0.5e-3, "{on} A lit");
    assert!(warnings(&circuit).is_empty(), "{:?}", warnings(&circuit));
    assert!(circuit.netlist.connectivity().floating_nets.is_empty());
}

#[test]
fn an_optocoupler_passes_its_transfer_ratio_at_the_sheet_s_test_point() {
    // 370 Ω from 5 V puts the sheet's 10 mA through its LED; a 1 kΩ load
    // on 22 V leaves 10 V across the transistor with 10 mA through it:
    // the sheet's 100 %, which is its least.
    let (mut circuit, switch, load) = isolated("370", "22", false);
    let collector = coupled_amps(&mut circuit, &switch, true, &load);
    let resistor = circuit
        .views()
        .into_iter()
        .find(|view| view.kind == "Resistor")
        .unwrap()
        .id;
    let led = volts_on(&circuit, &resistor) / 370.0;
    assert!((led - 10e-3).abs() < 0.2e-3, "{led} A through its LED");
    assert!(
        (collector / led - 1.0).abs() < 0.02,
        "{collector} A for {led}"
    );
}

#[test]
fn the_two_sides_of_an_optocoupler_need_not_share_a_ground() {
    // Apart, each side is a circuit with its own zero; joined, the second
    // sits 5 V up. Either way the same current crosses: only the signal
    // does, and it does not care where either side's zero is.
    let (mut apart, switch, load) = isolated("330", "9", false);
    let (mut joined, joined_switch, joined_load) = isolated("330", "9", true);
    let alone = coupled_amps(&mut apart, &switch, true, &load);
    let shifted = coupled_amps(&mut joined, &joined_switch, true, &joined_load);
    assert!((alone - shifted).abs() < 1e-9, "{alone} against {shifted}");
}

/// A relay whose coil is on a 9 V battery through a switch, and whose COM
/// and NO light a red LED through 330 Ω from a 5 V battery of their own.
/// Nothing joins the two circuits but the relay. Returns the circuit, the
/// switch and the LED.
fn relay_between_batteries() -> (Circuit, String, String) {
    let mut circuit = Circuit::new();
    let nine = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let switch = circuit.try_add("Switch", 80.0, 0.0).unwrap();
    let relay = circuit.try_add("Relay", 160.0, 0.0).unwrap();
    circuit.try_connect(&nine, "+", &switch, "a").unwrap();
    circuit.try_connect(&switch, "b", &relay, "coil+").unwrap();
    circuit.try_connect(&relay, "coil-", &nine, "-").unwrap();

    let five = circuit.try_add("DcSource", 300.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 380.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 460.0, 0.0).unwrap();
    circuit.try_set_value(&five, "5").unwrap();
    circuit.try_set_value(&resistor, "330").unwrap();
    circuit.try_connect(&five, "+", &relay, "com").unwrap();
    circuit.try_connect(&relay, "no", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
    circuit.try_connect(&led, "cathode", &five, "-").unwrap();
    (circuit, switch, led)
}

#[test]
fn a_relay_switches_a_circuit_on_a_battery_of_its_own() {
    let (mut circuit, switch, led) = relay_between_batteries();
    let lit = |circuit: &mut Circuit, closed: bool| {
        circuit.try_set_closed(&switch, closed).unwrap();
        for _ in 0..6 {
            circuit.advance(1.0 / 60.0).unwrap();
        }
        let solution = circuit.solution();
        assert!(solution.failure.is_none(), "{:?}", solution.failure);
        led_current(circuit, &led)
    };
    let off = lit(&mut circuit, false);
    let on = lit(&mut circuit, true);
    let again = lit(&mut circuit, false);
    // The contacts' circuit is its own, measured from its own battery's
    // −: before the relay declared sides it was hung off the coil's and
    // floated, its − at −1.75 V and its + at 3.25 V, the currents right
    // and every potential on it wrong.
    let five = circuit
        .views()
        .into_iter()
        .filter(|view| view.kind == "DcSource")
        .nth(1)
        .unwrap()
        .id;
    assert_eq!(pin_volts(&circuit, &five, "-"), 0.0);
    assert!((pin_volts(&circuit, &five, "+") - 5.0).abs() < 1e-9);
    assert_eq!(circuit.netlist.compile().unwrap().references().len(), 2);
    assert!(off < 1e-6, "{off} A before the coil is fed");
    // 5 V less a red LED's 1.8 V across 330 Ω: about 10 mA.
    assert!((on - 9.7e-3).abs() < 1e-3, "{on} A with the contact made");
    assert!(again < 1e-6, "{again} A once it lets go");
    assert!(warnings(&circuit).is_empty(), "{:?}", warnings(&circuit));
    assert!(circuit.netlist.connectivity().floating_nets.is_empty());
}

/// `chips` 74HC595s on a 5 V battery, chained QH′ to SER, each with eight
/// red LEDs through 330 Ω on QA to QH. Their SRCLK, RCLK, SRCLR and OE are
/// shared, and with the first one's SER each is a switch up to the + with
/// a 10 kΩ pull-down. Returns the circuit, the switches (data, shift,
/// latch, clear, enable) and each chip's LEDs, QA's first.
fn shifting(chips: usize) -> (Circuit, [String; 5], Vec<Vec<String>>) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    circuit.try_set_value(&battery, "5").unwrap();
    let ids: Vec<String> = (0..chips)
        .map(|chip| {
            let id = circuit
                .try_add("ShiftRegister", 200.0, 200.0 * chip as f64)
                .unwrap();
            circuit.try_connect(&id, "VCC", &battery, "+").unwrap();
            circuit.try_connect(&id, "GND", &battery, "-").unwrap();
            id
        })
        .collect();
    for pair in ids.windows(2) {
        circuit
            .try_connect(&pair[0], "QH'", &pair[1], "SER")
            .unwrap();
    }
    let mut switched = |input: &str| {
        let switch = circuit.try_add("Switch", 100.0, 0.0).unwrap();
        let pull = circuit.try_add("Resistor", 100.0, 80.0).unwrap();
        circuit.try_set_value(&pull, "10k").unwrap();
        circuit.try_connect(&battery, "+", &switch, "a").unwrap();
        circuit.try_connect(&battery, "-", &pull, "b").unwrap();
        circuit.try_connect(&switch, "b", &pull, "a").unwrap();
        let targets: Vec<&String> = if input == "SER" {
            ids.iter().take(1).collect()
        } else {
            ids.iter().collect()
        };
        for chip in targets {
            circuit.try_connect(&switch, "b", chip, input).unwrap();
        }
        switch
    };
    let switches = ["SER", "SRCLK", "RCLK", "SRCLR", "OE"].map(&mut switched);
    let leds = ids
        .iter()
        .map(|chip| {
            ["QA", "QB", "QC", "QD", "QE", "QF", "QG", "QH"]
                .iter()
                .map(|output| {
                    let resistor = circuit.try_add("Resistor", 300.0, 0.0).unwrap();
                    let led = circuit.try_add("Led", 400.0, 0.0).unwrap();
                    circuit.try_set_value(&resistor, "330").unwrap();
                    circuit.try_connect(chip, output, &resistor, "a").unwrap();
                    circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
                    circuit.try_connect(&led, "cathode", &battery, "-").unwrap();
                    led
                })
                .collect()
        })
        .collect();
    // Clear held high, out of the way; the outputs enabled.
    circuit.try_set_closed(&switches[3], true).unwrap();
    frame(&mut circuit);
    (circuit, switches, leds)
}

fn frame(circuit: &mut Circuit) {
    circuit.advance(1.0 / 60.0).unwrap();
    let solution = circuit.solution();
    assert!(solution.failure.is_none(), "{:?}", solution.failure);
}

/// A switch up for a frame and down for one: one rising edge.
fn tap(circuit: &mut Circuit, switch: &str) {
    circuit.try_set_closed(switch, true).unwrap();
    frame(circuit);
    circuit.try_set_closed(switch, false).unwrap();
    frame(circuit);
}

/// Clocks `bits` in, first first, through the data and shift switches.
fn shift_in(circuit: &mut Circuit, switches: &[String; 5], bits: &[bool]) {
    for &bit in bits {
        circuit.try_set_closed(&switches[0], bit).unwrap();
        frame(circuit);
        tap(circuit, &switches[1]);
    }
}

/// Which of `leds` are lit, QA in bit 0.
fn shown(circuit: &Circuit, leds: &[String]) -> u8 {
    leds.iter()
        .enumerate()
        .filter(|(_, led)| led_current(circuit, led) > 1e-3)
        .map(|(bit, _)| 1u8 << bit)
        .sum()
}

#[test]
fn a_shift_register_shows_a_byte_once_it_is_latched() {
    // 1, 0, 1, 1 in: nothing changes on the LEDs until RCLK rises, and
    // then QA, QB and QD light — the last bit in is QA's (C-93).
    let (mut circuit, switches, leds) = shifting(1);
    shift_in(&mut circuit, &switches, &[true, false, true, true]);
    assert_eq!(shown(&circuit, &leds[0]), 0, "not before the latch");
    tap(&mut circuit, &switches[2]);
    assert_eq!(shown(&circuit, &leds[0]), 0b1011);
    assert!(warnings(&circuit).is_empty(), "{:?}", warnings(&circuit));
}

#[test]
fn a_shift_register_drives_an_led_as_the_sheet_says() {
    // 5 V behind 33 Ω, then 330 Ω and a red LED's 1.8 V: about 9 mA.
    let (mut circuit, switches, leds) = shifting(1);
    shift_in(&mut circuit, &switches, &[true]);
    tap(&mut circuit, &switches[2]);
    let amps = led_current(&circuit, &leds[0][0]);
    assert!((amps - 8.8e-3).abs() < 0.5e-3, "{amps} A");
}

#[test]
fn clearing_empties_what_goes_in_and_not_what_is_shown() {
    let (mut circuit, switches, leds) = shifting(1);
    shift_in(&mut circuit, &switches, &[true, true, true]);
    tap(&mut circuit, &switches[2]);
    // SRCLR low, then up again, and latched: the LEDs went dark only on
    // the latch.
    circuit.try_set_closed(&switches[3], false).unwrap();
    frame(&mut circuit);
    circuit.try_set_closed(&switches[3], true).unwrap();
    frame(&mut circuit);
    assert_eq!(shown(&circuit, &leds[0]), 0b111, "still shown");
    tap(&mut circuit, &switches[2]);
    assert_eq!(shown(&circuit, &leds[0]), 0, "cleared, once latched");
}

#[test]
fn output_enable_high_turns_every_led_off_and_low_brings_them_back() {
    let (mut circuit, switches, leds) = shifting(1);
    shift_in(&mut circuit, &switches, &[true; 8]);
    tap(&mut circuit, &switches[2]);
    assert_eq!(shown(&circuit, &leds[0]), 0xff);
    circuit.try_set_closed(&switches[4], true).unwrap();
    frame(&mut circuit);
    assert_eq!(shown(&circuit, &leds[0]), 0, "OE high");
    circuit.try_set_closed(&switches[4], false).unwrap();
    frame(&mut circuit);
    assert_eq!(shown(&circuit, &leds[0]), 0xff, "and back");
}

#[test]
fn two_shift_registers_chained_hold_sixteen_bits() {
    // Sixteen bits in, through the first chip's QH′ into the second's
    // SER: the first eight in end up on the second chip.
    let (mut circuit, switches, leds) = shifting(2);
    let first: [bool; 8] = [true, true, false, false, false, false, false, true];
    let second: [bool; 8] = [false, true, false, true, false, true, false, true];
    shift_in(&mut circuit, &switches, &first);
    shift_in(&mut circuit, &switches, &second);
    tap(&mut circuit, &switches[2]);
    let byte = |bits: &[bool; 8]| {
        bits.iter()
            .rev()
            .enumerate()
            .filter(|(_, bit)| **bit)
            .map(|(at, _)| 1u8 << at)
            .sum::<u8>()
    };
    assert_eq!(shown(&circuit, &leds[0]), byte(&second));
    assert_eq!(shown(&circuit, &leds[1]), byte(&first));
}

/// A `kind` gate chip ("Gate" or "Inverter") on a battery of `volts`,
/// each of its first gate's inputs a switch up to the + with a 10 kΩ
/// pull-down, and its output lighting a red LED through 330 Ω. Returns
/// the circuit, the chip, the switches (A, then B for a quad) and the LED.
fn gated(kind: &str, volts: &str) -> (Circuit, String, Vec<String>, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    circuit.try_set_value(&battery, volts).unwrap();
    let chip = circuit.try_add(kind, 200.0, 0.0).unwrap();
    circuit.try_connect(&chip, "VCC", &battery, "+").unwrap();
    circuit.try_connect(&chip, "GND", &battery, "-").unwrap();
    let inputs: &[&str] = if kind == "Gate" {
        &["1A", "1B"]
    } else {
        &["1A"]
    };
    let switches = inputs
        .iter()
        .map(|input| {
            let switch = circuit.try_add("Switch", 100.0, 0.0).unwrap();
            let pull = circuit.try_add("Resistor", 100.0, 80.0).unwrap();
            circuit.try_set_value(&pull, "10k").unwrap();
            circuit.try_connect(&battery, "+", &switch, "a").unwrap();
            circuit.try_connect(&battery, "-", &pull, "b").unwrap();
            circuit.try_connect(&switch, "b", &pull, "a").unwrap();
            circuit.try_connect(&switch, "b", &chip, input).unwrap();
            switch
        })
        .collect();
    let resistor = circuit.try_add("Resistor", 300.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 400.0, 0.0).unwrap();
    circuit.try_set_value(&resistor, "330").unwrap();
    circuit.try_connect(&chip, "1Y", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
    circuit.try_connect(&led, "cathode", &battery, "-").unwrap();
    frame(&mut circuit);
    (circuit, chip, switches, led)
}

/// The LED's state for each of the four rows of a truth table, a frame
/// apart: a gate answers one step after its inputs change.
fn truth_table(circuit: &mut Circuit, switches: &[String], led: &str) -> [bool; 4] {
    [(false, false), (false, true), (true, false), (true, true)].map(|(a, b)| {
        circuit.try_set_closed(&switches[0], a).unwrap();
        circuit.try_set_closed(&switches[1], b).unwrap();
        frame(circuit);
        frame(circuit);
        led_current(circuit, led) > 1e-3
    })
}

#[test]
fn each_gate_lights_its_led_by_its_truth_table() {
    // Through a whole circuit: switches, pull-downs, the chip's outputs
    // and an LED (C-108).
    let (mut circuit, chip, switches, led) = gated("Gate", "5");
    for (logic, table) in [
        ("NAND", [true, true, true, false]),
        ("AND", [false, false, false, true]),
        ("OR", [false, true, true, true]),
        ("XOR", [false, true, true, false]),
    ] {
        circuit.try_set_choice(&chip, logic).unwrap();
        assert_eq!(truth_table(&mut circuit, &switches, &led), table, "{logic}");
    }
    assert!(warnings(&circuit).is_empty(), "{:?}", warnings(&circuit));
}

#[test]
fn an_inverter_lights_its_led_while_its_input_is_low() {
    let (mut circuit, _, switches, led) = gated("Inverter", "5");
    assert!(led_current(&circuit, &led) > 1e-3, "low in, high out");
    circuit.try_set_closed(&switches[0], true).unwrap();
    frame(&mut circuit);
    frame(&mut circuit);
    assert!(led_current(&circuit, &led) < 1e-6, "high in, low out");
}

#[test]
fn a_gate_drives_an_led_as_the_sheet_says() {
    // 5 V behind 50 Ω, then 330 Ω and a red LED's 1.8 V: about 8.4 mA.
    let (circuit, _, _, led) = gated("Gate", "5");
    let amps = led_current(&circuit, &led);
    assert!((amps - 8.4e-3).abs() < 0.5e-3, "{amps} A");
}

#[test]
fn two_nands_wired_in_a_loop_remember() {
    // The set–reset latch: each NAND's output on one of the other's
    // inputs, the other inputs held high by pull-ups and pulled low by a
    // button each. A press sets or resets it; let go, it stays.
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    circuit.try_set_value(&battery, "5").unwrap();
    let chip = circuit.try_add("Gate", 200.0, 0.0).unwrap();
    circuit.try_connect(&chip, "VCC", &battery, "+").unwrap();
    circuit.try_connect(&chip, "GND", &battery, "-").unwrap();
    circuit.try_connect(&chip, "1Y", &chip, "2A").unwrap();
    circuit.try_connect(&chip, "2Y", &chip, "1B").unwrap();
    let mut button = |input: &str| {
        let switch = circuit.try_add("Switch", 100.0, 0.0).unwrap();
        let pull = circuit.try_add("Resistor", 100.0, 80.0).unwrap();
        circuit.try_set_value(&pull, "10k").unwrap();
        circuit.try_connect(&battery, "+", &pull, "a").unwrap();
        circuit.try_connect(&pull, "b", &switch, "a").unwrap();
        circuit.try_connect(&switch, "b", &battery, "-").unwrap();
        circuit.try_connect(&switch, "a", &chip, input).unwrap();
        switch
    };
    let set = button("1A");
    let reset = button("2B");
    let resistor = circuit.try_add("Resistor", 300.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 400.0, 0.0).unwrap();
    circuit.try_set_value(&resistor, "330").unwrap();
    circuit.try_connect(&chip, "1Y", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
    circuit.try_connect(&led, "cathode", &battery, "-").unwrap();
    let press = |circuit: &mut Circuit, switch: &str| {
        circuit.try_set_closed(switch, true).unwrap();
        for _ in 0..3 {
            frame(circuit);
        }
        circuit.try_set_closed(switch, false).unwrap();
        for _ in 0..3 {
            frame(circuit);
        }
    };
    press(&mut circuit, &set);
    assert!(led_current(&circuit, &led) > 1e-3, "set, and held");
    press(&mut circuit, &reset);
    assert!(led_current(&circuit, &led) < 1e-6, "reset, and held");
    press(&mut circuit, &set);
    assert!(led_current(&circuit, &led) > 1e-3, "set again");
}

#[test]
fn a_used_gate_with_a_free_input_is_said() {
    let (mut circuit, chip, switches, _) = gated("Gate", "5");
    // 1B's switch and pull-down both gone.
    let pull = circuit
        .netlist
        .elements()
        .filter(
            |(_, element)| matches!(element.kind, ElementKind::Resistor { ohms } if ohms == 10e3),
        )
        .map(|(id, _)| encode(id))
        .nth(1)
        .unwrap();
    circuit.try_remove(&switches[1]).unwrap();
    circuit.try_remove(&pull).unwrap();
    frame(&mut circuit);
    assert_eq!(
        warnings(&circuit),
        [
            "This 74HC00's input 1B is not wired — a free input reads whatever is near it; tie it to + or −."
        ]
    );
    // An unused gate's free inputs are nobody's business.
    circuit.try_set_choice(&chip, "OR").unwrap();
    frame(&mut circuit);
    assert_eq!(warnings(&circuit).len(), 1, "{:?}", warnings(&circuit));
}

#[test]
fn a_gate_on_a_9_volt_battery_is_told_to_have_5() {
    let (circuit, _, _, _) = gated("Inverter", "9");
    assert_eq!(
        warnings(&circuit),
        ["This 74HC04 runs on 2 to 6 V and has 9.0 V — give it 5 V, not the battery."]
    );
}

/// What each pin of `id` pushes into its wire, by pin name, as the
/// solution reports it.
fn pushed(circuit: &Circuit, id: &str) -> Vec<(String, f64)> {
    let mut found: Vec<(String, f64)> = circuit
        .solution()
        .pin_currents
        .into_iter()
        .filter(|current| current.pin.element_id == id)
        .map(|current| (current.pin.pin, current.amps))
        .collect();
    found.sort_by(|a, b| a.0.cmp(&b.0));
    found
}

/// An NPN, its base fed through 10 kΩ from 9 V, switching a 9 V relay's
/// coil in its collector. Returns the circuit, the transistor and the
/// relay.
fn transistor_on_a_coil() -> (Circuit, String, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let base = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
    let npn = circuit.try_add("Bjt", 160.0, 0.0).unwrap();
    let relay = circuit.try_add("Relay", 160.0, -80.0).unwrap();
    circuit.try_set_value(&base, "10k").unwrap();
    circuit.try_connect(&battery, "+", &base, "a").unwrap();
    circuit.try_connect(&base, "b", &npn, "b").unwrap();
    circuit.try_connect(&battery, "+", &relay, "coil+").unwrap();
    circuit.try_connect(&relay, "coil-", &npn, "c").unwrap();
    circuit.try_connect(&npn, "e", &battery, "-").unwrap();
    (circuit, npn, relay)
}

#[test]
fn a_wire_between_a_transistor_and_a_coil_carries_its_current() {
    // C-101: neither end is a two-pin part, so the coil–collector wire
    // had no current to draw though the relay had pulled in.
    let (mut circuit, npn, relay) = transistor_on_a_coil();
    run_for(&mut circuit, 0.1);
    assert!(circuit.solution().energised.contains(&relay), "pulled in");
    let collector = pushed(&circuit, &npn)
        .into_iter()
        .find(|(pin, _)| pin == "c")
        .unwrap()
        .1;
    let coil = pushed(&circuit, &relay)
        .into_iter()
        .find(|(pin, _)| pin == "coil-")
        .unwrap()
        .1;
    // Out of the coil into the wire, and the same into the collector.
    assert!(coil > 0.03, "{coil} A out of the coil");
    assert!(
        (coil + collector).abs() < 1e-6 * coil.abs().max(1.0),
        "{coil} against {collector}"
    );
}

#[test]
fn what_enters_a_transistor_leaves_it() {
    let (mut circuit, npn, _) = transistor_on_a_coil();
    run_for(&mut circuit, 0.1);
    let pins = pushed(&circuit, &npn);
    assert_eq!(pins.len(), 3, "{pins:?}");
    let total: f64 = pins.iter().map(|(_, amps)| amps).sum();
    assert!(total.abs() < 1e-9, "{pins:?}");
}

#[test]
fn a_wiper_straight_into_a_base_carries_the_base_current() {
    // A potentiometer across 9 V, its wiper into an NPN's base, an LED
    // and 470 Ω in the collector.
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let knob = circuit.try_add("Potentiometer", 80.0, 0.0).unwrap();
    let npn = circuit.try_add("Bjt", 160.0, 0.0).unwrap();
    let load = circuit.try_add("Resistor", 160.0, -80.0).unwrap();
    let led = circuit.try_add("Led", 240.0, -80.0).unwrap();
    circuit.try_set_value(&load, "470").unwrap();
    circuit.try_connect(&battery, "+", &knob, "a").unwrap();
    circuit.try_connect(&knob, "b", &battery, "-").unwrap();
    circuit.try_connect(&knob, "wiper", &npn, "b").unwrap();
    circuit.try_connect(&battery, "+", &load, "a").unwrap();
    circuit.try_connect(&load, "b", &led, "anode").unwrap();
    circuit.try_connect(&led, "cathode", &npn, "c").unwrap();
    circuit.try_connect(&npn, "e", &battery, "-").unwrap();
    circuit.advance(1.0 / 60.0).unwrap();
    let wiper = pushed(&circuit, &knob)
        .into_iter()
        .find(|(pin, _)| pin == "wiper")
        .unwrap()
        .1;
    let base = pushed(&circuit, &npn)
        .into_iter()
        .find(|(pin, _)| pin == "b")
        .unwrap()
        .1;
    assert!(wiper > 1e-6, "{wiper} A out of the wiper");
    assert!((wiper + base).abs() < 1e-9, "{wiper} against {base}");
    // And the knob's three pins balance too.
    let total: f64 = pushed(&circuit, &knob).iter().map(|(_, amps)| amps).sum();
    assert!(total.abs() < 1e-9);
}

#[test]
fn a_wire_s_arrow_does_not_depend_on_which_end_was_wired_first() {
    // A resistor into a base, wired from either end.
    let arrow = |resistor_first: bool| {
        let mut circuit = Circuit::new();
        let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
        let base = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
        let npn = circuit.try_add("Bjt", 160.0, 0.0).unwrap();
        circuit.try_set_value(&base, "10k").unwrap();
        circuit.try_connect(&battery, "+", &base, "a").unwrap();
        if resistor_first {
            circuit.try_connect(&base, "b", &npn, "b").unwrap();
        } else {
            circuit.try_connect(&npn, "b", &base, "b").unwrap();
        }
        circuit.try_connect(&npn, "e", &battery, "-").unwrap();
        circuit.try_connect(&npn, "c", &battery, "+").unwrap();
        circuit.advance(1.0 / 60.0).unwrap();
        let base_net = circuit.netlist.element(decode(&npn).unwrap()).unwrap().pins[0]
            .net
            .unwrap();
        circuit
            .solution()
            .net_currents
            .into_iter()
            .find(|current| current.net_id == encode_net(base_net))
            .map(|current| (current.from.pin, current.to.pin, current.amps))
    };
    let one = arrow(true).expect("an arrow, resistor first");
    let other = arrow(false).expect("an arrow, transistor first");
    assert_eq!((&one.0, &one.1), (&other.0, &other.1));
    assert!((one.2 - other.2).abs() < 1e-12);
}

#[test]
fn a_servo_under_its_four_volts_says_so() {
    // C-102: it does not move under 4 V, and used to say nothing.
    let (mut circuit, button, _, battery) = servoed(true);
    circuit.try_set_value(&battery, "3").unwrap();
    pulse(&mut circuit, &button, 2, 2);
    assert_eq!(
        warnings(&circuit),
        ["This servo needs at least 4 V across +5V and GND."]
    );
}

#[test]
fn an_l293d_s_logic_under_four_and_a_half_volts_says_so() {
    let (mut circuit, switches, _, _) = bridged(true, true, true);
    let five = circuit
        .views()
        .into_iter()
        .find(|view| view.kind == "DcSource")
        .unwrap()
        .id;
    circuit.try_set_value(&five, "3.3").unwrap();
    steer(&mut circuit, &switches, true, false);
    assert_eq!(
        warnings(&circuit),
        ["This L293D needs at least 4.5 V across VCC1 and GND."]
    );
}

#[test]
fn a_chip_powered_the_wrong_way_round_is_told_so_and_not_that_it_has_no_power() {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let timer = circuit.try_add("Timer", 100.0, 0.0).unwrap();
    circuit.try_connect(&timer, "VCC", &battery, "-").unwrap();
    circuit.try_connect(&timer, "GND", &battery, "+").unwrap();
    circuit.advance(1.0 / 60.0).unwrap();
    assert_eq!(
        warnings(&circuit),
        ["This 555 is powered the wrong way round — swap its VCC and GND."]
    );
}

/// A 7805 fed from a battery of `volts` through its input, if `fed`, with
/// 1 kΩ on its output.
fn regulating(volts: &str, fed: bool) -> Circuit {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let regulator = circuit.try_add("Regulator", 100.0, 0.0).unwrap();
    let load = circuit.try_add("Resistor", 200.0, 0.0).unwrap();
    circuit.try_set_value(&battery, volts).unwrap();
    circuit.try_set_value(&load, "1k").unwrap();
    if fed {
        circuit
            .try_connect(&battery, "+", &regulator, "in")
            .unwrap();
    }
    circuit
        .try_connect(&regulator, "gnd", &battery, "-")
        .unwrap();
    circuit.try_connect(&regulator, "out", &load, "a").unwrap();
    circuit.try_connect(&load, "b", &battery, "-").unwrap();
    circuit.advance(1.0 / 60.0).unwrap();
    circuit
}

#[test]
fn a_regulator_below_its_dropout_says_what_it_needs_and_what_it_gives() {
    // A note, not a warning: it works as the sheet says, a few volts short.
    let circuit = regulating("6", true);
    let findings = circuit.solution().findings;
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].severity, "note");
    assert!(
        findings[0]
            .message
            .starts_with("This 7805 needs 7 V in to give 5 V out; on 6.0 V"),
        "{}",
        findings[0].message
    );
    assert!(
        regulating("9", true).solution().findings.is_empty(),
        "on 9 V, nothing to say"
    );
}

#[test]
fn a_regulator_with_nothing_on_its_input_says_so() {
    assert_eq!(
        warnings(&regulating("9", false)),
        ["This 7805 has nothing on its input — wire in to the battery's +."]
    );
}

#[test]
fn a_digit_of_the_other_common_type_is_told_which_way_its_common_goes() {
    // A common-anode digit wired as a common-cathode one would be: dark,
    // and now said.
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
    let digit = circuit.try_add("SevenSegment", 160.0, 0.0).unwrap();
    circuit.try_set_choice(&digit, "Common anode").unwrap();
    circuit.try_set_value(&resistor, "330").unwrap();
    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &digit, "a").unwrap();
    circuit.try_connect(&digit, "com", &battery, "-").unwrap();
    circuit.advance(1.0 / 60.0).unwrap();
    assert_eq!(
        warnings(&circuit),
        [
            "This digit is wired the wrong way round: it is common anode, so its common pin goes to +."
        ]
    );
    // And wired as its own type, nothing.
    let (mut right, _) = one_segment("Common anode");
    right.advance(1.0 / 60.0).unwrap();
    assert!(warnings(&right).is_empty(), "{:?}", warnings(&right));
}

#[test]
fn a_signal_generator_in_one_circuit_does_not_excuse_a_reversed_led_in_another() {
    // Before C-102 one generator anywhere silenced the rule everywhere.
    let (mut circuit, led) = reversed_led();
    let generator = circuit.try_add("SignalGenerator", 0.0, 200.0).unwrap();
    let other = circuit.try_add("Led", 100.0, 200.0).unwrap();
    let resistor = circuit.try_add("Resistor", 200.0, 200.0).unwrap();
    circuit
        .try_connect(&generator, "+", &resistor, "a")
        .unwrap();
    circuit
        .try_connect(&resistor, "b", &other, "anode")
        .unwrap();
    circuit
        .try_connect(&other, "cathode", &generator, "-")
        .unwrap();
    circuit.advance(1.0 / 60.0).unwrap();
    let findings = circuit.solution().findings;
    assert!(
        findings
            .iter()
            .any(|finding| finding.elements == [led.clone()]
                && finding.message.contains("wrong way round")),
        "{findings:?}"
    );
}

/// An op-amp or comparator on a 9 V battery with in+ on its + and in− on
/// its − (or the other way round if not `high`), and, if given, a load of
/// `ohms` from its output to the battery's `to` terminal. Returns the
/// circuit, the part and the load.
fn driven(kind: &str, high: bool, load: Option<(&str, &str)>) -> (Circuit, String, Option<String>) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let part = circuit.try_add(kind, 100.0, 0.0).unwrap();
    circuit.try_connect(&part, "V+", &battery, "+").unwrap();
    circuit.try_connect(&part, "V-", &battery, "-").unwrap();
    let (plus, minus) = if high { ("+", "-") } else { ("-", "+") };
    circuit.try_connect(&part, "in+", &battery, plus).unwrap();
    circuit.try_connect(&part, "in-", &battery, minus).unwrap();
    let resistor = load.map(|(ohms, to)| {
        let resistor = circuit.try_add("Resistor", 200.0, 0.0).unwrap();
        circuit.try_set_value(&resistor, ohms).unwrap();
        circuit.try_connect(&part, "out", &resistor, "a").unwrap();
        circuit.try_connect(&resistor, "b", &battery, to).unwrap();
        resistor
    });
    circuit.advance(1.0 / 60.0).unwrap();
    assert!(
        circuit.solution().failure.is_none(),
        "{:?}",
        circuit.solution().failure
    );
    (circuit, part, resistor)
}

#[test]
fn a_comparator_with_nothing_on_its_output_reads_its_v_plus() {
    // C-105, holding comparator.rs: let go with no pull-up, a number is
    // the least wrong thing to show, and it is V+.
    // Read by a voltmeter, which takes nothing.
    let (mut circuit, part, _) = driven("Comparator", true, None);
    let meter = circuit.try_add("Voltmeter", 200.0, 0.0).unwrap();
    let battery = circuit
        .views()
        .into_iter()
        .find(|view| view.kind == "DcSource")
        .unwrap()
        .id;
    circuit.try_connect(&part, "out", &meter, "+").unwrap();
    circuit.try_connect(&meter, "-", &battery, "-").unwrap();
    circuit.advance(1.0 / 60.0).unwrap();
    assert!((pin_volts(&circuit, &part, "out") - 9.0).abs() < 0.01);
}

#[test]
fn a_comparator_never_sources_current() {
    // Let go, into 1 kΩ to −: open collector, a microamp of leak at most.
    let (circuit, _, load) = driven("Comparator", true, Some(("1k", "-")));
    let amps = volts_on(&circuit, &load.unwrap()) / 1e3;
    assert!(amps.abs() < 2e-6, "{amps} A sourced");
}

#[test]
fn a_comparator_sinks_no_more_than_its_sheet_s_16_ma() {
    // Pulled down against 100 Ω from 9 V, which would be 90 mA.
    let (circuit, _, load) = driven("Comparator", false, Some(("100", "+")));
    let amps = volts_on(&circuit, &load.unwrap()).abs() / 100.0;
    assert!((amps - 16e-3).abs() < 1.5e-3, "{amps} A sunk");
}

#[test]
fn a_comparator_pulls_down_to_about_a_quarter_of_a_volt_at_4_ma() {
    // 9 V through 2.2 kΩ is about 4 mA: the sheet's 0.25 V.
    let (circuit, part, _) = driven("Comparator", false, Some(("2.2k", "+")));
    let low = pin_volts(&circuit, &part, "out");
    assert!((low - 0.26).abs() < 0.05, "{low} V");
}

#[test]
fn an_op_amp_stops_short_of_its_rails_by_what_its_sheet_promises() {
    // Driven high, a volt and a half under V+; driven low, lightly loaded,
    // within a few tens of millivolts of V−.
    let (high, part, _) = driven("OpAmp", true, Some(("100k", "-")));
    let top = pin_volts(&high, &part, "out");
    assert!((top - 7.5).abs() < 0.05, "{top} V high");
    let (low, part, _) = driven("OpAmp", false, Some(("100k", "+")));
    let bottom = pin_volts(&low, &part, "out");
    assert!(bottom < 0.05, "{bottom} V low");
}

#[test]
fn an_op_amp_sources_no_more_than_30_ma() {
    // Driven high into 10 Ω, which would be 750 mA.
    let (circuit, _, load) = driven("OpAmp", true, Some(("10", "-")));
    let amps = volts_on(&circuit, &load.unwrap()) / 10.0;
    assert!((amps - 30e-3).abs() < 3e-3, "{amps} A sourced");
}

#[test]
fn an_op_amp_s_inputs_take_what_10_megohms_would() {
    // in+ fed from 9 V through 10 MΩ, in− on −: the input's own 10 MΩ
    // makes the other half of a divider, and in+ sits near 4.5 V.
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let part = circuit.try_add("OpAmp", 100.0, 0.0).unwrap();
    let feed = circuit.try_add("Resistor", 50.0, 0.0).unwrap();
    circuit.try_set_value(&feed, "10M").unwrap();
    circuit.try_connect(&part, "V+", &battery, "+").unwrap();
    circuit.try_connect(&part, "V-", &battery, "-").unwrap();
    circuit.try_connect(&battery, "+", &feed, "a").unwrap();
    circuit.try_connect(&feed, "b", &part, "in+").unwrap();
    circuit.try_connect(&part, "in-", &battery, "-").unwrap();
    // Its output wired too: a part is in the circuit once every pin is.
    let load = circuit.try_add("Resistor", 200.0, 0.0).unwrap();
    circuit.try_set_value(&load, "100k").unwrap();
    circuit.try_connect(&part, "out", &load, "a").unwrap();
    circuit.try_connect(&load, "b", &battery, "-").unwrap();
    circuit.advance(1.0 / 60.0).unwrap();
    let input = pin_volts(&circuit, &part, "in+");
    assert!((input - 4.5).abs() < 0.1, "{input} V at in+");
}

/// The note a passive buzzer is sounding, if it is.
fn tone(circuit: &Circuit, buzzer: &str) -> Option<f64> {
    circuit
        .solution()
        .tones
        .into_iter()
        .find(|entry| entry.element_id == buzzer)
        .map(|entry| entry.hertz)
}

#[test]
fn a_passive_buzzer_sounds_the_note_it_is_fed() {
    // A signal generator straight across it, at three notes and two
    // shapes: what it reports is what it was fed (C-85).
    for (waveform, hertz) in [("Square", 440.0), ("Square", 1000.0), ("Sine", 262.0)] {
        let mut circuit = Circuit::new();
        let generator = circuit.try_add("SignalGenerator", 0.0, 0.0).unwrap();
        let buzzer = circuit.try_add("PassiveBuzzer", 80.0, 0.0).unwrap();
        circuit.try_set_choice(&generator, waveform).unwrap();
        circuit
            .try_set_frequency(&generator, &hertz.to_string())
            .unwrap();
        circuit.try_connect(&generator, "+", &buzzer, "+").unwrap();
        circuit.try_connect(&buzzer, "-", &generator, "-").unwrap();
        run_for(&mut circuit, 0.25);
        let heard = tone(&circuit, &buzzer).expect("a note");
        assert!(
            (heard / hertz - 1.0).abs() < 0.01,
            "{waveform} at {hertz} Hz heard as {heard}"
        );
        assert!(circuit.solution().energised.contains(&buzzer));
    }
}

#[test]
fn a_passive_buzzer_on_dc_is_silent() {
    // Where an active buzzer would sound: it has no oscillator of its own.
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let buzzer = circuit.try_add("PassiveBuzzer", 80.0, 0.0).unwrap();
    circuit.try_set_value(&battery, "5").unwrap();
    circuit.try_connect(&battery, "+", &buzzer, "+").unwrap();
    circuit.try_connect(&buzzer, "-", &battery, "-").unwrap();
    run_for(&mut circuit, 0.5);
    assert_eq!(tone(&circuit, &buzzer), None);
    assert!(!circuit.solution().energised.contains(&buzzer));
    // And it is its coil: 5 V across 16 Ω.
    let amps = circuit
        .solution()
        .element_currents
        .into_iter()
        .find(|entry| entry.element_id == buzzer)
        .unwrap()
        .amps;
    assert!((amps - 5.0 / 16.0).abs() < 1e-9, "{amps}");
}

/// A 555 on 5 V, astable on 1 kΩ, `rb` and 100 nF, its output through
/// 1 kΩ into an NPN's base, and a passive buzzer with 100 Ω from the +
/// to the collector, as a kit drives one. Returns the circuit, the
/// buzzer and the timing resistor.
fn organ(rb: &str) -> (Circuit, String, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    circuit.try_set_value(&battery, "5").unwrap();
    let timer = circuit.try_add("Timer", 200.0, 0.0).unwrap();
    let ra = circuit.try_add("Resistor", 100.0, 0.0).unwrap();
    let timing = circuit.try_add("Resistor", 100.0, 80.0).unwrap();
    let capacitor = circuit.try_add("Capacitor", 100.0, 160.0).unwrap();
    let base = circuit.try_add("Resistor", 300.0, 0.0).unwrap();
    let transistor = circuit.try_add("Bjt", 400.0, 0.0).unwrap();
    let limit = circuit.try_add("Resistor", 400.0, -80.0).unwrap();
    let buzzer = circuit.try_add("PassiveBuzzer", 400.0, -160.0).unwrap();
    circuit.try_set_value(&ra, "1k").unwrap();
    circuit.try_set_value(&timing, rb).unwrap();
    circuit.try_set_value(&capacitor, "100n").unwrap();
    circuit.try_set_value(&base, "1k").unwrap();
    circuit.try_set_value(&limit, "100").unwrap();
    circuit.try_connect(&timer, "VCC", &battery, "+").unwrap();
    circuit.try_connect(&timer, "GND", &battery, "-").unwrap();
    circuit.try_connect(&ra, "a", &battery, "+").unwrap();
    circuit.try_connect(&ra, "b", &timer, "DIS").unwrap();
    circuit.try_connect(&timing, "a", &timer, "DIS").unwrap();
    circuit.try_connect(&timing, "b", &timer, "THR").unwrap();
    circuit.try_connect(&timer, "TRIG", &timer, "THR").unwrap();
    circuit.try_connect(&capacitor, "a", &timer, "THR").unwrap();
    circuit.try_connect(&capacitor, "b", &battery, "-").unwrap();
    circuit.try_connect(&timer, "OUT", &base, "a").unwrap();
    circuit.try_connect(&base, "b", &transistor, "b").unwrap();
    circuit
        .try_connect(&transistor, "e", &battery, "-")
        .unwrap();
    circuit.try_connect(&battery, "+", &limit, "a").unwrap();
    circuit.try_connect(&limit, "b", &buzzer, "+").unwrap();
    circuit.try_connect(&buzzer, "-", &transistor, "c").unwrap();
    (circuit, buzzer, timing)
}

#[test]
fn a_555_plays_its_own_frequency_on_a_passive_buzzer() {
    // 1 / (ln 2 · (1 k + 2 · 15 k) · 100 n) = 465 Hz, the textbook's
    // formula; the 555 here is its thresholds and latch, so it should
    // land close to it, and what the buzzer hears is what it makes.
    let (mut circuit, buzzer, _) = organ("15k");
    run_for(&mut circuit, 0.3);
    let expected = 1.0 / (std::f64::consts::LN_2 * 31e3 * 100e-9);
    let heard = tone(&circuit, &buzzer).expect("a note");
    assert!(
        (heard / expected - 1.0).abs() < 0.05,
        "heard {heard} Hz against the formula's {expected}"
    );
}

#[test]
fn a_passive_buzzer_s_note_follows_the_circuit() {
    // A larger timing resistor, a lower note — read off the run, with
    // nothing anywhere set to a frequency.
    let (mut circuit, buzzer, timing) = organ("15k");
    run_for(&mut circuit, 0.3);
    let high = tone(&circuit, &buzzer).unwrap();
    circuit.try_set_value(&timing, "33k").unwrap();
    run_for(&mut circuit, 0.3);
    let low = tone(&circuit, &buzzer).unwrap();
    assert!(low < high * 0.6, "{high} Hz, then {low}");
}

/// How bright an LED of a colour is off 9 V through 330 Ω.
fn alone(colour: &str) -> f64 {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 160.0, 0.0).unwrap();
    circuit.try_set_choice(&led, colour).unwrap();
    circuit.try_set_value(&resistor, "330").unwrap();
    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
    circuit.try_connect(&led, "cathode", &battery, "-").unwrap();
    circuit.advance(1.0 / 60.0).unwrap();
    circuit
        .solution()
        .element_currents
        .into_iter()
        .find(|entry| entry.element_id == led)
        .and_then(|entry| entry.brightness)
        .unwrap()
}

/// An RGB LED off 9 V, each of the named dice through 330 Ω of its own
/// and the rest left free, the other way round for a common-anode one.
/// Returns the circuit and the LED.
fn rgb(common: &str, dice: &[&str]) -> (Circuit, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let led = circuit.try_add("RgbLed", 160.0, 0.0).unwrap();
    circuit.try_set_choice(&led, common).unwrap();
    let (to_die, to_common) = if common == "Common cathode" {
        ("+", "-")
    } else {
        ("-", "+")
    };
    for die in dice {
        let resistor = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
        circuit.try_set_value(&resistor, "330").unwrap();
        circuit
            .try_connect(&battery, to_die, &resistor, "a")
            .unwrap();
        circuit.try_connect(&resistor, "b", &led, die).unwrap();
    }
    circuit
        .try_connect(&led, "com", &battery, to_common)
        .unwrap();
    circuit.advance(1.0 / 60.0).unwrap();
    (circuit, led)
}

#[test]
fn each_die_of_an_rgb_led_is_as_bright_as_the_led_of_its_colour() {
    // Red, green and blue through 330 Ω off 9 V, each lit on its own and
    // all three at once, either way round: each die the LED of its colour
    // alone, because that is what it is (C-84).
    let expected = [alone("Red"), alone("Green"), alone("Blue")];
    for common in ["Common cathode", "Common anode"] {
        for (die, name) in ["R", "G", "B"].iter().enumerate() {
            let (circuit, led) = rgb(common, &[name]);
            assert!(circuit.solution().failure.is_none());
            let dice = glow(&circuit, &led);
            assert_eq!(dice.len(), 3);
            for (other, &brightness) in dice.iter().enumerate() {
                if other == die {
                    assert!(
                        (brightness - expected[die]).abs() < 1e-9,
                        "{common}, {name}: {brightness} against {}",
                        expected[die]
                    );
                } else {
                    assert_eq!(brightness, 0.0, "{common}, {name} alone: {dice:?}");
                }
            }
        }
        let (circuit, led) = rgb(common, &["R", "G", "B"]);
        let dice = glow(&circuit, &led);
        for (brightness, expected) in dice.iter().zip(expected) {
            assert!((brightness - expected).abs() < 1e-9, "{common}: {dice:?}");
        }
    }
}

#[test]
fn an_rgb_led_s_red_die_lights_from_less_than_the_other_two() {
    // Its dice are the single LEDs', drops and all: two cells light the
    // red, not the blue — which is why a kit gives the red its own,
    // larger resistor (C-84).
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    circuit.try_set_value(&battery, "2.4").unwrap();
    let led = circuit.try_add("RgbLed", 160.0, 0.0).unwrap();
    for die in ["R", "B"] {
        let resistor = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
        circuit.try_set_value(&resistor, "47").unwrap();
        circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
        circuit.try_connect(&resistor, "b", &led, die).unwrap();
    }
    circuit.try_connect(&led, "com", &battery, "-").unwrap();
    circuit.advance(1.0 / 60.0).unwrap();
    let dice = glow(&circuit, &led);
    assert!(dice[0] > 0.5, "red: {dice:?}");
    assert!(dice[2] < 0.05, "blue: {dice:?}");
}

#[test]
fn an_rgb_led_wired_the_wrong_way_round_stays_dark() {
    let (mut circuit, led) = rgb("Common cathode", &["R", "G", "B"]);
    circuit.try_set_choice(&led, "Common anode").unwrap();
    circuit.advance(1.0 / 60.0).unwrap();
    assert!(glow(&circuit, &led).iter().all(|&dark| dark < 1e-3));
}

#[test]
fn an_rgb_led_gives_back_at_its_common_what_its_dice_take() {
    // Three currents in, their sum out, so the dots run on (C-73).
    let (circuit, led) = rgb("Common cathode", &["R", "G", "B"]);
    let solution = circuit.solution();
    let into = |pin: &str| {
        solution
            .pin_currents
            .iter()
            .find(|entry| entry.pin.element_id == led && entry.pin.pin == pin)
            .map(|entry| entry.amps)
            .unwrap()
    };
    for die in ["R", "G", "B"] {
        assert!(into(die) < -0.01, "{die} takes current from its wire");
    }
    assert!((into("R") + into("G") + into("B") + into("com")).abs() < 1e-12);
}

#[test]
fn an_rgb_led_with_no_common_is_dark_and_no_trouble() {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
    let led = circuit.try_add("RgbLed", 160.0, 0.0).unwrap();
    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &led, "G").unwrap();
    circuit.advance(1.0 / 60.0).unwrap();
    assert!(circuit.solution().failure.is_none());
    assert!(glow(&circuit, &led).iter().all(|&dark| dark == 0.0));
}

/// An LCD digit on two 32 Hz squares of 2.5 V: one on COM, one the other
/// way round on segment a, and a third the same way round on segment b.
/// So a swings in antiphase with the backplane — 5 V RMS across it — and
/// b in phase, with nothing across it (C-76). Returns the circuit and the
/// digit.
fn glass_in_phase_and_antiphase() -> (Circuit, String) {
    let mut circuit = Circuit::new();
    let digit = circuit.try_add("LcdDigit", 300.0, 0.0).unwrap();
    let square = |circuit: &mut Circuit| {
        let generator = circuit.try_add("SignalGenerator", 0.0, 0.0).unwrap();
        circuit.try_set_value(&generator, "2.5").unwrap();
        circuit.try_set_frequency(&generator, "32").unwrap();
        generator
    };
    let backplane = square(&mut circuit);
    let antiphase = square(&mut circuit);
    let in_phase = square(&mut circuit);
    circuit.try_connect(&backplane, "+", &digit, "com").unwrap();
    // Every generator's other end on one net, the one they swing about.
    circuit
        .try_connect(&antiphase, "+", &backplane, "-")
        .unwrap();
    circuit
        .try_connect(&in_phase, "-", &backplane, "-")
        .unwrap();
    circuit.try_connect(&antiphase, "-", &digit, "a").unwrap();
    circuit.try_connect(&in_phase, "+", &digit, "b").unwrap();
    (circuit, digit)
}

fn plated(circuit: &Circuit, digit: &str) -> bool {
    circuit.solution().findings.iter().any(|finding| {
        finding.elements.iter().any(|element| element == digit) && finding.message.contains("DC")
    })
}

#[test]
fn glass_in_antiphase_with_its_backplane_shows_and_in_phase_does_not() {
    let (mut circuit, digit) = glass_in_phase_and_antiphase();
    run_for(&mut circuit, 0.5);
    let segments = glow(&circuit, &digit);
    assert_eq!(segments.len(), 8);
    assert!(segments[0] > 0.99, "a, in antiphase: {segments:?}");
    assert!(segments[1] < 1e-6, "b, in phase: {segments:?}");
    assert!(segments[2..].iter().all(|&clear| clear == 0.0));
    assert!(
        !plated(&circuit, &digit),
        "AC is how glass is meant to be driven"
    );
}

#[test]
fn glass_on_dc_shows_and_says_it_is_being_damaged() {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let digit = circuit.try_add("LcdDigit", 200.0, 0.0).unwrap();
    circuit.try_set_value(&battery, "5").unwrap();
    circuit.try_connect(&battery, "+", &digit, "a").unwrap();
    circuit.try_connect(&battery, "-", &digit, "com").unwrap();
    // A battery is no past, so this is the path with no run behind it:
    // the glass turns frame by frame all the same.
    run_for(&mut circuit, 1.0);
    assert!(glow(&circuit, &digit)[0] > 0.99);
    assert!(plated(&circuit, &digit));

    // Put on a signal instead, it is not told about the battery any more:
    // the edit starts the DC reading over.
    circuit.try_remove(&battery).unwrap();
    let generator = circuit.try_add("SignalGenerator", 0.0, 0.0).unwrap();
    circuit.try_set_frequency(&generator, "32").unwrap();
    circuit.try_connect(&generator, "+", &digit, "a").unwrap();
    circuit.try_connect(&generator, "-", &digit, "com").unwrap();
    run_for(&mut circuit, 0.5);
    assert!(glow(&circuit, &digit)[0] > 0.99);
    assert!(!plated(&circuit, &digit));
}

#[test]
fn glass_turned_down_to_10_hz_is_not_told_about_dc_at_any_frame() {
    // "A watch's digit" at 10 Hz: the warning used to come and go several
    // times a second, and the status bar with it.
    let mut circuit = Circuit::new();
    let generator = circuit.try_add("SignalGenerator", 0.0, 0.0).unwrap();
    let digit = circuit.try_add("LcdDigit", 200.0, 0.0).unwrap();
    circuit.try_set_frequency(&generator, "10").unwrap();
    circuit.try_connect(&generator, "+", &digit, "b").unwrap();
    circuit.try_connect(&generator, "+", &digit, "c").unwrap();
    circuit.try_connect(&generator, "-", &digit, "com").unwrap();
    for frame in 0..600 {
        circuit.advance(1.0 / 60.0).unwrap();
        assert!(!plated(&circuit, &digit), "frame {frame}");
    }
}

#[test]
fn glass_draws_almost_nothing() {
    // 5 V RMS across one segment is a hundredth of a microamp through
    // its leak, and what leaves the wire at the segment arrives at COM.
    let (mut circuit, digit) = glass_in_phase_and_antiphase();
    run_for(&mut circuit, 0.1);
    let solution = circuit.solution();
    let into = |pin: &str| {
        solution
            .pin_currents
            .iter()
            .find(|entry| entry.pin.element_id == digit && entry.pin.pin == pin)
            .map(|entry| entry.amps)
            .unwrap()
    };
    assert!(into("a").abs() < 1e-7 && into("a") != 0.0, "{}", into("a"));
    assert!((into("a") + into("b") + into("com")).abs() < 1e-18);
}

#[test]
fn glass_with_no_backplane_is_clear_and_no_trouble() {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 100.0, 0.0).unwrap();
    let digit = circuit.try_add("LcdDigit", 200.0, 0.0).unwrap();
    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &battery, "-").unwrap();
    circuit.try_connect(&battery, "+", &digit, "a").unwrap();
    run_for(&mut circuit, 0.2);
    assert!(glow(&circuit, &digit).iter().all(|&clear| clear == 0.0));
    assert!(!plated(&circuit, &digit));
}

/// Every wire an op-amp or a comparator is on gives back what it is given,
/// counting what the part itself says it puts in, frame after frame.
fn ic_wires_balance(circuit: &mut Circuit, part: &str, frames: usize, moment: &str) {
    let id = circuit
        .views()
        .into_iter()
        .find(|view| view.kind == part)
        .map(|view| view.id)
        .unwrap();
    for frame in 0..frames {
        circuit.advance(1.0 / 60.0).unwrap();
        let solution = circuit.solution();
        assert!(solution.failure.is_none(), "{:?}", solution.failure);
        let own: Vec<&PinCurrent> = solution
            .pin_currents
            .iter()
            .filter(|entry| entry.pin.element_id == id)
            .collect();
        assert_eq!(own.len(), 5, "{part}: every pin read, {moment}");
        for pin in &own {
            let others: Vec<&PinCurrent> = solution
                .pin_currents
                .iter()
                .filter(|entry| entry.net_id == pin.net_id)
                .collect();
            let sum: f64 = others.iter().map(|entry| entry.amps).sum();
            // The rest of the wire is resistors, a source, an LED: what
            // they carry is what the part's pin must, to a nanoamp.
            assert!(
                sum.abs() < 1e-9,
                "{part} {}, {moment}, frame {frame}: {} A, but the wire sums to {sum} A ({others:?})",
                pin.pin.pin,
                pin.amps,
            );
            assert!(
                pin.amps.abs() < 0.1,
                "{part} {}, {moment}, frame {frame}: {} A",
                pin.pin.pin,
                pin.amps
            );
        }
    }
}

#[test]
fn an_ic_pin_carries_what_the_rest_of_its_wire_does() {
    // An ideal comparator, linearised from a cold history at a point
    // where it is saturated, was evaluated just inside its rail instead
    // of where it was; its 5 kS of gain over that step was read as kiloamps
    // on its output and V− wires.
    let mut circuit = Circuit::new();
    circuit
        .load(include_str!("../../../../app/src/examples/threshold.json"))
        .unwrap();
    ic_wires_balance(&mut circuit, "Comparator", 12, "as it opens");
    let pot = circuit.views()[1].id.clone();
    for wiper in [0.0, 0.25, 0.5, 0.75, 1.0] {
        circuit.try_set_wiper(&pot, wiper).unwrap();
        ic_wires_balance(&mut circuit, "Comparator", 6, &format!("wiper at {wiper}"));
    }

    // The op-amp at twice its input, and asked for more than its supply.
    let mut circuit = Circuit::new();
    circuit
        .load(include_str!("../../../../app/src/examples/twice.json"))
        .unwrap();
    ic_wires_balance(&mut circuit, "OpAmp", 12, "as it opens");
    let pot = circuit.views()[1].id.clone();
    for wiper in [0.0, 0.25, 0.5, 0.75, 1.0] {
        circuit.try_set_wiper(&pot, wiper).unwrap();
        ic_wires_balance(&mut circuit, "OpAmp", 6, &format!("wiper at {wiper}"));
    }
}

#[test]
fn a_555_held_at_its_current_limit_says_what_its_output_carries() {
    // Latched high, its output into 1 Ω: 7.3 V behind 10 Ω asks for
    // 0.66 A, and the part holds it near its 0.2 A limit. Read from a cold
    // history, the output was linearised at half that limit instead.
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let timer = circuit.try_add("Timer", 100.0, 0.0).unwrap();
    let load = circuit.try_add("Resistor", 200.0, 0.0).unwrap();
    let pull = circuit.try_add("Resistor", 200.0, 80.0).unwrap();
    circuit.try_set_value(&load, "1").unwrap();
    circuit.try_set_value(&pull, "10k").unwrap();
    circuit.try_connect(&timer, "VCC", &battery, "+").unwrap();
    circuit.try_connect(&timer, "GND", &battery, "-").unwrap();
    // TRIG and THR on GND: set, and never reset.
    circuit.try_connect(&timer, "TRIG", &battery, "-").unwrap();
    circuit.try_connect(&timer, "THR", &battery, "-").unwrap();
    circuit.try_connect(&timer, "DIS", &pull, "a").unwrap();
    circuit.try_connect(&pull, "b", &battery, "+").unwrap();
    circuit.try_connect(&timer, "OUT", &load, "a").unwrap();
    circuit.try_connect(&load, "b", &battery, "-").unwrap();
    run_for(&mut circuit, 0.1);

    let solution = circuit.solution();
    let into = |element: &str, pin: &str| {
        solution
            .pin_currents
            .iter()
            .find(|entry| entry.pin.element_id == element && entry.pin.pin == pin)
            .map(|entry| entry.amps)
            .unwrap()
    };
    let delivered = into(&timer, "OUT");
    assert!(
        (delivered + into(&load, "a")).abs() < 1e-9,
        "OUT gives {delivered} A, the 1 Ω takes {} A",
        -into(&load, "a")
    );
    assert!(delivered > 0.15, "held near its limit: {delivered} A");
}

#[test]
fn a_wire_where_three_pins_meet_says_what_each_one_puts_in() {
    // "One way only": a battery feeding two diodes, one forwards and one
    // reversed, each through a resistor. The battery's + joins three pins,
    // so no one current can be said for it — but what each pin puts in
    // can, and it adds up to nothing, since what enters a wire leaves it
    // (C-73). The forward diode takes all of it; the reversed one, none.
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let forwards = circuit.try_add("Diode", 80.0, 0.0).unwrap();
    let backwards = circuit.try_add("Diode", 80.0, 80.0).unwrap();
    let top = circuit.try_add("Resistor", 160.0, 0.0).unwrap();
    let bottom = circuit.try_add("Resistor", 160.0, 80.0).unwrap();
    circuit
        .try_connect(&battery, "+", &forwards, "anode")
        .unwrap();
    circuit
        .try_connect(&battery, "+", &backwards, "cathode")
        .unwrap();
    circuit
        .try_connect(&forwards, "cathode", &top, "a")
        .unwrap();
    circuit
        .try_connect(&backwards, "anode", &bottom, "a")
        .unwrap();
    circuit.try_connect(&top, "b", &battery, "-").unwrap();
    circuit.try_connect(&bottom, "b", &battery, "-").unwrap();
    circuit.advance(1.0 / 60.0).unwrap();

    let solution = circuit.solution();
    assert!(solution.failure.is_none());
    let mut by_net: std::collections::BTreeMap<&str, Vec<&PinCurrent>> = Default::default();
    for pin in &solution.pin_currents {
        by_net.entry(&pin.net_id).or_default().push(pin);
    }
    // Four wires: both resistors come back to the battery's − on one.
    assert_eq!(by_net.len(), 4, "every wire's pins are all known");
    for pins in by_net.values() {
        let sum: f64 = pins.iter().map(|pin| pin.amps).sum();
        assert!(
            sum.abs() < 1e-9,
            "a wire gives back what it is given: {sum} A"
        );
    }

    let into = |element: &str, pin: &str| {
        solution
            .pin_currents
            .iter()
            .find(|entry| entry.pin.element_id == element && entry.pin.pin == pin)
            .map(|entry| entry.amps)
            .unwrap()
    };
    let supplied = into(&battery, "+");
    assert!(supplied > 0.01, "the battery feeds the wire: {supplied} A");
    assert!(
        (into(&forwards, "anode") + supplied).abs() < 1e-9,
        "and the forward diode takes all of it"
    );
    assert!(
        into(&backwards, "cathode").abs() < 1e-6,
        "the reversed one, none"
    );
}

#[test]
fn current_runs_along_every_wire_of_a_ringing_loop() {
    // A charged capacitor, a resistor and a coil in one loop: the same
    // current in all three, so every wire carries it. Each wire takes its
    // current from the first element on it, and for two of the three that
    // is the capacitor or the coil — which used to report none, so only
    // the wire leaving the resistor was drawn moving.
    let mut circuit = Circuit::new();
    let capacitor = circuit.try_add("Capacitor", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
    let coil = circuit.try_add("Inductor", 160.0, 0.0).unwrap();
    circuit.try_set_value(&capacitor, "10m").unwrap();
    circuit.try_set_initial_condition(&capacitor, "5").unwrap();
    circuit.try_set_value(&resistor, "1").unwrap();
    circuit.try_set_value(&coil, "2.5").unwrap();
    circuit
        .try_connect(&capacitor, "a", &resistor, "a")
        .unwrap();
    circuit.try_connect(&resistor, "b", &coil, "a").unwrap();
    circuit.try_connect(&coil, "b", &capacitor, "b").unwrap();

    // A quarter of a swing in, where the current is near its largest.
    for _ in 0..15 {
        circuit.advance(1.0 / 60.0).unwrap();
    }
    let solution = circuit.solution();
    assert!(solution.failure.is_none());
    assert_eq!(solution.net_currents.len(), 3, "every wire carries one");

    let through = current_through(&circuit, &resistor);
    assert!(through.abs() > 0.1, "{through} A");
    // The same current each way round the loop. Its sign is each part's
    // own: the capacitor is what drives it, so it leaves by the pin it
    // enters the resistor by.
    for element in [&capacitor, &coil] {
        let amps = current_through(&circuit, element);
        assert!(
            (amps.abs() - through.abs()).abs() < 1e-3 * through.abs(),
            "{element}: {amps} A against the resistor's {through} A"
        );
    }
    for wire in &solution.net_currents {
        assert!(
            (wire.amps - through.abs()).abs() < 1e-3 * through.abs(),
            "a wire at {} A against {} A",
            wire.amps,
            through.abs()
        );
    }
}

#[test]
fn a_relay_pulls_in_late_and_lets_go_later() {
    // A ±9 V triangle at 1 Hz straight across a 9 V coil: its magnitude
    // climbs from 0 to 9 V in a quarter second. The contact closes at
    // three quarters of rated, 6.75 V, at 0.1875 s — and, on the way back
    // down, does not open until a tenth of rated, 0.9 V, at 0.475 s. The
    // contact takes a battery through to a resistor on NO (C-64).
    let mut circuit = Circuit::new();
    let signal = circuit.try_add("SignalGenerator", 0.0, 0.0).unwrap();
    let relay = circuit.try_add("Relay", 80.0, 0.0).unwrap();
    let battery = circuit.try_add("DcSource", 160.0, 0.0).unwrap();
    let load = circuit.try_add("Resistor", 240.0, 0.0).unwrap();
    circuit.try_set_choice(&signal, "Triangle").unwrap();
    circuit.try_set_value(&signal, "9").unwrap();
    circuit.try_set_value(&load, "1k").unwrap();
    circuit.try_connect(&signal, "+", &relay, "coil+").unwrap();
    circuit.try_connect(&signal, "-", &relay, "coil-").unwrap();
    circuit.try_connect(&battery, "+", &relay, "com").unwrap();
    circuit.try_connect(&relay, "no", &load, "a").unwrap();
    circuit.try_connect(&load, "b", &battery, "-").unwrap();

    let mut frame = 0;
    let mut until = |circuit: &mut Circuit, seconds: f64| {
        while f64::from(frame) / 60.0 < seconds - 1e-9 {
            circuit.advance(1.0 / 60.0).unwrap();
            frame += 1;
        }
        let solution = circuit.solution();
        assert!(solution.failure.is_none());
        solution.energised.contains(&relay)
    };

    assert!(!until(&mut circuit, 0.15), "open below pull-in");
    assert!(until(&mut circuit, 0.2), "closed past 6.75 V");
    assert!(
        until(&mut circuit, 0.45),
        "still closed at 1.8 V on the way down"
    );
    assert!(!until(&mut circuit, 0.5), "open once under 0.9 V");
    // And the contact is what it says: current through the load only
    // while it is made.
    assert!(current_through(&circuit, &load).abs() < 1e-6);
    assert!(
        until(&mut circuit, 0.72),
        "closed again on the negative swing"
    );
    assert!(current_through(&circuit, &load).abs() > 8e-3);
}

#[test]
fn a_three_volt_regulator_holds_three_volts() {
    let (mut circuit, _, regulator, load) = regulated("9");
    circuit.try_set_choice(&regulator, "LM1117-3.3").unwrap();
    circuit.advance(0.016).unwrap();
    let out = across_pins(&circuit, &load);
    assert!((out - 3.3).abs() < 1e-3, "{out} V out");
}

#[test]
fn an_led_reports_its_brightness() {
    let (mut circuit, _, resistor, led) = milestone();
    circuit.advance(0.016).unwrap();

    let brightness = |circuit: &Circuit| {
        circuit
            .solution()
            .element_currents
            .iter()
            .find(|current| current.element_id == led)
            .and_then(|current| current.brightness)
            .unwrap()
    };

    // 470 Ω lights it at some 15 mA, inside its 20 mA: bright, short of
    // as bright as it gets.
    let lit = brightness(&circuit);
    assert!(lit > 0.8 && lit < 1.0, "{lit}");

    circuit.try_set_value(&resistor, "2k2").unwrap();
    circuit.advance(0.016).unwrap();
    let dimmed = brightness(&circuit);
    assert!(dimmed > 0.0 && dimmed < 1.0, "{dimmed}");
}

#[test]
fn every_two_terminal_element_reports_its_current() {
    let (mut circuit, battery, resistor, led) = milestone();
    circuit.advance(0.016).unwrap();

    let currents = circuit.solution().element_currents;
    let of = |id: &str| {
        currents
            .iter()
            .find(|current| current.element_id == id)
            .unwrap_or_else(|| panic!("no current for {id}"))
            .amps
    };

    // One loop, so all three carry the same current. The signs differ, and
    // that is the convention doing its job: current enters the resistor and
    // the LED by their first pin, and leaves the battery by its `+`, so the
    // battery is the one reporting a negative — the one delivering.
    let expected = 1.534_827_142_501_358_5e-2;
    assert!((of(&resistor) - expected).abs() < 1e-9, "{}", of(&resistor));
    assert!((of(&led) - expected).abs() < 1e-9, "{}", of(&led));
    assert!((of(&battery) + expected).abs() < 1e-9, "{}", of(&battery));
}

#[test]
fn a_wire_carries_a_current_with_a_direction() {
    let (mut circuit, _, _, _) = milestone();
    circuit.advance(0.016).unwrap();

    let state = circuit.solution();

    assert_eq!(state.net_currents.len(), 3);
    for wire in &state.net_currents {
        assert!((wire.amps - 1.534_827_142_501_358_5e-2).abs() < 1e-9);
        // Magnitude is always positive; the direction is the pin it leaves.
        assert!(wire.amps > 0.0);
    }
}

#[test]
fn current_leaves_the_battery_by_its_positive_terminal() {
    // The direction a beginner draws, and the one the overlay has to show:
    // out of `+`, round the loop, back into `-`.
    let (mut circuit, battery, resistor, _) = milestone();
    circuit.advance(0.016).unwrap();

    let state = circuit.solution();
    let from_battery = state
        .net_currents
        .iter()
        .find(|wire| wire.from.element_id == battery && wire.from.pin == "+")
        .expect("a wire leaving the battery's + terminal");

    // And that wire is the one joining the battery to the resistor.
    let net = state
        .net_voltages
        .iter()
        .find(|net| net.net_id == from_battery.net_id)
        .unwrap();
    assert!((net.volts - 9.0).abs() < 1e-9);
    assert!(!resistor.is_empty());
}

#[test]
fn both_ends_of_a_wire_name_a_pin_that_is_really_there() {
    // The `from` end used to be built out of one element's handle and,
    // when the current ran the other way, the *other* element's pin name.
    // It read perfectly well and nothing looked it up — until C-21 needed
    // both ends to find which stretch of the drawing the current crosses.
    let (mut circuit, _, _, _) = milestone();
    circuit.advance(0.016).unwrap();

    for wire in &circuit.solution().net_currents {
        for end in [&wire.from, &wire.to] {
            let element = circuit
                .netlist()
                .element(decode(&end.element_id).unwrap())
                .expect("an end names an element that exists");
            assert!(
                element.pin_index(&end.pin).is_some(),
                "{end:?} names a pin that element does not have"
            );
        }
        assert_ne!(wire.from, wire.to);
    }
}

#[test]
fn a_wire_where_three_pins_meet_carries_no_single_current() {
    // Current splits there, and one arrow would be a lie. C-21's probes are
    // what answers the question properly.
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let left = circuit.try_add("Resistor", 40.0, 0.0).unwrap();
    let right = circuit.try_add("Resistor", 40.0, 40.0).unwrap();

    circuit.try_connect(&battery, "+", &left, "a").unwrap();
    circuit.try_connect(&battery, "+", &right, "a").unwrap();
    circuit.try_connect(&left, "b", &battery, "-").unwrap();
    circuit.try_connect(&right, "b", &battery, "-").unwrap();
    circuit.advance(0.016).unwrap();

    // Two nets, both with three pins: neither reports a current.
    assert_eq!(circuit.net_count(), 2);
    assert!(circuit.solution().net_currents.is_empty());
}

#[test]
fn the_led_dims_as_the_resistor_rises() {
    // The acceptance of M-20, measured rather than looked at.
    let (mut circuit, _, resistor, led) = milestone();

    let brightness = |circuit: &Circuit| {
        circuit
            .solution()
            .element_currents
            .iter()
            .find(|current| current.element_id == led)
            .and_then(|current| current.brightness)
            .unwrap()
    };

    let mut previous = 1.1;
    for ohms in ["220", "470", "1k", "4k7", "22k"] {
        circuit.try_set_value(&resistor, ohms).unwrap();
        circuit.advance(0.016).unwrap();
        let now = brightness(&circuit);
        assert!(now < previous, "{ohms}: {now} is not less than {previous}");
        previous = now;
    }
    assert!(previous > 0.0);
}

#[test]
fn an_led_says_what_colour_it_is() {
    let mut circuit = Circuit::new();
    circuit.try_add("Led", 0.0, 0.0).unwrap();
    circuit.try_add("Resistor", 40.0, 0.0).unwrap();

    let kinds: Vec<Option<String>> = circuit
        .netlist()
        .elements()
        .map(|(_, element)| match element.kind {
            ElementKind::Led { colour } => Some(format!("{colour:?}")),
            _ => None,
        })
        .collect();

    assert_eq!(kinds, vec![Some("Red".to_string()), None]);
}

#[test]
fn moving_an_element_moves_its_wires_with_it() {
    let (mut circuit, battery, resistor, _) = milestone();

    circuit.try_move(&resistor, 200.0, 160.0).unwrap();

    // The wire is drawn from where the pins are, so there is nothing else to
    // update — and the net still joins what it joined.
    assert_eq!(
        circuit.placement_of(decode(&resistor).unwrap()),
        Some(Placement {
            x: 200.0,
            y: 160.0,
            turns: 0
        })
    );
    assert_eq!(circuit.net_count(), 3);
    assert!(circuit.placement_of(decode(&battery).unwrap()).unwrap().x == 0.0);
}

#[test]
fn moving_does_not_restart_the_solve() {
    // A drag is sixty moves a second. Treating each as a circuit change
    // would rebuild the engine sixty times and reset nothing usefully.
    let (mut circuit, _, resistor, _) = milestone();
    circuit.advance(0.016).unwrap();
    let before = circuit.revision;

    for step in 0..60 {
        circuit.try_move(&resistor, step as f64, 0.0).unwrap();
    }

    assert_eq!(circuit.revision, before);
}

#[test]
fn deleting_takes_the_dangling_wires_with_it() {
    let (mut circuit, _, resistor, _) = milestone();
    assert_eq!(circuit.net_count(), 3);

    circuit.try_remove(&resistor).unwrap();

    // The two wires that reached the resistor now join one thing each, so
    // they are stubs pointing at nothing and go too.
    assert_eq!(circuit.size(), 2);
    assert_eq!(circuit.net_count(), 1);
}

#[test]
fn a_resistor_of_nothing_is_refused_and_told_where_the_wire_is() {
    // A beginner's "make it a wire": before C-96 it took the part out of
    // the circuit without a word and reported an infinite current.
    let (mut circuit, _, resistor, _) = milestone();
    let undoable = circuit.can_undo();
    for typed in ["0", "-100"] {
        let refused = circuit.try_set_value(&resistor, typed).unwrap_err();
        assert!(
            matches!(refused, CircuitError::Refused(_)),
            "{typed}: {refused}"
        );
    }
    let said = circuit
        .try_set_value(&resistor, "0")
        .unwrap_err()
        .to_string();
    assert!(said.contains("draw a wire"), "{said}");
    assert_eq!(circuit.can_undo(), undoable, "a refused edit is not a step");
}

#[test]
fn a_part_s_temperature_is_always_a_number() {
    // Before C-96, "0" gave infinite watts, then a NaN °C carried through
    // every later edit and undo.
    let (mut circuit, _, resistor, _) = milestone();
    let _ = circuit.try_set_value(&resistor, "0");
    circuit.advance(0.5).unwrap();
    circuit.try_set_value(&resistor, "1k").unwrap();
    circuit.advance(0.5).unwrap();
    circuit.undo();
    circuit.undo();
    circuit.advance(0.5).unwrap();
    let heat = circuit.solution().heat;
    assert!(!heat.is_empty());
    for part in heat {
        assert!(part.celsius.is_finite(), "{part:?}");
    }
}

#[test]
fn an_edit_a_part_has_no_value_for_is_not_a_step_either() {
    let (mut circuit, _, _, led) = milestone();
    let undoable = circuit.can_undo();
    assert!(circuit.try_set_value(&led, "5").is_err());
    assert_eq!(circuit.can_undo(), undoable);
}

#[test]
fn opening_a_circuit_over_another_is_a_step_that_an_undo_takes_back() {
    // C-98: a file opened over an hour of work used to lose it.
    let (mut circuit, _, _, _) = milestone();
    circuit.try_set_description("Mine.");
    let mut other = Circuit::new();
    other.try_add("Resistor", 0.0, 0.0).unwrap();
    other.try_set_description("Somebody else's.");
    let text = circuit_text(&other);

    circuit.try_open(&text).unwrap();
    assert_eq!(circuit.netlist.element_count(), 1);
    assert_eq!(circuit.description(), "Somebody else's.");

    assert!(circuit.undo());
    assert_eq!(circuit.netlist.element_count(), 3);
    assert_eq!(circuit.netlist.net_count(), 3);
    assert_eq!(circuit.description(), "Mine.");
}

#[test]
fn a_file_that_will_not_open_leaves_the_circuit_and_its_history_alone() {
    let (mut circuit, _, _, _) = milestone();
    let undoable = circuit.can_undo();
    assert!(circuit.try_open("{ not a circuit").is_err());
    assert_eq!(circuit.netlist.element_count(), 3);
    assert_eq!(circuit.can_undo(), undoable);
}

fn circuit_text(circuit: &Circuit) -> String {
    serde_json::to_string(&circuit.to_document()).unwrap()
}

#[test]
fn a_description_is_an_edit_like_any_other() {
    let (mut circuit, _, _, _) = milestone();
    assert!(circuit.try_set_description("A battery, a resistor, an LED."));
    assert!(!circuit.try_set_description(" A battery, a resistor, an LED. "));
    assert!(circuit.undo());
    assert_eq!(circuit.description(), "");
    assert!(circuit.redo());
    assert_eq!(circuit.description(), "A battery, a resistor, an LED.");
}

#[test]
fn describing_a_circuit_does_not_restart_it() {
    // A run carries on through a description, as it does through a wire
    // being bent: the clock keeps its time.
    let (mut circuit, _, _, _) = milestone();
    circuit.advance(0.5).unwrap();
    let before = circuit.revision;
    circuit.try_set_description("Watch it.");
    assert_eq!(circuit.revision, before);
}

#[test]
fn clearing_empties_the_canvas_in_one_step_and_keeps_the_room() {
    let (mut circuit, _, _, _) = milestone();
    circuit.try_set_room("30").unwrap();
    circuit.try_set_description("The first circuit.");
    assert!(circuit.try_clear());
    assert_eq!(circuit.netlist.element_count(), 0);
    assert_eq!(circuit.netlist.net_count(), 0);
    assert_eq!(circuit.description(), "");
    assert_eq!(circuit.room_celsius(), 30.0);
    assert!(!circuit.try_clear(), "an empty canvas has nothing to clear");

    // One undo brings all of it back, wires included.
    assert!(circuit.undo());
    assert_eq!(circuit.netlist.element_count(), 3);
    assert_eq!(circuit.netlist.net_count(), 3);
    assert_eq!(circuit.description(), "The first circuit.");
}

#[test]
fn undo_puts_back_what_was_deleted() {
    let (mut circuit, _, resistor, _) = milestone();

    circuit.try_remove(&resistor).unwrap();
    assert_eq!(circuit.size(), 2);

    assert!(circuit.undo());

    assert_eq!(circuit.size(), 3);
    assert_eq!(circuit.net_count(), 3);
    // And the circuit solves again, which a half-restored netlist would not.
    circuit.advance(0.016).unwrap();
    assert!(circuit.solution().failure.is_none());
}

#[test]
fn undo_and_redo_walk_the_same_path() {
    let mut circuit = Circuit::new();
    circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    circuit.try_add("Resistor", 40.0, 0.0).unwrap();
    circuit.try_add("Led", 80.0, 0.0).unwrap();
    assert_eq!(circuit.size(), 3);

    for expected in [2, 1, 0] {
        assert!(circuit.undo());
        assert_eq!(circuit.size(), expected);
    }
    assert!(!circuit.undo(), "nothing left to undo");

    for expected in [1, 2, 3] {
        assert!(circuit.redo());
        assert_eq!(circuit.size(), expected);
    }
    assert!(!circuit.redo(), "nothing left to redo");
}

#[test]
fn undoing_a_value_edit_restores_the_old_value() {
    let (mut circuit, _, resistor, _) = milestone();
    circuit.try_set_value(&resistor, "1k").unwrap();

    circuit.undo();

    let element = circuit
        .netlist()
        .element(decode(&resistor).unwrap())
        .unwrap();
    assert_eq!(element.kind, ElementKind::Resistor { ohms: 470.0 });
}

#[test]
fn undoing_a_wire_unmakes_the_net() {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 40.0, 0.0).unwrap();
    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    assert_eq!(circuit.net_count(), 1);

    circuit.undo();

    assert_eq!(circuit.net_count(), 0);
    assert_eq!(circuit.size(), 2, "only the wire was undone");
}

#[test]
fn a_whole_drag_is_one_step_of_history() {
    // Sixty snapshots a second would make undo useless: pressing it would
    // nudge the component by a pixel.
    let (mut circuit, _, resistor, _) = milestone();
    let start = circuit.placement_of(decode(&resistor).unwrap()).unwrap();

    circuit.begin_gesture();
    for step in 1..=60 {
        circuit.try_move(&resistor, step as f64 * 2.0, 0.0).unwrap();
    }

    assert!(circuit.undo());
    assert_eq!(
        circuit.placement_of(decode(&resistor).unwrap()),
        Some(start)
    );
}

#[test]
fn deleting_a_selection_is_one_step_of_history() {
    // Four components deleted in one keystroke is one thing the user did,
    // so it is one press of undo — not four.
    let mut circuit = Circuit::new();
    let ids: Vec<String> = (0..4)
        .map(|i| circuit.try_add("Resistor", i as f64 * 40.0, 0.0).unwrap())
        .collect();

    circuit.begin_gesture();
    for id in &ids {
        circuit.try_remove(id).unwrap();
    }
    circuit.end_gesture();
    assert_eq!(circuit.size(), 0);

    assert!(circuit.undo());
    assert_eq!(circuit.size(), 4);
}

#[test]
fn an_unclosed_gesture_cannot_swallow_the_history() {
    // If a pointerup is ever missed, the next undo should still work rather
    // than the app quietly stopping to record anything.
    let mut circuit = Circuit::new();
    circuit.try_add("Resistor", 0.0, 0.0).unwrap();
    circuit.begin_gesture();

    assert!(circuit.undo());
    circuit.try_add("Led", 0.0, 0.0).unwrap();
    assert!(circuit.can_undo());
}

#[test]
fn a_gesture_that_changes_nothing_is_not_a_step_of_history() {
    // Every pointer down on a component opens a gesture, including the
    // ones that only select it. Each of those used to push a step
    // identical to the current state, so undo did nothing until it had
    // been pressed past all of them.
    let mut circuit = Circuit::new();
    circuit.try_add("Resistor", 0.0, 0.0).unwrap();
    let steps = circuit.past.len();

    for _ in 0..5 {
        circuit.begin_gesture();
        circuit.end_gesture();
    }

    assert_eq!(circuit.past.len(), steps);
    assert!(circuit.undo());
    assert_eq!(circuit.size(), 0);
}

#[test]
fn a_gesture_that_changes_nothing_keeps_the_redo_branch() {
    // Selecting something is not an edit, so it does not start a new
    // branch: what was undone is still ahead.
    let mut circuit = Circuit::new();
    circuit.try_add("Resistor", 0.0, 0.0).unwrap();
    circuit.undo();

    circuit.begin_gesture();
    circuit.end_gesture();

    assert!(circuit.can_redo());
    assert!(circuit.redo());
    assert_eq!(circuit.size(), 1);
}

#[test]
fn a_second_drag_is_a_second_step_of_history() {
    // The first change inside a gesture is the one that records, so the
    // next gesture has to be able to record again — otherwise every drag
    // after the first would have nowhere to go back to.
    let (mut circuit, _, resistor, _) = milestone();
    let id = decode(&resistor).unwrap();
    let start = circuit.placement_of(id).unwrap();

    circuit.begin_gesture();
    circuit.try_move(&resistor, 40.0, 0.0).unwrap();
    circuit.end_gesture();
    circuit.begin_gesture();
    circuit.try_move(&resistor, 80.0, 0.0).unwrap();
    circuit.end_gesture();

    assert!(circuit.undo());
    assert_eq!(circuit.placement_of(id).map(|at| at.x), Some(40.0));
    assert!(circuit.undo());
    assert_eq!(circuit.placement_of(id), Some(start));
}

#[test]
fn a_new_edit_abandons_the_redo_branch() {
    let mut circuit = Circuit::new();
    circuit.try_add("Resistor", 0.0, 0.0).unwrap();
    circuit.undo();
    assert!(circuit.can_redo());

    circuit.try_add("Led", 0.0, 0.0).unwrap();

    assert!(!circuit.can_redo(), "the undone branch is gone");
    assert_eq!(circuit.size(), 1);
}

#[test]
fn a_refused_edit_leaves_no_history_behind() {
    // Nothing changed, so there is nothing to undo — otherwise undo would
    // silently do nothing and look broken.
    let (mut circuit, _, resistor, _) = milestone();
    let steps = circuit.past.len();

    assert!(circuit.try_set_value(&resistor, "bananas").is_err());

    assert_eq!(circuit.past.len(), steps);
}

#[test]
fn the_history_is_bounded() {
    let mut circuit = Circuit::new();
    for _ in 0..(HISTORY_LIMIT + 50) {
        circuit.try_add("Resistor", 0.0, 0.0).unwrap();
    }

    assert_eq!(circuit.past.len(), HISTORY_LIMIT);

    // The oldest steps are gone, so undoing everything available does not
    // reach an empty circuit.
    while circuit.undo() {}
    assert_eq!(circuit.size(), 50);
}

/// The milestone circuit with the LED wired the wrong way round.
fn reversed_led() -> (Circuit, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 160.0, 0.0).unwrap();

    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit
        .try_connect(&resistor, "b", &led, "cathode")
        .unwrap();
    circuit.try_connect(&led, "anode", &battery, "-").unwrap();

    (circuit, led)
}

#[test]
fn a_reversed_led_is_named_rather_than_left_silent() {
    // Backwards, the circuit does exactly what it does when anything else
    // is wrong: nothing lights. The app knows which pin the wire landed on
    // and should say so.
    let (mut circuit, led) = reversed_led();
    circuit.advance(0.016).unwrap();

    let findings = circuit.solution().findings;

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].severity, "warning");
    assert_eq!(findings[0].elements, vec![led]);
    assert!(findings[0].message.contains("wrong way round"));
}

#[test]
fn the_message_blames_neither_component_alone() {
    // Which of the two is backwards is not something the circuit can know,
    // only that they disagree. Naming both is honest, and it is also the
    // sentence that teaches the rule.
    let (mut circuit, _) = reversed_led();
    circuit.advance(0.016).unwrap();

    let message = circuit.solution().findings[0].message.clone();

    assert!(message.contains("LED"), "{message}");
    assert!(message.contains("battery"), "{message}");
    // Short enough to read at a glance: naming the pins as well is the
    // hover's job, and a wrapping status line gets read less, not more.
    assert!(
        message.len() < 70,
        "{} characters: {message}",
        message.len()
    );
}

#[test]
fn a_correctly_wired_circuit_is_left_alone() {
    let (mut circuit, _, _, _) = milestone();
    circuit.advance(0.016).unwrap();

    assert!(circuit.solution().findings.is_empty());
}

#[test]
fn an_led_with_nothing_driving_it_is_not_accused() {
    // No source means no opinion about which way round anything is. Saying
    // "this is backwards" about an unpowered circuit would be noise at the
    // exact moment someone is still building it.
    let mut circuit = Circuit::new();
    let resistor = circuit.try_add("Resistor", 0.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 80.0, 0.0).unwrap();
    circuit
        .try_connect(&resistor, "b", &led, "cathode")
        .unwrap();
    circuit.advance(0.016).unwrap();

    assert!(circuit.solution().findings.is_empty());
}

#[test]
fn turning_the_led_round_clears_the_warning() {
    let (mut circuit, led) = reversed_led();
    circuit.advance(0.016).unwrap();
    assert_eq!(circuit.solution().findings.len(), 1);

    // Rewire it the right way: the warning goes on its own, with nothing
    // dismissed and nothing restarted.
    let battery = circuit
        .netlist()
        .elements()
        .find(|(_, element)| matches!(element.kind, ElementKind::DcSource { .. }))
        .map(|(id, _)| encode(id))
        .unwrap();
    let resistor = circuit
        .netlist()
        .elements()
        .find(|(_, element)| matches!(element.kind, ElementKind::Resistor { .. }))
        .map(|(id, _)| encode(id))
        .unwrap();

    let mut fixed = Circuit::new();
    let b = fixed.try_add("DcSource", 0.0, 0.0).unwrap();
    let r = fixed.try_add("Resistor", 80.0, 0.0).unwrap();
    let d = fixed.try_add("Led", 160.0, 0.0).unwrap();
    fixed.try_connect(&b, "+", &r, "a").unwrap();
    fixed.try_connect(&r, "b", &d, "anode").unwrap();
    fixed.try_connect(&d, "cathode", &b, "-").unwrap();
    fixed.advance(0.016).unwrap();

    assert!(fixed.solution().findings.is_empty());
    assert!(!battery.is_empty() && !resistor.is_empty() && !led.is_empty());
}
