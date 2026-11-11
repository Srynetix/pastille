// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

const FRAME: f64 = 1.0 / 60.0;

struct Watched {
    circuit: Circuit,
    resistor: String,
    meter: String,
}

/// A 10 kΩ and 1000 µF charging curve — τ of ten seconds — with a
/// voltmeter across the capacitor.
fn charging() -> Watched {
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 40.0, 0.0).unwrap();
    let capacitor = circuit.try_add("Capacitor", 80.0, 0.0).unwrap();
    let meter = circuit.try_add("Voltmeter", 80.0, 40.0).unwrap();
    circuit.try_set_value(&resistor, "10k").unwrap();
    circuit.try_set_value(&capacitor, "1000u").unwrap();
    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit
        .try_connect(&resistor, "b", &capacitor, "a")
        .unwrap();
    circuit.try_connect(&capacitor, "b", &battery, "-").unwrap();
    circuit.try_connect(&meter, "+", &capacitor, "a").unwrap();
    circuit.try_connect(&meter, "-", &capacitor, "b").unwrap();

    Watched {
        circuit,
        resistor,
        meter,
    }
}

fn run(watched: &mut Watched, seconds: f64) {
    for _ in 0..(seconds / FRAME).round() as usize {
        watched.circuit.advance(FRAME).unwrap();
    }
}

/// The plotted block, read back the way the app reads it.
fn plot(watched: &mut Watched, span: f64, columns: usize) -> Vec<f64> {
    let written = watched
        .circuit
        .plot_trace(&watched.meter, span, columns)
        .unwrap();
    let block = watched
        .circuit
        .simulation
        .as_ref()
        .unwrap()
        .engine()
        .traces()
        .plotted();
    assert_eq!(block.len(), written);
    block.to_vec()
}

#[test]
fn a_frame_records_far_more_than_one_sample() {
    // The whole reason the buffer is written from inside the run. At sixty
    // frames a second an app-side buffer would hold one sample per frame,
    // and anything faster than about thirty hertz would not exist.
    let mut watched = charging();

    run(&mut watched, FRAME);

    let depth = watched.circuit.trace_depth(&watched.meter).unwrap();
    assert!(depth > 1, "one frame recorded {depth} samples");
}

#[test]
fn the_trace_is_the_curve_the_voltmeter_read() {
    // Plotted over the whole run, the first column is near zero and the
    // last is where the charge has got to — which for one time constant is
    // 1 − 1/e of the supply.
    let mut watched = charging();
    run(&mut watched, 10.0);

    let plotted = plot(&mut watched, 10.0, 100);

    assert_eq!(plotted.len(), 200, "one low and one high per column");
    let first = plotted[1];
    let last = plotted[plotted.len() - 1];
    assert!(first < 1.0, "started at {first} V");
    let expected = 9.0 * (1.0 - (-1.0f64).exp());
    assert!(
        (last - expected).abs() / expected < 0.05,
        "ended at {last} V against {expected} V"
    );
    // And it only ever went up.
    for column in 0..100 {
        let low = plotted[column * 2];
        let high = plotted[column * 2 + 1];
        assert!(low <= high, "column {column}: {low} above {high}");
    }
}

#[test]
fn a_column_reports_the_lowest_and_highest_it_reached() {
    // A trace longer than the place it is drawn: a hundred columns over a
    // span the trace crosses in thousands of samples. Each column spans
    // real movement, so its low and high are not the same number.
    let mut watched = charging();
    run(&mut watched, 2.0);

    let plotted = plot(&mut watched, 2.0, 100);

    let moved = (0..100)
        .filter(|column| plotted[column * 2] < plotted[column * 2 + 1])
        .count();
    assert!(moved > 50, "only {moved} columns of a hundred spanned any");
}

#[test]
fn a_window_shorter_than_the_run_shows_only_its_end() {
    // The rolling half of a rolling buffer: asking for the last second of
    // a ten-second run starts where that second started, not at zero.
    let mut watched = charging();
    run(&mut watched, 10.0);

    let whole = plot(&mut watched, 10.0, 50);
    let recent = plot(&mut watched, 1.0, 50);

    assert!(whole[1] < 1.0, "the whole run starts near zero");
    assert!(recent[1] > 4.0, "the last second starts at {} V", recent[1]);
    // Both end in the same place, because both end now.
    let ends = (whole[whole.len() - 1], recent[recent.len() - 1]);
    assert!((ends.0 - ends.1).abs() < 0.2, "{ends:?}");
}

#[test]
fn an_edit_bends_the_trace_rather_than_clearing_it() {
    // The charge survives an edit (C-48), so the history should too — and a
    // curve that visibly bends where the resistor changed is the most
    // useful thing a scope in this app can show.
    let mut watched = charging();
    run(&mut watched, 2.0);
    let before = watched.circuit.trace_depth(&watched.meter).unwrap();

    watched
        .circuit
        .try_set_value(&watched.resistor, "1k")
        .unwrap();
    run(&mut watched, 1.0);

    let after = watched.circuit.trace_depth(&watched.meter).unwrap();
    assert!(after > before, "{before} samples became {after}");
}

#[test]
fn a_probe_that_is_deleted_takes_its_trace_with_it() {
    let mut watched = charging();
    run(&mut watched, 1.0);
    assert!(watched.circuit.trace_depth(&watched.meter).unwrap() > 0);

    watched.circuit.try_remove(&watched.meter).unwrap();
    watched.circuit.advance(FRAME).unwrap();

    assert_eq!(watched.circuit.trace_depth(&watched.meter).unwrap(), 0);
}

#[test]
fn a_probe_placed_late_is_read_against_the_right_moments() {
    // Every trace shares one list of times, so a probe that arrives after
    // the run started has to be padded back to the beginning of the buffer.
    // Without that its samples line up against the wrong times for ever.
    let mut watched = charging();
    run(&mut watched, 2.0);

    let ammeter = watched.circuit.try_add("Ammeter", 0.0, 40.0).unwrap();
    run(&mut watched, 1.0);

    let volts = watched.circuit.trace_depth(&watched.meter).unwrap();
    let amps = watched.circuit.trace_depth(&ammeter).unwrap();
    assert_eq!(
        volts, amps,
        "the two traces disagree about how long the run is"
    );
}

#[test]
fn a_resistive_circuit_still_has_a_trace() {
    // Nothing is stored and nothing changes, so every sample is the same
    // number — but a flat line is an answer and an empty scope is not.
    let mut circuit = Circuit::new();
    let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 40.0, 0.0).unwrap();
    let meter = circuit.try_add("Voltmeter", 40.0, 40.0).unwrap();
    circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &battery, "-").unwrap();
    circuit.try_connect(&meter, "+", &resistor, "a").unwrap();
    circuit.try_connect(&meter, "-", &resistor, "b").unwrap();

    for _ in 0..10 {
        circuit.advance(FRAME).unwrap();
    }

    assert!(circuit.trace_depth(&meter).unwrap() > 0);
    let written = circuit.plot_trace(&meter, 1.0, 20).unwrap();
    assert_eq!(written, 40);
    let block = circuit
        .simulation
        .as_ref()
        .unwrap()
        .engine()
        .traces()
        .plotted();
    // The supply, flat, everywhere it was recorded.
    for value in block.iter().filter(|value| value.is_finite()) {
        assert!((value - 9.0).abs() < 1e-9, "{value} V");
    }
}

#[test]
fn nothing_is_plotted_before_anything_is_recorded() {
    let mut circuit = Circuit::new();
    let meter = circuit.try_add("Voltmeter", 0.0, 0.0).unwrap();

    assert_eq!(circuit.plot_trace(&meter, 1.0, 20).unwrap(), 0);
    assert_eq!(circuit.trace_time(), 0.0);
}

#[test]
fn a_nonsensical_window_is_refused_rather_than_guessed() {
    let mut watched = charging();
    run(&mut watched, 1.0);

    assert_eq!(
        watched.circuit.plot_trace(&watched.meter, 0.0, 20).unwrap(),
        0
    );
    assert_eq!(
        watched
            .circuit
            .plot_trace(&watched.meter, -1.0, 20)
            .unwrap(),
        0
    );
    assert_eq!(
        watched
            .circuit
            .plot_trace(&watched.meter, f64::NAN, 20)
            .unwrap(),
        0
    );
    assert_eq!(
        watched.circuit.plot_trace(&watched.meter, 1.0, 0).unwrap(),
        0
    );
}
