// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

/// The milestone circuit with a contact in the supply line.
fn switched(kind: &str) -> (Circuit, String, String) {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let contact = circuit.try_add(kind, 40.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 120.0, 0.0).unwrap();

    circuit.try_connect(&battery, "+", &contact, "a").unwrap();
    circuit.try_connect(&contact, "b", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
    circuit.try_connect(&led, "cathode", &battery, "-").unwrap();

    (circuit, contact, led)
}

fn led_current(circuit: &Circuit, led: &str) -> f64 {
    circuit
        .solution()
        .element_currents
        .iter()
        .find(|current| current.element_id == led)
        .map(|current| current.amps)
        .unwrap_or_else(|| panic!("no current for {led}"))
}

#[test]
fn a_switch_turns_the_led_off_and_on_again() {
    // The whole of C-19 from where a user stands: the circuit is built, the
    // LED is dark, the switch is flipped and it lights. No run button and
    // no rebuild the caller has to ask for — the next advance solves the
    // circuit that is there now.
    let (mut circuit, contact, led) = switched("Switch");
    circuit.advance(0.016).unwrap();
    assert!(led_current(&circuit, &led).abs() < 1e-12, "lit while open");

    circuit.try_set_closed(&contact, true).unwrap();
    circuit.advance(0.016).unwrap();
    let lit = led_current(&circuit, &led);
    assert!(lit > 1e-3, "{lit} A");

    circuit.try_set_closed(&contact, false).unwrap();
    circuit.advance(0.016).unwrap();
    assert!(led_current(&circuit, &led).abs() < 1e-12, "still lit");
}

#[test]
fn a_tilt_switch_is_shut_upright_and_open_tipped() {
    // Placed upright, as it comes out of the bag, so the LED is lit from
    // the start; tipped, the ball rolls off its legs and it goes out; set
    // upright again, it comes back (C-88).
    let (mut circuit, tilt, led) = switched("TiltSwitch");
    circuit.advance(0.016).unwrap();
    assert!(led_current(&circuit, &led) > 1e-3, "upright and lit");

    circuit.try_set_closed(&tilt, false).unwrap();
    circuit.advance(0.016).unwrap();
    assert!(led_current(&circuit, &led).abs() < 1e-12, "tipped and dark");

    circuit.try_set_closed(&tilt, true).unwrap();
    circuit.advance(0.016).unwrap();
    assert!(led_current(&circuit, &led) > 1e-3, "upright again");
}

#[test]
fn a_button_is_lit_only_while_it_is_held() {
    // Same circuit, same calls: what makes it momentary is that the app
    // sends the release, not that the core models it differently.
    let (mut circuit, button, led) = switched("PushButton");

    // Held as long as a finger holds one, at the least (C-123).
    circuit.try_set_closed(&button, true).unwrap();
    circuit.advance(0.05).unwrap();
    assert!(led_current(&circuit, &led) > 1e-3);

    circuit.try_set_closed(&button, false).unwrap();
    circuit.advance(0.016).unwrap();
    assert!(led_current(&circuit, &led).abs() < 1e-12);
}

#[test]
fn the_view_says_whether_a_contact_is_made() {
    // How the canvas knows to draw the blade down, and the only field that
    // distinguishes the two states on screen.
    let (mut circuit, contact, _) = switched("Switch");

    let open = circuit.elements_for_test();
    assert_eq!(
        open.iter().find(|view| view.id == contact).unwrap().closed,
        Some(false)
    );
    // Nothing else claims to be a contact.
    assert!(open.iter().filter(|view| view.closed.is_some()).count() == 1);

    circuit.try_set_closed(&contact, true).unwrap();

    let shut = circuit.elements_for_test();
    assert_eq!(
        shut.iter().find(|view| view.id == contact).unwrap().closed,
        Some(true)
    );
}

#[test]
fn a_flip_is_one_press_of_undo() {
    let (mut circuit, contact, _) = switched("Switch");

    circuit.try_set_closed(&contact, true).unwrap();
    assert!(circuit.can_undo());

    circuit.undo();

    let back = circuit.elements_for_test();
    assert_eq!(
        back.iter().find(|view| view.id == contact).unwrap().closed,
        Some(false)
    );
}

#[test]
fn a_release_that_was_never_a_press_changes_nothing() {
    // A pointer coming up off the canvas reports a release whether or not
    // it went down on the button, and every one of those would otherwise be
    // a step of history to walk back through.
    let (mut circuit, button, _) = switched("PushButton");
    let before = circuit.can_undo();

    circuit.try_set_closed(&button, false).unwrap();

    assert_eq!(circuit.can_undo(), before);
}

#[test]
fn a_press_and_its_release_are_one_thing_that_happened() {
    // The canvas opens a gesture on the way down and closes it after the
    // release, so a button that went down and came up again is one step of
    // history — and never a step with the button stuck down in it. Down
    // and up at once, it stays down for a finger's shortest press, so
    // steps see it (C-123), and is up after it.
    let (mut circuit, button, _) = switched("PushButton");
    let state = |circuit: &Circuit| {
        circuit
            .elements_for_test()
            .iter()
            .find(|view| view.id == button)
            .unwrap()
            .closed
    };

    circuit.begin_gesture();
    circuit.try_set_closed(&button, true).unwrap();
    circuit.try_set_closed(&button, false).unwrap();
    circuit.end_gesture();

    assert_eq!(state(&circuit), Some(true), "held");
    for _ in 0..4 {
        circuit.advance(1.0 / 60.0).unwrap();
    }
    assert_eq!(
        state(&circuit),
        Some(false),
        "let go once it has lasted a press"
    );
    circuit.undo();
    assert_eq!(state(&circuit), Some(false));
}

#[test]
fn a_resistor_has_no_contact_to_make() {
    let mut circuit = Circuit::new();
    let resistor = circuit.try_add("Resistor", 0.0, 0.0).unwrap();

    assert_eq!(
        circuit.try_set_closed(&resistor, true),
        Err(CircuitError::NotEditable("Resistor".to_string()))
    );
}

#[test]
fn a_contact_has_no_value_to_type() {
    // It is open or shut, which is not a number. The inspector reads this
    // and offers no field rather than an empty one.
    let (circuit, contact, _) = switched("Switch");

    let view = circuit.elements_for_test();
    let view = view.iter().find(|view| view.id == contact).unwrap();
    assert_eq!(view.value, None);
    assert_eq!(view.label, "");
}

#[test]
fn a_contact_survives_being_saved_and_opened() {
    let (mut circuit, contact, _) = switched("PushButton");
    circuit.try_set_closed(&contact, true).unwrap();
    let saved = serde_json::to_string(&circuit.to_document()).unwrap();

    let document: crate::document::Document = serde_json::from_str(&saved).unwrap();
    let reloaded = Circuit::from_document(&document).unwrap();

    assert_eq!(
        reloaded.to_document().elements[1].kind,
        ElementKind::PushButton {
            closed: true,
            bounces: false
        }
    );
}

/// Whether the contact is made after each of `frames` frames of a
/// millisecond.
fn made(circuit: &mut Circuit, contact: &str, frames: usize) -> Vec<bool> {
    let id = decode(contact).unwrap();
    (0..frames)
        .map(|_| {
            circuit.advance(0.001).unwrap();
            circuit.netlist.element(id).unwrap().kind.contact().unwrap()
        })
        .collect()
}

#[test]
fn a_bouncy_button_chatters_for_five_milliseconds_and_ends_where_it_was_left() {
    // Pressed, it breaks and makes twice more before it stays made: two
    // more makes than the press (C-109). Let go, the same the other way.
    let (mut circuit, contact, _) = switched("PushButton");
    circuit.try_set_choice(&contact, "Bouncy").unwrap();
    circuit.advance(0.05).unwrap();
    // Which millisecond each flip lands on depends on what the scheduler
    // carried over from the last frame, so what is held is how many and
    // where they end.
    let flips = |states: &[bool], from: bool| {
        let mut last = from;
        states
            .iter()
            .filter(|&&state| std::mem::replace(&mut last, state) != state)
            .count()
    };
    circuit.try_set_closed(&contact, true).unwrap();
    let pressed = made(&mut circuit, &contact, 7);
    assert_eq!(flips(&pressed, true), 4, "{pressed:?}");
    assert_eq!(pressed.last(), Some(&true));
    // Held as long as a finger holds one, so it lets go when let go.
    circuit.advance(0.05).unwrap();
    circuit.try_set_closed(&contact, false).unwrap();
    let released = made(&mut circuit, &contact, 7);
    assert_eq!(flips(&released, false), 4, "{released:?}");
    assert_eq!(released.last(), Some(&false));
}

#[test]
fn an_ideal_button_never_bounces() {
    let (mut circuit, contact, _) = switched("PushButton");
    let id = decode(&contact).unwrap();
    assert_eq!(
        circuit.netlist.element(id).unwrap().kind.choice(),
        Some("Ideal")
    );
    circuit.advance(0.05).unwrap();
    circuit.try_set_closed(&contact, true).unwrap();
    assert_eq!(made(&mut circuit, &contact, 7), [true; 7]);
}

#[test]
fn a_whole_frame_takes_in_every_flip_of_the_chatter() {
    // At sixty frames a second all four flips fall inside the first
    // frame after the press, each on its own step; the frame ends made.
    let (mut circuit, contact, led) = switched("PushButton");
    circuit.try_set_choice(&contact, "Bouncy").unwrap();
    circuit.advance(1.0 / 60.0).unwrap();
    circuit.try_set_closed(&contact, true).unwrap();
    circuit.advance(1.0 / 60.0).unwrap();
    assert_eq!(
        circuit
            .netlist
            .element(decode(&contact).unwrap())
            .unwrap()
            .kind
            .contact(),
        Some(true)
    );
    assert!(circuit.chatter.is_empty());
    let lit = circuit
        .solution()
        .element_currents
        .iter()
        .find(|current| current.element_id == led)
        .map(|current| current.amps)
        .unwrap();
    assert!(lit > 1e-3, "{lit} A");
}

#[test]
fn undo_ends_a_chatter_rather_than_letting_it_flip_what_was_undone() {
    let (mut circuit, contact, _) = switched("PushButton");
    circuit.try_set_choice(&contact, "Bouncy").unwrap();
    circuit.advance(0.05).unwrap();
    circuit.try_set_closed(&contact, true).unwrap();
    circuit.undo();
    assert_eq!(made(&mut circuit, &contact, 7), [false; 7]);
}

#[test]
fn a_bouncy_button_says_so_in_its_file_and_an_ideal_one_says_nothing() {
    let (mut circuit, contact, _) = switched("PushButton");
    let ideal = serde_json::to_string(&circuit.to_document()).unwrap();
    assert!(!ideal.contains("bounces"), "{ideal}");
    circuit.try_set_choice(&contact, "Bouncy").unwrap();
    let saved = serde_json::to_string(&circuit.to_document()).unwrap();
    let document: crate::document::Document = serde_json::from_str(&saved).unwrap();
    assert_eq!(
        Circuit::from_document(&document)
            .unwrap()
            .to_document()
            .elements[1]
            .kind,
        ElementKind::PushButton {
            closed: false,
            bounces: true
        }
    );
}
