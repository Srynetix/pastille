// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

const SUPPLY: f64 = 9.0;
/// A frame at sixty a second.
const FRAME: f64 = 1.0 / 60.0;
/// 1 kΩ and 1000 µF: τ is one second, which is a curve a person can watch.
const TAU: f64 = 1.0;

struct Rc {
    circuit: Circuit,
    resistor: String,
    capacitor: String,
}

/// A 9 V supply charging a capacitor through a resistor.
fn rc() -> Rc {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 40.0, 0.0).unwrap();
    let capacitor = circuit.try_add("Capacitor", 80.0, 0.0).unwrap();
    circuit.try_set_value(&resistor, "1k").unwrap();
    circuit.try_set_value(&capacitor, "1000u").unwrap();
    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit
        .try_connect(&resistor, "b", &capacitor, "a")
        .unwrap();
    circuit.try_connect(&capacitor, "b", &battery, "-").unwrap();

    Rc {
        circuit,
        resistor,
        capacitor,
    }
}

/// Volts on the capacitor, whose other pin sits on the reference.
///
/// The net is looked up afresh every time rather than remembered: a wire
/// left with one end loose is debris and is swept away (ADR 0006), so
/// taking a component out and putting it back gives its neighbour a new
/// net — which is exactly the case one of these tests is about.
fn charged(rc: &Rc) -> f64 {
    let anode = rc
        .circuit
        .netlist()
        .element(decode(&rc.capacitor).unwrap())
        .and_then(|element| element.pins.first()?.net)
        .expect("the capacitor's first pin is on a net");

    rc.circuit
        .solution()
        .net_voltages
        .iter()
        .find(|net| net.net_id == encode_net(anode))
        .map(|net| net.volts)
        .expect("the capacitor's node has a voltage")
}

fn run(rc: &mut Rc, seconds: f64) {
    for _ in 0..(seconds / FRAME).round() as usize {
        rc.circuit.advance(FRAME).unwrap();
    }
}

#[test]
fn a_capacitor_charges_over_time_rather_than_arriving_charged() {
    // What C-48 was filed for. Before it, the capacitor was an open circuit
    // at DC, the resistor dropped nothing, and the node sat at the supply
    // from the first frame — measured at 9 V after a full second, where τ
    // is a second.
    let mut rc = rc();

    run(&mut rc, 0.1 * TAU);
    let early = charged(&rc);
    run(&mut rc, 0.9 * TAU);
    let after_tau = charged(&rc);

    // A tenth of the way in it has barely started.
    assert!(early < SUPPLY * 0.2, "{early} V after 0.1τ");
    // And at one time constant it is 1 − 1/e of the way there.
    let expected = SUPPLY * (1.0 - (-1.0f64).exp());
    assert!(
        (after_tau - expected).abs() / expected < 0.02,
        "{after_tau} V at τ against {expected} V"
    );
}

#[test]
fn simulated_time_keeps_up_with_the_frames() {
    let mut rc = rc();

    run(&mut rc, 0.5);

    assert!((rc.circuit.solution().time - 0.5).abs() < 1e-9);
}

#[test]
fn an_edit_does_not_empty_the_capacitor() {
    // Simulated time was already kept out of the engine so that changing a
    // resistor could not rewind the clock. The charge is the same argument,
    // and an edit that emptied the capacitor would be a jump to zero volts
    // in the middle of a curve somebody is watching.
    let mut rc = rc();
    run(&mut rc, 0.5 * TAU);
    let before = charged(&rc);
    assert!(before > 1.0, "nothing to lose: {before} V");

    rc.circuit.try_set_value(&rc.resistor, "2k").unwrap();
    rc.circuit.advance(FRAME).unwrap();

    // Still where it was, give or take the one frame that has passed.
    let after = charged(&rc);
    assert!((after - before).abs() < 0.05, "{before} V became {after} V");
}

#[test]
fn changing_what_it_starts_at_starts_it_again() {
    // The exception that proves the rule. "Starts at" (C-18) can only mean
    // something at the moment it is set, in a simulation that never stops —
    // so this is the one edit that puts the element back where it says.
    let mut rc = rc();
    run(&mut rc, TAU);
    assert!(charged(&rc) > 4.0);

    rc.circuit
        .try_set_initial_condition(&rc.capacitor, "1")
        .unwrap();
    rc.circuit.advance(FRAME).unwrap();

    let after = charged(&rc);
    assert!((0.9..1.3).contains(&after), "{after} V");
}

#[test]
fn a_circuit_with_nothing_to_remember_answers_exactly_as_it_did() {
    // A resistive circuit has no past, so a step over time is the operating
    // point and nothing else — no companion, no error estimate, and none of
    // the three solves a step the adaptive machinery costs. The number is
    // the one the milestone has been pinned to since M-20.
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 40.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 80.0, 0.0).unwrap();
    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
    circuit.try_connect(&led, "cathode", &battery, "-").unwrap();

    circuit.advance(FRAME).unwrap();

    let amps = circuit
        .solution()
        .element_currents
        .iter()
        .find(|current| current.element_id == led)
        .map(|current| current.amps)
        .unwrap();
    assert!((amps - 1.534_827_142_501_358_5e-2).abs() < 1e-9, "{amps} A");
}

#[test]
fn a_coil_takes_time_to_reach_its_current() {
    // The capacitor's mirror image, and the other component C-18 placed
    // that did nothing: an inductor was a plain wire, so the current was
    // there from the first frame instead of climbing to it.
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 40.0, 0.0).unwrap();
    let coil = circuit.try_add("Inductor", 80.0, 0.0).unwrap();
    circuit.try_set_value(&resistor, "100").unwrap();
    // 1 H through 100 Ω: τ = L/R is ten milliseconds.
    circuit.try_set_value(&coil, "1").unwrap();
    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &coil, "a").unwrap();
    circuit.try_connect(&coil, "b", &battery, "-").unwrap();

    let through = |circuit: &Circuit| {
        circuit
            .solution()
            .element_currents
            .iter()
            .find(|current| current.element_id == resistor)
            .map(|current| current.amps.abs())
            .unwrap_or(0.0)
    };

    circuit.advance(FRAME).unwrap();
    let settled = SUPPLY / 100.0;
    // One frame is under two time constants, so it is on its way and not
    // yet there.
    let early = through(&circuit);
    assert!(early > 0.0 && early < settled * 0.95, "{early} A");

    for _ in 0..30 {
        circuit.advance(FRAME).unwrap();
    }
    let later = through(&circuit);
    assert!((later - settled).abs() < settled * 0.01, "{later} A");
}

#[test]
fn a_circuit_that_stops_solving_keeps_what_it_was_holding() {
    // Half a wire is not a circuit, and half a wire is the normal state of
    // one being built. The engine is kept through it, so the charge comes
    // back with the circuit rather than starting again from nothing.
    let mut rc = rc();
    run(&mut rc, 0.5 * TAU);
    let before = charged(&rc);

    // Take the resistor out and put an identical one back.
    rc.circuit.try_remove(&rc.resistor).unwrap();
    rc.circuit.advance(FRAME).unwrap();
    let replacement = rc.circuit.try_add("Resistor", 40.0, 0.0).unwrap();
    rc.circuit.try_set_value(&replacement, "1k").unwrap();
    let battery = rc
        .circuit
        .elements_for_test()
        .into_iter()
        .find(|view| view.kind == "DcSource")
        .unwrap()
        .id;
    rc.circuit
        .try_connect(&battery, "+", &replacement, "a")
        .unwrap();
    rc.circuit
        .try_connect(&replacement, "b", &rc.capacitor, "a")
        .unwrap();
    rc.circuit.advance(FRAME).unwrap();

    // Where it was, plus the frame of charging that has happened since —
    // and nowhere near the zero it would have restarted from.
    let after = charged(&rc);
    assert!(after >= before, "{before} V became {after} V");
    assert!(after - before < 0.2, "{before} V jumped to {after} V");
}

#[test]
fn a_circuit_that_will_not_solve_is_not_solved_again_every_frame() {
    // Retrying would run the whole convergence ladder of C-11 sixty times a
    // second for an answer that is not there. The state says so: a failure
    // stands until something about the circuit changes.
    let mut circuit = Circuit::new();
    circuit.try_add("Resistor", 0.0, 0.0).unwrap();

    circuit.advance(FRAME).unwrap();
    let failure = circuit.solution().failure;
    assert!(failure.is_some());

    for _ in 0..10 {
        circuit.advance(FRAME).unwrap();
    }

    assert_eq!(circuit.solution().failure, failure);
    // And it recovers the moment the circuit becomes one.
    let mut whole = rc();
    whole.circuit.advance(FRAME).unwrap();
    assert!(whole.circuit.solution().failure.is_none());
}
