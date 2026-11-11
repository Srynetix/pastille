// SPDX-License-Identifier: MIT OR Apache-2.0

//! The solver, against circuits whose answers are known independently of it.
//!
//! C-14. Every other test in the crate checks a part: that a companion has the
//! right conductance, that a step is rejected, that a matrix is not singular.
//! These check the whole thing against arithmetic done outside it — the six
//! circuits of the acceptance, each with the closed form it is held to and the
//! tolerance it is held to within.
//!
//! An integration test on purpose. It reaches the crate the way anything else
//! would, through `netlist` and `solver`, so a change that keeps every unit
//! test passing while breaking the public path fails here.
//!
//! Where a tolerance is loose, the comment says what makes it loose. A number
//! with no reason behind it is a number that will be widened the first time it
//! fails.

use pastille_core::netlist::{ElementId, ElementKind, LedColour, NetId, Netlist};
use pastille_core::solver::device::{
    Channel, Diode, Led, Mosfet, Polarity, SaturatingSource, Waveform, ZenerPart,
};
use pastille_core::solver::{
    CoupledDevice, InitialCondition, Integration, MAX_TERMINALS, NewtonOptions, NewtonSolution,
    Stepper, Transient, TransientOptions, assemble, operating_point, solve_nonlinear_coupled,
    transient,
};

/// Builder for the circuits below: elements joined by named nets.
struct Builder {
    netlist: Netlist,
}

impl Builder {
    fn new() -> Self {
        Builder {
            netlist: Netlist::new(),
        }
    }

    fn net(&mut self) -> NetId {
        self.netlist.add_net()
    }

    /// Adds an element and wires its pins, in pin order, to the given nets.
    fn place(&mut self, kind: ElementKind, pins: &[NetId]) -> ElementId {
        let names = kind.pin_names();
        let id = self.netlist.add_element(kind);
        for (name, net) in names.iter().zip(pins) {
            self.netlist.connect_named(id, name, *net).unwrap();
        }
        id
    }
}

/// Asserts a value is within a relative tolerance of what the maths says.
#[track_caller]
fn close(found: f64, expected: f64, tolerance: f64, what: &str) {
    let error = (found - expected).abs() / expected.abs().max(f64::MIN_POSITIVE);
    assert!(
        error <= tolerance,
        "{what}: {found} against {expected}, {:.3}% out (allowed {:.3}%)",
        error * 100.0,
        tolerance * 100.0
    );
}

#[test]
fn a_resistive_divider_splits_the_supply() {
    // V_mid = V · R2 / (R1 + R2). Exact arithmetic, so the tolerance is there
    // to catch a wrong answer, not a slightly wrong one.
    const SUPPLY: f64 = 9.0;
    const TOP: f64 = 1_000.0;
    const BOTTOM: f64 = 2_000.0;

    let mut builder = Builder::new();
    let (rail, middle, ground) = (builder.net(), builder.net(), builder.net());
    builder.place(
        ElementKind::DcSource {
            volts: SUPPLY,
            series_ohms: 0.0,
        },
        &[rail, ground],
    );
    builder.place(ElementKind::Resistor { ohms: TOP }, &[rail, middle]);
    builder.place(ElementKind::Resistor { ohms: BOTTOM }, &[middle, ground]);

    let point = operating_point(&builder.netlist).unwrap();

    let found = point.voltage(middle).unwrap() - point.voltage(ground).unwrap();
    close(
        found,
        SUPPLY * BOTTOM / (TOP + BOTTOM),
        1e-9,
        "divider output",
    );
}

/// Runs a transient from empty and returns the run.
fn from_empty(netlist: &Netlist, step: f64, duration: f64) -> Transient {
    transient(
        netlist,
        &TransientOptions {
            step,
            duration,
            start: InitialCondition::AsPlaced,
            integration: Integration::default(),
        },
    )
    .expect("the circuit runs")
}

#[test]
fn an_rc_circuit_follows_its_exponential() {
    // v(t) = V·(1 − e^(−t/RC)).
    const SUPPLY: f64 = 9.0;
    const OHMS: f64 = 1_000.0;
    const FARADS: f64 = 1e-6;
    let tau = OHMS * FARADS;
    let step = tau / 1000.0;

    let mut builder = Builder::new();
    let (rail, middle, ground) = (builder.net(), builder.net(), builder.net());
    builder.place(
        ElementKind::DcSource {
            volts: SUPPLY,
            series_ohms: 0.0,
        },
        &[rail, ground],
    );
    builder.place(ElementKind::Resistor { ohms: OHMS }, &[rail, middle]);
    builder.place(
        ElementKind::Capacitor {
            farads: FARADS,
            initial_volts: 0.0,
        },
        &[middle, ground],
    );

    let run = from_empty(&builder.netlist, step, 5.0 * tau);
    let volts = run.series(middle).unwrap();

    for multiple in [1.0, 2.0, 5.0] {
        let index = (multiple * tau / step).round() as usize - 1;
        // 1%: backward Euler is first order, so the error is proportional to
        // the step, and a thousandth of τ leaves it far inside this.
        close(
            volts[index],
            SUPPLY * (1.0 - (-multiple).exp()),
            0.01,
            &format!("RC at {multiple}τ"),
        );
    }
}

#[test]
fn an_rl_circuit_follows_its_exponential() {
    // i(t) = (V/R)·(1 − e^(−t·R/L)), read off the resistor's drop.
    const SUPPLY: f64 = 9.0;
    const OHMS: f64 = 100.0;
    const HENRIES: f64 = 1e-3;
    let tau = HENRIES / OHMS;
    let step = tau / 1000.0;

    let mut builder = Builder::new();
    let (rail, middle, ground) = (builder.net(), builder.net(), builder.net());
    builder.place(
        ElementKind::DcSource {
            volts: SUPPLY,
            series_ohms: 0.0,
        },
        &[rail, ground],
    );
    builder.place(ElementKind::Resistor { ohms: OHMS }, &[rail, middle]);
    builder.place(
        ElementKind::Inductor {
            henries: HENRIES,
            initial_amps: 0.0,
        },
        &[middle, ground],
    );

    let run = from_empty(&builder.netlist, step, 5.0 * tau);
    let volts = run.series(middle).unwrap();

    for multiple in [1.0, 2.0, 5.0] {
        let index = (multiple * tau / step).round() as usize - 1;
        let amps = (SUPPLY - volts[index]) / OHMS;
        close(
            amps,
            SUPPLY / OHMS * (1.0 - (-multiple).exp()),
            0.01,
            &format!("RL at {multiple}τ"),
        );
    }
}

#[test]
fn an_lc_circuit_rings_at_the_frequency_it_should() {
    // A series RLC with a small resistance, stepped: the closed form is the
    // underdamped step response, ringing at ω_d = √(ω₀² − α²) with
    // ω₀ = 1/√(LC) and α = R/2L.
    //
    // A resistance is what makes it excitable from a step at all — a pure LC
    // needs an initial condition, and nothing yet sets one per element.
    const SUPPLY: f64 = 9.0;
    const OHMS: f64 = 2.0;
    const HENRIES: f64 = 1e-3;
    const FARADS: f64 = 1e-6;

    let natural = 1.0 / (HENRIES * FARADS).sqrt();
    let damping = OHMS / (2.0 * HENRIES);
    let ringing = (natural * natural - damping * damping).sqrt();
    let period = 2.0 * core::f64::consts::TAU / (2.0 * ringing);

    let mut builder = Builder::new();
    let (rail, middle, ground) = (builder.net(), builder.net(), builder.net());
    builder.place(
        ElementKind::DcSource {
            volts: SUPPLY,
            series_ohms: 0.0,
        },
        &[rail, ground],
    );
    builder.place(ElementKind::Resistor { ohms: OHMS }, &[rail, middle]);
    let across = builder.net();
    builder.place(
        ElementKind::Inductor {
            henries: HENRIES,
            initial_amps: 0.0,
        },
        &[middle, across],
    );
    builder.place(
        ElementKind::Capacitor {
            farads: FARADS,
            initial_volts: 0.0,
        },
        &[across, ground],
    );

    // Fine enough that the integrator's own damping does not swamp the
    // circuit's: backward Euler loses amplitude, which is the trade C-08
    // documents and offers trapezoidal as the other side of, so this measures
    // the period rather than the peak.
    let step = period / 500.0;
    let run = from_empty(&builder.netlist, step, 3.0 * period);
    let volts = run.series(across).unwrap();

    let peaks = peak_times(run.times(), &volts);
    assert!(
        peaks.len() >= 2,
        "expected it to ring, found {} peaks",
        peaks.len()
    );

    // 2%: the period a first-order scheme reports is slightly long, and the
    // peak is located to within one step either way.
    close(peaks[1] - peaks[0], period, 0.02, "ringing period");

    // It does ring rather than merely wobbling: the first peak overshoots the
    // supply it is settling towards.
    let highest = volts.iter().cloned().fold(f64::MIN, f64::max);
    assert!(highest > SUPPLY * 1.2, "peaked at only {highest} V");
}

/// Times of the local maxima of a series.
fn peak_times(times: &[f64], values: &[f64]) -> Vec<f64> {
    values
        .windows(3)
        .enumerate()
        .filter(|(_, window)| window[1] > window[0] && window[1] >= window[2])
        .map(|(index, _)| times[index + 1])
        .collect()
}

#[test]
fn a_diode_clamp_lands_where_the_junction_equation_says() {
    // A junction with a series resistor has no elementary closed form — the
    // answer is a Lambert W — so the expectation is computed independently by
    // bisecting the one equation the circuit has to satisfy:
    //
    //     (V − v) / R  =  i_diode(v)
    //
    // The independence that matters here is not from the device model, which
    // is the physics both sides agree on, but from the machinery under test: a
    // scalar root-find outside the matrix, against MNA and Newton inside it.
    // Asking the model for its own current rather than restating the Shockley
    // equation is deliberate — a restatement would drift the moment the model
    // gained an emission coefficient, which is exactly what it has.
    const SUPPLY: f64 = 9.0;
    const OHMS: f64 = 1_000.0;

    let mut builder = Builder::new();
    let (rail, middle, ground) = (builder.net(), builder.net(), builder.net());
    builder.place(
        ElementKind::DcSource {
            volts: SUPPLY,
            series_ohms: 0.0,
        },
        &[rail, ground],
    );
    builder.place(ElementKind::Resistor { ohms: OHMS }, &[rail, middle]);
    builder.place(
        ElementKind::Led {
            colour: LedColour::Red,
        },
        &[middle, ground],
    );

    let point = operating_point(&builder.netlist).unwrap();
    let found = point.voltage(middle).unwrap() - point.voltage(ground).unwrap();

    let led = Led::new(LedColour::Red);
    let expected = bisect(|v| (SUPPLY - v) / OHMS - led.forward_current(v));

    close(found, expected, 1e-6, "clamped diode voltage");
}

/// Finds the root of a monotonically decreasing function between 0 and 10 V.
fn bisect(f: impl Fn(f64) -> f64) -> f64 {
    let (mut low, mut high) = (0.0_f64, 10.0_f64);
    for _ in 0..200 {
        let middle = 0.5 * (low + high);
        if f(middle) > 0.0 {
            low = middle;
        } else {
            high = middle;
        }
    }
    0.5 * (low + high)
}

#[test]
fn a_common_emitter_stage_biases_where_beta_says() {
    // Ib = (V − Vbe)/Rb, Ic = β·Ib, Vc = V − Ic·Rc. Vbe is the one term
    // without a closed form, so it is taken from the answer and everything
    // else is held to it.
    const SUPPLY: f64 = 9.0;
    const BASE_OHMS: f64 = 470e3;
    const COLLECTOR_OHMS: f64 = 1e3;
    const BETA: f64 = 100.0;

    let mut builder = Builder::new();
    let (rail, base, collector, ground) =
        (builder.net(), builder.net(), builder.net(), builder.net());
    builder.place(
        ElementKind::DcSource {
            volts: SUPPLY,
            series_ohms: 0.0,
        },
        &[rail, ground],
    );
    builder.place(ElementKind::Resistor { ohms: BASE_OHMS }, &[rail, base]);
    builder.place(
        ElementKind::Resistor {
            ohms: COLLECTOR_OHMS,
        },
        &[rail, collector],
    );
    builder.place(
        ElementKind::Bjt {
            polarity: Polarity::Npn,
        },
        &[base, collector, ground],
    );

    let point = operating_point(&builder.netlist).unwrap();
    let vbe = point.voltage(base).unwrap() - point.voltage(ground).unwrap();
    let vc = point.voltage(collector).unwrap() - point.voltage(ground).unwrap();

    let base_current = (SUPPLY - vbe) / BASE_OHMS;
    let expected = SUPPLY - BETA * base_current * COLLECTOR_OHMS;

    // 2%: Ebers-Moll's collector current is β·Ib only while the reverse
    // junction contributes nothing, which in the active region it nearly does.
    close(vc, expected, 0.02, "collector voltage");
    assert!(
        (0.55..0.75).contains(&vbe),
        "a silicon junction should be around 0.65 V, found {vbe}"
    );
}

// ---------------------------------------------------------------------------
// The saturating source (C-53), which every integrated circuit of ADR 0010
// stands on. No element places one yet, so these reach it the way the solver
// does: the netlist compiled and assembled, and the device handed to Newton on
// the rows of the nets it spans.
//
// Plain Newton, with none of C-11's aids. The acceptance is that a gain of
// 1e5 converges on its own; a test that went through `operating_point` would
// pass by gmin stepping and prove nothing about the device.
// ---------------------------------------------------------------------------

/// An op-amp's numbers, roughly an LM358 on a split supply.
fn amplifier() -> SaturatingSource {
    SaturatingSource {
        gain: 1e5,
        offset: 0.0,
        reference: 0.5,
        headroom_high: 1.5,
        headroom_low: 0.05,
        output_ohms: 50.0,
        source_limit: 0.03,
        sink_limit: 0.02,
        quiescent: 1e-3,
        input_ohms: 1e7,
    }
}

/// A 7805's numbers.
fn regulator() -> SaturatingSource {
    SaturatingSource {
        gain: 0.0,
        offset: 5.0,
        reference: 0.0,
        headroom_high: 2.0,
        headroom_low: 0.0,
        output_ohms: 0.01,
        source_limit: 1.5,
        sink_limit: 1e-3,
        quiescent: 5e-3,
        input_ohms: f64::INFINITY,
    }
}

/// A solve with saturating sources placed on the given nets, in terminal
/// order: non-inverting, inverting, output, high rail, low rail.
struct Solved {
    solution: NewtonSolution,
    rows: Vec<(NetId, Option<usize>)>,
    sources: Vec<(ElementId, Option<usize>)>,
}

impl Solved {
    fn voltage(&self, net: NetId) -> f64 {
        let (_, row) = self
            .rows
            .iter()
            .find(|(candidate, _)| *candidate == net)
            .expect("a net the circuit was built with");
        row.map_or(0.0, |row| self.solution.x[row])
    }

    /// Current a source delivers from its `+` terminal.
    fn delivered(&self, source: ElementId) -> f64 {
        let (_, row) = self
            .sources
            .iter()
            .find(|(candidate, _)| *candidate == source)
            .expect("a source the circuit was built with");
        // The branch unknown is the current *into* `+`.
        -self.solution.x[row.expect("a wired source has a branch row")]
    }
}

fn solve_saturating(
    builder: &Builder,
    parts: &[(SaturatingSource, [NetId; 5])],
    nets: &[NetId],
    sources: &[ElementId],
) -> Solved {
    let map = builder.netlist.compile().unwrap();
    let system = assemble(&builder.netlist, &map);
    let row = |net: NetId| map.index_of(net).and_then(|node| system.row_of_node(node));

    let placed: Vec<CoupledDevice<'_>> = parts
        .iter()
        .map(|(model, pins)| {
            let mut rows = [None; MAX_TERMINALS];
            for (slot, net) in rows.iter_mut().zip(pins) {
                *slot = row(*net);
            }
            CoupledDevice { rows, model }
        })
        .collect();

    let solution = solve_nonlinear_coupled(&system, &[], &placed, &NewtonOptions::default(), &[])
        .expect("plain Newton converges");
    Solved {
        solution,
        rows: nets.iter().map(|&net| (net, row(net))).collect(),
        sources: sources
            .iter()
            .map(|&id| (id, system.row_of_source(id)))
            .collect(),
    }
}

/// An inverting stage on ±9 V: R1 from the input to the inverting input, R2
/// from there to the output, a load on the output.
fn inverting_stage(input: f64) -> (Builder, [NetId; 6]) {
    const R1: f64 = 1e3;
    const R2: f64 = 10e3;

    let mut builder = Builder::new();
    let [signal, summing, output, positive, negative, ground] = [(); 6].map(|_| builder.net());
    builder.place(ElementKind::Ground, &[ground]);
    let supply = |volts| ElementKind::DcSource {
        volts,
        series_ohms: 0.0,
    };
    builder.place(supply(9.0), &[positive, ground]);
    builder.place(supply(9.0), &[ground, negative]);
    builder.place(supply(input), &[signal, ground]);
    builder.place(ElementKind::Resistor { ohms: R1 }, &[signal, summing]);
    builder.place(ElementKind::Resistor { ohms: R2 }, &[summing, output]);
    builder.place(ElementKind::Resistor { ohms: 10e3 }, &[output, ground]);
    (
        builder,
        [signal, summing, output, positive, negative, ground],
    )
}

#[test]
fn an_inverting_stage_gains_minus_r2_over_r1() {
    // Vout = −(R2/R1)·Vin, less a fraction (1 + R2/R1)/A for the finite gain:
    // 1.1e-4 at A = 1e5, which is what the 0.1% is for. The output resistance
    // is inside the loop and costs another factor of A below that.
    let (builder, [_, summing, output, positive, negative, ground]) = inverting_stage(0.5);
    let solved = solve_saturating(
        &builder,
        &[(amplifier(), [ground, summing, output, positive, negative])],
        &[summing, output, ground],
        &[],
    );

    close(solved.voltage(output), -5.0, 1e-3, "inverting stage output");
    // The inverting input is a virtual ground: within a gain's worth of zero.
    assert!(
        solved.voltage(summing).abs() < 1e-3,
        "virtual ground at {} V",
        solved.voltage(summing)
    );
    // Bounded steps, not a crawl: from a cold start the output is thrown at a
    // rail on the first iteration, and the bound is what brings it back.
    assert!(
        solved.solution.iterations <= 20,
        "{} iterations",
        solved.solution.iterations
    );
}

#[test]
fn an_overdriven_stage_sits_at_its_rail_and_no_further() {
    // Asked for −20 V on a −9 V supply. The output stops short of the rail by
    // its headroom, and rises from there only by what the sink current drops
    // across the output resistance — a few milliamps through 50 Ω.
    let (builder, [_, summing, output, positive, negative, ground]) = inverting_stage(2.0);
    let model = amplifier();
    let solved = solve_saturating(
        &builder,
        &[(model, [ground, summing, output, positive, negative])],
        &[output],
        &[],
    );

    let floor = -9.0 + model.headroom_low;
    let found = solved.voltage(output);
    assert!(
        (floor..floor + 0.3).contains(&found),
        "saturated output at {found} V, rail allows {floor} V"
    );
}

#[test]
fn a_gain_of_1e5_swept_through_its_threshold_converges_at_every_point() {
    // Open loop, as a comparator is: the non-inverting input swept across a
    // fixed 4.5 V on the inverting one. The linear region is 90 µV wide, so
    // the sweep crosses it in steps of ten and walks out a volt either side.
    // Every point is a cold start, which is what the live loop does after an
    // edit, and every one has to converge with nothing but plain Newton.
    let fine = (-20..=20).map(|step| 4.5 + f64::from(step) * 1e-5);
    let coarse = (-10..=10).map(|step| 4.5 + f64::from(step) * 0.1);
    let mut sweep: Vec<f64> = fine.chain(coarse).collect();
    sweep.sort_by(f64::total_cmp);

    let mut last = f64::NEG_INFINITY;
    for input in sweep {
        let mut builder = Builder::new();
        let [probe, threshold, output, rail, ground] = [(); 5].map(|_| builder.net());
        let supply = |volts| ElementKind::DcSource {
            volts,
            series_ohms: 0.0,
        };
        builder.place(supply(9.0), &[rail, ground]);
        builder.place(supply(input), &[probe, ground]);
        builder.place(ElementKind::Resistor { ohms: 10e3 }, &[rail, threshold]);
        builder.place(ElementKind::Resistor { ohms: 10e3 }, &[threshold, ground]);
        builder.place(ElementKind::Resistor { ohms: 10e3 }, &[output, ground]);

        let solved = solve_saturating(
            &builder,
            &[(amplifier(), [probe, threshold, output, rail, ground])],
            &[output],
            &[],
        );
        let found = solved.voltage(output);
        // Six iterations at worst today. Twenty leaves room for a model
        // change without letting a crawl through.
        assert!(
            solved.solution.iterations <= 20,
            "{} iterations at {input} V",
            solved.solution.iterations
        );
        // Monotonic across the threshold: an output that went back down as
        // the input went up would be a solve that landed on the wrong side.
        assert!(
            found >= last - 1e-9,
            "at {input} V the output fell to {found} V from {last} V"
        );
        last = found;
    }
    // And it got from one rail to the other.
    assert!(last > 7.0, "fully high at the end of the sweep: {last} V");
}

#[test]
fn a_regulator_holds_its_output_and_its_battery_pays_for_it() {
    // 9 V in, 5 V across 100 Ω: 50 mA to the load, and the battery supplies
    // that plus the regulator's quiescent current. The output sits below 5 V
    // by the load current through 10 mΩ, which is half a millivolt.
    let mut builder = Builder::new();
    let [input, output, ground] = [(); 3].map(|_| builder.net());
    let battery = builder.place(
        ElementKind::DcSource {
            volts: 9.0,
            series_ohms: 0.0,
        },
        &[input, ground],
    );
    builder.place(ElementKind::Resistor { ohms: 100.0 }, &[output, ground]);
    // A regulator has no inputs to compare: both sit on ground, where they
    // change nothing because its gain is zero.
    let model = regulator();
    let solved = solve_saturating(
        &builder,
        &[(model, [ground, ground, output, input, ground])],
        &[output],
        &[battery],
    );

    close(solved.voltage(output), 5.0, 1e-3, "regulated output");
    let load = solved.voltage(output) / 100.0;
    // The quiescent current is at 99% of its value by 9 V (its knee is at 1 V).
    close(
        solved.delivered(battery),
        load + model.quiescent,
        1e-2,
        "battery current",
    );
}

#[test]
fn below_its_dropout_a_regulator_follows_its_input_down() {
    // 6 V in with 2 V of dropout: the most it can make is 4 V, and it does not
    // invent the fifth.
    let mut builder = Builder::new();
    let [input, output, ground] = [(); 3].map(|_| builder.net());
    builder.place(
        ElementKind::DcSource {
            volts: 6.0,
            series_ohms: 0.0,
        },
        &[input, ground],
    );
    builder.place(ElementKind::Resistor { ohms: 100.0 }, &[output, ground]);
    let solved = solve_saturating(
        &builder,
        &[(regulator(), [ground, ground, output, input, ground])],
        &[output],
        &[],
    );

    close(solved.voltage(output), 4.0, 1e-3, "output in dropout");
}

#[test]
fn a_shorted_regulator_delivers_its_limit() {
    // Output straight to ground through a milliohm: an ideal clamp would ask
    // for five thousand amps and a solver that gives up. The part delivers its
    // current limit instead, and survives, which is what the real one does.
    let mut builder = Builder::new();
    let [input, output, ground] = [(); 3].map(|_| builder.net());
    builder.place(
        ElementKind::DcSource {
            volts: 9.0,
            series_ohms: 0.0,
        },
        &[input, ground],
    );
    builder.place(ElementKind::Resistor { ohms: 1e-3 }, &[output, ground]);
    let model = regulator();
    let solved = solve_saturating(
        &builder,
        &[(model, [ground, ground, output, input, ground])],
        &[output],
        &[],
    );

    let current = solved.voltage(output) / 1e-3;
    assert!(
        (0.95 * model.source_limit..=model.source_limit).contains(&current),
        "{current} A into a short, limit {} A",
        model.source_limit
    );
}

// ---------------------------------------------------------------------------
// The op-amp (C-55): the saturating source placed as a component, and solved
// the way the app solves it — through `operating_point`, aids and all. The
// closed forms are the ideal amplifier's; what the finite gain costs is said
// beside each tolerance.
// ---------------------------------------------------------------------------

/// An op-amp stage on a single 9 V battery, as a beginner would build one:
/// V+ on the battery, V− on its negative terminal, and the signal biased to
/// half the supply by a divider so it has room to swing both ways.
struct Stage {
    builder: Builder,
    ground: NetId,
    output: NetId,
}

fn stage(build: impl FnOnce(&mut Builder, [NetId; 5])) -> Stage {
    let mut builder = Builder::new();
    let [plus, minus, output, rail, ground] = [(); 5].map(|_| builder.net());
    builder.place(
        ElementKind::DcSource {
            volts: 9.0,
            series_ohms: 0.0,
        },
        &[rail, ground],
    );
    builder.place(ElementKind::OpAmp, &[plus, minus, output, rail, ground]);
    // A load, so the output is driving something.
    builder.place(ElementKind::Resistor { ohms: 10e3 }, &[output, ground]);
    build(&mut builder, [plus, minus, output, rail, ground]);
    Stage {
        builder,
        ground,
        output,
    }
}

impl Stage {
    fn out(&self) -> f64 {
        let point = operating_point(&self.builder.netlist).expect("an op-amp stage solves");
        point.voltage(self.output).unwrap() - point.voltage(self.ground).unwrap()
    }
}

/// A source of `volts` on `net`, measured from ground.
fn drive(builder: &mut Builder, net: NetId, ground: NetId, volts: f64) {
    builder.place(
        ElementKind::DcSource {
            volts,
            series_ohms: 0.0,
        },
        &[net, ground],
    );
}

#[test]
fn a_follower_follows() {
    // Output wired to the inverting input: Vout = Vin · A/(1 + A), which at
    // A = 1e5 is Vin to one part in 1e5 — 1e-4 covers it with room.
    let stage = stage(|builder, [plus, minus, output, _, ground]| {
        drive(builder, plus, ground, 3.0);
        builder.place(ElementKind::Resistor { ohms: 1.0 }, &[minus, output]);
    });
    close(stage.out(), 3.0, 1e-4, "follower output");
}

#[test]
fn a_non_inverting_stage_gains_one_plus_r2_over_r1() {
    // R1 from the inverting input to ground, R2 from there to the output:
    // Vout = Vin · (1 + R2/R1). 1 V in, a gain of three, 3 V out. The finite
    // gain costs (1 + R2/R1)/A = 3e-5.
    let stage = stage(|builder, [plus, minus, output, _, ground]| {
        drive(builder, plus, ground, 1.0);
        builder.place(ElementKind::Resistor { ohms: 10e3 }, &[minus, ground]);
        builder.place(ElementKind::Resistor { ohms: 20e3 }, &[minus, output]);
    });
    close(stage.out(), 3.0, 1e-3, "non-inverting output");
}

#[test]
fn an_inverting_stage_on_one_battery_swings_about_its_bias() {
    // A single supply cannot go below zero, so the non-inverting input is
    // held at 4.5 V and the stage inverts about that: Vout = Vb − (R2/R1)·
    // (Vin − Vb). 4 V in, half a volt under the bias, times two: 5.5 V out.
    let stage = stage(|builder, [plus, minus, output, _, ground]| {
        let signal = builder.net();
        drive(builder, plus, ground, 4.5);
        drive(builder, signal, ground, 4.0);
        builder.place(ElementKind::Resistor { ohms: 10e3 }, &[signal, minus]);
        builder.place(ElementKind::Resistor { ohms: 20e3 }, &[minus, output]);
    });
    close(stage.out(), 5.5, 1e-3, "inverting output");
}

#[test]
fn an_op_amp_asked_for_more_than_its_supply_gives_what_the_part_can() {
    // A gain of three on 4 V asks for 12 V from a 9 V battery. An LM358 gets
    // to within a volt and a half of V+, and a little less than that into a
    // load, so the answer is just under 7.5 V — never 12, and never 9.
    let stage = stage(|builder, [plus, minus, output, _, ground]| {
        drive(builder, plus, ground, 4.0);
        builder.place(ElementKind::Resistor { ohms: 10e3 }, &[minus, ground]);
        builder.place(ElementKind::Resistor { ohms: 20e3 }, &[minus, output]);
    });
    let found = stage.out();
    assert!((7.3..=7.5).contains(&found), "saturated at {found} V");
}

// ---------------------------------------------------------------------------
// The signal generator (ADR 0011), through the live stepper the canvas uses:
// sixty calls a second, each picking up where the last one stopped, each told
// the time it starts at.
// ---------------------------------------------------------------------------

/// Every step the live stepper takes over `seconds`, as (time, volts on `net`
/// measured from `ground`), a frame at a time.
fn live(netlist: &Netlist, net: NetId, ground: NetId, seconds: f64) -> Vec<(f64, f64)> {
    let mut stepper = Stepper::start(
        netlist,
        InitialCondition::AsPlaced,
        Integration::BackwardEuler,
    )
    .expect("a signal circuit starts");
    let frame = 1.0 / 60.0;
    let mut samples = Vec::new();
    let mut from = 0.0;
    while from < seconds - 1e-12 {
        stepper
            .advance(netlist, from, frame, &mut |into, point| {
                let volts = point.voltage(net).unwrap() - point.voltage(ground).unwrap();
                samples.push((from + into, volts));
            })
            .expect("every frame solves");
        from += frame;
    }
    samples
}

fn generator(waveform: Waveform, volts: f64, hertz: f64) -> ElementKind {
    ElementKind::SignalGenerator {
        waveform,
        volts,
        hertz,
    }
}

#[test]
fn a_divider_on_a_sine_is_half_of_it_at_every_instant() {
    // Nothing reactive, so each sample is the source's value at the sample's
    // own time, halved. A step read at its start rather than its end would lag
    // by a step and fail here by the slope times the step.
    const HERTZ: f64 = 2.0;
    let mut builder = Builder::new();
    let [live_net, middle, ground] = [(); 3].map(|_| builder.net());
    builder.place(generator(Waveform::Sine, 5.0, HERTZ), &[live_net, ground]);
    builder.place(ElementKind::Resistor { ohms: 1e3 }, &[live_net, middle]);
    builder.place(ElementKind::Resistor { ohms: 1e3 }, &[middle, ground]);

    let samples = live(&builder.netlist, middle, ground, 1.0);
    assert!(samples.len() >= 60, "{} samples in a second", samples.len());
    for (at, volts) in samples {
        let expected = 2.5 * (core::f64::consts::TAU * HERTZ * at).sin();
        assert!(
            (volts - expected).abs() < 1e-6,
            "{volts} V at {at} s, expected {expected}"
        );
    }
}

#[test]
fn the_stepper_lands_on_every_edge_of_a_square_wave() {
    // 1 Hz: edges every half second. Some step must end on each one, to the
    // billionth of a cycle the signal counts as on the edge.
    let mut builder = Builder::new();
    let [live_net, ground] = [(); 2].map(|_| builder.net());
    builder.place(generator(Waveform::Square, 5.0, 1.0), &[live_net, ground]);
    builder.place(ElementKind::Resistor { ohms: 1e3 }, &[live_net, ground]);

    let samples = live(&builder.netlist, live_net, ground, 2.0);
    for edge in [0.5, 1.0, 1.5] {
        assert!(
            samples.iter().any(|(at, _)| (at - edge).abs() < 1e-9),
            "no step ended on the edge at {edge} s"
        );
    }
    // And each half is flat: no sample shows a value between the two levels.
    for (at, volts) in &samples {
        assert!((volts.abs() - 5.0).abs() < 1e-9, "{volts} V at {at} s");
    }
}

#[test]
fn a_square_wave_into_an_rc_charges_and_discharges_as_the_textbook_says() {
    // τ = RC = 0.1 s against a half period of 0.5 s: each half settles to
    // within e^-5 of its level before the next edge. So after the first
    // edge the capacitor starts from +5 (less e^-5 of the swing) and heads
    // for −5: v(t) = −5 + (v₀ + 5)·e^−(t − 0.5)/τ.
    const TAU: f64 = 0.1;
    let mut builder = Builder::new();
    let [live_net, top, ground] = [(); 3].map(|_| builder.net());
    builder.place(generator(Waveform::Square, 5.0, 1.0), &[live_net, ground]);
    builder.place(ElementKind::Resistor { ohms: 1e3 }, &[live_net, top]);
    builder.place(
        ElementKind::Capacitor {
            farads: TAU / 1e3,
            initial_volts: 0.0,
        },
        &[top, ground],
    );

    let samples = live(&builder.netlist, top, ground, 1.0);
    let start = 5.0 * (1.0 - (-0.5 / TAU).exp());
    // The low half only: at 1 s the wave goes high again, and sixty frames of
    // a sixtieth can end a hair either side of it.
    let low_half = samples
        .iter()
        .filter(|(at, _)| *at > 0.5 + 1e-9 && *at < 1.0 - 1e-9);
    for &(at, volts) in low_half {
        let expected = -5.0 + (start + 5.0) * (-(at - 0.5) / TAU).exp();
        // Backward Euler at the steps the live loop can afford: its local
        // error is held to a millivolt per step by C-10, and it gathers over
        // a few dozen of them — 50 mV is a percent of the swing.
        assert!(
            (volts - expected).abs() < 0.05,
            "{volts} V at {at} s, expected {expected}"
        );
    }
}

// ---------------------------------------------------------------------------
// The 555 (ADR 0012), whose latch is carried from step to step the way charge
// is, and whose timing is the textbook's.
// ---------------------------------------------------------------------------

#[test]
fn a_555_astable_blinks_at_the_frequency_the_textbook_gives() {
    // R1 from VCC to DIS, R2 from DIS to the capacitor, THR and TRIG on the
    // capacitor. It charges through R1 + R2 from a third of the supply to two
    // thirds, and discharges through R2 alone: high for ln 2·(R1 + R2)·C, low
    // for ln 2·R2·C. The first high is longer — the capacitor starts empty,
    // not at a third — so the edges are measured after it.
    const R1: f64 = 1e3;
    const R2: f64 = 10e3;
    const C: f64 = 47e-6;
    let high = core::f64::consts::LN_2 * (R1 + R2) * C;
    let low = core::f64::consts::LN_2 * R2 * C;

    let mut builder = Builder::new();
    let [vcc, discharge, timing, output, ground] = [(); 5].map(|_| builder.net());
    builder.place(
        ElementKind::DcSource {
            volts: 9.0,
            series_ohms: 0.0,
        },
        &[vcc, ground],
    );
    builder.place(
        ElementKind::Timer,
        &[vcc, ground, timing, timing, discharge, output],
    );
    builder.place(ElementKind::Resistor { ohms: R1 }, &[vcc, discharge]);
    builder.place(ElementKind::Resistor { ohms: R2 }, &[discharge, timing]);
    builder.place(
        ElementKind::Capacitor {
            farads: C,
            initial_volts: 0.0,
        },
        &[timing, ground],
    );
    builder.place(ElementKind::Resistor { ohms: 1e3 }, &[output, ground]);

    let samples = live(&builder.netlist, output, ground, 6.0);
    // Edges: where the output crosses half the supply, one way or the other.
    let mut rising = Vec::new();
    let mut falling = Vec::new();
    for pair in samples.windows(2) {
        let ((_, before), (at, after)) = (pair[0], pair[1]);
        if before < 4.5 && after >= 4.5 {
            rising.push(at);
        }
        if before >= 4.5 && after < 4.5 {
            falling.push(at);
        }
    }
    assert!(rising.len() >= 5, "rising edges at {rising:?}");

    // Skip the first cycle, then average: one sixtieth of a second is the
    // live loop's frame, and each edge can land anywhere in a step.
    let periods: Vec<f64> = rising
        .windows(2)
        .skip(1)
        .map(|pair| pair[1] - pair[0])
        .collect();
    let period = periods.iter().sum::<f64>() / periods.len() as f64;
    // 3%: the latch moves once per step, and the step the stepper settles on
    // near a crossing is a few milliseconds against a period of 684.
    close(period, high + low, 0.03, "astable period");

    let highs: Vec<f64> = rising
        .iter()
        .skip(1)
        .filter_map(|&up| {
            falling
                .iter()
                .find(|&&down| down > up)
                .map(|down| down - up)
        })
        .collect();
    let high_time = highs.iter().sum::<f64>() / highs.len() as f64;
    close(high_time, high, 0.05, "time high");
}

#[test]
fn a_zener_holds_a_battery_down_to_its_voltage() {
    // 9 V through 330 Ω into a 5.1 V Zener, cathode up. The current is
    // (9 − V)/330, and the Zener's reverse junction carries Is·(e^(w/Vt) − 1)
    // with w = V − (Vz − Vt·ln(1 + Izt/Is)): rated Vz at its test current Izt.
    // The answer is found by bisecting that, not by asking the solver.
    const IS: f64 = 1e-14;
    const VT: f64 = 0.025_852;
    let part = ZenerPart::V5_1;
    let shift = part.volts() - VT * (1.0 + part.test_amps() / IS).ln();
    let expected = bisect(|v| (9.0 - v) / 330.0 - IS * (((v - shift) / VT).exp() - 1.0));

    let mut builder = Builder::new();
    let [rail, top, ground] = [(); 3].map(|_| builder.net());
    builder.place(
        ElementKind::DcSource {
            volts: 9.0,
            series_ohms: 0.0,
        },
        &[rail, ground],
    );
    builder.place(ElementKind::Resistor { ohms: 330.0 }, &[rail, top]);
    builder.place(ElementKind::Zener { part }, &[ground, top]);

    let point = operating_point(&builder.netlist).unwrap();
    let found = point.voltage(top).unwrap() - point.voltage(ground).unwrap();
    close(found, expected, 1e-6, "Zener voltage");
    // And it is the rated voltage less what a lighter current costs: a few
    // tens of millivolts at 12 mA against a 49 mA rating.
    assert!((5.0..5.1).contains(&found), "{found} V");
}

/// A MOSFET with its gate held at `gate` volts and its drain through `ohms`
/// to 9 V; the drain's voltage above ground.
fn mosfet_stage(gate: f64, ohms: f64) -> f64 {
    let mut builder = Builder::new();
    let [rail, gate_net, drain, ground] = [(); 4].map(|_| builder.net());
    let supply = |volts| ElementKind::DcSource {
        volts,
        series_ohms: 0.0,
    };
    builder.place(supply(9.0), &[rail, ground]);
    builder.place(supply(gate), &[gate_net, ground]);
    builder.place(ElementKind::Resistor { ohms }, &[rail, drain]);
    builder.place(
        ElementKind::Mosfet {
            channel: Channel::N,
        },
        &[gate_net, drain, ground],
    );
    let point = operating_point(&builder.netlist).unwrap();
    point.voltage(drain).unwrap() - point.voltage(ground).unwrap()
}

#[test]
fn a_mosfet_switched_on_is_the_resistance_the_square_law_gives() {
    // 5 V on a 2N7000's gate is 2.9 V above its threshold, and with 220 Ω
    // above it the drain sits low, in the linear region, where
    //   (9 − V)/220 = K·(Vov·V − V²/2).
    // Bisected from that, not asked of the solver.
    let fet = Mosfet::new(Channel::N);
    let (k, overdrive) = (fet.transconductance, 5.0 - fet.threshold);
    let expected = bisect(|v| (9.0 - v) / 220.0 - k * (overdrive * v - 0.5 * v * v));
    assert!(
        expected < overdrive,
        "the textbook answer is in the linear region"
    );

    // 0.1%: the threshold's rounding moves the overdrive by e^(−58) of a volt.
    close(
        mosfet_stage(5.0, 220.0),
        expected,
        1e-3,
        "drain, switched on",
    );
}

#[test]
fn a_mosfet_barely_on_passes_the_square_laws_current_whatever_its_load() {
    // Half a volt above threshold, and the channel pinches off: the drain
    // current is K/2·Vov² — 8.8 mA — whether the load is 100 Ω or 500.
    let fet = Mosfet::new(Channel::N);
    let overdrive = 0.5;
    let expected = 0.5 * fet.transconductance * overdrive * overdrive;
    for ohms in [100.0, 500.0] {
        let amps = (9.0 - mosfet_stage(fet.threshold + overdrive, ohms)) / ohms;
        // 0.2%: the rounding at the threshold adds a few microvolts of
        // overdrive at ten widths above it.
        close(amps, expected, 2e-3, "saturated drain current");
    }
}

// ---------------------------------------------------------------------------
// The parts that came after the acceptance: each placed as a component and
// solved the way the app solves it, against the formula its datasheet or its
// textbook gives — written out here, not asked of the model, wherever the
// formula is the thing being promised.
// ---------------------------------------------------------------------------

/// Five volts across `top` over `bottom`, and what the middle settles at.
fn divider(top: ElementKind, bottom: ElementKind) -> f64 {
    let mut builder = Builder::new();
    let [rail, middle, ground] = [(); 3].map(|_| builder.net());
    drive(&mut builder, rail, ground, 5.0);
    builder.place(top, &[rail, middle]);
    builder.place(bottom, &[middle, ground]);
    let point = operating_point(&builder.netlist).unwrap();
    point.voltage(middle).unwrap() - point.voltage(ground).unwrap()
}

#[test]
fn a_thermistor_follows_its_beta_equation() {
    // R(T) = R25 · e^(B·(1/T − 1/298.15 K)), with the sheet's 10 kΩ and
    // B = 3950 written out rather than borrowed from the model: the curve is
    // what the part promises, so it is what is checked. Under a 10 kΩ top,
    // 2.5 V at 25 °C exactly, and the divider's arithmetic either side.
    const AT_25: f64 = 10e3;
    const BETA: f64 = 3950.0;
    for celsius in [-20.0, 0.0, 25.0, 60.0, 100.0] {
        let ohms = AT_25 * (BETA * (1.0 / (celsius + 273.15) - 1.0 / 298.15)).exp();
        let found = divider(
            ElementKind::Resistor { ohms: 10e3 },
            ElementKind::Thermistor { celsius },
        );
        close(
            found,
            5.0 * ohms / (10e3 + ohms),
            1e-9,
            &format!("thermistor divider at {celsius} °C"),
        );
    }
}

#[test]
fn a_loaded_potentiometer_is_its_thevenin_equivalent() {
    // A 10 kΩ track across 9 V with its wiper at x, loaded by 4.7 kΩ: the
    // wiper is V·x behind R·x·(1 − x), divided by the load. Linear, so the
    // tolerance only catches a wrong answer — a wiper read from the wrong
    // end, or a load that is not seen.
    const SUPPLY: f64 = 9.0;
    const TRACK: f64 = 10e3;
    const LOAD: f64 = 4.7e3;
    for wiper in [0.1, 0.3, 0.5, 0.9] {
        let mut builder = Builder::new();
        let [rail, tap, ground] = [(); 3].map(|_| builder.net());
        drive(&mut builder, rail, ground, SUPPLY);
        // Wiper at 0 is pin a, so a on ground makes x the fraction of the
        // supply the unloaded wiper sees.
        builder.place(
            ElementKind::Potentiometer { ohms: TRACK, wiper },
            &[ground, tap, rail],
        );
        builder.place(ElementKind::Resistor { ohms: LOAD }, &[tap, ground]);
        let point = operating_point(&builder.netlist).unwrap();
        let found = point.voltage(tap).unwrap() - point.voltage(ground).unwrap();

        let open = SUPPLY * wiper;
        let behind = TRACK * wiper * (1.0 - wiper);
        close(
            found,
            open * LOAD / (LOAD + behind),
            1e-9,
            &format!("wiper at {wiper}"),
        );
    }
}

#[test]
fn a_tmp36_reads_half_a_volt_and_ten_millivolts_a_degree() {
    // Vout = 0.5 V + 10 mV/°C (Analog Devices' TMP36 sheet), on 5 V and read
    // by a meter's worth of load. Its 60 Ω output costs 60 ppm into 1 MΩ,
    // and its finite gain a few more: 1e-3 is well clear of both and still
    // a tenth of a degree.
    for celsius in [-40.0, 0.0, 25.0, 100.0, 125.0] {
        let mut builder = Builder::new();
        let [supply, output, ground] = [(); 3].map(|_| builder.net());
        drive(&mut builder, supply, ground, 5.0);
        builder.place(ElementKind::Tmp36 { celsius }, &[supply, output, ground]);
        builder.place(ElementKind::Resistor { ohms: 1e6 }, &[output, ground]);
        let point = operating_point(&builder.netlist).unwrap();
        let found = point.voltage(output).unwrap() - point.voltage(ground).unwrap();
        close(
            found,
            0.5 + 0.01 * celsius,
            1e-3,
            &format!("TMP36 at {celsius} °C"),
        );
    }
}

#[test]
fn a_7805_on_the_board_holds_five_volts() {
    // The saturating source's regulator is held to its sheet above; this is
    // the part as it is placed — in, gnd, out — through every aid the app
    // solves with. 9 V in and 100 Ω out: 5 V less 50 mA through its 10 mΩ.
    use pastille_core::solver::device::RegulatorPart;
    let mut builder = Builder::new();
    let [input, output, ground] = [(); 3].map(|_| builder.net());
    drive(&mut builder, input, ground, 9.0);
    builder.place(
        ElementKind::Regulator {
            part: RegulatorPart::L7805,
        },
        &[input, ground, output],
    );
    builder.place(ElementKind::Resistor { ohms: 100.0 }, &[output, ground]);
    let point = operating_point(&builder.netlist).unwrap();
    let found = point.voltage(output).unwrap() - point.voltage(ground).unwrap();
    close(found, 5.0, 1e-3, "7805 output");
}

/// A DB107 fed `volts` on its AC pins, with `ohms` across + and −: what the
/// load sees.
fn rectified(volts: f64, ohms: f64) -> f64 {
    let mut builder = Builder::new();
    let [ac1, ac2, plus, minus] = [(); 4].map(|_| builder.net());
    drive(&mut builder, ac1, ac2, volts);
    builder.place(ElementKind::Bridge, &[ac1, ac2, plus, minus]);
    builder.place(ElementKind::Resistor { ohms }, &[plus, minus]);
    let point = operating_point(&builder.netlist).unwrap();
    point.voltage(plus).unwrap() - point.voltage(minus).unwrap()
}

#[test]
fn a_bridge_costs_two_junctions_whichever_way_it_is_fed() {
    // Two of the four diodes in series with the load, the same two drops
    // whichever polarity: (V − 2v)/R = i_diode(v), bisected outside the
    // matrix as the clamp above is. The other two leak, and each diode has
    // the sheet's 10 µA at 1000 V across it besides — 10 nS, which
    // moves the answer by about a ten-millionth: 1e-6 holds it with room.
    const OHMS: f64 = 1e3;
    let junction = Diode::default();
    for volts in [5.0, 12.0] {
        let drop = bisect(|v| (volts - 2.0 * v) / OHMS - junction.current(v));
        let expected = volts - 2.0 * drop;
        close(rectified(volts, OHMS), expected, 1e-6, "bridge, fed +");
        close(rectified(-volts, OHMS), expected, 1e-6, "bridge, fed −");
    }
}

#[test]
fn a_transformer_steps_a_sine_down_by_its_turns_ratio() {
    // 10 V at 50 Hz across a 2:1 primary, 1 kΩ across the secondary. Coupled
    // perfectly, a transformer is an ideal one behind its primary copper R1,
    // with the magnetising inductance L1 across it and the secondary's copper
    // R2 = R1/n² in series with the load:
    //
    //     Zp = jωL1 ∥ n²(R2 + RL)          the primary's load, reflected
    //     Vs = V · Zp/(R1 + Zp) / n · RL/(R2 + RL)
    //
    // The figures are the model's typical ones, said in its module (10 H,
    // 10 Ω); the arithmetic is the textbook's.
    use pastille_core::solver::device::transformer::{PRIMARY_HENRIES, PRIMARY_OHMS};
    const VOLTS: f64 = 10.0;
    const HERTZ: f64 = 50.0;
    const RATIO: f64 = 2.0;
    const LOAD: f64 = 1e3;

    let mut builder = Builder::new();
    let [p1, p2, s1, s2] = [(); 4].map(|_| builder.net());
    builder.place(generator(Waveform::Sine, VOLTS, HERTZ), &[p1, p2]);
    builder.place(ElementKind::Transformer { ratio: RATIO }, &[p1, p2, s1, s2]);
    builder.place(ElementKind::Resistor { ohms: LOAD }, &[s1, s2]);

    // Complex arithmetic by hand, as (re, im) pairs.
    let omega = core::f64::consts::TAU * HERTZ;
    let r2 = PRIMARY_OHMS / (RATIO * RATIO);
    let reflected = RATIO * RATIO * (r2 + LOAD);
    let x = omega * PRIMARY_HENRIES;
    // jX ∥ R = jXR/(R + jX) = (X²R + jXR²)/(R² + X²)
    let denominator = reflected * reflected + x * x;
    let zp = (
        x * x * reflected / denominator,
        x * reflected * reflected / denominator,
    );
    // |Zp / (R1 + Zp)|
    let gain = (zp.0.hypot(zp.1)) / (PRIMARY_OHMS + zp.0).hypot(zp.1);
    let expected = VOLTS * gain / RATIO * LOAD / (r2 + LOAD);

    // Half a second is twenty-five cycles; the last is measured peak to peak,
    // which leaves out the magnetising current's start-up offset, still
    // decaying over its L1/R1 second but flat across one cycle.
    let step = 1e-5;
    let run = from_empty(&builder.netlist, step, 0.5);
    let (secondary, back) = (run.series(s1).unwrap(), run.series(s2).unwrap());
    let cycle = (1.0 / HERTZ / step).round() as usize;
    let last = secondary.len() - cycle..secondary.len();
    let across = last.map(|index| secondary[index] - back[index]);
    let (low, high) = across.fold((f64::MAX, f64::MIN), |(low, high), v| {
        (low.min(v), high.max(v))
    });
    // 0.1%: it lands within a few hundredths of that. The copper alone moves
    // the answer by half a percent, so a model that lost either winding's
    // resistance, or wound its secondary to the wrong inductance, fails here.
    close((high - low) / 2.0, expected, 1e-3, "secondary amplitude");
}
