// SPDX-License-Identifier: MIT OR Apache-2.0

//! The solver interface: what the app is allowed to know about simulation.
//!
//! Seam #2 of the design doc (7.1.2). The app drives an [`Engine`] — advance
//! time, drive a source, read a node back — and never names the MNA core, so
//! that ngspice behind the same interface (L-04) or a second engine entirely is
//! a substitution rather than a rewrite.
//!
//! Time is advanced by whoever calls [`Engine::step`], which is deliberately
//! not the render loop. C-13 puts a scheduler in that position; the point of
//! this trait is that it can.

mod mna_engine;

pub use mna_engine::MnaEngine;

use crate::netlist::CompileError;
use crate::netlist::{ElementId, NetId, Netlist};
use crate::solver::{Charge, DcError, NewtonError, SolveError};

/// Why an engine call failed.
#[derive(Debug, Clone, PartialEq)]
pub enum EngineError {
    /// The circuit could not be solved at this point.
    Solve(DcError),
    /// No element with that id.
    UnknownElement(ElementId),
    /// That element is not a source, so there is nothing to drive.
    NotASource(ElementId),
}

/// What kind of failure this is, for the app to put into words (C-25).
///
/// A code rather than a sentence, because the sentence is beginner English
/// written for a particular audience in a particular language, and that is the
/// app's (ADR 0008). A closed set rather than a string, so the app's table of
/// explanations is exhaustive at compile time and a new failure mode cannot
/// reach a user as a blank.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, tsify::Tsify)]
#[serde(rename_all = "kebab-case")]
pub enum FailureCode {
    /// No wire joins anything, so there is no circuit to solve.
    NothingWired,
    /// The matrix is singular: some part of the circuit has nothing limiting
    /// the current through it, and the answer is unbounded rather than large.
    NoLimit,
    /// Newton ran out of iterations without settling.
    Unsettled,
    /// A fault in the simulator rather than in the circuit. Nothing the user
    /// did causes one, and nothing they can do fixes one.
    Internal,
}

impl EngineError {
    /// Which kind of failure this is.
    ///
    /// Exhaustive on purpose, at every level: a new variant anywhere below
    /// here stops compiling until somebody decides what a user should be told
    /// about it.
    pub fn code(&self) -> FailureCode {
        match self {
            EngineError::Solve(DcError::Compile(CompileError::NoReferenceNode)) => {
                FailureCode::NothingWired
            }
            EngineError::Solve(DcError::Solve(error)) => match error {
                NewtonError::Solve(SolveError::Singular { .. }) => FailureCode::NoLimit,
                // A device's exponential overflowed to infinity before the
                // matrix was even factorised, which is the same runaway
                // arriving one step earlier.
                NewtonError::Solve(SolveError::NotFinite) => FailureCode::NoLimit,
                // The right-hand side is the wrong length, which no circuit
                // can cause and no user can fix.
                NewtonError::Solve(SolveError::DimensionMismatch { .. }) => FailureCode::Internal,
                NewtonError::NotConverged { .. } => FailureCode::Unsettled,
            },
            EngineError::UnknownElement(_) | EngineError::NotASource(_) => FailureCode::Internal,
        }
    }
}

impl From<DcError> for EngineError {
    fn from(error: DcError) -> Self {
        EngineError::Solve(error)
    }
}

impl core::fmt::Display for EngineError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            EngineError::Solve(error) => write!(f, "{error}"),
            EngineError::UnknownElement(id) => write!(f, "no such element: {id:?}"),
            EngineError::NotASource(id) => write!(f, "element {id:?} is not a source"),
        }
    }
}

impl core::error::Error for EngineError {}

/// A simulation engine.
pub trait Engine {
    /// Advances the simulation by `dt` seconds and solves for the new state.
    ///
    /// The engine decides what that means. The MNA core runs a circuit with
    /// a past — a charge held, or anything carried from one step to the next:
    /// a latch, a temperature, a program — as an adaptive transient that
    /// chooses its own steps inside `dt` (C-10, C-48), and re-solves the
    /// operating point of one without (`mna_engine::run_for` decides which).
    fn step(&mut self, dt: f64) -> Result<(), EngineError>;

    /// Hands the engine an edited circuit to go on solving.
    ///
    /// The circuit is replaced; what the engine has accumulated about it is
    /// the engine's to keep or discard. The MNA core keeps the charge in the
    /// reactive elements, because a capacitor does not empty itself because
    /// somebody changed a resistor — which is the same argument that already
    /// keeps simulated time out of here (C-48).
    ///
    /// No result: an edited circuit that cannot be solved is not an error to
    /// report now, it is a failure the next step will run into, in the same
    /// words as any other.
    fn adopt(&mut self, netlist: Netlist);

    /// What the probes have been reading, over time (C-22).
    ///
    /// The engine's, because only the engine knows how finely it stepped: a
    /// circuit moving fast is solved hundreds of times between two frames, and
    /// a history gathered from outside would be a history of the frames.
    fn traces(&self) -> &crate::scope::Traces;

    /// The same, to be read into — reducing a trace to a plot writes into a
    /// buffer the traces own, so the caller gets a view instead of a copy.
    fn traces_mut(&mut self) -> &mut crate::scope::Traces;

    /// Drives a source to a new value, taking effect on the next step.
    fn set_source_voltage(&mut self, source: ElementId, volts: f64) -> Result<(), EngineError>;

    /// Potential of a net at the last solved point, relative to the reference.
    fn node_voltage(&self, net: NetId) -> Option<f64>;

    /// Voltage across a two-terminal element, `+` terminal minus `-` terminal.
    fn voltage_across(&self, element: ElementId) -> Option<f64>;

    /// Current a source is delivering, positive out of its `+` terminal.
    fn source_current(&self, source: ElementId) -> Option<f64>;

    /// Simulated time, in seconds.
    fn time(&self) -> f64;

    /// The bit a part carries between steps — a relay energised, a 555's
    /// output high — or nothing for a part that carries none, or an engine
    /// that does not know (C-64).
    fn latched(&self, _element: ElementId) -> Option<bool> {
        None
    }

    /// What a reactive element is carrying into the next step — a
    /// capacitor's voltage and how fast it is moving, an inductor's current —
    /// or nothing for one that is carrying nothing yet, or an engine that does
    /// not step.
    fn carried(&self, _element: ElementId) -> Option<Charge> {
        None
    }

    /// How hot a part is running and what it is dissipating, for a part
    /// that has a temperature — one with a package in the registry — or
    /// nothing for any other, or an engine that does not know (C-74).
    fn heat(&self, _element: ElementId) -> Option<Heat> {
        None
    }

    /// What an LCD digit's segments have been seeing — how dark each is and
    /// the DC across it — for an LCD digit, or nothing for any other part, or
    /// an engine that does not know (C-76).
    fn glass(&self, _element: ElementId) -> Option<crate::solver::device::lcd::Glass> {
        None
    }

    /// The note a passive buzzer is being fed, in hertz, or nothing while it
    /// is silent, for any other part, or for an engine that does not know
    /// (C-85).
    fn pitch(&self, _element: ElementId) -> Option<f64> {
        None
    }

    /// What an electrolytic has been put through lately — wired backwards,
    /// past its rating — for an electrolytic, or nothing for any other part
    /// or an engine that does not know (C-86).
    fn strain(&self, _element: ElementId) -> Option<crate::solver::device::electrolytic::Strain> {
        None
    }

    /// What a microcontroller's serial monitor has heard since this was last
    /// asked (A-06); nothing for anything else.
    fn take_serial(&mut self, _element: ElementId) -> Vec<u8> {
        Vec::new()
    }

    /// Types `bytes` into a microcontroller's serial monitor.
    fn send_serial(&mut self, _element: ElementId, _bytes: &[u8]) {}

    /// Sets the baud a microcontroller's serial monitor runs at.
    fn set_monitor_baud(&mut self, _element: ElementId, _baud: u32) {}

    /// A microcontroller's monitor baud, and its program's while the
    /// program's serial port is on.
    fn serial_bauds(&self, _element: ElementId) -> Option<(u32, Option<u32>)> {
        None
    }
}

/// A part's temperature, and the power putting it there (C-74).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Heat {
    /// Watts it is turning into heat now.
    pub watts: f64,
    /// Where that has taken it so far, in °C.
    pub celsius: f64,
}

/// Builds the engine for a circuit.
///
/// The only place an engine is chosen. Callers get something that solves
/// circuits and are told nothing about how — which is the whole point of the
/// seam, and the line L-04 will change to offer ngspice as "accurate mode".
pub fn engine_for(netlist: Netlist) -> Result<Box<dyn Engine>, EngineError> {
    Ok(Box::new(MnaEngine::new(netlist)?))
}

#[cfg(test)]
mod tests {
    use crate::netlist::{ElementKind, LedColour, NetId, Netlist};

    use super::*;

    struct Milestone {
        netlist: Netlist,
        battery: ElementId,
        led: ElementId,
        middle: NetId,
    }

    /// Battery, series resistor, LED.
    fn milestone(volts: f64, ohms: f64) -> Milestone {
        let mut netlist = Netlist::new();
        let battery = netlist.add_element(ElementKind::DcSource {
            volts,
            series_ohms: 0.0,
        });
        let resistor = netlist.add_element(ElementKind::Resistor { ohms });
        let led = netlist.add_element(ElementKind::Led {
            colour: LedColour::Red,
        });

        let top = netlist.add_net();
        let middle = netlist.add_net();
        let bottom = netlist.add_net();

        netlist.connect_named(battery, "+", top).unwrap();
        netlist.connect_named(resistor, "a", top).unwrap();
        netlist.connect_named(resistor, "b", middle).unwrap();
        netlist.connect_named(led, "anode", middle).unwrap();
        netlist.connect_named(led, "cathode", bottom).unwrap();
        netlist.connect_named(battery, "-", bottom).unwrap();

        Milestone {
            netlist,
            battery,
            led,
            middle,
        }
    }

    #[test]
    fn a_caller_can_simulate_without_naming_the_engine() {
        // This test is the acceptance: everything below goes through the trait.
        // If the MNA core were renamed tomorrow, nothing here would change.
        let circuit = milestone(9.0, 220.0);
        let led = circuit.led;
        let mut engine: Box<dyn Engine> = engine_for(circuit.netlist).unwrap();

        engine.step(1e-3).unwrap();

        assert!((engine.voltage_across(led).unwrap() - 1.825_281_138_046_504).abs() < 1e-9);
        assert_eq!(engine.time(), 1e-3);
    }

    #[test]
    fn a_circuit_is_solved_before_the_first_step() {
        // A caller that only ever reads still sees a real state, rather than
        // zeroes that look like a flat circuit.
        let circuit = milestone(9.0, 220.0);
        let led = circuit.led;
        let engine = engine_for(circuit.netlist).unwrap();

        assert!(engine.voltage_across(led).unwrap() > 1.5);
        assert_eq!(engine.time(), 0.0);
    }

    #[test]
    fn driving_a_source_changes_the_answer() {
        // Source injection: the value moves, the circuit is not rebuilt, and the
        // next step reflects it. This is what M-19 does on every edit.
        let circuit = milestone(9.0, 220.0);
        let battery = circuit.battery;
        let mut engine: Box<dyn Engine> = engine_for(circuit.netlist).unwrap();

        engine.step(0.0).unwrap();
        let bright = engine.source_current(battery).unwrap();

        engine.set_source_voltage(battery, 3.0).unwrap();
        engine.step(0.0).unwrap();
        let dim = engine.source_current(battery).unwrap();

        assert!(dim < bright / 4.0, "{dim} vs {bright}");
        assert!(dim > 0.0);
    }

    #[test]
    fn a_driven_value_only_takes_effect_on_the_next_step() {
        // The read-back state belongs to the last solve. Anything else would
        // hand the UI a circuit that is part old and part new.
        let circuit = milestone(9.0, 220.0);
        let (battery, led) = (circuit.battery, circuit.led);
        let mut engine: Box<dyn Engine> = engine_for(circuit.netlist).unwrap();

        let before = engine.voltage_across(led).unwrap();
        engine.set_source_voltage(battery, 3.0).unwrap();

        assert_eq!(engine.voltage_across(led), Some(before));
        engine.step(0.0).unwrap();
        assert!(engine.voltage_across(led).unwrap() < before);
    }

    #[test]
    fn node_voltages_read_back_by_net() {
        let circuit = milestone(9.0, 220.0);
        let middle = circuit.middle;
        let mut engine: Box<dyn Engine> = engine_for(circuit.netlist).unwrap();

        engine.step(0.0).unwrap();

        // The LED's anode sits at its forward drop above the reference.
        assert!((engine.node_voltage(middle).unwrap() - 1.825_281_138_046_504).abs() < 1e-9);
    }

    #[test]
    fn time_accumulates_across_steps() {
        let circuit = milestone(9.0, 220.0);
        let mut engine: Box<dyn Engine> = engine_for(circuit.netlist).unwrap();

        for _ in 0..10 {
            engine.step(1e-4).unwrap();
        }

        assert!((engine.time() - 1e-3).abs() < 1e-15);
    }

    #[test]
    fn driving_something_that_is_not_a_source_is_refused() {
        let mut netlist = Netlist::new();
        let battery = netlist.add_element(ElementKind::DcSource {
            volts: 9.0,
            series_ohms: 0.0,
        });
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: 220.0 });
        let top = netlist.add_net();
        let bottom = netlist.add_net();
        netlist.connect_named(battery, "+", top).unwrap();
        netlist.connect_named(resistor, "a", top).unwrap();
        netlist.connect_named(resistor, "b", bottom).unwrap();
        netlist.connect_named(battery, "-", bottom).unwrap();

        let mut engine: Box<dyn Engine> = engine_for(netlist).unwrap();

        assert_eq!(
            engine.set_source_voltage(resistor, 5.0),
            Err(EngineError::NotASource(resistor))
        );
    }

    #[test]
    fn an_unsolvable_circuit_fails_at_construction() {
        // Nothing is wired to anything, so there is nothing to solve.
        let mut netlist = Netlist::new();
        netlist.add_element(ElementKind::Resistor { ohms: 220.0 });

        assert!(matches!(
            engine_for(netlist),
            Err(EngineError::Solve(crate::solver::DcError::Compile(_)))
        ));
    }

    #[test]
    fn a_value_edit_cannot_turn_one_element_into_another() {
        // The netlist guard behind set_source_voltage: pins would stop meaning
        // what the nets think they mean.
        let mut netlist = Netlist::new();
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: 220.0 });

        assert_eq!(
            netlist.set_kind(
                resistor,
                ElementKind::DcSource {
                    volts: 9.0,
                    series_ohms: 0.0
                }
            ),
            Err(crate::netlist::NetlistError::KindMismatch { element: resistor })
        );
        assert_eq!(
            netlist.set_kind(resistor, ElementKind::Resistor { ohms: 470.0 }),
            Ok(())
        );
    }
}
