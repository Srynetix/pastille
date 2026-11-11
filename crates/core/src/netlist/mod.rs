// SPDX-License-Identifier: MIT OR Apache-2.0

//! The netlist: the single source of truth every other subsystem reads
//! (design doc 7.1.1).
//!
//! It knows nothing about canvases, coordinates or rendering. A circuit is
//! elements, the pins they expose, and the nets those pins sit on.
//!
//! Ids are slotmap keys rather than plain indices, because removal arrives in
//! M-04 and undo/redo in C-23: both hand out ids that outlive the thing they
//! point at, and a stale generational key is a clean `None` rather than a
//! silent hit on whatever took the slot.

mod compile;
mod connectivity;
mod pin;

pub use compile::{CompileError, GroundOrigin, NodeMap, REFERENCE_NODE, Reference};
pub use connectivity::Connectivity;
pub use pin::{ElectricalModel, InputThresholds, LogicDrive, Pin, PinDiscipline, PinRef};

use slotmap::{SlotMap, new_key_type};

new_key_type! {
    /// Handle to an element. Stays valid until that element is removed.
    pub struct ElementId;

    /// Handle to a net.
    pub struct NetId;
}

/// Colour of an LED. Forward voltage per colour arrives with the model in M-11.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LedColour {
    Red,
    Green,
    Blue,
    White,
}

impl LedColour {
    /// Every colour the core models, in the order a chooser should offer them.
    ///
    /// The one place the four are written. A copy of this list in the app would
    /// drift the first time a fifth arrived, which is what ADR 0008 is about —
    /// so the app is given it rather than keeping one, and a test holds the
    /// spelling of each to what `name` says (C-46).
    pub const ALL: &'static [LedColour] = &[
        LedColour::Red,
        LedColour::Green,
        LedColour::Blue,
        LedColour::White,
    ];

    /// The name this colour crosses the boundary under.
    pub fn name(&self) -> &'static str {
        match self {
            LedColour::Red => "Red",
            LedColour::Green => "Green",
            LedColour::Blue => "Blue",
            LedColour::White => "White",
        }
    }

    /// The colour of that name, or nothing if it is not one.
    pub fn named(name: &str) -> Option<LedColour> {
        LedColour::ALL
            .iter()
            .copied()
            .find(|colour| colour.name() == name)
    }
}

/// A digit's eight segments, each the red LED (C-75).
const SEGMENT_COLOURS: [LedColour; 8] = [LedColour::Red; 8];

/// What an element *is*, and the parameters that belong to it.
///
/// Values are SI throughout the core — volts, ohms, amps, seconds — and are
/// converted for display at the edge, never in here.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ElementKind {
    /// A DC source, ideal unless given an internal resistance.
    DcSource {
        volts: f64,
        /// Resistance in series with it, inside the element, in ohms.
        ///
        /// Zero is an ideal source, which is the default and what every
        /// textbook circuit assumes. A real cell has milliohms, and it is what
        /// stops a short circuit being an unbounded current — the difference
        /// between a battery that gets hot and a solver that gives up (C-17).
        series_ohms: f64,
    },
    Resistor {
        ohms: f64,
    },
    /// A source whose voltage changes with time: a square, a sine or a
    /// triangle, swinging between −volts and +volts (ADR 0011).
    ///
    /// Bench equipment rather than a part, like the meters. Its size is its
    /// value, as a battery's is; its shape is a choice; its frequency is the
    /// second number, edited the way a battery's internal resistance is.
    SignalGenerator {
        waveform: crate::solver::device::Waveform,
        /// Peak volts either side of zero.
        volts: f64,
        hertz: f64,
    },
    /// Stores charge, and so has a past. Open to DC; a companion model over a
    /// timestep (C-06).
    Capacitor {
        farads: f64,
        /// Volts across it when a run starts, before anything has charged it.
        ///
        /// Zero for a component straight out of the palette. It is the
        /// quantity a capacitor carries between steps, so it is the quantity
        /// it can be given a head start in (C-18).
        initial_volts: f64,
    },
    /// An aluminium electrolytic: the capacitor, polarised, with the leak its
    /// sheet allows and the voltage it is rated for (C-86). Wired backwards or
    /// past its rating it says so and goes on working.
    Electrolytic {
        farads: f64,
        /// Volts across it, + over −, when a run starts, as a capacitor's.
        initial_volts: f64,
        rated: crate::solver::device::electrolytic::Rated,
    },
    /// Stores current, and so has a past. A plain wire to DC; a companion model
    /// over a timestep (C-07).
    Inductor {
        henries: f64,
        /// Amps through it when a run starts. The inductor's equivalent of a
        /// capacitor's initial voltage, and zero for the same reason.
        initial_amps: f64,
    },
    /// Two windings on one core, nothing conducting between them (C-118):
    /// `ratio` turns on the primary for one on the secondary.
    Transformer {
        ratio: f64,
    },
    /// A battery that runs down (C-121): a real one of its `part`, with
    /// `charge` left on its dial, 0 to 1. What it has given since is the run's.
    Cell {
        part: crate::solver::device::cell::CellPart,
        charge: f64,
    },
    /// A fuse rated `amps` (C-120): a fast-blow one that melts and stays
    /// `blown` until it is replaced, or a polyfuse that trips hot and resets
    /// once cool. How hot it is, is the run's; blown is the part's, as a
    /// switch's contact is, so it is drawn and saved blown.
    Fuse {
        amps: f64,
        part: crate::solver::device::fuse::FusePart,
        blown: bool,
    },
    /// An LM3914: a voltage shown on ten LEDs, as a bar or a dot (C-119).
    /// One part; which LEDs are lit is the run's.
    Lm3914,
    /// Ten red LEDs side by side in one package, each with its own anode and
    /// cathode (C-119). One part, so no fields.
    Bargraph,
    /// A DB107 bridge rectifier: four diodes in one package, two AC pins in
    /// and + and − out (C-118). One part, so no fields.
    Bridge,
    /// A pn junction, and nothing else: one way round it conducts, the other
    /// way it does not.
    ///
    /// The model M-10 built has been in use since, but only inside an LED. A
    /// plain one has no property to set — no colour, no value — because what
    /// makes a diode a diode is the direction, and the direction is which way
    /// it is wired (C-50).
    Diode,
    /// A diode that also conducts backwards, at a voltage it is made for
    /// (C-62). Which voltage is which part, so it is a choice, as a
    /// regulator's is.
    Zener {
        part: crate::solver::device::ZenerPart,
    },
    Led {
        colour: LedColour,
    },
    /// A contact that latches: flipped, it stays where it was put.
    ///
    /// Ideal, as §5 says — shut it is a piece of wire, open it is a gap. There
    /// is no on-resistance to pick and no leakage to explain, which is the
    /// right amount of physics for the component whose whole job is to be
    /// obvious.
    Switch {
        /// Whether the contact is made. Normally open, so a fresh one is not.
        closed: bool,
    },
    /// A contact that springs back: shut while it is held, open the moment it
    /// is let go.
    ///
    /// Electrically the same ideal contact as [`ElementKind::Switch`] — the
    /// difference is entirely in what the user's finger does, which is why the
    /// two share a field name and every stamp that reads it.
    PushButton {
        closed: bool,
        /// Whether it bounces as a real one does: a few more makes and breaks
        /// in the first milliseconds of every press and release (C-109). Off
        /// by default, and left out of a saved file while it is, so every
        /// circuit saved before it existed reads the same.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        bounces: bool,
    },
    /// A ball tilt switch, an SW-520D: shut while it stands upright, the ball
    /// resting across its two legs, and open once it is tipped and the ball
    /// rolls away (C-88). The same ideal contact again; `closed` is upright.
    /// Its ball's bounce is not modelled.
    TiltSwitch {
        closed: bool,
    },
    /// A light-dependent resistor: less resistance the more light falls on
    /// it (C-65). `light` is a dial from 0, a dark room, to 1, daylight.
    Ldr {
        light: f64,
    },
    /// An NPN phototransistor, collector and emitter, its base fed by the
    /// light (C-89). `light` is the LDR's dial, the same light on the same
    /// scale, so the two can be set side by side.
    Phototransistor {
        light: f64,
    },
    /// A small solar panel, + and − (C-117). `sun` is its dial: the share
    /// of the full sun its sheet was measured in, from 0, night, to 1.
    SolarCell {
        sun: f64,
    },
    /// An NTC thermistor: less resistance the warmer it is (C-65).
    Thermistor {
        celsius: f64,
    },
    /// A resistive track with a wiper that taps it somewhere along its length.
    ///
    /// Two resistors that always add up to the same total, which is what a real
    /// one is: the track is `ohms` end to end, and the wiper splits it. Nothing
    /// about it is nonlinear and nothing about it is coupled — it is stamped as
    /// the two conductances it is (C-20).
    Potentiometer {
        /// End to end, in ohms. What the part is sold as.
        ohms: f64,
        /// Where the wiper sits, 0 at `a` and 1 at `b`.
        ///
        /// A fraction rather than a resistance, because that is what turning a
        /// knob changes and what stays meaningful when the track's value is
        /// edited underneath it.
        wiper: f64,
    },
    /// Reads the potential difference between its two pins.
    ///
    /// Ideal, which for a voltmeter means infinite resistance: it is not
    /// stamped at all, so no current passes through it and putting one across
    /// a component cannot change what that component is doing. That is the
    /// whole property an instrument needs — a meter that loads the circuit is
    /// measuring a circuit that only exists while it is being measured.
    ///
    /// Two pins rather than one, because a voltage is always *between* two
    /// points. ADR 0003 hid the reference so a beginner never has to place a
    /// ground, and a one-pin probe would smuggle it straight back in by
    /// answering "volts against what?" with silence.
    Voltmeter,
    /// Reads the current flowing through it.
    ///
    /// Ideal, which for an ammeter means no resistance: the same piece of wire
    /// a shut contact is, and stamped the same way. It goes *in* the branch it
    /// measures, where a voltmeter goes *across* it — the difference a beginner
    /// meets by wiring one the wrong way round and blowing a fuse.
    Ammeter,
    /// Bipolar transistor. Three terminals, and the first element whose
    /// terminals are coupled (C-12). Nothing in v1 places one — C-28 is the
    /// symbol and C-16's registry is what would describe it.
    Bjt {
        polarity: crate::solver::device::Polarity,
    },
    /// Explicit reference node. One pin, zero volts, no current of its own.
    /// A MOSFET: steered by the voltage on its gate, which takes no current
    /// (C-63). N- or P-channel, as a transistor is NPN or PNP.
    Mosfet {
        channel: crate::solver::device::Channel,
    },
    /// A linear regulator: a fixed voltage out whatever it is fed, as long as
    /// it is fed enough (C-54).
    ///
    /// Modelled from its datasheet rather than its die (ADR 0010), and which
    /// datasheet is the part: the output voltage is not a number to type but a
    /// thing printed on the case.
    Regulator {
        part: crate::solver::device::RegulatorPart,
    },
    /// An operational amplifier: its output is the difference between its
    /// two inputs, times a hundred thousand, as far as its supply allows
    /// (C-55).
    ///
    /// No fields. One part is offered, an LM358, and there is nothing about
    /// it to set: what it does is decided entirely by what it is wired to.
    OpAmp,
    /// A comparator: its output is pulled down when its inverting input is
    /// the higher of the two, and let go otherwise (C-56).
    ///
    /// No fields, like the op-amp: one part, an LM393, and what it does is
    /// decided by what it is wired to — including the pull-up its output
    /// needs, because it cannot drive high on its own.
    Comparator,
    /// A TMP36: 0.5 V + 10 mV per degree out, at the temperature its dial
    /// reads (C-87). Like the thermistor, the temperature is the part's own,
    /// for a beginner to turn.
    Tmp36 {
        celsius: f64,
    },
    /// A 555 timer (ADR 0012): two thresholds and a latch, the part a
    /// hobbyist makes a blinking LED or a timed light from.
    ///
    /// No fields — one part, an NE555 — and the one element whose state is a
    /// bit rather than a charge. The bit is carried by the stepper, not here:
    /// what a part is set to is the netlist's, what it is doing is the run's.
    Timer,
    /// A relay: a coil that throws a contact from NC to NO once it carries
    /// enough current, and lets it go again only when it carries far less
    /// (C-64). Which coil is the part; whether it is energised is the run's,
    /// carried as a 555's latch is.
    Relay {
        part: crate::solver::device::RelayPart,
    },
    /// An active buzzer: sounds with enough voltage across it, the right way
    /// round (C-66).
    Buzzer,
    /// A small DC motor: turns with enough voltage across it, either way
    /// (C-66).
    Motor,
    /// A passive buzzer: a small speaker with no oscillator of its own, which
    /// sounds at the frequency the circuit feeds it (C-85). One part, so no
    /// fields; the note it is hearing is the run's.
    PassiveBuzzer,
    /// A seven-segment LED digit: eight LEDs sharing one pin (C-75). Which end
    /// they share is the choice; the segments are the red LED.
    SevenSegment {
        common: crate::solver::device::digit::Common,
    },
    /// An RGB LED: a red, a green and a blue LED sharing one pin (C-84). Which
    /// end they share is the choice, as a digit's is; the dice are the LEDs of
    /// their colours.
    RgbLed {
        common: crate::solver::device::digit::Common,
    },
    /// A seven-segment liquid-crystal digit: eight segments of glass sharing
    /// a backplane, dark when the RMS voltage across one is high enough
    /// (C-76). One part, so no fields.
    LcdDigit,
    /// A CD4511: four bits to seven segments, latched (C-77). No fields — one
    /// part — and, like the 555, what it is holding is the run's.
    Decoder,
    /// A hobby servo, an SG90: a horn turned to the angle its pulses ask for
    /// (C-90). One part, so no fields; where its horn is is the run's.
    Servo,
    /// A CD4026: a decade counter that drives a digit (C-78). One part; what
    /// it has counted is the run's.
    Counter,
    /// An L293D: four half-H drivers, two motors either way (C-91). One part;
    /// what its inputs read at the last step is the run's.
    MotorDriver,
    /// A 4N35: an LED lighting a phototransistor, and no path between them
    /// (C-92). One part, so no fields.
    Optocoupler,
    /// A 74HC595: eight bits shifted in, then latched onto eight outputs
    /// (C-93). One part; both its registers are the run's.
    ShiftRegister,
    /// Four two-input gates in one package: a 74HC00, 08, 32 or 86 (C-108).
    /// Which is the choice; what each output was last set to is the run's.
    Gate {
        logic: crate::solver::device::gate::Logic,
    },
    /// A 74HC04: six inverters (C-108). One part, so no fields.
    Inverter,
    /// A 74HC14: six inverters whose inputs switch up and down at different
    /// voltages (C-109). Which way each last went is the run's.
    Schmitt,
    /// A CD4017: a decade counter, one of ten outputs high at a time
    /// (C-110). One part; what it has counted is the run's.
    Decade,
    /// A CD4013: two D flip-flops (C-111). One part; what each holds is the
    /// run's.
    FlipFlop,
    /// An LM317: an adjustable regulator, 1.25 V held between its output and
    /// its ADJ pin (C-113). One part, so no fields; the regulator's model.
    Adjustable,
    /// A 1N5819 Schottky diode: a lower drop than the 1N4148's, and more
    /// leak (C-114). One part, so no fields.
    Schottky,
    /// A reed switch: shut while a magnet is near (C-115). `magnet` is its
    /// dial, 0 as far away as it goes and 1 touching; the contact follows it.
    Reed {
        magnet: f64,
    },
    /// An A3144 Hall sensor: pulls its output low while a magnet is near,
    /// with hysteresis (C-115). Its dial is the reed's; whether it is on is
    /// the run's.
    Hall {
        magnet: f64,
    },
    /// An HC-SR501 PIR module (C-116). `moving` is someone in front of it —
    /// a press on the part, as a push-button's is — `hold` its delay in
    /// seconds, `retriggers` its jumper.
    Pir {
        moving: bool,
        hold: f64,
        retriggers: bool,
    },
    /// An Arduino Uno (A-01, ADR 0017): an ATmega328P running the program
    /// the netlist holds for it, its pins the circuit's. What the program is
    /// doing is the run's, as a 555's latch is.
    Uno,
    Ground,
}

/// What a push-button can be: one that never bounces, and one that does as a
/// real one does (C-109). Ideal first, which is what every button was before.
pub const BUTTON_CHOICES: [&str; 2] = ["Ideal", "Bouncy"];

/// Where a PIR module's jumper can be: H, held movement keeps restarting its
/// delay; L, it triggers once and must rest (C-116).
pub const PIR_CHOICES: [&str; 2] = ["Retriggers", "Once"];

fn button_choice(bounces: bool) -> &'static str {
    BUTTON_CHOICES[usize::from(bounces)]
}

impl ElementKind {
    /// The name this kind crosses the boundary under, and is filed under in
    /// the registry.
    pub fn name(&self) -> &'static str {
        match self {
            ElementKind::DcSource { .. } => "DcSource",
            ElementKind::Resistor { .. } => "Resistor",
            ElementKind::SignalGenerator { .. } => "SignalGenerator",
            ElementKind::Capacitor { .. } => "Capacitor",
            ElementKind::Electrolytic { .. } => "Electrolytic",
            ElementKind::Inductor { .. } => "Inductor",
            ElementKind::Transformer { .. } => "Transformer",
            ElementKind::Bridge => "Bridge",
            ElementKind::Lm3914 => "Lm3914",
            ElementKind::Fuse { .. } => "Fuse",
            ElementKind::Cell { .. } => "Cell",
            ElementKind::Bargraph => "Bargraph",
            ElementKind::Diode => "Diode",
            ElementKind::Zener { .. } => "Zener",
            ElementKind::Led { .. } => "Led",
            ElementKind::Switch { .. } => "Switch",
            ElementKind::PushButton { .. } => "PushButton",
            ElementKind::TiltSwitch { .. } => "TiltSwitch",
            ElementKind::Potentiometer { .. } => "Potentiometer",
            ElementKind::Ldr { .. } => "Ldr",
            ElementKind::Phototransistor { .. } => "Phototransistor",
            ElementKind::SolarCell { .. } => "SolarCell",
            ElementKind::Thermistor { .. } => "Thermistor",
            ElementKind::Voltmeter => "Voltmeter",
            ElementKind::Ammeter => "Ammeter",
            ElementKind::Bjt { .. } => "Bjt",
            ElementKind::Mosfet { .. } => "Mosfet",
            ElementKind::Regulator { .. } => "Regulator",
            ElementKind::OpAmp => "OpAmp",
            ElementKind::Comparator => "Comparator",
            ElementKind::Tmp36 { .. } => "Tmp36",
            ElementKind::Timer => "Timer",
            ElementKind::Relay { .. } => "Relay",
            ElementKind::Buzzer => "Buzzer",
            ElementKind::Motor => "Motor",
            ElementKind::PassiveBuzzer => "PassiveBuzzer",
            ElementKind::SevenSegment { .. } => "SevenSegment",
            ElementKind::RgbLed { .. } => "RgbLed",
            ElementKind::LcdDigit => "LcdDigit",
            ElementKind::Decoder => "Decoder",
            ElementKind::Servo => "Servo",
            ElementKind::Counter => "Counter",
            ElementKind::MotorDriver => "MotorDriver",
            ElementKind::Optocoupler => "Optocoupler",
            ElementKind::ShiftRegister => "ShiftRegister",
            ElementKind::Gate { .. } => "Gate",
            ElementKind::Inverter => "Inverter",
            ElementKind::Schmitt => "Schmitt",
            ElementKind::Decade => "Decade",
            ElementKind::FlipFlop => "FlipFlop",
            ElementKind::Adjustable => "Adjustable",
            ElementKind::Schottky => "Schottky",
            ElementKind::Reed { .. } => "Reed",
            ElementKind::Hall { .. } => "Hall",
            ElementKind::Pir { .. } => "Pir",
            ElementKind::Uno => "Uno",
            ElementKind::Ground => "Ground",
        }
    }

    /// Whether this is a contact, and whether it is currently made.
    ///
    /// A switch and a push-button are one electrical fact under two names
    /// (C-19), and this is that fact. Everything that stamps, draws or reports
    /// a contact asks here rather than listing both kinds, so a third kind of
    /// contact — a relay, a reed — is one arm rather than a sweep.
    pub fn contact(&self) -> Option<bool> {
        match *self {
            ElementKind::Switch { closed }
            | ElementKind::PushButton { closed, .. }
            | ElementKind::TiltSwitch { closed } => Some(closed),
            // Its contact is the magnet's to make, not a finger's: a contact
            // to everything that stamps or draws one, and none to a click.
            ElementKind::Reed { magnet } => Some(crate::solver::device::magnet::reed_shut(magnet)),
            _ => None,
        }
    }

    /// Whether someone is moving in front of it, for a PIR module (C-116):
    /// what a press means on it, as `contact` is what one means on a button.
    pub fn moving(&self) -> Option<bool> {
        match *self {
            ElementKind::Pir { moving, .. } => Some(moving),
            _ => None,
        }
    }

    /// The same kind with its contact made or broken, for a caller that has
    /// one and does not want to know which of the two it is.
    ///
    /// `None` for anything that is not a contact, which is how a caller finds
    /// out it was asking the wrong element.
    pub fn with_contact(&self, closed: bool) -> Option<ElementKind> {
        match *self {
            ElementKind::Switch { .. } => Some(ElementKind::Switch { closed }),
            ElementKind::PushButton { bounces, .. } => {
                Some(ElementKind::PushButton { closed, bounces })
            }
            ElementKind::TiltSwitch { .. } => Some(ElementKind::TiltSwitch { closed }),
            // Not a contact — nothing in it is shut — but pressed and let go
            // as one is: held is someone moving in front of it (C-116).
            ElementKind::Pir {
                hold, retriggers, ..
            } => Some(ElementKind::Pir {
                moving: closed,
                hold,
                retriggers,
            }),
            _ => None,
        }
    }

    /// Which of its alternatives this element is currently set to.
    ///
    /// The second shape of property a component can have (C-46). Everything
    /// editable until now was a number parsed from text; this is a choice from
    /// a closed set, and the two need different controls, different calls and
    /// different errors. `None` for a component with nothing to choose.
    pub fn choice(&self) -> Option<&'static str> {
        match *self {
            ElementKind::Led { colour } => Some(colour.name()),
            ElementKind::Bjt { polarity } => Some(polarity.name()),
            ElementKind::Mosfet { channel } => Some(channel.name()),
            ElementKind::Gate { logic } => Some(logic.name()),
            ElementKind::Pir { retriggers, .. } => Some(PIR_CHOICES[usize::from(!retriggers)]),
            ElementKind::Fuse { part, .. } => Some(part.name()),
            ElementKind::Cell { part, .. } => Some(part.name()),
            ElementKind::PushButton { bounces, .. } => Some(button_choice(bounces)),
            ElementKind::Regulator { part } => Some(part.name()),
            ElementKind::Zener { part } => Some(part.name()),
            ElementKind::Relay { part } => Some(part.name()),
            ElementKind::Electrolytic { rated, .. } => Some(rated.name()),
            ElementKind::SevenSegment { common } | ElementKind::RgbLed { common } => {
                Some(common.name())
            }
            ElementKind::SignalGenerator { waveform, .. } => Some(waveform.name()),
            _ => None,
        }
    }

    /// The same kind set to one of its alternatives, by name.
    ///
    /// `None` when this kind has nothing to choose or has never heard of that
    /// one, which the caller has to tell apart — hence [`ElementKind::choice`]
    /// to ask first.
    pub fn with_choice(&self, choice: &str) -> Option<ElementKind> {
        match *self {
            ElementKind::Led { .. } => {
                LedColour::named(choice).map(|colour| ElementKind::Led { colour })
            }
            ElementKind::Bjt { .. } => crate::solver::device::Polarity::named(choice)
                .map(|polarity| ElementKind::Bjt { polarity }),
            ElementKind::Mosfet { .. } => crate::solver::device::Channel::named(choice)
                .map(|channel| ElementKind::Mosfet { channel }),
            ElementKind::Gate { .. } => crate::solver::device::gate::Logic::named(choice)
                .map(|logic| ElementKind::Gate { logic }),
            ElementKind::Pir { moving, hold, .. } => PIR_CHOICES
                .iter()
                .position(|&name| name == choice)
                .map(|index| ElementKind::Pir {
                    moving,
                    hold,
                    retriggers: index == 0,
                }),
            ElementKind::PushButton { closed, .. } => BUTTON_CHOICES
                .iter()
                .position(|&name| name == choice)
                .map(|index| ElementKind::PushButton {
                    closed,
                    bounces: index == 1,
                }),
            // Another battery, as full as this one was (C-121).
            ElementKind::Cell { charge, .. } => {
                crate::solver::device::cell::CellPart::from_name(choice)
                    .map(|part| ElementKind::Cell { part, charge })
            }
            // A fuse of the other kind starts whole (C-120).
            ElementKind::Fuse { amps, .. } => {
                crate::solver::device::fuse::FusePart::from_name(choice).map(|part| {
                    ElementKind::Fuse {
                        amps,
                        part,
                        blown: false,
                    }
                })
            }
            ElementKind::Regulator { .. } => crate::solver::device::RegulatorPart::named(choice)
                .map(|part| ElementKind::Regulator { part }),
            ElementKind::Zener { .. } => crate::solver::device::ZenerPart::named(choice)
                .map(|part| ElementKind::Zener { part }),
            ElementKind::Relay { .. } => crate::solver::device::RelayPart::named(choice)
                .map(|part| ElementKind::Relay { part }),
            ElementKind::Electrolytic {
                farads,
                initial_volts,
                ..
            } => crate::solver::device::electrolytic::Rated::named(choice).map(|rated| {
                ElementKind::Electrolytic {
                    farads,
                    initial_volts,
                    rated,
                }
            }),
            ElementKind::SevenSegment { .. } => crate::solver::device::digit::Common::named(choice)
                .map(|common| ElementKind::SevenSegment { common }),
            ElementKind::RgbLed { .. } => crate::solver::device::digit::Common::named(choice)
                .map(|common| ElementKind::RgbLed { common }),
            ElementKind::SignalGenerator { volts, hertz, .. } => {
                crate::solver::device::Waveform::named(choice).map(|waveform| {
                    ElementKind::SignalGenerator {
                        waveform,
                        volts,
                        hertz,
                    }
                })
            }
            _ => None,
        }
    }

    /// Where this element's wiper sits, for the one kind that has one.
    ///
    /// `None` for everything else, which is how a caller finds out it was
    /// asking the wrong element (C-20).
    ///
    /// Generalised to the one dial a part has (C-65): a potentiometer's
    /// wiper, the light on an LDR, a thermistor's temperature — each from 0
    /// to 1, and what it means is the part's.
    pub fn wiper(&self) -> Option<f64> {
        match *self {
            ElementKind::Potentiometer { wiper, .. } => Some(wiper),
            ElementKind::Ldr { light } | ElementKind::Phototransistor { light } => Some(light),
            ElementKind::SolarCell { sun } => Some(sun),
            ElementKind::Cell { charge, .. } => Some(charge),
            ElementKind::Thermistor { celsius } => {
                Some(crate::solver::device::sensor::dial_for(celsius))
            }
            ElementKind::Tmp36 { celsius } => Some(crate::solver::device::tmp36::dial_for(celsius)),
            ElementKind::Reed { magnet } | ElementKind::Hall { magnet } => Some(magnet),
            _ => None,
        }
    }

    /// The same kind with its dial turned to `dial`, or `None` for a kind
    /// with no dial.
    pub fn with_wiper(&self, dial: f64) -> Option<ElementKind> {
        let dial = dial.clamp(0.0, 1.0);
        match *self {
            ElementKind::Potentiometer { ohms, .. } => {
                Some(ElementKind::Potentiometer { ohms, wiper: dial })
            }
            ElementKind::Ldr { .. } => Some(ElementKind::Ldr { light: dial }),
            ElementKind::Phototransistor { .. } => {
                Some(ElementKind::Phototransistor { light: dial })
            }
            ElementKind::SolarCell { .. } => Some(ElementKind::SolarCell { sun: dial }),
            ElementKind::Cell { part, .. } => Some(ElementKind::Cell { part, charge: dial }),
            ElementKind::Thermistor { .. } => Some(ElementKind::Thermistor {
                celsius: crate::solver::device::sensor::celsius(dial),
            }),
            ElementKind::Tmp36 { .. } => Some(ElementKind::Tmp36 {
                celsius: crate::solver::device::tmp36::celsius(dial),
            }),
            ElementKind::Reed { .. } => Some(ElementKind::Reed { magnet: dial }),
            ElementKind::Hall { .. } => Some(ElementKind::Hall { magnet: dial }),
            _ => None,
        }
    }

    /// Whether current can pass through it at all.
    ///
    /// False for an ideal voltmeter and nothing else: infinite resistance is
    /// what makes it a measurement rather than a load (C-21). It is also why a
    /// wire with a voltmeter tapped onto it still has exactly two ends as far
    /// as the current through it is concerned — three pins meet there, but the
    /// current does not divide between them.
    /// For a part made of LEDs sharing one pin — a digit's segments, an RGB
    /// LED's dice — which end they share and each one's colour, in pin order.
    /// The shared pin is the one after them (C-75, C-84).
    ///
    /// The one question everything that stamps, reads or draws such a part
    /// asks, so a third one is an arm here rather than a sweep.
    pub fn shared_leds(
        &self,
    ) -> Option<(crate::solver::device::digit::Common, &'static [LedColour])> {
        match *self {
            ElementKind::SevenSegment { common } => Some((common, &SEGMENT_COLOURS)),
            ElementKind::RgbLed { common } => Some((common, &crate::solver::device::rgb::DICE)),
            _ => None,
        }
    }

    /// The groups of pins nothing conducts between, for a part that has more
    /// than one: an optocoupler's LED and its transistor (C-92), a relay's
    /// coil and its contacts. `None` for
    /// everything else, whose pins are all one circuit. Two nets are in the
    /// same circuit only when one side of a part joins them, so each side
    /// gets a reference of its own.
    pub fn sides(&self) -> Option<&'static [&'static [usize]]> {
        match self {
            ElementKind::Optocoupler => Some(&crate::solver::device::opto::SIDES),
            ElementKind::Relay { .. } => Some(&crate::solver::device::relay::SIDES),
            ElementKind::Transformer { .. } => Some(&crate::solver::device::transformer::SIDES),
            _ => None,
        }
    }

    pub fn carries_current(&self) -> bool {
        !matches!(self, ElementKind::Voltmeter)
    }

    /// What a reactive element starts a run holding — volts for a capacitor,
    /// amps for an inductor (C-18). `None` for everything that carries nothing
    /// between steps.
    ///
    /// Read by the view the app draws and by the live run, which restarts an
    /// element whose initial condition has just been edited: "starts at" can
    /// only mean something at the moment it is set, in a simulation that never
    /// stops (C-48).
    pub fn initial_condition(&self) -> Option<f64> {
        match *self {
            ElementKind::Capacitor { initial_volts, .. }
            | ElementKind::Electrolytic { initial_volts, .. } => Some(initial_volts),
            ElementKind::Inductor { initial_amps, .. } => Some(initial_amps),
            _ => None,
        }
    }

    /// Pin names for this kind, in pin order.
    ///
    /// From the registry, which is the one place they are written (ADR 0008).
    /// Polarity is part of the name because it is part of the electrical
    /// meaning: reversing an LED is a circuit mistake, not a drawing choice.
    pub fn pin_names(&self) -> &'static [&'static str] {
        crate::registry::definition(self.name())
            .map(|component| component.pins)
            .unwrap_or(&[])
    }
}

/// An element and its pins.
#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub kind: ElementKind,
    pub pins: Vec<Pin>,
}

impl Element {
    fn new(kind: ElementKind) -> Self {
        let pins = kind
            .pin_names()
            .iter()
            .copied()
            .map(Pin::electrical)
            .collect();
        Element { kind, pins }
    }

    /// Index of a pin by name, for callers that would rather say `"anode"` than
    /// remember that it is pin 0.
    pub fn pin_index(&self, name: &str) -> Option<usize> {
        self.pins.iter().position(|pin| pin.name == name)
    }
}

/// An electrical connection: every pin on a net is at the same potential.
///
/// This is the editing-level view, the thing a wire creates. Turning nets into
/// numbered solver nodes, with exactly one reference node, is M-05.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Net {
    pins: Vec<PinRef>,
}

impl Net {
    pub fn pins(&self) -> &[PinRef] {
        &self.pins
    }

    pub fn is_floating(&self) -> bool {
        self.pins.len() < 2
    }
}

/// A net once it has been numbered for the solver.
///
/// Node numbering and the choice of the single reference node are M-05's job;
/// this is the shape that work produces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Node {
    pub index: u32,
    pub net: NetId,
}

/// Every way a netlist operation can legitimately fail. Nothing in here
/// panics: a bad edit is an expected outcome of a UI, not a bug.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetlistError {
    UnknownElement(ElementId),
    UnknownNet(NetId),
    UnknownPin {
        element: ElementId,
        index: usize,
    },
    UnknownPinName {
        element: ElementId,
        name: String,
    },
    /// A pin sits on at most one net. Connecting a connected pin is a mistake
    /// on the caller's side, not something to silently rewire.
    PinAlreadyConnected {
        pin: PinRef,
        net: NetId,
    },
    /// Disconnecting a pin that sits on no net. Reported rather than ignored,
    /// so a double disconnect in an undo stack cannot pass unnoticed.
    PinNotConnected {
        pin: PinRef,
    },
    /// Retuning an element into a different kind of element. A resistor and a
    /// diode do not have the same pins, so this would leave the nets wrong.
    KindMismatch {
        element: ElementId,
    },
}

impl core::fmt::Display for NetlistError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            NetlistError::UnknownElement(id) => write!(f, "no such element: {id:?}"),
            NetlistError::UnknownNet(id) => write!(f, "no such net: {id:?}"),
            NetlistError::UnknownPin { element, index } => {
                write!(f, "element {element:?} has no pin {index}")
            }
            NetlistError::UnknownPinName { element, name } => {
                write!(f, "element {element:?} has no pin named {name:?}")
            }
            NetlistError::PinAlreadyConnected { pin, net } => {
                write!(f, "pin {pin:?} is already on net {net:?}")
            }
            NetlistError::PinNotConnected { pin } => write!(f, "pin {pin:?} is not connected"),
            NetlistError::KindMismatch { element } => {
                write!(f, "element {element:?} cannot change kind")
            }
        }
    }
}

impl core::error::Error for NetlistError {}

/// The room a circuit is in until somebody says otherwise, in °C (C-74).
pub const ROOM_CELSIUS: f64 = 25.0;

/// A circuit.
#[derive(Debug, Clone)]
pub struct Netlist {
    elements: SlotMap<ElementId, Element>,
    nets: SlotMap<NetId, Net>,
    /// The temperature of the air around it, in °C (C-74).
    ///
    /// The circuit's rather than any part's: every part that warms starts
    /// from it and settles back to it. Here rather than beside the circuit,
    /// so it goes wherever the circuit goes — to the engine, into a step of
    /// undo — without anything having to remember to take it along.
    room_celsius: f64,
    /// The program each microcontroller runs, in flash words (ADR 0017):
    /// the part's, as its value is, so it is saved and undone with it.
    /// Shared, because a netlist is cloned for every step of undo and every
    /// edit the engine adopts, and 32 KB a clone would add up.
    programs: slotmap::SecondaryMap<ElementId, std::sync::Arc<Program>>,
}

/// A microcontroller's program (ADR 0017): what it was loaded as, and the
/// flash it fills.
///
/// Kept as the Intel HEX it came in as, so a saved circuit writes back
/// exactly what was loaded; the words are read from it once. Shared, and
/// compared by identity: the run starts a microcontroller from reset when
/// its program is not the same one it was running — another file, or the
/// same one loaded again, which is what a reset is.
#[derive(Debug, PartialEq)]
pub struct Program {
    /// What it is called: the file it came from, or an example's name.
    pub name: String,
    pub hex: String,
    pub words: std::sync::Arc<[u16]>,
}

impl Program {
    /// Reads `hex` into an ATmega328P's flash, or says which line is wrong.
    pub fn from_hex(name: &str, hex: &str) -> Result<Program, crate::avr::hex::HexError> {
        let mut words = vec![0u16; crate::avr::ATMEGA328P_FLASH_WORDS];
        crate::avr::hex::load_hex(hex, &mut words)?;
        Ok(Program {
            name: name.to_string(),
            hex: hex.to_string(),
            words: words.into(),
        })
    }
}

impl Default for Netlist {
    fn default() -> Self {
        Netlist {
            elements: SlotMap::default(),
            nets: SlotMap::default(),
            room_celsius: ROOM_CELSIUS,
            programs: slotmap::SecondaryMap::new(),
        }
    }
}

impl Netlist {
    pub fn new() -> Self {
        Netlist::default()
    }

    /// The temperature of the room the circuit is in, in °C.
    pub fn room_celsius(&self) -> f64 {
        self.room_celsius
    }

    /// The program a microcontroller runs, if it has been given one.
    pub fn program(&self, id: ElementId) -> Option<&std::sync::Arc<Program>> {
        self.programs.get(id)
    }

    /// Gives a microcontroller a program (ADR 0017); it starts from reset.
    pub fn set_program(&mut self, id: ElementId, program: std::sync::Arc<Program>) {
        self.programs.insert(id, program);
    }

    /// Puts the circuit in a warmer or cooler room.
    pub fn set_room_celsius(&mut self, celsius: f64) {
        self.room_celsius = celsius;
    }

    /// Adds an element with its pins unconnected.
    pub fn add_element(&mut self, kind: ElementKind) -> ElementId {
        self.elements.insert(Element::new(kind))
    }

    /// Creates an empty net. Nets exist independently of the pins on them so
    /// that a wire can be drawn before it reaches anything.
    pub fn add_net(&mut self) -> NetId {
        self.nets.insert(Net::default())
    }

    /// Puts a pin on a net.
    ///
    /// Disconnecting, and the invariants that come with it, are M-04.
    pub fn connect(&mut self, pin: PinRef, net: NetId) -> Result<(), NetlistError> {
        let element = self
            .elements
            .get_mut(pin.element)
            .ok_or(NetlistError::UnknownElement(pin.element))?;
        let slot = element
            .pins
            .get_mut(pin.index)
            .ok_or(NetlistError::UnknownPin {
                element: pin.element,
                index: pin.index,
            })?;

        if let Some(existing) = slot.net {
            return Err(NetlistError::PinAlreadyConnected { pin, net: existing });
        }

        // Checked after the pin, so a bad net id cannot leave the pin half-wired.
        let target = self
            .nets
            .get_mut(net)
            .ok_or(NetlistError::UnknownNet(net))?;

        slot.net = Some(net);
        target.pins.push(pin);
        Ok(())
    }

    /// Connects a pin by name — `connect_named(led, "anode", net)`.
    pub fn connect_named(
        &mut self,
        element: ElementId,
        pin_name: &str,
        net: NetId,
    ) -> Result<(), NetlistError> {
        let index = self
            .element(element)
            .ok_or(NetlistError::UnknownElement(element))?
            .pin_index(pin_name)
            .ok_or_else(|| NetlistError::UnknownPinName {
                element,
                name: pin_name.to_string(),
            })?;
        self.connect(PinRef::new(element, index), net)
    }

    /// Changes an element's parameters, leaving its wiring alone.
    ///
    /// The kind has to stay the same kind — a resistor cannot become a diode,
    /// because the pins would no longer mean what the nets think they mean.
    /// Retuning a value is the common case: an inspector edit (M-18), or a
    /// source driven live (M-12).
    pub fn set_kind(&mut self, id: ElementId, kind: ElementKind) -> Result<(), NetlistError> {
        let element = self
            .elements
            .get_mut(id)
            .ok_or(NetlistError::UnknownElement(id))?;

        if core::mem::discriminant(&element.kind) != core::mem::discriminant(&kind) {
            return Err(NetlistError::KindMismatch { element: id });
        }

        element.kind = kind;
        Ok(())
    }

    /// Makes an element another kind of part, in place (C-137): its pins
    /// become the new kind's, and each `(old, new)` pin index in `carry`
    /// keeps the old pin's net. Every other old pin is taken off its net,
    /// and the nets themselves stay, as [`Netlist::remove_element`] leaves
    /// them, for the caller to sweep.
    pub fn replace(
        &mut self,
        id: ElementId,
        kind: ElementKind,
        carry: &[(usize, usize)],
    ) -> Result<(), NetlistError> {
        let element = self
            .elements
            .get(id)
            .ok_or(NetlistError::UnknownElement(id))?;
        let nets: Vec<Option<NetId>> = element.pins.iter().map(|pin| pin.net).collect();
        for (index, net) in nets.iter().enumerate() {
            if let Some(net) = net.and_then(|net| self.nets.get_mut(net)) {
                let reference = PinRef::new(id, index);
                net.pins.retain(|&candidate| candidate != reference);
            }
        }
        // A program is a microcontroller's: another kind has none.
        if core::mem::discriminant(&self.elements[id].kind) != core::mem::discriminant(&kind) {
            self.programs.remove(id);
        }
        self.elements[id] = Element::new(kind);
        for &(old, new) in carry {
            if let Some(net) = nets.get(old).copied().flatten() {
                self.connect(PinRef::new(id, new), net)?;
            }
        }
        Ok(())
    }

    /// Folds one net into another, moving every pin across.
    ///
    /// Wiring two already-wired pins together is the common case — two halves of
    /// a circuit meeting — and it has to end with one net, not two that happen
    /// to touch. Merging into the older net keeps ids stable for whatever is
    /// already pointing at them.
    pub fn merge_nets(&mut self, into: NetId, from: NetId) -> Result<(), NetlistError> {
        if into == from {
            return Ok(());
        }
        if !self.nets.contains_key(into) {
            return Err(NetlistError::UnknownNet(into));
        }
        let absorbed = self
            .nets
            .remove(from)
            .ok_or(NetlistError::UnknownNet(from))?;

        for pin in absorbed.pins {
            if let Some(slot) = self
                .elements
                .get_mut(pin.element)
                .and_then(|element| element.pins.get_mut(pin.index))
            {
                slot.net = Some(into);
            }
            // The net is present: checked above, and nothing removed it since.
            self.nets[into].pins.push(pin);
        }

        Ok(())
    }

    /// Takes a pin off its net.
    ///
    /// The net itself stays, even if it ends up empty: a net with no pins is
    /// still a wire the user may be in the middle of drawing. Sweeping those
    /// up is `remove_empty_nets`, called when the caller decides it is time.
    pub fn disconnect(&mut self, pin: PinRef) -> Result<(), NetlistError> {
        let slot = self.pin_mut(pin)?;
        let net_id = slot.net.ok_or(NetlistError::PinNotConnected { pin })?;
        slot.net = None;

        // The net is guaranteed to exist: a pin only ever holds an id that
        // `connect` put there, and nothing removes a net without clearing it.
        let net = self
            .nets
            .get_mut(net_id)
            .expect("pin referenced a net that is gone");
        net.pins.retain(|&candidate| candidate != pin);
        Ok(())
    }

    /// Removes an element and takes its pins off every net they sat on, so no
    /// net is left pointing at something that no longer exists.
    pub fn remove_element(&mut self, id: ElementId) -> Result<Element, NetlistError> {
        let element = self
            .elements
            .remove(id)
            .ok_or(NetlistError::UnknownElement(id))?;
        self.programs.remove(id);

        for (index, pin) in element.pins.iter().enumerate() {
            let Some(net_id) = pin.net else { continue };
            let reference = PinRef::new(id, index);
            if let Some(net) = self.nets.get_mut(net_id) {
                net.pins.retain(|&candidate| candidate != reference);
            }
        }

        Ok(element)
    }

    /// Removes a net, disconnecting every pin that sat on it. The pins stay,
    /// unconnected — deleting a wire does not delete what it joined.
    pub fn remove_net(&mut self, id: NetId) -> Result<(), NetlistError> {
        let net = self.nets.remove(id).ok_or(NetlistError::UnknownNet(id))?;

        for pin in net.pins {
            if let Some(slot) = self
                .elements
                .get_mut(pin.element)
                .and_then(|element| element.pins.get_mut(pin.index))
            {
                slot.net = None;
            }
        }

        Ok(())
    }

    /// Drops every net that no longer joins anything, and reports how many went.
    ///
    /// Deleting an element leaves its nets behind on purpose; this is the call
    /// that clears the debris, at a moment the caller chooses rather than in
    /// the middle of an edit.
    pub fn remove_empty_nets(&mut self) -> usize {
        let before = self.nets.len();
        self.nets.retain(|_, net| !net.pins.is_empty());
        before - self.nets.len()
    }

    fn pin_mut(&mut self, pin: PinRef) -> Result<&mut Pin, NetlistError> {
        self.elements
            .get_mut(pin.element)
            .ok_or(NetlistError::UnknownElement(pin.element))?
            .pins
            .get_mut(pin.index)
            .ok_or(NetlistError::UnknownPin {
                element: pin.element,
                index: pin.index,
            })
    }

    pub fn element(&self, id: ElementId) -> Option<&Element> {
        self.elements.get(id)
    }

    pub fn net(&self, id: NetId) -> Option<&Net> {
        self.nets.get(id)
    }

    pub fn elements(&self) -> impl Iterator<Item = (ElementId, &Element)> {
        self.elements.iter()
    }

    pub fn nets(&self) -> impl Iterator<Item = (NetId, &Net)> {
        self.nets.iter()
    }

    pub fn element_count(&self) -> usize {
        self.elements.len()
    }

    pub fn net_count(&self) -> usize {
        self.nets.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Checks the one invariant the whole module rests on: a pin names a net
    /// if and only if that net names the pin back, exactly once.
    fn assert_consistent(netlist: &Netlist) {
        for (id, element) in netlist.elements() {
            for (index, pin) in element.pins.iter().enumerate() {
                let Some(net_id) = pin.net else { continue };
                let reference = PinRef::new(id, index);
                let net = netlist
                    .net(net_id)
                    .unwrap_or_else(|| panic!("{reference:?} points at missing net {net_id:?}"));
                let mentions = net.pins().iter().filter(|&&p| p == reference).count();
                assert_eq!(
                    mentions, 1,
                    "{reference:?} appears {mentions} times on its net"
                );
            }
        }

        for (net_id, net) in netlist.nets() {
            for &pin in net.pins() {
                let element = netlist
                    .element(pin.element)
                    .unwrap_or_else(|| panic!("net {net_id:?} points at removed element"));
                let slot = element
                    .pins
                    .get(pin.index)
                    .unwrap_or_else(|| panic!("net {net_id:?} points at missing pin {pin:?}"));
                assert_eq!(slot.net, Some(net_id), "{pin:?} disagrees with its net");
            }
        }
    }

    /// The M-21 circuit: a 9 V battery, a 220 Ω resistor and a red LED in one
    /// loop. Built here with no canvas, no coordinates and no solver.
    fn battery_resistor_led() -> (Netlist, ElementId, ElementId, ElementId) {
        let mut netlist = Netlist::new();

        let battery = netlist.add_element(ElementKind::DcSource {
            volts: 9.0,
            series_ohms: 0.0,
        });
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: 220.0 });
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

        (netlist, battery, resistor, led)
    }

    #[test]
    fn builds_a_battery_resistor_led_loop() {
        let (netlist, battery, resistor, led) = battery_resistor_led();

        assert_eq!(netlist.element_count(), 3);
        assert_eq!(netlist.net_count(), 3);

        // Every pin of every element is on a net: the loop is closed.
        for (_, element) in netlist.elements() {
            assert!(element.pins.iter().all(|pin| pin.net.is_some()));
        }

        // Every net joins exactly two pins, so none of them is floating.
        for (_, net) in netlist.nets() {
            assert_eq!(net.pins().len(), 2);
            assert!(!net.is_floating());
        }

        // The resistor really does sit between the battery and the LED.
        let shared = |a: ElementId, a_pin: &str, b: ElementId, b_pin: &str| {
            let net_of = |id: ElementId, name: &str| {
                let element = netlist.element(id).unwrap();
                element.pins[element.pin_index(name).unwrap()].net.unwrap()
            };
            net_of(a, a_pin) == net_of(b, b_pin)
        };
        assert!(shared(battery, "+", resistor, "a"));
        assert!(shared(resistor, "b", led, "anode"));
        assert!(shared(led, "cathode", battery, "-"));
    }

    #[test]
    fn element_parameters_survive_construction() {
        let (netlist, battery, resistor, led) = battery_resistor_led();

        assert_eq!(
            netlist.element(battery).unwrap().kind,
            ElementKind::DcSource {
                volts: 9.0,
                series_ohms: 0.0
            }
        );
        assert_eq!(
            netlist.element(resistor).unwrap().kind,
            ElementKind::Resistor { ohms: 220.0 }
        );
        assert_eq!(
            netlist.element(led).unwrap().kind,
            ElementKind::Led {
                colour: LedColour::Red
            }
        );
    }

    #[test]
    fn a_new_element_starts_unconnected() {
        let mut netlist = Netlist::new();
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: 1_000.0 });

        let element = netlist.element(resistor).unwrap();
        assert_eq!(element.pins.len(), 2);
        assert!(element.pins.iter().all(|pin| pin.net.is_none()));
    }

    #[test]
    fn a_net_with_one_pin_is_floating() {
        let mut netlist = Netlist::new();
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: 1_000.0 });
        let net = netlist.add_net();

        assert!(netlist.net(net).unwrap().is_floating());
        netlist.connect_named(resistor, "a", net).unwrap();
        assert!(netlist.net(net).unwrap().is_floating());
    }

    #[test]
    fn a_pin_sits_on_at_most_one_net() {
        let mut netlist = Netlist::new();
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: 220.0 });
        let first = netlist.add_net();
        let second = netlist.add_net();

        netlist.connect_named(resistor, "a", first).unwrap();
        let err = netlist
            .connect(PinRef::new(resistor, 0), second)
            .unwrap_err();

        assert_eq!(
            err,
            NetlistError::PinAlreadyConnected {
                pin: PinRef::new(resistor, 0),
                net: first,
            }
        );
        assert_eq!(netlist.net(second).unwrap().pins(), &[]);
    }

    #[test]
    fn a_failed_connect_leaves_the_pin_untouched() {
        let mut netlist = Netlist::new();
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: 220.0 });

        // The null key is the one id slotmap guarantees never resolves. Keys
        // borrowed from another netlist would only fail here by luck.
        let unknown = NetId::default();
        let err = netlist
            .connect(PinRef::new(resistor, 0), unknown)
            .unwrap_err();

        assert_eq!(err, NetlistError::UnknownNet(unknown));
        assert!(netlist.element(resistor).unwrap().pins[0].net.is_none());
    }

    #[test]
    fn unknown_pins_are_reported_rather_than_panicking() {
        let mut netlist = Netlist::new();
        let led = netlist.add_element(ElementKind::Led {
            colour: LedColour::Green,
        });
        let net = netlist.add_net();

        assert_eq!(
            netlist.connect(PinRef::new(led, 7), net).unwrap_err(),
            NetlistError::UnknownPin {
                element: led,
                index: 7
            }
        );
        assert_eq!(
            netlist.connect_named(led, "gate", net).unwrap_err(),
            NetlistError::UnknownPinName {
                element: led,
                name: "gate".to_string()
            }
        );
    }

    #[test]
    fn disconnecting_clears_both_sides() {
        let (mut netlist, _, resistor, _) = battery_resistor_led();
        let pin = PinRef::new(resistor, 0);
        let net = netlist.element(resistor).unwrap().pins[0].net.unwrap();

        netlist.disconnect(pin).unwrap();

        assert!(netlist.element(resistor).unwrap().pins[0].net.is_none());
        assert!(!netlist.net(net).unwrap().pins().contains(&pin));
        assert_consistent(&netlist);
    }

    #[test]
    fn disconnecting_twice_is_reported() {
        let (mut netlist, _, resistor, _) = battery_resistor_led();
        let pin = PinRef::new(resistor, 0);

        netlist.disconnect(pin).unwrap();
        assert_eq!(
            netlist.disconnect(pin).unwrap_err(),
            NetlistError::PinNotConnected { pin }
        );
    }

    #[test]
    fn removing_an_element_leaves_no_net_pointing_at_it() {
        let (mut netlist, _, resistor, _) = battery_resistor_led();

        let removed = netlist.remove_element(resistor).unwrap();

        assert_eq!(removed.kind, ElementKind::Resistor { ohms: 220.0 });
        assert_eq!(netlist.element_count(), 2);
        for (_, net) in netlist.nets() {
            assert!(net.pins().iter().all(|pin| pin.element != resistor));
        }
        assert_consistent(&netlist);
    }

    #[test]
    fn an_id_does_not_survive_its_element() {
        let (mut netlist, _, resistor, _) = battery_resistor_led();

        netlist.remove_element(resistor).unwrap();

        // The payoff of generational keys: the stale id reads as absent rather
        // than as whatever later took the slot.
        assert!(netlist.element(resistor).is_none());
        assert_eq!(
            netlist.remove_element(resistor).unwrap_err(),
            NetlistError::UnknownElement(resistor)
        );

        let replacement = netlist.add_element(ElementKind::Resistor { ohms: 470.0 });
        assert_ne!(replacement, resistor);
        assert!(netlist.element(resistor).is_none());
    }

    #[test]
    fn removing_a_net_disconnects_its_pins_but_keeps_them() {
        let (mut netlist, battery, resistor, _) = battery_resistor_led();
        let net = netlist.element(resistor).unwrap().pins[0].net.unwrap();

        netlist.remove_net(net).unwrap();

        assert!(netlist.net(net).is_none());
        assert_eq!(netlist.element_count(), 3);
        assert!(netlist.element(resistor).unwrap().pins[0].net.is_none());
        let plus = netlist.element(battery).unwrap().pin_index("+").unwrap();
        assert!(netlist.element(battery).unwrap().pins[plus].net.is_none());
        assert_consistent(&netlist);
    }

    #[test]
    fn empty_nets_are_swept_only_when_asked() {
        let (mut netlist, _, resistor, _) = battery_resistor_led();
        let drawing = netlist.add_net();

        netlist.remove_element(resistor).unwrap();
        // Two nets lost a pin each, neither is empty; the fresh one is.
        assert_eq!(netlist.net_count(), 4);

        assert_eq!(netlist.remove_empty_nets(), 1);
        assert_eq!(netlist.net_count(), 3);
        assert!(netlist.net(drawing).is_none());
        assert_consistent(&netlist);
    }

    #[test]
    fn a_pin_can_be_rewired_after_disconnecting() {
        let (mut netlist, _, resistor, led) = battery_resistor_led();
        let pin = PinRef::new(resistor, 0);
        let led_cathode_net = {
            let element = netlist.element(led).unwrap();
            element.pins[element.pin_index("cathode").unwrap()]
                .net
                .unwrap()
        };

        netlist.disconnect(pin).unwrap();
        netlist.connect(pin, led_cathode_net).unwrap();

        assert_eq!(netlist.net(led_cathode_net).unwrap().pins().len(), 3);
        assert_consistent(&netlist);
    }

    #[test]
    fn stays_consistent_through_a_round_of_edits() {
        let (mut netlist, battery, resistor, led) = battery_resistor_led();
        assert_consistent(&netlist);

        netlist.disconnect(PinRef::new(resistor, 1)).unwrap();
        assert_consistent(&netlist);

        netlist.remove_element(led).unwrap();
        assert_consistent(&netlist);

        let replacement = netlist.add_element(ElementKind::Led {
            colour: LedColour::Blue,
        });
        let free = netlist.add_net();
        netlist.connect_named(replacement, "anode", free).unwrap();
        netlist.connect_named(resistor, "b", free).unwrap();
        assert_consistent(&netlist);

        netlist.remove_empty_nets();
        assert_consistent(&netlist);
        assert_eq!(netlist.element_count(), 3);
        assert!(netlist.element(battery).is_some());
    }
}
