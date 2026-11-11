// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

/// The milestone circuit, whose LED is the thing being recoloured.
fn milestone() -> (Circuit, String, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 40.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 80.0, 0.0).unwrap();
    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
    circuit.try_connect(&led, "cathode", &battery, "-").unwrap();
    (circuit, resistor, led)
}

fn view_of(circuit: &Circuit, id: &str) -> ElementView {
    circuit
        .elements_for_test()
        .into_iter()
        .find(|view| view.id == id)
        .unwrap_or_else(|| panic!("no element {id}"))
}

#[test]
fn an_led_offers_its_colours_and_says_which_one_it_is() {
    let (circuit, _, led) = milestone();

    let view = view_of(&circuit, &led);

    assert_eq!(view.choice.as_deref(), Some("Red"));
    assert_eq!(view.choices, ["Red", "Green", "Blue", "White"]);
    // And no value to type: a colour is not a number, which is the whole
    // reason it needed a second shape of property.
    assert_eq!(view.value, None);
}

#[test]
fn a_component_with_nothing_to_choose_offers_nothing() {
    let (circuit, resistor, _) = milestone();

    let view = view_of(&circuit, &resistor);

    assert_eq!(view.choice, None);
    assert!(view.choices.is_empty());
}

#[test]
fn choosing_a_colour_changes_what_the_led_needs_to_light() {
    // The reason it is worth doing. Blue wants 3.2 V where red wants 1.8,
    // so the same resistor lets less through — and the brightness the
    // canvas draws follows. Picking a colour and watching it dim is the
    // shortest route to what a forward voltage is.
    let (mut circuit, resistor, led) = milestone();
    // A kilohm, not the palette's 220 Ω: at 220 a 9 V supply drives both
    // colours past the 20 mA an indicator is rated for, so both saturate
    // the brightness curve and the difference is invisible. That over-drive
    // is B-03's story, not this test's.
    circuit.try_set_value(&resistor, "1k").unwrap();
    circuit.advance(0.016).unwrap();
    let current_of = |circuit: &Circuit| {
        circuit
            .solution()
            .element_currents
            .iter()
            .find(|current| current.element_id == led)
            .map(|current| (current.amps, current.brightness.unwrap_or(0.0)))
            .unwrap()
    };
    let (red_amps, red_light) = current_of(&circuit);

    circuit.try_set_choice(&led, "Blue").unwrap();
    circuit.advance(0.016).unwrap();
    let (blue_amps, blue_light) = current_of(&circuit);

    assert!(blue_amps < red_amps, "{blue_amps} A against {red_amps} A");
    assert!(blue_light < red_light, "{blue_light} against {red_light}");
    // And the drop across it really is the higher one.
    assert!(
        circuit
            .solution()
            .net_voltages
            .iter()
            .any(|net| net.volts > 3.0)
    );
}

#[test]
fn the_label_keeps_saying_which_colour_it_is() {
    // An LED that is not lit draws no glow, so the word beside it is the
    // only thing that says which colour it is — and unlit is most of them,
    // most of the time.
    let (mut circuit, _, led) = milestone();
    assert_eq!(view_of(&circuit, &led).label, "Red");

    circuit.try_set_choice(&led, "White").unwrap();

    assert_eq!(view_of(&circuit, &led).label, "White");
}

#[test]
fn a_colour_it_has_never_heard_of_is_refused_by_name() {
    // Distinct from asking a resistor for a colour: one is about the set,
    // the other about the component.
    let (mut circuit, resistor, led) = milestone();

    assert_eq!(
        circuit.try_set_choice(&led, "Purple"),
        Err(CircuitError::UnknownChoice {
            kind: "Led".to_string(),
            choice: "Purple".to_string()
        })
    );
    assert_eq!(
        circuit.try_set_choice(&resistor, "Red"),
        Err(CircuitError::NotEditable("Resistor".to_string()))
    );
    // Neither left a mark.
    assert_eq!(view_of(&circuit, &led).choice.as_deref(), Some("Red"));
}

#[test]
fn choosing_the_colour_it_already_is_is_not_a_step_of_history() {
    let (mut circuit, _, led) = milestone();
    let before = circuit.can_undo();

    circuit.try_set_choice(&led, "Red").unwrap();

    assert_eq!(circuit.can_undo(), before);
}

#[test]
fn a_recoloured_led_survives_being_saved_and_opened() {
    let (mut circuit, _, led) = milestone();
    circuit.try_set_choice(&led, "Green").unwrap();
    let saved = serde_json::to_string(&circuit.to_document()).unwrap();

    let document: crate::document::Document = serde_json::from_str(&saved).unwrap();
    let reloaded = Circuit::from_document(&document).unwrap();

    assert_eq!(
        reloaded.to_document().elements[2].kind,
        ElementKind::Led {
            colour: crate::netlist::LedColour::Green
        }
    );
}
