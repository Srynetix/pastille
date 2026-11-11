// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

#[test]
fn a_capacitor_and_an_inductor_can_be_placed() {
    // C-18: the solver has modelled both since C-06 and C-07, and until
    // the registry offered them nothing could put one on a canvas.
    let mut circuit = Circuit::new();

    assert!(circuit.try_add("Capacitor", 0.0, 0.0).is_ok());
    assert!(circuit.try_add("Inductor", 40.0, 0.0).is_ok());
}

#[test]
fn their_values_take_the_prefixes_people_write() {
    // `100n`, `10u`, `1m` — the way the part is labelled and sold.
    let mut circuit = Circuit::new();
    let capacitor = circuit.try_add("Capacitor", 0.0, 0.0).unwrap();
    let inductor = circuit.try_add("Inductor", 40.0, 0.0).unwrap();

    circuit.try_set_value(&capacitor, "10u").unwrap();
    circuit.try_set_value(&inductor, "1m").unwrap();

    // Compared with a tolerance rather than exactly: `10u` is ten times a
    // millionth, and in binary that is 9.999999999999999e-6. Displaying it
    // rounds back to "10u"; asserting on it must not pretend otherwise.
    let of = |id: &str| circuit.netlist.element(decode(id).unwrap()).unwrap().kind;
    let ElementKind::Capacitor {
        farads,
        initial_volts,
    } = of(&capacitor)
    else {
        panic!("not a capacitor")
    };
    assert!((farads - 10e-6).abs() < 1e-18, "{farads} F");
    assert_eq!(initial_volts, 0.0);

    let ElementKind::Inductor {
        henries,
        initial_amps,
    } = of(&inductor)
    else {
        panic!("not an inductor")
    };
    assert!((henries - 1e-3).abs() < 1e-15, "{henries} H");
    assert_eq!(initial_amps, 0.0);
}

#[test]
fn a_fresh_capacitor_is_the_one_in_the_kit() {
    let mut circuit = Circuit::new();
    let capacitor = circuit.try_add("Capacitor", 0.0, 0.0).unwrap();

    let views = circuit.elements_for_test();
    let view = views.iter().find(|view| view.id == capacitor).unwrap();

    assert_eq!(view.label, "100 nF");
    assert_eq!(view.value.as_deref(), Some("100n"));
    assert_eq!(view.unit, "F");
}

#[test]
fn an_initial_condition_can_be_set_and_cleared() {
    let mut circuit = Circuit::new();
    let capacitor = circuit.try_add("Capacitor", 0.0, 0.0).unwrap();

    circuit.try_set_initial_condition(&capacitor, "5").unwrap();
    let charged = circuit.elements_for_test();
    assert_eq!(
        charged
            .iter()
            .find(|view| view.id == capacitor)
            .unwrap()
            .initial_condition,
        Some(5.0)
    );

    circuit.try_set_initial_condition(&capacitor, "").unwrap();
    let cleared = circuit.elements_for_test();
    assert_eq!(
        cleared
            .iter()
            .find(|view| view.id == capacitor)
            .unwrap()
            .initial_condition,
        Some(0.0)
    );
}

#[test]
fn the_initial_condition_is_in_the_unit_the_element_carries() {
    // Volts for a capacitor, amps for an inductor: the quantity it holds
    // between steps is the only one it can be given a head start in.
    let mut circuit = Circuit::new();
    let capacitor = circuit.try_add("Capacitor", 0.0, 0.0).unwrap();
    let inductor = circuit.try_add("Inductor", 40.0, 0.0).unwrap();

    let views = circuit.elements_for_test();
    let unit = |id: &str| {
        views
            .iter()
            .find(|view| view.id == id)
            .unwrap()
            .initial_unit
            .clone()
    };

    assert_eq!(unit(&capacitor), "V");
    assert_eq!(unit(&inductor), "A");
}

#[test]
fn only_something_with_a_past_has_an_initial_condition() {
    let mut circuit = Circuit::new();
    let resistor = circuit.try_add("Resistor", 0.0, 0.0).unwrap();

    assert!(circuit.try_set_initial_condition(&resistor, "1").is_err());
}

#[test]
fn editing_the_value_leaves_the_initial_condition_alone() {
    let mut circuit = Circuit::new();
    let capacitor = circuit.try_add("Capacitor", 0.0, 0.0).unwrap();
    circuit.try_set_initial_condition(&capacitor, "5").unwrap();

    circuit.try_set_value(&capacitor, "10u").unwrap();

    let ElementKind::Capacitor {
        farads,
        initial_volts,
    } = circuit
        .netlist
        .element(decode(&capacitor).unwrap())
        .unwrap()
        .kind
    else {
        panic!("not a capacitor")
    };
    assert!((farads - 10e-6).abs() < 1e-18, "{farads} F");
    assert_eq!(initial_volts, 5.0);
}

#[test]
fn a_charged_capacitor_starts_a_run_charged() {
    // What the field is for: a transient from `AsPlaced` begins where the
    // component says it begins, so a discharge curve is as reachable as a
    // charging one.
    use crate::solver::{InitialCondition, Integration, TransientOptions, transient};

    let mut netlist = Netlist::new();
    let capacitor = netlist.add_element(ElementKind::Capacitor {
        farads: 1e-6,
        initial_volts: 5.0,
    });
    let resistor = netlist.add_element(ElementKind::Resistor { ohms: 1_000.0 });
    let top = netlist.add_net();
    let bottom = netlist.add_net();
    netlist.connect_named(capacitor, "a", top).unwrap();
    netlist.connect_named(capacitor, "b", bottom).unwrap();
    netlist.connect_named(resistor, "a", top).unwrap();
    netlist.connect_named(resistor, "b", bottom).unwrap();

    let tau = 1e-3;
    let run = transient(
        &netlist,
        &TransientOptions {
            step: tau / 1000.0,
            duration: tau,
            start: InitialCondition::AsPlaced,
            integration: Integration::default(),
        },
    )
    .unwrap();

    // Discharging through the resistor: V·e^(−t/τ), so 1/e of 5 V at τ.
    let volts = run.series(top).unwrap();
    let settled = volts.last().unwrap() - run.series(bottom).unwrap().last().unwrap();
    let expected = 5.0 * (-1.0f64).exp();
    assert!(
        (settled - expected).abs() / expected < 0.01,
        "{settled} V against {expected} V"
    );
}
