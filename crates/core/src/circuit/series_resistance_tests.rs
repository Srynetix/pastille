// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

/// A battery wired straight to an LED, which is the circuit the second
/// M-22 session produced and the solver refused.
fn led_across_battery(series: &str) -> Circuit {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 80.0, 0.0).unwrap();
    circuit.try_connect(&battery, "+", &led, "anode").unwrap();
    circuit.try_connect(&led, "cathode", &battery, "-").unwrap();
    if !series.is_empty() {
        circuit.try_set_series_resistance(&battery, series).unwrap();
    }
    circuit
}

#[test]
fn a_fresh_source_is_ideal() {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();

    let view = circuit.placement_of(decode(&battery).unwrap());

    assert!(view.is_some());
    assert_eq!(
        circuit
            .netlist
            .element(decode(&battery).unwrap())
            .unwrap()
            .kind,
        ElementKind::DcSource {
            volts: 9.0,
            series_ohms: 0.0
        }
    );
}

#[test]
fn an_ideal_source_across_a_diode_still_has_no_answer() {
    // Nothing limits the current, so the model's answer really is
    // unbounded. This is the state C-11 investigated and handed here.
    let mut circuit = led_across_battery("");
    circuit.advance(0.016).unwrap();

    assert!(circuit.solution().failure.is_some());
}

#[test]
fn the_same_circuit_solves_once_the_battery_is_real() {
    // A cell has milliohms, and milliohms are the difference between a
    // battery that gets hot and a solver that gives up.
    let mut circuit = led_across_battery("0.1");
    circuit.advance(0.016).unwrap();

    let state = circuit.solution();
    assert!(state.failure.is_none(), "{:?}", state.failure);

    // Enormous, and finite, and the linter's business rather than the
    // solver's (B-02, B-03).
    let amps = state
        .element_currents
        .iter()
        .map(|current| current.amps.abs())
        .fold(0.0, f64::max);
    assert!(
        amps > 1.0,
        "{amps} A — it should be far past the LED's rating"
    );
}

#[test]
fn a_shorted_source_gives_a_finite_current() {
    // The acceptance: + straight to −, which is the other circuit with no
    // answer at all when the source is ideal.
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    circuit.try_set_series_resistance(&battery, "0.5").unwrap();
    circuit.try_connect(&battery, "+", &battery, "-").unwrap();
    circuit.advance(0.016).unwrap();

    let state = circuit.solution();
    assert!(state.failure.is_none(), "{:?}", state.failure);

    let amps = state
        .element_currents
        .iter()
        .find(|current| current.element_id == battery)
        .map(|current| current.amps)
        .expect("the battery reports its current");
    // V/R = 9 / 0.5 = 18 A, leaving by the + terminal.
    assert!((amps.abs() - 18.0).abs() < 1e-6, "{amps} A");
}

#[test]
fn the_internal_resistance_drops_the_terminal_voltage() {
    // What an internal resistance *is*: a 1 Ω cell driving 1 Ω sees half
    // its own voltage at its terminals.
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let load = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
    circuit.try_set_series_resistance(&battery, "1").unwrap();
    circuit.try_set_value(&load, "1").unwrap();
    circuit.try_connect(&battery, "+", &load, "a").unwrap();
    circuit.try_connect(&load, "b", &battery, "-").unwrap();
    circuit.advance(0.016).unwrap();

    let state = circuit.solution();
    let across = state
        .element_currents
        .iter()
        .find(|current| current.element_id == load)
        .map(|current| current.amps)
        .unwrap();
    assert!((across.abs() - 4.5).abs() < 1e-9, "{across} A");
}

#[test]
fn editing_the_voltage_leaves_the_resistance_alone() {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    circuit.try_set_series_resistance(&battery, "0.5").unwrap();

    circuit.try_set_value(&battery, "12").unwrap();

    assert_eq!(
        circuit
            .netlist
            .element(decode(&battery).unwrap())
            .unwrap()
            .kind,
        ElementKind::DcSource {
            volts: 12.0,
            series_ohms: 0.5
        }
    );
}

#[test]
fn only_a_source_has_one() {
    let mut circuit = Circuit::new();
    let resistor = circuit.try_add("Resistor", 0.0, 0.0).unwrap();

    assert!(circuit.try_set_series_resistance(&resistor, "1").is_err());
}

#[test]
fn a_negative_resistance_is_refused() {
    // It would be a source of energy pretending to be a limit on one.
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();

    assert!(circuit.try_set_series_resistance(&battery, "-1").is_err());
}

#[test]
fn clearing_it_makes_the_source_ideal_again() {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    circuit.try_set_series_resistance(&battery, "0.5").unwrap();

    circuit.try_set_series_resistance(&battery, "").unwrap();

    assert_eq!(
        circuit
            .netlist
            .element(decode(&battery).unwrap())
            .unwrap()
            .kind,
        ElementKind::DcSource {
            volts: 9.0,
            series_ohms: 0.0
        }
    );
}
