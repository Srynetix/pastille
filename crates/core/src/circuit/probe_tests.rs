// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

/// The milestone circuit: battery, resistor, LED, wired into a loop.
fn milestone() -> (Circuit, String, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 40.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 80.0, 0.0).unwrap();
    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
    circuit.try_connect(&led, "cathode", &battery, "-").unwrap();
    (circuit, battery, resistor)
}

fn view_of(circuit: &Circuit, id: &str) -> ElementView {
    circuit
        .elements_for_test()
        .into_iter()
        .find(|view| view.id == id)
        .unwrap_or_else(|| panic!("no element {id}"))
}

#[test]
fn a_voltmeter_tapped_onto_a_wire_leaves_its_current_alone() {
    // Three pins meet on that wire once the meter is on it, and the rule
    // for the arrow is that a wire joining three pins carries no single
    // current — because the current divides. It does not divide here: an
    // ideal voltmeter is an open circuit, and the dots should go on running
    // exactly as they did.
    let (mut circuit, battery, resistor) = milestone();
    circuit.advance(0.016).unwrap();
    let before = circuit.solution().net_currents;
    assert_eq!(before.len(), 3);

    let meter = circuit.try_add("Voltmeter", 40.0, 40.0).unwrap();
    circuit.try_connect(&meter, "+", &resistor, "b").unwrap();
    circuit.try_connect(&meter, "-", &battery, "-").unwrap();
    circuit.advance(0.016).unwrap();

    let after = circuit.solution().net_currents;
    assert_eq!(after.len(), 3);
    for (was, now) in before.iter().zip(&after) {
        assert_eq!(was.net_id, now.net_id);
        assert!((was.amps - now.amps).abs() < 1e-12, "{was:?} then {now:?}");
        assert_eq!(was.from, now.from);
    }
}

#[test]
fn an_ammeter_reports_the_current_running_through_it() {
    // Wired in series, between the resistor and the LED, where it reads
    // what the whole loop is passing.
    let (mut circuit, _, resistor) = milestone();
    let led = circuit
        .elements_for_test()
        .into_iter()
        .find(|view| view.kind == "Led")
        .unwrap()
        .id;
    let meter = circuit.try_add("Ammeter", 60.0, 0.0).unwrap();
    // Cut the wire the meter goes into, then put it in the gap.
    let middle = circuit
        .nets_for_test()
        .into_iter()
        .find(|net| {
            let id = decode_net(net).unwrap();
            circuit
                .netlist()
                .net(id)
                .unwrap()
                .pins()
                .iter()
                .any(|pin| encode(pin.element) == led)
                && circuit
                    .netlist()
                    .net(id)
                    .unwrap()
                    .pins()
                    .iter()
                    .any(|pin| encode(pin.element) == resistor)
        })
        .unwrap();
    circuit.try_remove_net(&middle).unwrap();
    circuit.try_connect(&resistor, "b", &meter, "+").unwrap();
    circuit.try_connect(&meter, "-", &led, "anode").unwrap();

    circuit.advance(0.016).unwrap();

    let reading = circuit
        .solution()
        .element_currents
        .into_iter()
        .find(|current| current.element_id == meter)
        .expect("the meter reads something");
    // The milestone circuit's own current, which the LED is also carrying.
    assert!((reading.amps.abs() - 1.534_827_142_501_358_5e-2).abs() < 1e-9);
}

#[test]
fn a_probe_reads_in_the_unit_the_core_says_it_does() {
    // No value to type and no label of its own: the unit is what it reads
    // in, and the reading itself is the app's to write beside it, because
    // it moves every frame and this does not.
    let mut circuit = Circuit::new();
    let voltmeter = circuit.try_add("Voltmeter", 0.0, 0.0).unwrap();
    let ammeter = circuit.try_add("Ammeter", 40.0, 0.0).unwrap();

    for (id, unit) in [(&voltmeter, "V"), (&ammeter, "A")] {
        let view = view_of(&circuit, id);
        assert_eq!(view.unit, unit);
        assert_eq!(view.value, None);
        assert_eq!(view.label, "");
        assert_eq!(view.closed, None);
    }
}

#[test]
fn a_voltmeter_survives_being_saved_and_opened() {
    let (mut circuit, battery, resistor) = milestone();
    let meter = circuit.try_add("Voltmeter", 40.0, 40.0).unwrap();
    circuit.try_connect(&meter, "+", &resistor, "b").unwrap();
    circuit.try_connect(&meter, "-", &battery, "-").unwrap();
    let saved = serde_json::to_string(&circuit.to_document()).unwrap();

    let document: crate::document::Document = serde_json::from_str(&saved).unwrap();
    let reloaded = Circuit::from_document(&document).unwrap();

    assert_eq!(
        reloaded.to_document().elements[3].kind,
        ElementKind::Voltmeter
    );
    assert_eq!(reloaded.net_count(), circuit.net_count());
}
