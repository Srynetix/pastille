// SPDX-License-Identifier: MIT OR Apache-2.0

//! Runs a saved circuit without a browser.
//!
//! C-30, modelled on MAME's `nltool`, which is what makes that project's
//! solver testable outside the emulator (the prior-art study, §2). The same argument
//! applies here: a solver reachable only through a canvas is a solver that can
//! only be checked by clicking, and a regression in it looks like "the LED
//! seems dimmer".
//!
//! Output is CSV, because it diffs, it plots, and it is the format every other
//! tool in this space already reads. Values are written to six significant
//! figures: enough to see a real change, few enough that the last bits of a
//! double do not churn a diff between two runs that agree.

use std::io::Read;
use std::process::ExitCode;

use pastille_core::circuit::Circuit;
use pastille_core::document::Document;
use pastille_core::solver::{
    AdaptiveOptions, InitialCondition, Integration, Transient, TransientOptions, adaptive,
    operating_point, transient,
};
use pastille_core::value;

const USAGE: &str = "\
pastille-run — run a saved circuit without a browser

    pastille-run <circuit.json> [options]
    pastille-run -            [options]      read the circuit from stdin

Options:
    --dc                  operating point only. The default.
    --transient           run over time
    --duration <time>     how long to run for, e.g. 5m, 1
    --step <time>         fixed step, e.g. 10u. Omit for an adaptive one
    --tolerance <volts>   largest local error an adaptive step allows
    --settled             start from the operating point rather than from
                          each component's own initial condition
    --trapezoidal         integrate by the trapezoidal rule rather than by
                          backward Euler: more accurate per step, and it rings
                          rather than damps when the step outruns the circuit
    -h, --help            this

Times and tolerances take the prefixes values do: 10u, 1m, 4k7.
";

fn main() -> ExitCode {
    match run() {
        Ok(output) => {
            print!("{output}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("pastille-run: {message}");
            ExitCode::FAILURE
        }
    }
}

/// What was asked for.
struct Options {
    source: String,
    transient: bool,
    duration: f64,
    step: Option<f64>,
    tolerance: f64,
    start: InitialCondition,
    integration: Integration,
}

fn run() -> Result<String, String> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.iter().any(|a| a == "-h" || a == "--help") {
        return Ok(USAGE.to_string());
    }

    let options = parse(&arguments)?;
    let circuit = open(&options.source)?;

    if options.transient {
        run_transient(&circuit, &options)
    } else {
        run_dc(&circuit)
    }
}

fn parse(arguments: &[String]) -> Result<Options, String> {
    let mut options = Options {
        source: String::new(),
        transient: false,
        duration: 1e-3,
        step: None,
        tolerance: 1e-3,
        start: InitialCondition::AsPlaced,
        integration: Integration::BackwardEuler,
    };

    let mut rest = arguments.iter();
    while let Some(argument) = rest.next() {
        // A flag needing a value, with the value missing, is a mistake worth
        // naming rather than a default worth guessing.
        let mut value_of = |flag: &str| -> Result<f64, String> {
            rest.next()
                .ok_or_else(|| format!("{flag} needs a value"))
                .and_then(|text| value::parse(text).map_err(|error| format!("{flag}: {error}")))
        };

        match argument.as_str() {
            "--dc" => options.transient = false,
            "--transient" => options.transient = true,
            "--settled" => options.start = InitialCondition::OperatingPoint,
            "--trapezoidal" => options.integration = Integration::Trapezoidal,
            "--duration" => options.duration = value_of("--duration")?,
            "--step" => options.step = Some(value_of("--step")?),
            "--tolerance" => options.tolerance = value_of("--tolerance")?,
            flag if flag.starts_with("--") => return Err(format!("unknown option {flag}")),
            path if options.source.is_empty() => options.source = path.to_string(),
            extra => return Err(format!("unexpected argument {extra}")),
        }
    }

    if options.source.is_empty() {
        return Err("no circuit given. Try --help".to_string());
    }
    // `--step` on its own implies a run over time: nobody passes one meaning
    // to discard it.
    if options.step.is_some() {
        options.transient = true;
    }

    Ok(options)
}

/// Reads a circuit from a path, or from stdin for `-`.
fn open(source: &str) -> Result<Circuit, String> {
    let text = if source == "-" {
        let mut text = String::new();
        std::io::stdin()
            .read_to_string(&mut text)
            .map_err(|error| format!("could not read stdin: {error}"))?;
        text
    } else {
        std::fs::read_to_string(source).map_err(|error| format!("{source}: {error}"))?
    };

    let document: Document =
        serde_json::from_str(&text).map_err(|error| format!("{source}: {error}"))?;
    Circuit::from_document(&document).map_err(|error| format!("{source}: {error}"))
}

fn run_dc(circuit: &Circuit) -> Result<String, String> {
    let point = operating_point(circuit.netlist()).map_err(|error| error.to_string())?;

    let mut output = header(circuit);
    let row: Vec<String> = net_order(circuit)
        .iter()
        .map(|&net| number(point.voltage(net).unwrap_or(0.0)))
        .collect();
    output.push_str(&format!("0,{}\n", row.join(",")));
    Ok(output)
}

fn run_transient(circuit: &Circuit, options: &Options) -> Result<String, String> {
    let run: Transient = match options.step {
        Some(step) => transient(
            circuit.netlist(),
            &TransientOptions {
                step,
                duration: options.duration,
                start: options.start,
                integration: options.integration,
            },
        ),
        None => adaptive(
            circuit.netlist(),
            &AdaptiveOptions {
                initial_step: options.duration / 1000.0,
                min_step: options.duration / 1e9,
                max_step: options.duration / 10.0,
                duration: options.duration,
                tolerance: options.tolerance,
                start: options.start,
                integration: options.integration,
            },
        ),
    }
    .map_err(|error| error.to_string())?;

    let order = net_order(circuit);
    let mut output = header(circuit);
    for (index, at) in run.times().iter().enumerate() {
        let row: Vec<String> = order
            .iter()
            .map(|&net| {
                run.series(net)
                    .and_then(|series| series.get(index).copied())
                    .unwrap_or(0.0)
            })
            .map(number)
            .collect();
        output.push_str(&format!("{},{}\n", number(*at), row.join(",")));
    }

    // Said on stderr rather than in the table, so a diff of two runs does not
    // turn over because one of them needed an extra step.
    if run.rejections() > 0 || run.forced() > 0 {
        eprintln!(
            "pastille-run: {} steps rejected, {} forced past the tolerance",
            run.rejections(),
            run.forced()
        );
    }

    Ok(output)
}

/// The nets, in the order the document lists them.
///
/// Stable across runs of the same file, which is what makes two outputs
/// comparable. A slotmap handle is not a name and never appears here.
fn net_order(circuit: &Circuit) -> Vec<pastille_core::netlist::NetId> {
    circuit.netlist().nets().map(|(id, _)| id).collect()
}

fn header(circuit: &Circuit) -> String {
    let names: Vec<String> = (0..net_order(circuit).len())
        .map(|index| format!("net{index}"))
        .collect();
    format!("time,{}\n", names.join(","))
}

/// Six significant figures: enough to see a real change, few enough that the
/// last bits of a double do not churn a diff between two runs that agree.
fn number(value: f64) -> String {
    if value == 0.0 {
        return "0".to_string();
    }
    format!("{value:.6e}")
}
