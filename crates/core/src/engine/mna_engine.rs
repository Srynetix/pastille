// SPDX-License-Identifier: MIT OR Apache-2.0

//! The MNA core behind the [`Engine`] interface.

use slotmap::SecondaryMap;

use crate::netlist::{ElementId, ElementKind, NetId, Netlist};
use crate::registry::{Thermal, rating_of};
use crate::scope::Traces;
use crate::solver::device::electrolytic::{self, Strain};
use crate::solver::device::lcd::{self, Glass};
use crate::solver::device::pitch::{self, Pitch};
use crate::solver::heat;
use crate::solver::{
    InitialCondition, Integration, OperatingPoint, Stepper, operating_point, operating_point_near,
};

use super::{Engine, EngineError, Heat};

/// Drives the in-house modified nodal analysis core.
///
/// Owns its netlist: the app edits its own copy and hands the engine a new one
/// when the circuit changes, so a half-applied edit can never be what gets
/// solved. Retuning a value in place — a source driven live — needs even less.
pub struct MnaEngine {
    netlist: Netlist,
    state: OperatingPoint,
    /// The run in progress, for a circuit that has a past to carry.
    ///
    /// `None` when nothing on the canvas stores anything, which is the common
    /// case and the cheap one: with no capacitor and no coil, a step over time
    /// answers exactly what the operating point answers, and there is no reason
    /// to pay three solves a step to find that out (C-48).
    run: Option<Stepper>,
    /// What every probe has read, kept over time (C-22).
    ///
    /// Here rather than a layer up because this is where the steps are: a
    /// circuit moving fast takes hundreds of them between two frames, and a
    /// buffer written from outside would only ever see the last of each frame.
    traces: Traces,
    time: f64,
    /// Every part that warms, with its package (C-74). Looked up once per
    /// circuit rather than once per step.
    packaged: Vec<(ElementId, Thermal)>,
    /// How hot each of them is, in °C. A part not in here yet is at the
    /// room's temperature, which is where every part starts.
    ///
    /// Beside the run rather than in it. The heat changes nothing electrical,
    /// so it needs no place in the solve, and a circuit of resistors — which
    /// has no run, because it has no past — still warms, without paying
    /// three solves a step for a past it does not have.
    celsius: SecondaryMap<ElementId, f64>,
    /// What each LCD digit's segments have been seeing (C-76). Beside the run
    /// for the heat's reasons: what a segment shows changes nothing
    /// electrical, and a digit on a battery — no past, so no run — still has
    /// to darken over the crystal's response. A digit not in here yet has
    /// seen nothing.
    glass: SecondaryMap<ElementId, Glass>,
    /// What each passive buzzer has been hearing (C-85). Beside the run as
    /// the glass is, and fed every step it takes, because a note is many
    /// cycles a frame and the last step of each would hear none of them.
    pitch: SecondaryMap<ElementId, Pitch>,
    /// What each electrolytic has been put through (C-86). Every step, so a
    /// reversal on an alternating drive, or a spike past its rating, is seen
    /// even when no frame ends on it.
    strain: SecondaryMap<ElementId, Strain>,
}

impl MnaEngine {
    /// Builds the engine and solves the circuit once, so a caller that never
    /// steps still has a state to read.
    pub fn new(netlist: Netlist) -> Result<Self, EngineError> {
        let state = operating_point(&netlist)?;
        Ok(MnaEngine {
            run: run_for(&netlist),
            packaged: packaged(&netlist),
            netlist,
            state,
            traces: Traces::default(),
            time: 0.0,
            celsius: SecondaryMap::new(),
            glass: SecondaryMap::new(),
            pitch: SecondaryMap::new(),
            strain: SecondaryMap::new(),
        })
    }

    /// The circuit being solved.
    pub fn netlist(&self) -> &Netlist {
        &self.netlist
    }
}

/// Every part with a package to warm, and the package.
fn packaged(netlist: &Netlist) -> Vec<(ElementId, Thermal)> {
    netlist
        .elements()
        .filter_map(|(id, element)| Some((id, rating_of(&element.kind).thermal?)))
        .collect()
}

/// Takes every part that warms `seconds` further, at the power `point` has it
/// dissipating. A part too loosely wired to say dissipates nothing, and cools.
fn warm(
    netlist: &Netlist,
    packaged: &[(ElementId, Thermal)],
    celsius: &mut SecondaryMap<ElementId, f64>,
    point: &OperatingPoint,
    seconds: f64,
) {
    let room = netlist.room_celsius();
    for &(id, thermal) in packaged {
        let Some(element) = netlist.element(id) else {
            continue;
        };
        // Never a temperature that is not one: a part whose power cannot be
        // said is taken to make none, and one carried in as nonsense starts
        // again from the room (C-96).
        let watts = heat::watts(element, point)
            .filter(|watts| watts.is_finite())
            .unwrap_or(0.0);
        let was = celsius
            .get(id)
            .copied()
            .filter(|was| was.is_finite())
            .unwrap_or(room);
        let now = heat::warm(&thermal, was, room, watts, seconds);
        celsius.insert(id, if now.is_finite() { now } else { room });
    }
}

/// Takes every LCD digit's glass `seconds` further, with what `point` has
/// across its segments.
fn turn(
    netlist: &Netlist,
    glass: &mut SecondaryMap<ElementId, Glass>,
    point: &OperatingPoint,
    seconds: f64,
) {
    for (id, element) in netlist.elements() {
        let Some(across) = lcd::across(element, point) else {
            continue;
        };
        let mut now = glass.get(id).copied().unwrap_or_default();
        now.advance(&across, seconds);
        glass.insert(id, now);
    }
}

/// Gives every passive buzzer what `point` has across it at `time`, in
/// seconds of the run.
fn listen(
    netlist: &Netlist,
    pitch: &mut SecondaryMap<ElementId, Pitch>,
    point: &OperatingPoint,
    time: f64,
) {
    for (id, element) in netlist.elements() {
        let Some(across) = pitch::across(element, point) else {
            continue;
        };
        let mut now = pitch.get(id).copied().unwrap_or_default();
        now.advance(across, time);
        pitch.insert(id, now);
    }
}

/// Tells every electrolytic what `point` has across it at `time`.
fn strain(
    netlist: &Netlist,
    strained: &mut SecondaryMap<ElementId, Strain>,
    point: &OperatingPoint,
    time: f64,
) {
    for (id, element) in netlist.elements() {
        let Some((across, rated)) = electrolytic::across(element, point) else {
            continue;
        };
        let mut now = strained.get(id).copied().unwrap_or_default();
        now.advance(across, rated, time);
        strained.insert(id, now);
    }
}

/// A run for a circuit that has something to remember or something that moves
/// on its own, and nothing for one that has neither.
///
/// A signal generator is the second kind (ADR 0011): a circuit of resistors on
/// one has no past, but every moment of it is a different answer, so it needs
/// a clock the operating point does not have.
fn run_for(netlist: &Netlist) -> Option<Stepper> {
    let reactive = netlist
        .elements()
        .any(|(_, element)| element.kind.initial_condition().is_some());
    // A 555 carries a bit from one step to the next (ADR 0012), which is a
    // past as surely as a charge is.
    let latched = netlist.elements().any(|(_, element)| {
        matches!(
            element.kind,
            ElementKind::Timer
                | ElementKind::Relay { .. }
                | ElementKind::Decoder
                | ElementKind::Servo
                | ElementKind::Counter
                | ElementKind::MotorDriver
                | ElementKind::ShiftRegister
                | ElementKind::Gate { .. }
                | ElementKind::Inverter
                | ElementKind::Schmitt
                | ElementKind::Decade
                | ElementKind::Lm3914
                // How hot it is, and whether it has given way (C-120).
                | ElementKind::Fuse { .. }
                // What it has given (C-121).
                | ElementKind::Cell { .. }
                // Two currents and their rates, as an inductor's is (C-118).
                | ElementKind::Transformer { .. }
                | ElementKind::FlipFlop
                | ElementKind::Hall { .. }
                | ElementKind::Pir { .. }
                // A program running (ADR 0017).
                | ElementKind::Uno
        )
    });
    if !reactive && !latched && !crate::solver::device::signal::has_signal(netlist) {
        return None;
    }

    // From what the elements were placed holding rather than from the settled
    // operating point: a capacitor that arrives already charged never shows the
    // one thing worth watching, and "starts at" (C-18) is the field that says
    // otherwise.
    //
    // Backward Euler, which is the default and stays it here: the canvas is
    // where somebody who cannot yet tell an artefact from a component is
    // watching, and trapezoidal's ringing is the artefact that would fool them
    // (C-08). Selecting it per analysis is what the runner is for.
    Stepper::start(
        netlist,
        InitialCondition::AsPlaced,
        Integration::BackwardEuler,
    )
    .ok()
}

impl Engine for MnaEngine {
    fn step(&mut self, dt: f64) -> Result<(), EngineError> {
        match self.run.as_mut() {
            // A circuit with a past: the run carries the charge from one call
            // to the next, and chooses its own steps inside this one (C-10).
            // Every step it takes is offered to the traces, which is the whole
            // reason a scope can show something a frame is too slow to see.
            Some(run) => {
                let netlist = &self.netlist;
                let traces = &mut self.traces;
                let (packaged, celsius) = (&self.packaged, &mut self.celsius);
                let glass = &mut self.glass;
                let heard = &mut self.pitch;
                let strained = &mut self.strain;
                let base = self.time;
                // Each part warms over every step the run takes, at that
                // step's power, so a 555 blinking faster than a frame heats
                // its resistor by what it averages rather than by whatever
                // the frame happened to end on.
                let mut reached = 0.0;
                let mut record = |into: f64, point: &OperatingPoint| {
                    traces.record(netlist, point, base + into);
                    warm(netlist, packaged, celsius, point, into - reached);
                    // And each LCD segment answers the RMS of every step, so
                    // a drive faster than a frame is seen as what it averages.
                    turn(netlist, glass, point, into - reached);
                    // And each passive buzzer hears every step, which is
                    // where the cycles of its note are.
                    listen(netlist, heard, point, base + into);
                    strain(netlist, strained, point, base + into);
                    reached = into;
                };

                if let Some(point) = run.advance(netlist, base, dt, &mut record)? {
                    self.state = point;
                }
            }
            // A resistive circuit has no past, so every moment is the operating
            // point and dt only moves the clock. One sample, because there is
            // only one answer however finely it is asked for.
            None => {
                // From the last answer rather than from nothing: the same
                // circuit a millisecond on is nearly always nearly there.
                self.state = operating_point_near(&self.netlist, &self.state)?;
                self.traces
                    .record(&self.netlist, &self.state, self.time + dt);
                warm(
                    &self.netlist,
                    &self.packaged,
                    &mut self.celsius,
                    &self.state,
                    dt,
                );
                turn(&self.netlist, &mut self.glass, &self.state, dt);
                listen(&self.netlist, &mut self.pitch, &self.state, self.time + dt);
                strain(&self.netlist, &mut self.strain, &self.state, self.time + dt);
            }
        }
        self.time += dt;
        Ok(())
    }

    fn traces(&self) -> &Traces {
        &self.traces
    }

    fn traces_mut(&mut self) -> &mut Traces {
        &mut self.traces
    }

    fn adopt(&mut self, netlist: Netlist) {
        // What the reactive elements are holding, less anything whose initial
        // condition has just been edited. "Starts at" can only mean something
        // at the moment it is set, in a simulation that never stops, so that
        // one edit restarts the element rather than being ignored for ever.
        // The programs running carry on through an edit, as the charge
        // does (ADR 0017).
        let (mcus, last) = self
            .run
            .as_mut()
            .map(Stepper::take_mcus)
            .unwrap_or_default();
        let mut carried = self
            .run
            .take()
            .map(Stepper::into_charge)
            .unwrap_or_default();
        for (id, element) in netlist.elements() {
            let before = self
                .netlist
                .element(id)
                .and_then(|was| was.kind.initial_condition());
            if before != element.kind.initial_condition() {
                carried.remove(id);
            }
            // A part replaced by another kind (C-137) is a new part: it starts
            // empty, cold and unread, whatever the old one was holding.
            if self.netlist.element(id).is_some_and(|was| {
                core::mem::discriminant(&was.kind) != core::mem::discriminant(&element.kind)
            }) {
                carried.remove(id);
                self.celsius.remove(id);
                self.traces.forget(id);
            }
            // A fuse replaced, or made another, starts cold (C-120); a
            // battery whose dial is turned, or swapped, starts from the dial
            // (C-121).
            if matches!(
                element.kind,
                ElementKind::Fuse { .. } | ElementKind::Cell { .. }
            ) && self.netlist.element(id).map(|was| was.kind) != Some(element.kind)
            {
                carried.remove(id);
            }
        }

        self.netlist = netlist;
        // A part keeps its temperature through an edit, as a capacitor keeps
        // its charge: a resistor does not cool because its neighbour changed.
        // One that is gone takes its temperature with it, and one just placed
        // starts at the room's.
        self.packaged = packaged(&self.netlist);
        let packaged = &self.packaged;
        self.celsius
            .retain(|id, _| packaged.iter().any(|&(part, _)| part == id));
        // A digit's glass keeps what it was showing through an edit, as a part
        // keeps its temperature. The DC it was averaging does not carry over:
        // it was the old circuit's drive, and it would take half a minute to
        // fade out of the new one's (C-76).
        let netlist = &self.netlist;
        self.glass.retain(|id, _| {
            netlist
                .element(id)
                .is_some_and(|element| matches!(element.kind, ElementKind::LcdDigit))
        });
        for (_, glass) in self.glass.iter_mut() {
            glass.restart_dc();
        }
        // A buzzer keeps hearing through an edit, so a knob turned slides the
        // note rather than stopping it.
        self.strain.retain(|id, _| {
            netlist
                .element(id)
                .is_some_and(|element| matches!(element.kind, ElementKind::Electrolytic { .. }))
        });
        self.pitch.retain(|id, _| {
            netlist
                .element(id)
                .is_some_and(|element| matches!(element.kind, ElementKind::PassiveBuzzer))
        });
        self.run = run_for(&self.netlist).map(|mut run| {
            run.resume(carried);
            run.resume_mcus(mcus, last);
            run
        });

        // A probe that has been deleted has no history worth keeping. The rest
        // keep theirs, so an edit shows up as a bend in the trace rather than
        // as a scope that went blank — which is the same continuity the charge
        // is kept for, and the more interesting half of watching a circuit.
        self.traces.prune(&self.netlist);
    }

    fn set_source_voltage(&mut self, source: ElementId, volts: f64) -> Result<(), EngineError> {
        let element = self
            .netlist
            .element(source)
            .ok_or(EngineError::UnknownElement(source))?;

        match element.kind {
            ElementKind::DcSource { series_ohms, .. } => self
                .netlist
                .set_kind(source, ElementKind::DcSource { volts, series_ohms })
                .map_err(|_| EngineError::UnknownElement(source)),
            _ => Err(EngineError::NotASource(source)),
        }
    }

    fn node_voltage(&self, net: NetId) -> Option<f64> {
        self.state.voltage(net)
    }

    fn voltage_across(&self, element: ElementId) -> Option<f64> {
        self.state.voltage_across(element)
    }

    fn source_current(&self, source: ElementId) -> Option<f64> {
        self.state.source_current(source)
    }

    fn time(&self) -> f64 {
        self.time
    }

    fn carried(&self, element: ElementId) -> Option<crate::solver::Charge> {
        self.run.as_ref()?.carried(element)
    }

    fn heat(&self, element: ElementId) -> Option<Heat> {
        self.packaged.iter().find(|&&(id, _)| id == element)?;
        let part = self.netlist.element(element)?;
        Some(Heat {
            watts: heat::watts(part, &self.state).unwrap_or(0.0),
            celsius: self
                .celsius
                .get(element)
                .copied()
                .unwrap_or(self.netlist.room_celsius()),
        })
    }

    fn glass(&self, element: ElementId) -> Option<Glass> {
        let part = self.netlist.element(element)?;
        if !matches!(part.kind, ElementKind::LcdDigit) {
            return None;
        }
        Some(self.glass.get(element).copied().unwrap_or_default())
    }

    fn strain(&self, element: ElementId) -> Option<Strain> {
        self.strain.get(element).copied()
    }

    fn pitch(&self, element: ElementId) -> Option<f64> {
        self.pitch.get(element)?.hertz()
    }

    fn latched(&self, element: ElementId) -> Option<bool> {
        let carried = self.run.as_ref()?.carried(element)?;
        Some(carried.latched)
    }

    fn take_serial(&mut self, element: ElementId) -> Vec<u8> {
        self.run
            .as_mut()
            .map(|run| run.take_serial(element))
            .unwrap_or_default()
    }

    fn send_serial(&mut self, element: ElementId, bytes: &[u8]) {
        if let Some(run) = self.run.as_mut() {
            run.send_serial(element, bytes);
        }
    }

    fn set_monitor_baud(&mut self, element: ElementId, baud: u32) {
        if let Some(run) = self.run.as_mut() {
            run.set_monitor_baud(element, baud);
        }
    }

    fn serial_bauds(&self, element: ElementId) -> Option<(u32, Option<u32>)> {
        self.run.as_ref()?.serial_bauds(element)
    }
}
