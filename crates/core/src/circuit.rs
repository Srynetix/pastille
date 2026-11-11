// SPDX-License-Identifier: MIT OR Apache-2.0

//! A circuit as the app holds it: the netlist, plus where things are drawn.
//!
//! Geometry sits *beside* the netlist rather than inside it. The netlist is
//! electrical and knows nothing about coordinates (design doc 7.1.1), but the
//! drawing has to be saved with the circuit and has to survive a reload, so it
//! cannot live in the app either. See `docs/adrs/0006-layout-beside-netlist.md`.
//!
//! This is also the only type the app talks to, so it is the wasm boundary for
//! everything structural: adding an element, removing it, listing what to draw.

use serde::{Deserialize, Serialize};
use slotmap::{Key, KeyData, SecondaryMap, SparseSecondaryMap};
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::{JsError, wasm_bindgen};

use crate::breadboard::{self, Hole};
use crate::engine::{Engine, FailureCode, engine_for};
use crate::netlist::{ElementId, ElementKind, NetId, Netlist, NetlistError, PinRef};
use crate::scheduler::Scheduler;
use crate::value::{self, ValueError};

/// Why an edit was refused.
///
/// A plain error rather than a `JsError`, because `JsError` cannot even be
/// constructed off-wasm — which would make every failure path untestable
/// without a browser. wasm-bindgen converts any `Error` at the boundary, so the
/// `?` in the exported methods is all the plumbing needed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitError {
    UnknownKind(String),
    BadHandle(String),
    /// Wiring a pin to itself. Reported rather than ignored: it means the
    /// interaction that produced it thought two different pins were involved.
    SamePin,
    /// The text in the inspector could not be read as a value.
    Value(ValueError),
    /// This kind has no value to edit — a ground, or an LED, whose colour is
    /// not a number.
    NotEditable(String),
    /// A choice this kind has never heard of (C-46). Distinct from
    /// `NotEditable`: asking a resistor for a colour and asking an LED for
    /// purple are different mistakes, and only one of them is about the set.
    UnknownChoice {
        kind: String,
        choice: String,
    },
    /// A saved circuit in a format this build does not know (C-04).
    UnknownSchema(u32),
    /// A saved circuit whose shape is right and whose contents are not.
    Malformed(String),
    /// A room no circuit is built in (C-74), as it was typed.
    RoomOutOfRange(String),
    /// A value no part can have, or a file holding one, said in a sentence
    /// that says what is accepted (C-96).
    Refused(String),
    Netlist(NetlistError),
}

/// The coldest and warmest rooms a circuit can be put in, in °C.
///
/// The range a part rated for industrial use is specified over. Past it, the
/// numbers every package's figures were measured at stop being the ones that
/// apply, and the answer would be a number with nothing behind it.
pub const ROOM_RANGE: (f64, f64) = (-40.0, 85.0);

impl core::fmt::Display for CircuitError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CircuitError::UnknownKind(kind) => write!(f, "unknown element kind: {kind}"),
            CircuitError::BadHandle(handle) => write!(f, "not an element handle: {handle}"),
            CircuitError::SamePin => write!(f, "a pin cannot be wired to itself"),
            CircuitError::Value(error) => write!(f, "{error}"),
            // A file that will not open is said in a sentence for a person,
            // then, on a line of its own, the detail for whoever asks (C-100):
            // the app shows the first line and keeps the rest for a tooltip.
            CircuitError::UnknownSchema(found) => write!(
                f,
                "This circuit was saved by a newer version of the app. Open it there, or \
                 update this one.\nIt is in format {found}; this version reads format {}.",
                crate::document::SCHEMA
            ),
            CircuitError::Malformed(detail) => write!(
                f,
                "This file is not a circuit saved by this app, or part of it is missing.\n{detail}"
            ),
            CircuitError::NotEditable(kind) => write!(f, "a {kind} has no value to set"),
            CircuitError::RoomOutOfRange(typed) => write!(
                f,
                "{typed} °C is not a room: it has to be between {} °C and {} °C",
                ROOM_RANGE.0, ROOM_RANGE.1
            ),
            CircuitError::UnknownChoice { kind, choice } => {
                write!(f, "a {kind} has no {choice:?} to be set to")
            }
            CircuitError::Refused(sentence) => write!(f, "{sentence}"),
            CircuitError::Netlist(error) => write!(f, "{error}"),
        }
    }
}

impl core::error::Error for CircuitError {}

/// [`crate::check::check`], as the error every setter returns.
fn checked(kind: &ElementKind) -> Result<(), CircuitError> {
    crate::check::check(kind).map_err(CircuitError::Refused)
}

/// Where an element sits in the schematic, and which way it faces.
///
/// Layout, beside the netlist rather than in the app (ADR 0006). An
/// orientation belongs here for the same reason a position does: it is not
/// something the canvas decides, it is something the circuit remembers.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Placement {
    pub x: f64,
    pub y: f64,
    /// Quarter turns clockwise from the way the symbol is drawn, 0 to 3.
    ///
    /// Quarter turns rather than an angle, because a pin two grid cells out
    /// has to land on a grid intersection after turning, and any angle but a
    /// right angle puts it between them — which orthogonal routing (M-17) and
    /// wiring by pointer both depend on.
    pub turns: u8,
}

/// How many quarter turns there are in a full one.
const TURNS: u8 = 4;

mod views;
pub use views::*;

/// A circuit as it was, for undo.
///
/// The whole state rather than a description of what changed. A command stack
/// would be smaller, but every mutation would need an inverse written and kept
/// correct, and the ones that merge nets do not have obvious inverses. A
/// circuit of a few dozen elements clones in microseconds, so the simple thing
/// is also the affordable one — revisit if C-15 ever says otherwise.
#[derive(Clone)]
struct Snapshot {
    netlist: Netlist,
    layout: SecondaryMap<ElementId, Placement>,
    board: SecondaryMap<ElementId, Vec<Option<Hole>>>,
    wires: SparseSecondaryMap<NetId, Vec<PointView>>,
    description: String,
}

/// Whether changes are being grouped into one step of history, and whether
/// that step has been taken yet.
///
/// Taken lazily, because a gesture opens on every pointer down — including the
/// ones that only select something. Snapshotting on the way in put a step
/// identical to the current state on the stack for each of those: undo
/// appeared to do nothing until it had been pressed past them, and a click
/// that changed nothing threw the redo branch away.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Gesture {
    /// No gesture open: every change is a step of its own.
    Closed,
    /// Open, and nothing has changed inside it yet.
    Open,
    /// Open, and the state from before its first change is on the stack.
    Recorded,
}

/// How many steps back the history goes.
///
/// Bounded because an afternoon of editing should not grow without limit. Deep
/// enough that reaching the end is a surprise rather than a routine annoyance.
const HISTORY_LIMIT: usize = 200;

/// The shortest press a finger makes, in seconds: a quick click is some
/// 50 to 100 ms down. A momentary press let go sooner is held to this
/// (C-123), so a tap is a press as a person's would be, long enough for a
/// debouncing capacitor to see, and never a press no step saw.
const SHORTEST_PRESS: f64 = 0.05;

/// When a bouncing push-button's contact flips again after it is pressed or
/// let go, in seconds after (C-109).
///
/// Four flips, so it ends where the finger left it: pressed, it is made, then
/// broken, made, broken and made for good — two more makes than the press, as
/// a counter on it counts. A real button's chatter lasts a few milliseconds,
/// in pulses shorter than one; here on the scheduler's millisecond, which is
/// the finest step a frame is cut into.
const BOUNCES: [f64; 4] = [0.001, 0.002, 0.003, 0.005];

/// A bouncing push-button's contact, due to flip at a moment of the run.
#[derive(Debug, Clone, Copy)]
struct Chatter {
    element: ElementId,
    /// Simulated seconds, on the engine's clock.
    at: f64,
}

/// The circuit being edited.
#[wasm_bindgen]
pub struct Circuit {
    netlist: Netlist,
    layout: SecondaryMap<ElementId, Placement>,
    /// Where someone has moved a part on the breadboard (B-14): each pin's
    /// hole. A part not here is wherever the placer puts it (B-13); the
    /// second geometry ADR 0006 made room for, keyed the same way.
    board: SecondaryMap<ElementId, Vec<Option<Hole>>>,
    /// Bumped by a move on the board, which changes nothing the simulation
    /// or the schematic reads, so the board view knows to look again.
    board_edits: u64,
    /// Where a wire has been bent, by net. Sparse: most wires are never bent,
    /// and an entry only exists once one has been.
    wires: SparseSecondaryMap<NetId, Vec<PointView>>,
    /// What the circuit is, in its author's words: shown on the canvas, saved
    /// with it, and empty for a circuit nobody has described.
    description: String,
    /// Rebuilt whenever the circuit changes. `None` until the first advance, or
    /// when the circuit cannot be solved at all.
    simulation: Option<Scheduler>,
    /// Why the last rebuild or step failed.
    failure: Option<Failure>,
    /// Bumped by every edit, so the simulation knows it is out of date.
    revision: u64,
    /// The revision the current simulation was built from.
    simulated: u64,
    /// States to go back to, oldest first.
    past: Vec<Snapshot>,
    /// States to go forward to, most recently undone last.
    future: Vec<Snapshot>,
    /// Whether a gesture is open, and whether it has recorded its step.
    gesture: Gesture,
    /// Simulated seconds, kept here rather than read off the engine.
    ///
    /// An edit rebuilds the engine, and a rebuilt engine starts its clock at
    /// zero. Time belongs to the circuit being simulated, not to the object
    /// currently doing the simulating — otherwise changing a resistor rewinds
    /// the clock, which is invisible in DC and wrong the moment C-09 lands.
    elapsed: f64,
    /// A bouncing push-button's flips still to come, soonest first (C-109).
    /// Not history: the press is one step of undo, and its chatter is what
    /// the press did.
    chatter: Vec<Chatter>,
    /// Momentary presses let go sooner than a finger can (C-116, C-123): a
    /// PIR module or a push-button, each with when it went down. A tap on a
    /// trackpad is a press and a release in a millisecond, and a click often
    /// lands in the same frame; someone who walks past for an instant has
    /// still walked past, as a button tapped has still been pressed. Held
    /// until it has lasted [`SHORTEST_PRESS`], then let go.
    unseen: Vec<ElementId>,
    /// When each momentary press went down, in run seconds.
    pressed_at: SecondaryMap<ElementId, f64>,
}

impl Default for Circuit {
    fn default() -> Self {
        Circuit::new()
    }
}

mod wasm;

mod edit;
mod exports;
mod read;

mod labels;
use labels::*;

fn encode_net(id: NetId) -> String {
    id.data().as_ffi().to_string()
}

fn encode(id: ElementId) -> String {
    id.data().as_ffi().to_string()
}

fn decode_net(id: &str) -> Result<NetId, CircuitError> {
    id.parse::<u64>()
        .map(|raw| NetId::from(KeyData::from_ffi(raw)))
        .map_err(|_| CircuitError::BadHandle(id.to_string()))
}

fn decode(id: &str) -> Result<ElementId, CircuitError> {
    id.parse::<u64>()
        .map(|raw| ElementId::from(KeyData::from_ffi(raw)))
        .map_err(|_| CircuitError::BadHandle(id.to_string()))
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod series_resistance_tests;

#[cfg(test)]
mod reactive_component_tests;

/// The switch and the push-button, from the app's side of the boundary (C-19).
#[cfg(test)]
mod contact_tests;

/// The instruments, from the app's side of the boundary (C-21).
#[cfg(test)]
mod probe_tests;

/// The live loop over a circuit that has a past (C-48).
///
/// Everything here is driven through `advance`, in frames, because that is how
/// the app drives it — `solver::transient` was already tested from the other
/// side and still answered nothing anybody could see.
#[cfg(test)]
mod live_transient_tests;

/// The scope: what the probes have been reading, over time (C-22).
#[cfg(test)]
mod scope_tests;

/// Choosing between a component's alternatives (C-46).
#[cfg(test)]
mod choice_tests;

#[cfg(test)]
mod uno_tests;

#[cfg(test)]
mod replace_tests;

#[cfg(test)]
mod purchase_tests;

#[cfg(test)]
mod bill_tests;

#[cfg(test)]
mod board_tests;
