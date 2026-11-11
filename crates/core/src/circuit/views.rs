// SPDX-License-Identifier: MIT OR Apache-2.0

//! What the circuit hands across the boundary: one serialisable view per
//! thing the app draws, reads or exports.

use super::*;

/// One element, as the canvas needs it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ElementView {
    /// Opaque handle. A string because a slotmap key is 64 bits and JavaScript
    /// numbers are not.
    pub id: String,
    /// Matches the palette and the symbol table: "DcSource", "Resistor", …
    #[tsify(type = "ComponentKind")]
    pub kind: String,
    pub x: f64,
    pub y: f64,
    /// Quarter turns clockwise from the way the symbol is drawn, 0 to 3.
    pub turns: u8,
    /// What to write beside the symbol — "220 Ω", "9 V".
    pub label: String,
    /// The value as the inspector should offer it for editing, without its
    /// unit. `None` for a component whose value is not a number.
    pub value: Option<String>,
    /// The unit that value is in, for the inspector to show beside the field.
    pub unit: String,
    /// The LED's colour, for whatever draws its light. Absent otherwise.
    pub colour: Option<String>,
    /// A source's internal resistance in ohms, absent for anything that is not
    /// a source (C-17). Zero means ideal.
    pub series_ohms: Option<f64>,
    /// A signal generator's frequency in hertz, absent for anything else
    /// (ADR 0011). The second number it has, beside its amplitude.
    pub hertz: Option<f64>,
    /// What a reactive component starts a run holding — volts for a capacitor,
    /// amps for an inductor (C-18). Absent for everything else.
    pub initial_condition: Option<f64>,
    /// The unit that initial condition is in, for the inspector to show.
    pub initial_unit: String,
    /// Which alternative this element is set to — an LED's colour, a
    /// transistor's polarity. Absent for a component with nothing to choose.
    pub choice: Option<String>,
    /// The real part it is modelled on, absent for an ideal one.
    pub based_on: Option<BasisView>,
    /// The program a microcontroller runs, by name (A-03); absent for one
    /// with none, and for anything that is not a microcontroller.
    pub program: Option<String>,
    /// The alternatives it could be set to instead, empty when there are none.
    ///
    /// Carried per element rather than looked up per kind, so whatever draws a
    /// chooser needs nothing but the element in front of it — and so the list
    /// can only ever come from the core (C-46, ADR 0008).
    pub choices: Vec<String>,
    /// Where a potentiometer's wiper sits, 0 to 1. Absent for anything that
    /// has not got one (C-20).
    pub wiper: Option<f64>,
    /// Whether a contact is made, absent for anything that is not one (C-19).
    ///
    /// The canvas draws a switch shut or open from this, and the same field
    /// covers a push-button: the two differ in what a press means, not in what
    /// there is to know about them.
    pub closed: Option<bool>,
    /// Whether a fuse has blown, absent for anything that is not one (C-120).
    pub blown: Option<bool>,
}

/// A point in world coordinates, crossing the boundary.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct PointView {
    pub x: f64,
    pub y: f64,
}

/// The interior points a wire passes through (ADR 0007).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(transparent)]
pub struct Waypoints(pub Vec<PointView>);

/// Why a circuit could not be solved, for the app to put into words.
///
/// A code and the solver's own sentence. The app says what the code means in
/// language its audience has (ADR 0008 keeps prose out of here); the detail is
/// for whoever is debugging the solver, and is not for the status bar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct Failure {
    pub code: FailureCode,
    /// The solver's own words — "matrix is singular at column 1".
    pub detail: String,
}

/// One pin of one element, named the way the app names them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct PinView {
    pub element_id: String,
    pub pin: String,
}

/// A net, as the canvas needs it: what it joins, not where it runs.
///
/// Where a wire is *drawn* is the app's business, because pin positions come
/// from the symbol and symbols live in the app. The core says what is connected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct NetView {
    pub id: String,
    pub pins: Vec<PinView>,
    /// Points the wire has been bent through, in order (ADR 0007). Empty for a
    /// wire nobody has touched, which routes exactly as it always has.
    pub waypoints: Vec<PointView>,
}

/// Every net.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(transparent)]
pub struct NetViews(pub Vec<NetView>);

/// Something the app noticed and should say: the linter's (B-01), and kept
/// here by name because `SolutionView` carries a list of them.
pub use crate::lint::{Finding, Fix, Severity};

/// What the simulation currently says, for the canvas to draw.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct SolutionView {
    /// Potential of each net, by net handle.
    pub net_voltages: Vec<NetVoltage>,
    /// Current through each element that carries one, by element handle.
    pub element_currents: Vec<ElementCurrent>,
    /// Current along each wire that carries an unambiguous one.
    pub net_currents: Vec<NetCurrent>,
    /// What each pin pushes into the wire it is on, wherever that is known
    /// (C-73). A wire where three pins meet has no one current, but it is drawn
    /// as a tree of runs, and on a tree each run's current is fixed by what
    /// the pins on either side of it put in — which is the app's to work out,
    /// since only it knows the tree.
    pub pin_currents: Vec<PinCurrent>,
    /// How brightly each segment of each seven-segment digit is lit (C-75),
    /// how dark each of an LCD's is (C-76), and how brightly each die of an
    /// RGB LED is lit, red, green and blue (C-84).
    pub digits: Vec<DigitGlow>,
    /// Why the circuit could not be solved (C-25).
    pub failure: Option<Failure>,
    /// Seconds of simulated time.
    pub time: f64,
    /// What the app noticed about this circuit.
    pub findings: Vec<Finding>,
    /// Relays whose coil has pulled their contact over to NO, by element
    /// handle (C-64). Run state rather than an element's, so it is here and
    /// not in the element views: it moves with the solution.
    pub energised: Vec<String>,
    /// How hot each part that warms is running (C-74).
    pub heat: Vec<ElementHeat>,
    /// The note each passive buzzer is sounding, for those that are (C-85).
    pub tones: Vec<ElementTone>,
    /// Where each servo's horn points, in degrees from 0 to 180 (C-90).
    pub angles: Vec<ElementAngle>,
    /// How much each battery has left, and how long that lasts (C-121).
    pub batteries: Vec<ElementCharge>,
    /// Motors turning backwards, by element handle: those with `b` above `a`
    /// (C-91). Every one here is in `energised` too, which says it turns at
    /// all; this says which way.
    pub reversed: Vec<String>,
    /// Each running microcontroller's serial port (A-06): the baud its
    /// monitor is set to, and its program's while the program has begun
    /// `Serial`.
    pub serial: Vec<ElementSerial>,
}

/// A microcontroller's serial port, as its monitor sees it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ElementSerial {
    pub element_id: String,
    pub monitor_baud: u32,
    /// None until the program begins `Serial`, and after it ends it.
    pub program_baud: Option<u32>,
}

/// A battery's charge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ElementCharge {
    pub element_id: String,
    /// How full it is, 0 to 1.
    pub left: f64,
    /// Hours left at the current it is giving now; none while it gives none.
    pub hours: Option<f64>,
}

/// A servo's horn.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ElementAngle {
    pub element_id: String,
    pub degrees: f64,
}

/// A passive buzzer's note.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ElementTone {
    pub element_id: String,
    /// What it is being fed, in hertz, read from the run.
    pub hertz: f64,
}

/// A part's temperature, what is putting it there, and what it is rated for.
///
/// The limits beside the reading, because they are this part's: a 7805 and
/// an LM1117 are both regulators and do not survive the same temperature,
/// and the kind's rating is only its default part's.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ElementHeat {
    pub element_id: String,
    /// Watts it is turning into heat.
    pub watts: f64,
    /// How hot it is, in °C.
    pub celsius: f64,
    /// Where it would settle if the power stayed as it is, in °C.
    pub settles_at: f64,
    /// Most power it is rated to dissipate, when it has a figure for one.
    pub rated_watts: Option<f64>,
    /// Hottest it may run, in °C.
    pub max_celsius: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct NetVoltage {
    pub net_id: String,
    pub volts: f64,
}

/// Current through a two-terminal element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ElementCurrent {
    pub element_id: String,
    /// Amps flowing *into* the element's first pin and out of its second.
    ///
    /// One convention for every element, so a sign means the same thing
    /// everywhere: positive current enters the `+` of a battery, the `a` of a
    /// resistor, the anode of an LED.
    pub amps: f64,
    /// 0 to 1 for an LED, absent for everything else.
    pub brightness: Option<f64>,
}

/// Current along a wire, and which way it goes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct NetCurrent {
    pub net_id: String,
    /// Magnitude, in amps. Always positive; the direction is `from` to `to`.
    pub amps: f64,
    /// The pin the current leaves to flow along this wire.
    pub from: PinView,
    /// The pin it arrives at.
    ///
    /// Both ends rather than one, because a wire is not always a straight run
    /// between two pins: a meter tapped onto it adds an end that carries
    /// nothing, and the app has to know which stretch of the drawing the
    /// current actually crosses (C-21).
    pub to: PinView,
}

/// A seven-segment digit's segments, lit or not — or an RGB LED's dice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct DigitGlow {
    pub element_id: String,
    /// 0 to 1 for each segment, in pin order: a to g, then the point. On an
    /// LED digit, the brightness an LED would have at the same current,
    /// because each segment is one; on an LCD, how dark the glass has turned
    /// (C-76); on an RGB LED, its red, green and blue dice's (C-84).
    pub segments: Vec<f64>,
}

/// What one pin pushes into the wire it is on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct PinCurrent {
    pub net_id: String,
    pub pin: PinView,
    /// Amps out of the element and into the wire at this pin; negative when
    /// the wire is feeding the element instead.
    ///
    /// Read pin by pin for a part with more than two — a transistor, an
    /// op-amp, a relay, a potentiometer, from its own model (C-101). Missing
    /// still for a chip's, which the app recovers when it is the only one
    /// missing on a wire, since what enters a wire leaves it.
    pub amps: f64,
}

/// The real part a component is modelled on, and its datasheet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct BasisView {
    /// "1N4148 (Vishay)", "NE555".
    pub part: String,
    /// Where its figures come from, when there is a sheet worth linking.
    pub datasheet: Option<String>,
    /// Whether the model is only close to it rather than held to its sheet.
    pub approximate: bool,
}

/// What buying a part would mean (B-09, design doc §6.2): the value it can be
/// bought in nearest the one it has, and what the circuit does on that value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct Purchase {
    pub element_id: String,
    /// The value to buy, as a field takes it: "220".
    pub value: String,
    /// The same, as it is said: "220 Ω".
    pub label: String,
    /// The series it is from: "E12".
    pub series: String,
    /// How far it is from the value the part has, as a percentage of that
    /// value: positive when the part bought is larger.
    pub error_percent: f64,
    /// The current through it on the bought value, for a resistor.
    pub amps: Option<f64>,
    /// What the linter says on the bought value and did not on the exact
    /// one: the rounding raising a warning of its own.
    pub findings: Vec<Finding>,
}

/// One line of a bill of materials (B-11): parts of one kind, value and part
/// number, how many, and what to know about them.
///
/// The kind rather than its name: what a part is called is the app's (ADR
/// 0008), and the app writes the bill out.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct BillLine {
    #[tsify(type = "ComponentKind")]
    pub kind: String,
    /// What to buy it as: the value it can be bought in, as it is said.
    pub value: String,
    /// The part number, when it is one in particular.
    pub part: Option<String>,
    pub datasheet: Option<String>,
    pub quantity: u32,
    /// Handles of the parts on the canvas this line is.
    pub elements: Vec<String>,
    /// What the linter says about them, and what rounding their value did.
    pub notes: Vec<String>,
}

/// A bill of materials, one line per part to buy, in the order the parts were
/// first placed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(transparent)]
pub struct Bill(pub Vec<BillLine>);

/// The breadboard, as the view draws it (B-14): the board, what is on it,
/// and where it builds something other than the schematic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct BoardView {
    /// "half" or "full".
    pub size: String,
    pub rows: u16,
    pub holes: Vec<BoardHole>,
    pub parts: Vec<BoardPart>,
    pub jumpers: Vec<BoardJumper>,
    pub problems: Vec<BoardProblem>,
    /// How to build it, in order (B-15).
    pub steps: Vec<BoardStep>,
}

/// One step of building it: what to do, where, and which parts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct BoardStep {
    pub text: String,
    /// Where the holes it is about are, in pitches: a rail's hole is not
    /// named apart from the rest of its rail, so it is pointed at instead.
    pub holes: Vec<BoardSpot>,
    pub elements: Vec<String>,
}

/// A place on the board, in pitches.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Tsify)]
pub struct BoardSpot {
    pub x: f64,
    pub y: f64,
}

/// A hole, where it is in 0.1″ pitches, and which strip it is part of.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct BoardHole {
    pub name: String,
    pub x: f64,
    pub y: f64,
    /// The same for every hole one clip joins: "12u", "12l", "top+".
    pub strip: String,
}

/// A part on the board, and where each of its legs is.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct BoardPart {
    pub element_id: String,
    #[tsify(type = "ComponentKind")]
    pub kind: String,
    pub label: String,
    /// A chip's package pin count, for one that straddles the channel.
    pub dip: Option<u8>,
    /// Whether someone has put it where it is, rather than the placer.
    pub moved: bool,
    /// Whether it can be moved: a battery's leads stay on the rails.
    pub movable: bool,
    pub legs: Vec<BoardLeg>,
}

/// One leg of a part, in its hole.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct BoardLeg {
    pub pin: String,
    pub hole: String,
    pub x: f64,
    pub y: f64,
    /// Its number on the package, for a chip's.
    pub number: Option<u8>,
}

/// A jumper wire between two holes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct BoardJumper {
    pub from: BoardHole,
    pub to: BoardHole,
}

/// Something the board builds that the schematic does not, and the holes it
/// is in.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct BoardProblem {
    pub message: String,
    pub holes: Vec<String>,
}

/// The circuit as a KiCad netlist (B-16), or the kinds of part that stop it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct KicadView {
    /// The file, when every part can be written.
    pub netlist: Option<String>,
    /// The same circuit as a schematic KiCad's schematic editor opens (B-21),
    /// when every part can be written.
    pub schematic: Option<String>,
    /// The kinds of part with no KiCad mapping yet, when some cannot: for the
    /// app to name as its palette does.
    #[tsify(type = "ComponentKind[]")]
    pub unmapped: Vec<String>,
}

/// A purchase, or nothing for a part already at a value it can be bought in,
/// or one with no value to buy.
///
/// A named wrapper because `Ts` crosses one declared type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(transparent)]
pub struct PurchaseView(pub Option<Purchase>);

/// What became of a part's wires when it was replaced by another kind
/// (C-137): each pin that kept its wires and the pin it is now, and each
/// that had wires and lost them. For the app to say.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct Replacement {
    /// `(was, now)` pin names, for every wired pin that kept its wires.
    pub kept: Vec<(String, String)>,
    /// The old part's wired pins that had nowhere to go.
    pub dropped: Vec<String>,
}

/// One value a field's − or + can step to (C-136).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct Step {
    /// As a field takes it: "4.7k".
    pub value: String,
    /// As it is said: "4.7 kΩ".
    pub label: String,
}

/// The values either side of a resistor's or a capacitor's own (C-136), in
/// the series it is bought in:
/// what its − and + step to, and the chips under its field. Nearest first on
/// each side; both empty for a part with no such value.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct Steps {
    pub below: Vec<Step>,
    pub above: Vec<Step>,
}

/// Everything to draw.
///
/// A named wrapper because `Ts` crosses one declared type, not a bare `Vec`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(transparent)]
pub struct ElementViews(pub Vec<ElementView>);
