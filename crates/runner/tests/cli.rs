// SPDX-License-Identifier: MIT OR Apache-2.0

//! The runner, driven the way anything else would drive it.
//!
//! Through the built binary rather than through its internals, because what
//! C-30 promises is a command: a wrong flag has to fail, a missing file has to
//! say which, and two runs of the same circuit have to produce the same bytes
//! or "diffable" means nothing.

use std::io::Write;
use std::process::{Command, Output, Stdio};

const RUNNER: &str = env!("CARGO_BIN_EXE_pastille-run");

/// The milestone circuit: battery, resistor, LED.
const MILESTONE: &str = r#"{"schema":1,"elements":[
 {"kind":{"DcSource":{"volts":9.0,"series_ohms":0.0}},"x":0.0,"y":0.0,"turns":0},
 {"kind":{"Resistor":{"ohms":220.0}},"x":80.0,"y":0.0,"turns":0},
 {"kind":{"Led":{"colour":"Red"}},"x":160.0,"y":0.0,"turns":0}],
 "nets":[
 {"pins":[{"element":0,"pin":"+"},{"element":1,"pin":"a"}]},
 {"pins":[{"element":1,"pin":"b"},{"element":2,"pin":"anode"}]},
 {"pins":[{"element":2,"pin":"cathode"},{"element":0,"pin":"-"}]}]}"#;

/// An RC circuit with a time constant of one millisecond.
const RC: &str = r#"{"schema":1,"elements":[
 {"kind":{"DcSource":{"volts":9.0,"series_ohms":0.0}},"x":0.0,"y":0.0,"turns":0},
 {"kind":{"Resistor":{"ohms":1000.0}},"x":80.0,"y":0.0,"turns":0},
 {"kind":{"Capacitor":{"farads":0.000001,"initial_volts":0.0}},"x":160.0,"y":0.0,"turns":0}],
 "nets":[
 {"pins":[{"element":0,"pin":"+"},{"element":1,"pin":"a"}]},
 {"pins":[{"element":1,"pin":"b"},{"element":2,"pin":"a"}]},
 {"pins":[{"element":2,"pin":"b"},{"element":0,"pin":"-"}]}]}"#;

/// Runs the binary with a circuit on stdin.
fn run(circuit: &str, arguments: &[&str]) -> Output {
    let mut child = Command::new(RUNNER)
        .arg("-")
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the runner is built");

    child
        .stdin
        .as_mut()
        .expect("stdin is piped")
        .write_all(circuit.as_bytes())
        .expect("the circuit is written");

    child.wait_with_output().expect("the runner finishes")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("output is text")
}

#[test]
fn it_solves_an_operating_point() {
    let output = run(MILESTONE, &[]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = stdout(&output);
    let mut lines = text.lines();
    assert_eq!(lines.next(), Some("time,net0,net1,net2"));

    let row: Vec<&str> = lines.next().expect("one row").split(',').collect();
    assert_eq!(row[0], "0", "a DC point is at time zero");
    let led: f64 = row[2].parse().expect("a number");
    assert!((1.7..1.9).contains(&led), "the LED sits at {led} V");
}

#[test]
fn it_runs_a_circuit_over_time() {
    let output = run(RC, &["--transient", "--duration", "5m", "--step", "500u"]);

    assert!(output.status.success());
    let text = stdout(&output);
    assert_eq!(text.lines().count(), 11, "a header and ten steps");

    // Charging, so every sample is above the one before it.
    let volts: Vec<f64> = text
        .lines()
        .skip(1)
        .map(|line| line.split(',').nth(2).unwrap().parse().unwrap())
        .collect();
    for pair in volts.windows(2) {
        assert!(pair[1] > pair[0], "{} then {}", pair[0], pair[1]);
    }
}

#[test]
fn it_chooses_its_own_steps_when_not_told() {
    // No `--step`, so the adaptive loop of C-10 runs and the samples are not
    // evenly spaced.
    let output = run(RC, &["--transient", "--duration", "5m"]);

    assert!(output.status.success());
    let times: Vec<f64> = stdout(&output)
        .lines()
        .skip(1)
        .map(|line| line.split(',').next().unwrap().parse().unwrap())
        .collect();

    assert!(times.len() > 5, "only {} samples", times.len());
    for pair in times.windows(2) {
        assert!(pair[1] > pair[0], "time must run forwards");
    }
    // The widest gap, not the last one: the final step is cut short so the run
    // lands exactly on the duration asked for, which says nothing about how
    // the step was chosen.
    let gaps: Vec<f64> = times.windows(2).map(|pair| pair[1] - pair[0]).collect();
    let widest = gaps.iter().cloned().fold(f64::MIN, f64::max);
    assert!(
        widest > gaps[0] * 2.0,
        "the step should grow as the circuit settles: started at {}, reached {widest}",
        gaps[0]
    );
}

#[test]
fn a_step_on_its_own_means_a_run_over_time() {
    // Nobody passes `--step` meaning to discard it.
    let output = run(RC, &["--step", "1m", "--duration", "5m"]);

    assert!(output.status.success());
    assert!(stdout(&output).lines().count() > 2);
}

#[test]
fn two_runs_of_the_same_circuit_agree_exactly() {
    // The whole of "diffable". Anything that made this untrue — a handle in
    // the output, a map iterated in hash order, a timestamp — would make the
    // tool useless for spotting a regression.
    let once = run(RC, &["--transient", "--duration", "5m", "--step", "500u"]);
    let twice = run(RC, &["--transient", "--duration", "5m", "--step", "500u"]);

    assert_eq!(stdout(&once), stdout(&twice));
}

#[test]
fn starting_settled_is_different_from_starting_empty() {
    let empty = run(RC, &["--transient", "--duration", "5m", "--step", "500u"]);
    let settled = run(
        RC,
        &[
            "--transient",
            "--duration",
            "5m",
            "--step",
            "500u",
            "--settled",
        ],
    );

    assert_ne!(stdout(&empty), stdout(&settled));
}

#[test]
fn the_integration_rule_is_selectable_and_changes_the_answer() {
    // C-08's "selectable per analysis", from outside: the same circuit, the
    // same schedule, and a different curve — trapezoidal reaching further up
    // the charging curve in each step than backward Euler does.
    let euler = run(RC, &["--transient", "--duration", "5m", "--step", "500u"]);
    let trapezoid = run(
        RC,
        &[
            "--transient",
            "--duration",
            "5m",
            "--step",
            "500u",
            "--trapezoidal",
        ],
    );

    assert!(trapezoid.status.success());
    assert_ne!(stdout(&euler), stdout(&trapezoid));

    // Both are still charging the same capacitor towards the same 9 V, which
    // is what says the difference is the rule and not a broken run. Column
    // two is the net between the resistor and the capacitor, as above.
    for output in [&euler, &trapezoid] {
        let last = stdout(output).lines().last().unwrap().to_string();
        let volts: f64 = last.split(',').nth(2).unwrap().parse().unwrap();
        assert!((volts - 9.0).abs() < 0.2, "ended at {volts} V: {last}");
    }
}

#[test]
fn it_explains_itself() {
    let output = run(MILESTONE, &["--help"]);

    assert!(output.status.success());
    assert!(stdout(&output).contains("--transient"));
}

#[test]
fn an_unknown_option_is_refused() {
    // Rather than ignored: a misspelled flag that runs anyway gives an answer
    // to a question nobody asked.
    let output = run(MILESTONE, &["--transiant"]);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--transiant"));
}

#[test]
fn a_flag_with_no_value_is_refused() {
    let output = run(MILESTONE, &["--duration"]);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--duration"));
}

#[test]
fn a_file_that_is_not_there_says_which() {
    let output = Command::new(RUNNER)
        .arg("no-such-circuit.json")
        .output()
        .expect("the runner runs");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("no-such-circuit.json"));
}

#[test]
fn something_that_is_not_a_circuit_is_refused() {
    let output = run("not a circuit", &[]);

    assert!(!output.status.success());
}

#[test]
fn a_circuit_that_will_not_solve_says_why() {
    // A battery straight across an LED: C-17's reproduction, and the runner
    // should report it rather than print an empty table.
    let shorted = r#"{"schema":1,"elements":[
     {"kind":{"DcSource":{"volts":9.0,"series_ohms":0.0}},"x":0.0,"y":0.0,"turns":0},
     {"kind":{"Led":{"colour":"Red"}},"x":80.0,"y":0.0,"turns":0}],
     "nets":[
     {"pins":[{"element":0,"pin":"+"},{"element":1,"pin":"anode"}]},
     {"pins":[{"element":1,"pin":"cathode"},{"element":0,"pin":"-"}]}]}"#;

    let output = run(shorted, &[]);

    assert!(!output.status.success());
    assert!(!String::from_utf8_lossy(&output.stderr).is_empty());
}

/// Every circuit the app ships, as files on disk.
fn shipped_examples() -> Vec<std::path::PathBuf> {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../app/src/examples");
    let mut found: Vec<_> = std::fs::read_dir(&folder)
        .expect("the examples are beside the app")
        .map(|entry| entry.expect("a readable entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect();
    found.sort();
    found
}

#[test]
fn every_shipped_example_opens_from_its_file_and_solves() {
    // C-105. `examples.ts` says `just run app/src/examples/milestone.json`
    // works on any of them: held here, by path rather than through stdin, at
    // its operating point and over a tenth of a second, so a change to the
    // file format that breaks an example fails in seconds rather than only
    // in the browser suite.
    let examples = shipped_examples();
    assert!(examples.len() > 50, "found {}", examples.len());
    for example in &examples {
        for arguments in [&["--dc"][..], &["--transient", "--duration", "100m"][..]] {
            let output = Command::new(RUNNER)
                .arg(example)
                .args(arguments)
                .output()
                .expect("the runner is built");
            assert!(
                output.status.success(),
                "{} {arguments:?}: {}",
                example.display(),
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(
                !output.stdout.is_empty(),
                "{} {arguments:?}",
                example.display()
            );
        }
    }
}
