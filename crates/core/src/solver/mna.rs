// SPDX-License-Identifier: MIT OR Apache-2.0

//! Modified nodal analysis: turning a numbered netlist into `A·x = z`.
//!
//! The unknown vector `x` is every node voltage except the reference, followed
//! by the current through each voltage source. The reference node's row and
//! column are not in the matrix at all — that is what makes the system
//! solvable, and it is why [`NodeMap`] has to pick a reference first (M-05).
//!
//! ```text
//!          nodes          sources
//!        ┌─────────┬─────────────┐
//! nodes  │    G    │      B      │   G: conductances
//!        ├─────────┼─────────────┤   B: which nodes a source spans
//! srcs   │   Bᵀ    │      0      │
//!        └─────────┴─────────────┘
//! ```
//!
//! Only linear elements are stamped here. Diodes and LEDs need a linearisation
//! step around an operating point (M-09, M-10), so they are reported as
//! unstamped rather than silently contributing nothing.

use slotmap::SecondaryMap;

use crate::netlist::{ElementId, ElementKind, Netlist, NodeMap, REFERENCE_NODE};
use crate::solver::device::{Capacitor, Inductor, relay};
use crate::solver::{Charge, Integration, MAX_TERMINALS};

/// Why an element contributed nothing to the matrix.
///
/// Reported rather than swallowed: the live loop (M-19) has to keep solving
/// while a circuit is half-built, but the app still needs to be able to say
/// what it ignored (C-25).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnstampedReason {
    /// At least one pin is not on a net, so the element spans nothing.
    PinNotConnected,
    /// A value that cannot be stamped as it stands — a zero-ohm resistor is an
    /// ideal short, which MNA expresses as a voltage source, not a conductance.
    DegenerateValue,
    /// A capacitor with no timestep to look at: open, which is what it is once
    /// nothing is changing.
    OpenAtDc,
    /// A switch or a button that is open. Nothing crosses a gap, so there is
    /// nothing to stamp — which is not a problem with the element but the
    /// whole of what it is for (C-19).
    ContactOpen,
    /// An ideal voltmeter, which is an open circuit by definition. Not stamping
    /// it is what stops the measurement changing the thing measured (C-21).
    DrawsNoCurrent,
}

/// A coupled element and the rows of its terminals (C-12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoupledPlacement {
    pub element: ElementId,
    /// One row per terminal, in the element's pin order. `None` is the
    /// reference node, which has no row.
    pub rows: [Option<usize>; MAX_TERMINALS],
    /// The bit a 555 or a relay is carrying into this step (ADR 0012, C-64),
    /// and false for everything else. Read from the step's carried state, because what a
    /// latch holds is the run's and not the netlist's.
    pub latched: bool,
}

/// A nonlinear element and where it sits in the matrix.
///
/// The linear part of the system is stamped once; these are handed to Newton,
/// which restamps them about a moving operating point every iteration (M-09).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NonlinearPlacement {
    pub element: ElementId,
    /// Matrix row of the `+` terminal, or `None` when it is the reference node.
    pub plus: Option<usize>,
    pub minus: Option<usize>,
    /// Which of the element's junctions this is: 0 for a part with one, and
    /// for a part whose LEDs share a pin, the pin of the LED it is — a
    /// digit's segment, an RGB LED's die (C-75, C-84). Several placements for
    /// one element, and not all the same LED.
    pub junction: usize,
    /// What it is set to carry, for a junction that is a current sink: an
    /// LM3914's output, lit or not (C-119). Nothing for every other.
    pub drive: f64,
}

/// An element left out of the matrix, and why.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unstamped {
    pub element: ElementId,
    pub reason: UnstampedReason,
}

/// The assembled system.
#[derive(Debug, Clone, PartialEq)]
pub struct MnaSystem {
    size: usize,
    node_count: usize,
    a: Vec<f64>,
    z: Vec<f64>,
    sources: Vec<ElementId>,
    nonlinear: Vec<NonlinearPlacement>,
    coupled: Vec<CoupledPlacement>,
    unstamped: Vec<Unstamped>,
}

impl MnaSystem {
    /// Side length of `A`: one row per non-reference node, plus one per source.
    pub fn size(&self) -> usize {
        self.size
    }

    /// `A[row][col]`.
    pub fn at(&self, row: usize, col: usize) -> f64 {
        self.a[row * self.size + col]
    }

    /// The matrix, row-major.
    pub fn a(&self) -> &[f64] {
        &self.a
    }

    /// The right-hand side.
    pub fn z(&self) -> &[f64] {
        &self.z
    }

    /// Row of the matrix carrying a node's KCL equation, or `None` for the
    /// reference node, which has no row.
    pub fn row_of_node(&self, node_index: u32) -> Option<usize> {
        if node_index == REFERENCE_NODE {
            None
        } else {
            Some(node_index as usize - 1)
        }
    }

    /// The elements with a branch-current unknown, in row order: the first
    /// is at row `node_count`, and so on.
    pub fn sources(&self) -> &[ElementId] {
        &self.sources
    }

    /// Row carrying a source's branch current, once M-08 needs to read it back.
    pub fn row_of_source(&self, element: ElementId) -> Option<usize> {
        let position = self.sources.iter().position(|&id| id == element)?;
        Some(self.node_count + position)
    }

    /// Elements that contributed nothing, and why.
    pub fn unstamped(&self) -> &[Unstamped] {
        &self.unstamped
    }

    /// Adds a conductance from every node to the reference (C-11).
    ///
    /// A node joined to the rest of the circuit only through devices that are
    /// currently off has almost no conductance anywhere, and its row is
    /// numerically zero — which is a singular matrix, reported as one, for a
    /// circuit that has a perfectly good answer. A tiny leak to ground at every
    /// node makes the matrix solvable; stepping it back down to nothing, and
    /// re-solving from each answer, arrives at the circuit that was asked for.
    ///
    /// Node rows only. A source's branch row carries a current unknown, not a
    /// potential, and leaking that to ground would change the source.
    pub fn add_gmin(&mut self, gmin: f64) {
        for row in 0..self.node_count {
            self.a[row * self.size + row] += gmin;
        }
    }

    /// Scales every source, for source stepping (C-11).
    ///
    /// A source's branch row carries its voltage on the right-hand side and
    /// nothing else does, so turning every supply down to a fraction of itself
    /// is one multiplication per source.
    pub fn scale_sources(&mut self, factor: f64) {
        for row in self.node_count..self.size {
            self.z[row] *= factor;
        }
    }

    /// Nonlinear elements and their rows, for the Newton loop to restamp.
    pub fn nonlinear(&self) -> &[NonlinearPlacement] {
        &self.nonlinear
    }

    /// Coupled elements and their rows, likewise.
    pub fn coupled(&self) -> &[CoupledPlacement] {
        &self.coupled
    }

    fn add(&mut self, row: usize, col: usize, value: f64) {
        self.a[row * self.size + col] += value;
    }

    /// Stamps a conductance between two nodes, skipping any row or column that
    /// belongs to the reference node.
    /// The current source half of a companion model.
    ///
    /// Same sign convention as Newton's, which stamps the linearised nonlinear
    /// devices: `current` flows from the `+` terminal to the `-` one inside the
    /// device, so it leaves the `+` node.
    fn stamp_companion_source(&mut self, plus: Option<usize>, minus: Option<usize>, current: f64) {
        if let Some(plus) = plus {
            self.z[plus] -= current;
        }
        if let Some(minus) = minus {
            self.z[minus] += current;
        }
    }

    /// A current through one pair of nodes set by the voltage across another:
    /// `g · (V(c⁺) − V(c⁻))` into `out.0` and out of `out.1`, through the
    /// device. Half of a coupled pair's conductance matrix — a transformer's
    /// winding driven by the other's voltage (C-118).
    fn stamp_transconductance(
        &mut self,
        out: (Option<usize>, Option<usize>),
        control: (Option<usize>, Option<usize>),
        g: f64,
    ) {
        for (row, sign) in [(out.0, 1.0), (out.1, -1.0)] {
            let Some(row) = row else { continue };
            if let Some(col) = control.0 {
                self.add(row, col, sign * g);
            }
            if let Some(col) = control.1 {
                self.add(row, col, -sign * g);
            }
        }
    }

    fn stamp_conductance(&mut self, a: Option<usize>, b: Option<usize>, g: f64) {
        if let Some(a) = a {
            self.add(a, a, g);
        }
        if let Some(b) = b {
            self.add(b, b, g);
        }
        if let (Some(a), Some(b)) = (a, b) {
            self.add(a, b, -g);
            self.add(b, a, -g);
        }
    }

    /// Stamps a voltage source spanning two nodes, driving `volts` from `plus`
    /// to `minus`.
    /// A voltage source, with its internal resistance if it has one.
    ///
    /// The branch equation for an ideal source is V(+) − V(−) = Vs. With a
    /// resistance inside the element the terminals see Vs less what the
    /// delivered current drops across it.
    ///
    /// The branch unknown is the current flowing *into* the `+` terminal, so
    /// the delivered current is its negation — which is why the new term is
    /// subtracted rather than added:
    ///
    /// ```text
    /// V(+) − V(−) = Vs − R·I_delivered = Vs + R·x
    ///     ⟹  V(+) − V(−) − R·x = Vs
    /// ```
    ///
    /// One entry on the branch's own diagonal. No extra node, and the branch
    /// unknown still means what it meant, so everything that reads a source's
    /// current is unaffected. Getting the sign the other way round makes the
    /// resistance a *source* of energy: a 1 Ω cell into a 1 Ω load cancels to
    /// nothing and the current runs away to 9e12 A, which is how this was
    /// found.
    fn stamp_voltage_source(
        &mut self,
        plus: Option<usize>,
        minus: Option<usize>,
        branch: usize,
        volts: f64,
        series_ohms: f64,
    ) {
        if let Some(plus) = plus {
            self.add(plus, branch, 1.0);
            self.add(branch, plus, 1.0);
        }
        if let Some(minus) = minus {
            self.add(minus, branch, -1.0);
            self.add(branch, minus, -1.0);
        }
        self.add(branch, branch, -series_ohms);
        self.z[branch] = volts;
    }
}

/// Stamps a potentiometer, and reports why if nothing could be stamped.
///
/// Two conductances that always add up to the same track. Linear, so it goes
/// into the matrix here rather than to Newton, and uncoupled, so it needs none
/// of C-12's machinery: a wiper is two resistors that share a number.
///
/// Each half is stamped on its own account, so a track with one terminal in the
/// air still conducts through the half that is wired. With the *wiper* in the
/// air and both ends wired it is a plain resistor of the whole track, which is
/// what a pot used as a fixed resistor is.
fn stamp_wiper(
    system: &mut MnaSystem,
    element: &crate::netlist::Element,
    map: &NodeMap,
    ohms: f64,
    wiper: f64,
) -> Option<UnstampedReason> {
    if ohms <= 0.0 || !ohms.is_finite() {
        return Some(UnstampedReason::DegenerateValue);
    }
    let [a, w, b] = element.pins.as_slice() else {
        return Some(UnstampedReason::PinNotConnected);
    };
    // `None` for a pin on no node at all, `Some(None)` for one on the
    // reference, which has no row of its own.
    let row = |pin: &crate::netlist::Pin| {
        pin.net
            .and_then(|net| map.index_of(net))
            .map(|node| system.row_of_node(node))
    };
    let (a, w, b) = (&row(a), &row(w), &row(b));

    // The wiper never reaches quite the end of its track. A real one does not
    // either — there is always a little metal between the contact and the
    // terminal — and the alternative is a zero-ohm half, which MNA can only say
    // as a voltage source, and which would make a knob at the stop behave
    // unlike a knob a hair away from it.
    let along = wiper.clamp(END_RESISTANCE, 1.0 - END_RESISTANCE);

    match (a, w, b) {
        (_, Some(w), _) => {
            let mut stamped = false;
            if let Some(a) = a {
                system.stamp_conductance(*a, *w, 1.0 / (ohms * along));
                stamped = true;
            }
            if let Some(b) = b {
                system.stamp_conductance(*w, *b, 1.0 / (ohms * (1.0 - along)));
                stamped = true;
            }
            // Only the wiper is wired, so the track leads nowhere.
            if stamped {
                None
            } else {
                Some(UnstampedReason::PinNotConnected)
            }
        }
        // No wiper, but both ends: the whole track, which is a resistor.
        (Some(a), None, Some(b)) => {
            system.stamp_conductance(*a, *b, 1.0 / ohms);
            None
        }
        _ => Some(UnstampedReason::PinNotConnected),
    }
}

/// The least of its own track a potentiometer's wiper can have behind it.
///
/// A ten-thousandth, so a 10 kΩ pot bottoms out at an ohm and a 100 Ω one at ten
/// milliohms. Small enough to be the short circuit a knob at the stop is, large
/// enough that no matrix goes singular over it.
pub(crate) const END_RESISTANCE: f64 = 1e-4;

/// One step of a transient analysis: how long it is, and where the reactive
/// elements were left by the step before it.
///
/// Passed rather than held, because the solver owns no history — C-09's loop
/// does, and C-10's step rejection needs to be able to throw a step away and
/// try a shorter one without anything else having moved on.
pub struct Timestep<'a> {
    /// Length of the step, in seconds.
    pub seconds: f64,
    /// What each reactive element was left holding at the end of the previous
    /// step. An element missing from the map starts from zero — uncharged, and
    /// not moving.
    pub previous: &'a SecondaryMap<ElementId, Charge>,
    /// Which difference stands in for the derivative over this step (C-08).
    pub method: Integration,
    /// When the step ends, in seconds from the start of the run.
    ///
    /// What a signal generator is stamped at (ADR 0011): the implicit rules
    /// solve for the end of the step, so that is when its sources are read.
    pub at: f64,
}

/// Assembles the linear system for a compiled netlist, at DC.
///
/// Capacitors are open circuits here, which is what they are once nothing is
/// changing: the operating point a transient starts from.
pub fn assemble(netlist: &Netlist, map: &NodeMap) -> MnaSystem {
    assemble_at(netlist, map, None)
}

/// Assembles the linear system for one step of a transient, or for DC when
/// there is no step.
/// The most pins any part has: a bar of ten LEDs' twenty (C-119).
const MOST_PINS: usize = 24;

/// Each of an element's pins' row: `None` for a pin on no node, `Some(None)`
/// for one on the reference, which has no row of its own; the element's own
/// pins first, and nothing after them.
///
/// On the stack, because it runs for every element of every solve, and a live
/// 555 at audio frequency solves some seventy thousand times a second (C-94).
fn pin_rows(
    element: &crate::netlist::Element,
    map: &NodeMap,
    system: &MnaSystem,
) -> [Option<Option<usize>>; MOST_PINS] {
    debug_assert!(
        element.pins.len() <= MOST_PINS,
        "a part with more pins than any"
    );
    let mut rows = [None; MOST_PINS];
    for (slot, pin) in rows.iter_mut().zip(&element.pins) {
        *slot = pin
            .net
            .and_then(|net| map.index_of(net))
            .map(|node| system.row_of_node(node));
    }
    rows
}

pub fn assemble_at(netlist: &Netlist, map: &NodeMap, step: Option<&Timestep>) -> MnaSystem {
    // References have no rows, hence the subtraction — one per circuit on the
    // canvas, not one overall.
    let node_count = map.len() - map.references().len();
    // A wire between two nodes is a zero-volt source: it needs a branch unknown
    // of its own, the same as a supply, because MNA has no way to say "these
    // two nodes are the same" with a conductance.
    //
    // Three elements are that wire. An inductor is one at DC only — during a
    // transient it is a companion instead and needs no row. A closed contact is
    // one always: a switch does not turn into something else when time starts
    // moving, and an ideal one is a piece of wire whatever the analysis (C-19).
    // An ammeter is one too, and its branch unknown is the reading (C-21).
    let wired = |element: &crate::netlist::Element| {
        element
            .pins
            .iter()
            .all(|pin| pin.net.and_then(|net| map.index_of(net)).is_some())
    };
    let shorted: Vec<ElementId> = netlist
        .elements()
        .filter(|(_, element)| match element.kind {
            ElementKind::Inductor { .. } => step.is_none(),
            ElementKind::Ammeter => true,
            kind => kind.contact() == Some(true),
        })
        .filter(|(_, element)| wired(element))
        .map(|(id, _)| id)
        .collect();

    // Only sources that will actually be stamped get a branch unknown. A source
    // with a pin off a net is skipped below, and a branch row nothing ever
    // stamps is a row of zeros — which is a singular matrix, reported to the
    // user as "matrix is singular at column 0" the moment they wire a battery
    // to one thing and not yet to the next.
    let sources: Vec<ElementId> = netlist
        .elements()
        .filter(|(_, element)| {
            matches!(
                element.kind,
                ElementKind::DcSource { .. } | ElementKind::SignalGenerator { .. }
            )
        })
        .filter(|(_, element)| {
            element
                .pins
                .iter()
                .all(|pin| pin.net.and_then(|net| map.index_of(net)).is_some())
        })
        .map(|(id, _)| id)
        .chain(shorted)
        .collect();

    let size = node_count + sources.len();
    let mut system = MnaSystem {
        size,
        node_count,
        a: vec![0.0; size * size],
        z: vec![0.0; size],
        sources,
        // Sized for the commonest worst case up front: grown a push at a
        // time, these were the largest share of a solve's allocations (C-94).
        nonlinear: Vec::with_capacity(netlist.element_count()),
        coupled: Vec::with_capacity(netlist.element_count()),
        unstamped: Vec::with_capacity(netlist.element_count()),
    };

    for (id, element) in netlist.elements() {
        // A potentiometer is the one element worth stamping in pieces. Wired as
        // a rheostat — one end and the wiper, the far end left in the air — it
        // is how a beginner dims an LED, and requiring every pin would make it
        // contribute nothing because of the terminal nobody attached (C-20).
        //
        // It looks up its own pins, before the rows below are built: three of
        // them on the stack, so the loop everything else goes through keeps the
        // one allocation per element it had.
        // A relay is the other: one of its two contacts is usually left free,
        // and a relay with its NC unused is still a relay (C-64). A pin on no
        // net gets no row, and the model is told which are wired so it stamps
        // nothing that touches the others.
        if let ElementKind::Relay { part } = element.kind {
            let mut terminals = [None; MAX_TERMINALS];
            for (slot, pin) in terminals.iter_mut().zip(&element.pins) {
                *slot = pin
                    .net
                    .and_then(|net| map.index_of(net))
                    .and_then(|node| system.row_of_node(node));
            }
            let carried = step
                .and_then(|step| step.previous.get(id))
                .copied()
                .unwrap_or_default();
            // The coil is linear, so it goes into the matrix here rather than
            // to Newton with the contacts: its winding at DC, and during a
            // step the winding and its inductance in series (C-67).
            let coil_wired = [relay::COIL_PLUS, relay::COIL_MINUS].iter().all(|&pin| {
                element.pins[pin]
                    .net
                    .and_then(|net| map.index_of(net))
                    .is_some()
            });
            if coil_wired {
                let (plus, minus) = (terminals[relay::COIL_PLUS], terminals[relay::COIL_MINUS]);
                match step {
                    None => system.stamp_conductance(plus, minus, 1.0 / part.coil_ohms()),
                    Some(step) => {
                        let coil = part.coil(step.method, step.seconds, carried);
                        system.stamp_conductance(plus, minus, coil.conductance);
                        system.stamp_companion_source(plus, minus, coil.current);
                    }
                }
            }
            let latched = step.is_some() && carried.latched;
            system.coupled.push(CoupledPlacement {
                element: id,
                rows: terminals,
                latched,
            });
            continue;
        }

        // A seven-segment digit is eight LEDs sharing a pin, and an RGB LED
        // three, and nothing couples them: each LED wired to something, with
        // the common wired too, is a junction of its own, and one left free
        // is simply dark (C-75, C-84). Several placements for one element,
        // each saying which LED it is, because an RGB LED's are not the same.
        if let Some((common, colours)) = element.kind.shared_leds() {
            // `None` for a pin on no node, `Some(None)` for one on the
            // reference, which has no row of its own.
            let rows = pin_rows(element, map, &system);
            let rows = &rows[..element.pins.len()];
            let shared_pin = colours.len();
            let mut stamped = false;
            if let Some(shared) = rows.get(shared_pin).copied().flatten() {
                for (led, row) in rows.iter().enumerate().take(shared_pin) {
                    let Some(own) = *row else {
                        continue;
                    };
                    let (anode, _) = common.junction(led, shared_pin);
                    let (plus, minus) = if anode == led {
                        (own, shared)
                    } else {
                        (shared, own)
                    };
                    system.nonlinear.push(NonlinearPlacement {
                        element: id,
                        plus,
                        minus,
                        junction: led,
                        drive: 0.0,
                    });
                    stamped = true;
                }
            }
            if !stamped {
                system.unstamped.push(Unstamped {
                    element: id,
                    reason: UnstampedReason::PinNotConnected,
                });
            }
            continue;
        }

        // A servo is linear within a solve (C-90): its supply a resistance
        // that is lower while the horn is turning, read from what the run
        // carries, and its signal a leak to GND. Each stamped as far as it is
        // wired, so a servo with its supply on and its signal still free holds
        // still and draws its holding current.
        if let ElementKind::Servo = element.kind {
            use crate::solver::device::servo;
            let rows = pin_rows(element, map, &system);
            let state = servo::State::decode(step.and_then(|step| step.previous.get(id).copied()));
            let mut stamped = false;
            if let (Some(supply), Some(ground)) = (rows[servo::SUPPLY], rows[servo::GROUND]) {
                system.stamp_conductance(supply, ground, 1.0 / state.supply_ohms());
                stamped = true;
            }
            if let (Some(signal), Some(ground)) = (rows[servo::SIGNAL], rows[servo::GROUND]) {
                system.stamp_conductance(signal, ground, servo::INPUT_LEAK);
                stamped = true;
            }
            if !stamped {
                system.unstamped.push(Unstamped {
                    element: id,
                    reason: UnstampedReason::PinNotConnected,
                });
            }
            continue;
        }

        // A CD4511 is linear within a solve (C-77): what it latched, read
        // after the last step, decides which outputs are sources towards VDD
        // and which are pulled to VSS, and its inputs take nothing but a leak.
        // It stamps what is wired, and nothing without both supply pins.
        if let ElementKind::Decoder = element.kind {
            use crate::solver::device::decoder;
            let mut rows = [None; 16];
            for (slot, pin) in rows.iter_mut().zip(&element.pins) {
                *slot = pin
                    .net
                    .and_then(|net| map.index_of(net))
                    .map(|node| system.row_of_node(node));
            }
            let (Some(vdd), Some(vss)) = (rows[decoder::VDD], rows[decoder::VSS]) else {
                system.unstamped.push(Unstamped {
                    element: id,
                    reason: UnstampedReason::PinNotConnected,
                });
                continue;
            };
            let state = decoder::State::decode(
                step.and_then(|step| step.previous.get(id))
                    .map_or(0.0, |carried| carried.value),
            );
            let lit = state.segments();
            for segment in 0..7 {
                let Some(out) = rows[decoder::FIRST_SEGMENT + segment] else {
                    continue;
                };
                if lit & (1 << segment) != 0 {
                    // A source a base–emitter drop below VDD, behind its own
                    // resistance: the current from VDD into the output is
                    // G·(V(VDD) − V(out) − drop).
                    let g = 1.0 / decoder::HIGH_OHMS;
                    system.stamp_conductance(vdd, out, g);
                    system.stamp_companion_source(vdd, out, -decoder::HIGH_DROP * g);
                } else {
                    system.stamp_conductance(out, vss, 1.0 / decoder::LOW_OHMS);
                }
            }
            for input in [
                decoder::BITS[0],
                decoder::BITS[1],
                decoder::BITS[2],
                decoder::BITS[3],
                decoder::LAMP_TEST,
                decoder::BLANKING,
                decoder::LATCH_ENABLE,
            ] {
                if let Some(pin) = rows[input] {
                    system.stamp_conductance(pin, vss, decoder::INPUT_LEAK);
                }
            }
            continue;
        }

        // A CD4026 is linear within a solve likewise (C-78): its count, read
        // after the last step, sets each output high or low — a CMOS output
        // either way, the same resistance to VDD or to VSS — and its inputs
        // take nothing but a leak.
        if let ElementKind::Counter = element.kind {
            use crate::solver::device::{cmos, counter};
            let mut rows = [None; 14];
            for (slot, pin) in rows.iter_mut().zip(&element.pins) {
                *slot = pin
                    .net
                    .and_then(|net| map.index_of(net))
                    .map(|node| system.row_of_node(node));
            }
            let (Some(vdd), Some(vss)) = (rows[counter::VDD], rows[counter::VSS]) else {
                system.unstamped.push(Unstamped {
                    element: id,
                    reason: UnstampedReason::PinNotConnected,
                });
                continue;
            };
            let carried = step.and_then(|step| step.previous.get(id));
            let state = counter::State::decode(carried.map_or(0.0, |carried| carried.value));
            let lit = state.segments();
            // At the supply the last step left it on (C-125).
            let supply = carried.map(|carried| carried.held[0]);
            let g = 1.0 / cmos::output_ohms(&counter::OUTPUT_CURVE, supply);
            let mut drive = |pin: usize, high: bool| {
                if let Some(out) = rows[pin] {
                    system.stamp_conductance(out, if high { vdd } else { vss }, g);
                }
            };
            for segment in 0..7 {
                drive(counter::FIRST_SEGMENT + segment, lit & (1 << segment) != 0);
            }
            drive(counter::CARRY, state.carry());
            for input in [
                counter::CLOCK,
                counter::INHIBIT,
                counter::RESET,
                counter::DISPLAY_ENABLE,
            ] {
                if let Some(pin) = rows[input] {
                    system.stamp_conductance(pin, vss, counter::INPUT_LEAK);
                }
            }
            continue;
        }

        // A CD4017 is linear within a solve likewise (C-110): its count, read
        // after the last step, sets the one output that is high, and the
        // carry; its inputs take a leak.
        if let ElementKind::Decade = element.kind {
            use crate::solver::device::{cmos, decade};
            let rows = pin_rows(element, map, &system);
            let (Some(vdd), Some(vss)) = (rows[decade::VDD], rows[decade::VSS]) else {
                system.unstamped.push(Unstamped {
                    element: id,
                    reason: UnstampedReason::PinNotConnected,
                });
                continue;
            };
            let carried = step.and_then(|step| step.previous.get(id));
            let state = decade::State::decode(carried.map_or(0.0, |carried| carried.value));
            // At the supply the last step left it on (C-124).
            let supply = carried.map(|carried| carried.held[0]);
            let g = 1.0 / cmos::output_ohms(&decade::OUTPUT_CURVE, supply);
            let mut drive = |pin: usize, high: bool| {
                if let Some(out) = rows[pin] {
                    system.stamp_conductance(out, if high { vdd } else { vss }, g);
                }
            };
            for (count, &pin) in decade::OUTPUTS.iter().enumerate() {
                drive(pin, usize::from(state.count) == count);
            }
            drive(decade::CARRY, state.carry());
            for &input in &decade::INPUTS {
                if let Some(pin) = rows[input] {
                    system.stamp_conductance(pin, vss, decade::INPUT_LEAK);
                }
            }
            continue;
        }

        // A CD4013 is linear within a solve likewise (C-111): what each
        // flip-flop holds, read after the last step, sets its Q and /Q; its
        // inputs take a leak.
        if let ElementKind::FlipFlop = element.kind {
            use crate::solver::device::{cmos, flipflop};
            let rows = pin_rows(element, map, &system);
            let (Some(vdd), Some(vss)) = (rows[flipflop::VDD], rows[flipflop::VSS]) else {
                system.unstamped.push(Unstamped {
                    element: id,
                    reason: UnstampedReason::PinNotConnected,
                });
                continue;
            };
            let carried = step.and_then(|step| step.previous.get(id));
            let state = carried.map_or(flipflop::resting(), |carried| {
                flipflop::decode(carried.value)
            });
            // At the supply the last step left it on (C-124).
            let supply = carried.map(|carried| carried.held[0]);
            let g = 1.0 / cmos::output_ohms(&flipflop::OUTPUT_CURVE, supply);
            for (half, bit) in flipflop::HALVES.iter().zip(state) {
                for (pin, high) in [(half.q, bit.q), (half.not_q, bit.not_q)] {
                    if let Some(out) = rows[pin] {
                        system.stamp_conductance(out, if high { vdd } else { vss }, g);
                    }
                }
                for input in [half.clock, half.reset, half.data, half.set] {
                    if let Some(pin) = rows[input] {
                        system.stamp_conductance(pin, vss, flipflop::INPUT_LEAK);
                    }
                }
            }
            continue;
        }

        // A 74HC595 is linear within a solve likewise (C-93): what it latched,
        // read after the last step, drives QA to QH high or low, or lets them
        // go while OE is high; QH′ is driven always, from the shift register.
        // Its inputs are a leak.
        if let ElementKind::ShiftRegister = element.kind {
            use crate::solver::device::shift;
            let rows = pin_rows(element, map, &system);
            let (Some(vcc), Some(gnd)) = (rows[shift::VCC], rows[shift::GND]) else {
                system.unstamped.push(Unstamped {
                    element: id,
                    reason: UnstampedReason::PinNotConnected,
                });
                continue;
            };
            let state = shift::State::decode(
                step.and_then(|step| step.previous.get(id))
                    .map_or(0.0, |carried| carried.value),
            );
            let mut drive = |pin: usize, level: Option<bool>, ohms: (f64, f64)| {
                let Some(out) = rows[pin] else {
                    return;
                };
                match level {
                    Some(true) => system.stamp_conductance(out, vcc, 1.0 / ohms.0),
                    Some(false) => system.stamp_conductance(out, gnd, 1.0 / ohms.1),
                    None => system.stamp_conductance(out, gnd, shift::LEAK),
                }
            };
            let shown = state.shown();
            for (bit, &pin) in shift::OUTPUTS.iter().enumerate() {
                let level = shown.map(|byte| byte & (1 << bit) != 0);
                drive(pin, level, (shift::HIGH_OHMS, shift::LOW_OHMS));
            }
            drive(
                shift::CARRY,
                Some(state.carry()),
                (shift::CARRY_HIGH_OHMS, shift::CARRY_LOW_OHMS),
            );
            for &pin in &shift::INPUTS {
                if let Some(input) = rows[pin] {
                    system.stamp_conductance(input, gnd, shift::LEAK);
                }
            }
            continue;
        }

        // A gate package is linear within a solve likewise (C-108): what each
        // output should be, read after the last step, drives it high or low;
        // its inputs are a leak.
        if let Some(chip) = crate::solver::device::gate::Chip::of(&element.kind) {
            use crate::solver::device::gate;
            let rows = pin_rows(element, map, &system);
            let (Some(vcc), Some(gnd)) = (rows[gate::VCC], rows[gate::GND]) else {
                system.unstamped.push(Unstamped {
                    element: id,
                    reason: UnstampedReason::PinNotConnected,
                });
                continue;
            };
            let bits = step
                .and_then(|step| step.previous.get(id))
                .map_or(gate::resting(chip), |carried| gate::decode(carried.value));
            for (index, wiring) in gate::wiring(chip).iter().enumerate() {
                if let Some(out) = rows[wiring.output] {
                    if bits & (1 << index) != 0 {
                        system.stamp_conductance(out, vcc, 1.0 / gate::HIGH_OHMS);
                    } else {
                        system.stamp_conductance(out, gnd, 1.0 / gate::LOW_OHMS);
                    }
                }
                for &pin in wiring.inputs {
                    if let Some(input) = rows[pin] {
                        system.stamp_conductance(input, gnd, gate::LEAK);
                    }
                }
            }
            continue;
        }

        // An A3144 is linear within a solve likewise (C-115): whether it was
        // on after the last step pulls its output low or lets it go; its
        // supply draws through a resistance.
        if let ElementKind::Hall { magnet } = element.kind {
            use crate::solver::device::magnet as hall;
            let rows = pin_rows(element, map, &system);
            let (Some(vcc), Some(gnd)) = (rows[hall::VCC], rows[hall::GND]) else {
                system.unstamped.push(Unstamped {
                    element: id,
                    reason: UnstampedReason::PinNotConnected,
                });
                continue;
            };
            // Before anything is carried, as it would be powered.
            let on = step.and_then(|step| step.previous.get(id)).map_or_else(
                || hall::hall_on(false, magnet, hall::MOST_SUPPLY),
                |carried| carried.value > 0.5,
            );
            system.stamp_conductance(vcc, gnd, 1.0 / hall::SUPPLY_OHMS);
            if let Some(out) = rows[hall::OUT] {
                let g = if on {
                    1.0 / hall::ON_OHMS
                } else {
                    hall::OFF_LEAK
                };
                system.stamp_conductance(out, gnd, g);
            }
            continue;
        }

        // An HC-SR501 is linear within a solve likewise (C-116): high, its
        // output is 3.3 V above its GND behind its resistance, a source and a
        // conductance; low, the same resistance to GND. Its supply draws its
        // standing current.
        if let ElementKind::Pir { .. } = element.kind {
            use crate::solver::device::pir;
            let rows = pin_rows(element, map, &system);
            let (Some(vcc), Some(gnd)) = (rows[pir::VCC], rows[pir::GND]) else {
                system.unstamped.push(Unstamped {
                    element: id,
                    reason: UnstampedReason::PinNotConnected,
                });
                continue;
            };
            let state = pir::State::decode(step.and_then(|step| step.previous.get(id)).copied());
            system.stamp_conductance(vcc, gnd, 1.0 / pir::SUPPLY_OHMS);
            if let Some(out) = rows[pir::OUT] {
                system.stamp_conductance(out, gnd, 1.0 / pir::OUT_OHMS);
                if state.high() {
                    system.stamp_companion_source(gnd, out, pir::HIGH / pir::OUT_OHMS);
                }
            }
            continue;
        }

        // An Uno is linear within a solve likewise (ADR 0017): each pin as
        // the program last left it, against the board's GND — high is 5 V
        // behind the output's resistance, low the resistance alone, a
        // pull-up 5 V behind its own, an input a leak — and its 5 V pin the
        // USB supply behind a little resistance. Each a conductance and,
        // where it drives, a source.
        if let ElementKind::Uno = element.kind {
            use crate::solver::device::uno::{self, Drive};
            let rows = pin_rows(element, map, &system);
            let Some(gnd) = rows[uno::GND] else {
                system.unstamped.push(Unstamped {
                    element: id,
                    reason: UnstampedReason::PinNotConnected,
                });
                continue;
            };
            let drives = step
                .and_then(|step| step.previous.get(id))
                .map_or([Drive::Input; uno::IO], |carried| {
                    uno::decode(carried.value)
                });
            for (pin, row) in rows.iter().enumerate() {
                let Some(at) = *row else { continue };
                for (ohms, volts) in uno::branches(pin, &drives).into_iter().flatten() {
                    system.stamp_conductance(at, gnd, 1.0 / ohms);
                    if volts != 0.0 {
                        system.stamp_companion_source(gnd, at, volts / ohms);
                    }
                }
            }
            // The ADC's sample-and-hold on the analog pin its mux is on
            // (A-04): 14 pF to GND, by backward Euler whatever the run's
            // rule — it is far too small to ring, and carries no slope.
            if let Some(step) = step
                && let Some(carried) = step.previous.get(id)
                && let Some(pin) = uno::sampled_pin(carried.held[1])
                && let Some(at) = rows[pin]
            {
                let g = uno::SAMPLE_HOLD_FARADS / step.seconds;
                system.stamp_conductance(at, gnd, g);
                system.stamp_companion_source(gnd, at, g * carried.held[0]);
            }
            continue;
        }

        // A transformer: each winding its copper at DC, and during a step the
        // two windings' companion — a conductance on each and one across from
        // each to the other, which is the coupling (C-118). A winding left
        // free is stamped as nothing; the other is still an inductor.
        if let ElementKind::Transformer { ratio } = element.kind {
            use crate::solver::device::transformer::{P1, P2, S1, S2, Transformer};
            let rows = pin_rows(element, map, &system);
            let windings = [(rows[P1], rows[P2]), (rows[S1], rows[S2])];
            let wired = windings.map(|(a, b)| a.is_some() && b.is_some());
            let at = windings.map(|(a, b)| (a.flatten(), b.flatten()));
            let transformer = Transformer { ratio };
            match step {
                None => {
                    for (side, copper) in transformer.resistance().into_iter().enumerate() {
                        if wired[side] {
                            system.stamp_conductance(at[side].0, at[side].1, 1.0 / copper);
                        }
                    }
                }
                Some(step) => {
                    let carried = step.previous.get(id).copied().unwrap_or_default();
                    let companion = transformer.companion(step.method, step.seconds, carried);
                    for side in 0..2 {
                        if !wired[side] {
                            continue;
                        }
                        let (plus, minus) = at[side];
                        system.stamp_conductance(plus, minus, companion.conductance[side][side]);
                        system.stamp_companion_source(plus, minus, companion.current[side]);
                        let other = 1 - side;
                        if wired[other] {
                            system.stamp_transconductance(
                                (plus, minus),
                                at[other],
                                companion.conductance[side][other],
                            );
                        }
                    }
                }
            }
            if !wired[0] && !wired[1] {
                system.unstamped.push(Unstamped {
                    element: id,
                    reason: UnstampedReason::PinNotConnected,
                });
            }
            continue;
        }

        // An LM3914 is linear within a solve but for its outputs (C-119): what
        // it read after the last step sets which of them sink and how hard,
        // and Newton holds each to its sink's curve. Its reference is 1.25 V
        // behind its load regulation's resistance, its divider a resistor,
        // its supply its standing current. Nothing stamps without V−.
        if let ElementKind::Lm3914 = element.kind {
            use crate::solver::device::lm3914::{self, State};
            let rows = pin_rows(element, map, &system);
            let Some(minus) = rows[lm3914::V_MINUS] else {
                system.unstamped.push(Unstamped {
                    element: id,
                    reason: UnstampedReason::PinNotConnected,
                });
                continue;
            };
            let state = step
                .and_then(|step| step.previous.get(id))
                .map_or_else(State::default, |&carried| State::decode(carried));
            let plus = rows[lm3914::V_PLUS];
            if let Some(plus) = plus {
                system.stamp_conductance(plus, minus, 1.0 / lm3914::STANDING_OHMS);
            }
            // The reference floats on REF ADJ and is fed from V+: REF OUT is
            // held 1.25 V above REF ADJ, and what it gives — R1's current —
            // comes from the supply and goes on down whatever is below REF
            // ADJ, which is how a resistor there raises the scale.
            if let (Some(out), Some(adj)) = (rows[lm3914::REF_OUT], rows[lm3914::REF_ADJ]) {
                match plus {
                    Some(supply) if state.powered => {
                        let g = 1.0 / lm3914::REFERENCE_OHMS;
                        system.stamp_transconductance((supply, out), (adj, out), g);
                        system.stamp_companion_source(supply, out, lm3914::REFERENCE_VOLTS * g);
                    }
                    _ => system.stamp_conductance(out, adj, lm3914::LEAK),
                }
            }
            if let (true, Some(plus), Some(adj)) = (state.powered, plus, rows[lm3914::REF_ADJ]) {
                system.stamp_companion_source(plus, adj, lm3914::ADJUST_AMPS);
            }
            if let (Some(high), Some(low)) = (rows[lm3914::RHI], rows[lm3914::RLO]) {
                system.stamp_conductance(high, low, 1.0 / lm3914::DIVIDER_OHMS);
            }
            for input in [lm3914::SIG, lm3914::MODE] {
                if let Some(pin) = rows[input] {
                    system.stamp_conductance(pin, minus, lm3914::LEAK);
                }
            }
            for k in 1..=10 {
                if let Some(out) = rows[lm3914::led_pin(k)] {
                    system.nonlinear.push(NonlinearPlacement {
                        element: id,
                        plus: out,
                        minus,
                        junction: k - 1,
                        drive: if state.is_lit(k) { state.amps } else { 0.0 },
                    });
                }
            }
            continue;
        }

        // A bar of ten LEDs: each anode and its own cathode, a junction for
        // Newton when both are wired, dark when not (C-119).
        if let ElementKind::Bargraph = element.kind {
            let rows = pin_rows(element, map, &system);
            let mut stamped = false;
            for led in 0..10 {
                if let (Some(anode), Some(cathode)) = (rows[led], rows[10 + led]) {
                    system.nonlinear.push(NonlinearPlacement {
                        element: id,
                        plus: anode,
                        minus: cathode,
                        junction: led,
                        drive: 0.0,
                    });
                    stamped = true;
                }
            }
            if !stamped {
                system.unstamped.push(Unstamped {
                    element: id,
                    reason: UnstampedReason::PinNotConnected,
                });
            }
            continue;
        }

        // A bridge rectifier is four junctions and nothing else (C-118): from
        // each AC pin up to +, and up to each from −. One with a pin free is
        // the diodes that are still wired.
        if let ElementKind::Bridge = element.kind {
            const AC: [usize; 2] = [0, 1];
            const PLUS: usize = 2;
            const MINUS: usize = 3;
            // What each diode leaks backwards: the sheet's 10 µA at its 1000 V.
            // It also keeps a transformer's secondary, which reaches the rest
            // of its circuit only through these, from floating between pulses.
            const LEAK: f64 = 10e-6 / 1000.0;
            let rows = pin_rows(element, map, &system);
            let mut stamped = false;
            for (index, &ac) in AC.iter().enumerate() {
                let Some(input) = rows[ac] else {
                    continue;
                };
                if let Some(plus) = rows[PLUS] {
                    system.stamp_conductance(input, plus, LEAK);
                    system.nonlinear.push(NonlinearPlacement {
                        element: id,
                        plus: input,
                        minus: plus,
                        junction: 2 * index,
                        drive: 0.0,
                    });
                    stamped = true;
                }
                if let Some(minus) = rows[MINUS] {
                    system.stamp_conductance(minus, input, LEAK);
                    system.nonlinear.push(NonlinearPlacement {
                        element: id,
                        plus: minus,
                        minus: input,
                        junction: 2 * index + 1,
                        drive: 0.0,
                    });
                    stamped = true;
                }
            }
            if !stamped {
                system.unstamped.push(Unstamped {
                    element: id,
                    reason: UnstampedReason::PinNotConnected,
                });
            }
            continue;
        }

        // An L293D is linear within a solve likewise (C-91), but for its clamp
        // diodes: what its inputs read after the last step makes each enabled
        // output a source under VCC2 or a pull to GND, and lets the others go.
        // The diodes, one from each output up to VCC2 and one up to it from
        // GND, are junctions for Newton. Nothing stamps without GND.
        if let ElementKind::MotorDriver = element.kind {
            use crate::solver::device::driver::{self, Drive};
            let rows = pin_rows(element, map, &system);
            let Some(ground) = rows[driver::GND] else {
                system.unstamped.push(Unstamped {
                    element: id,
                    reason: UnstampedReason::PinNotConnected,
                });
                continue;
            };
            let state = driver::State::decode(
                step.and_then(|step| step.previous.get(id))
                    .map_or(0.0, |carried| carried.value),
            );
            let motor = rows[driver::VCC2];
            for (channel, &pin) in driver::OUTPUTS.iter().enumerate() {
                let Some(out) = rows[pin] else {
                    continue;
                };
                match (state.drive(channel), motor) {
                    // A source a drop below VCC2, behind its own resistance:
                    // the current from VCC2 into the output is
                    // G·(V(VCC2) − V(out) − drop).
                    (Drive::High, Some(motor)) => {
                        let g = 1.0 / driver::HIGH_OHMS;
                        system.stamp_conductance(motor, out, g);
                        system.stamp_companion_source(motor, out, -driver::HIGH_DROP * g);
                    }
                    (Drive::Low, _) => {
                        system.stamp_conductance(out, ground, 1.0 / driver::LOW_OHMS);
                    }
                    _ => system.stamp_conductance(out, ground, driver::LEAK),
                }
                if let Some(motor) = motor {
                    system.nonlinear.push(NonlinearPlacement {
                        element: id,
                        plus: out,
                        minus: motor,
                        junction: 2 * channel,
                        drive: 0.0,
                    });
                }
                system.nonlinear.push(NonlinearPlacement {
                    element: id,
                    plus: ground,
                    minus: out,
                    junction: 2 * channel + 1,
                    drive: 0.0,
                });
            }
            if let Some(logic) = rows[driver::VCC1] {
                system.stamp_conductance(logic, ground, 1.0 / driver::LOGIC_OHMS);
            }
            for &pin in driver::INPUTS.iter().chain(&driver::ENABLES) {
                if let Some(input) = rows[pin] {
                    system.stamp_conductance(input, ground, driver::LEAK);
                }
            }
            continue;
        }

        // An LCD digit is the same shape with glass in place of the LEDs: each
        // segment wired, with COM wired too, is its leak to the backplane, and
        // a segment left free sees no field (C-76). Linear, so stamped straight
        // into the matrix.
        if let ElementKind::LcdDigit = element.kind {
            use crate::solver::device::digit::{COMMON, PIN_COUNT, SEGMENTS};
            let mut rows = [None; PIN_COUNT];
            for (slot, pin) in rows.iter_mut().zip(&element.pins) {
                *slot = pin
                    .net
                    .and_then(|net| map.index_of(net))
                    .map(|node| system.row_of_node(node));
            }
            let mut stamped = false;
            if let Some(shared) = rows[COMMON] {
                for own in rows.iter().take(SEGMENTS.len()).flatten() {
                    system.stamp_conductance(
                        *own,
                        shared,
                        1.0 / crate::solver::device::lcd::LEAK_OHMS,
                    );
                    stamped = true;
                }
            }
            if !stamped {
                system.unstamped.push(Unstamped {
                    element: id,
                    reason: UnstampedReason::PinNotConnected,
                });
            }
            continue;
        }

        if let ElementKind::Potentiometer { ohms, wiper } = element.kind {
            if let Some(reason) = stamp_wiper(&mut system, element, map, ohms, wiper) {
                system.unstamped.push(Unstamped {
                    element: id,
                    reason,
                });
            }
            continue;
        }

        // A pin off a net leaves the element spanning nothing.
        let found = pin_rows(element, map, &system);
        let pins = element.pins.len();
        if found[..pins].iter().any(Option::is_none) {
            system.unstamped.push(Unstamped {
                element: id,
                reason: UnstampedReason::PinNotConnected,
            });
            continue;
        }
        let mut wired = [None; MOST_PINS];
        for (slot, row) in wired.iter_mut().zip(&found[..pins]) {
            *slot = row.flatten();
        }
        let rows = &wired[..pins];

        match element.kind {
            // Loads, which to the circuit are resistances (C-66).
            ElementKind::Buzzer => {
                let ohms = crate::solver::device::load::BUZZER_OHMS;
                system.stamp_conductance(rows[0], rows[1], 1.0 / ohms);
            }
            // Its coil, whatever it is being fed (C-85).
            ElementKind::PassiveBuzzer => {
                system.stamp_conductance(
                    rows[0],
                    rows[1],
                    1.0 / crate::solver::device::pitch::OHMS,
                );
            }
            ElementKind::Motor => {
                let ohms = crate::solver::device::load::MOTOR_OHMS;
                system.stamp_conductance(rows[0], rows[1], 1.0 / ohms);
            }
            // Resistors whose value is the world's (C-65).
            ElementKind::Ldr { light } => {
                let ohms = crate::solver::device::sensor::ldr_ohms(light);
                system.stamp_conductance(rows[0], rows[1], 1.0 / ohms);
            }
            ElementKind::Thermistor { celsius } => {
                let ohms = crate::solver::device::sensor::thermistor_ohms(celsius);
                system.stamp_conductance(rows[0], rows[1], 1.0 / ohms);
            }
            ElementKind::Resistor { ohms } => {
                if ohms <= 0.0 || !ohms.is_finite() {
                    system.unstamped.push(Unstamped {
                        element: id,
                        reason: UnstampedReason::DegenerateValue,
                    });
                    continue;
                }
                system.stamp_conductance(rows[0], rows[1], 1.0 / ohms);
            }
            // Handled before the match, because it is stampable in pieces.
            ElementKind::Potentiometer { .. } => unreachable!("stamped above"),
            // A battery is its voltage at the depth it has reached, behind its
            // resistance, which never vanishes: a Norton pair, so it needs no
            // branch of its own (C-121).
            ElementKind::Cell { part, charge } => {
                use crate::solver::device::cell::State;
                let sheet = part.sheet();
                let state = step
                    .and_then(|step| step.previous.get(id))
                    .map_or_else(State::default, |&carried| State::decode(carried));
                let g = 1.0 / sheet.series_ohms();
                system.stamp_conductance(rows[0], rows[1], g);
                system.stamp_companion_source(
                    rows[1],
                    rows[0],
                    sheet.open_volts(state.depth(charge)) * g,
                );
            }
            // A fuse is its resistance while whole, a tripped polyfuse's
            // while tripped, and a gap once blown — the gap a switch is
            // when open (C-120).
            ElementKind::Fuse { amps, part, blown } => {
                use crate::solver::device::fuse::{self, State};
                let Some(model) = fuse::model(part, amps) else {
                    system.unstamped.push(Unstamped {
                        element: id,
                        reason: UnstampedReason::DegenerateValue,
                    });
                    continue;
                };
                let state = step
                    .and_then(|step| step.previous.get(id))
                    .map_or_else(State::default, |&carried| State::decode(carried));
                match model.ohms_in(state).filter(|_| !blown) {
                    Some(ohms) => system.stamp_conductance(rows[0], rows[1], 1.0 / ohms),
                    None => system.unstamped.push(Unstamped {
                        element: id,
                        reason: UnstampedReason::ContactOpen,
                    }),
                }
            }
            ElementKind::Capacitor { farads, .. } => {
                let Some(step) = step else {
                    // Open at DC: no stamp at all, which is the same thing as
                    // an infinite resistance and costs nothing to say.
                    system.unstamped.push(Unstamped {
                        element: id,
                        reason: UnstampedReason::OpenAtDc,
                    });
                    continue;
                };

                if farads <= 0.0 || !farads.is_finite() || step.seconds <= 0.0 {
                    system.unstamped.push(Unstamped {
                        element: id,
                        reason: UnstampedReason::DegenerateValue,
                    });
                    continue;
                }

                let companion = Capacitor { farads }.companion(
                    step.method,
                    step.seconds,
                    step.previous.get(id).copied().unwrap_or_default(),
                );
                system.stamp_conductance(rows[0], rows[1], companion.conductance);
                system.stamp_companion_source(rows[0], rows[1], companion.current);
            }
            // The capacitor and its leak (C-86). The leak is there at DC too,
            // so an electrolytic is never quite open: what charges it can be
            // watched draining away.
            ElementKind::Electrolytic { farads, .. } => {
                if farads <= 0.0
                    || !farads.is_finite()
                    || step.is_some_and(|step| step.seconds <= 0.0)
                {
                    system.unstamped.push(Unstamped {
                        element: id,
                        reason: UnstampedReason::DegenerateValue,
                    });
                    continue;
                }
                let leak = 1.0 / crate::solver::device::electrolytic::leak_ohms(farads);
                system.stamp_conductance(rows[0], rows[1], leak);
                if let Some(step) = step {
                    let companion = Capacitor { farads }.companion(
                        step.method,
                        step.seconds,
                        step.previous.get(id).copied().unwrap_or_default(),
                    );
                    system.stamp_conductance(rows[0], rows[1], companion.conductance);
                    system.stamp_companion_source(rows[0], rows[1], companion.current);
                }
            }
            ElementKind::Inductor { henries, .. } => {
                let Some(step) = step else {
                    // A wire, expressed the only way MNA can express one.
                    let branch = system
                        .row_of_source(id)
                        .expect("every wired inductor was collected above");
                    system.stamp_voltage_source(rows[0], rows[1], branch, 0.0, 0.0);
                    continue;
                };

                if henries <= 0.0 || !henries.is_finite() || step.seconds <= 0.0 {
                    system.unstamped.push(Unstamped {
                        element: id,
                        reason: UnstampedReason::DegenerateValue,
                    });
                    continue;
                }

                let companion = Inductor { henries }.companion(
                    step.method,
                    step.seconds,
                    step.previous.get(id).copied().unwrap_or_default(),
                );
                system.stamp_conductance(rows[0], rows[1], companion.conductance);
                system.stamp_companion_source(rows[0], rows[1], companion.current);
            }
            ElementKind::DcSource { volts, series_ohms } => {
                let branch = system
                    .row_of_source(id)
                    .expect("every stampable DC source was collected above");
                let series = if series_ohms.is_finite() && series_ohms > 0.0 {
                    series_ohms
                } else {
                    0.0
                };
                system.stamp_voltage_source(rows[0], rows[1], branch, volts, series);
            }
            // A voltage source whose value is the time's (ADR 0011). At DC it is
            // read just after zero, which is where a run starts; in a step, just
            // before the step's end, so a step landing on a square wave's edge
            // sees the half that is ending and the next step the one beginning.
            ElementKind::SignalGenerator {
                waveform,
                volts,
                hertz,
            } => {
                let branch = system
                    .row_of_source(id)
                    .expect("every stampable signal generator was collected above");
                let now = match step {
                    Some(step) => waveform.before(volts, hertz, step.at),
                    None => waveform.after(volts, hertz, 0.0),
                };
                system.stamp_voltage_source(rows[0], rows[1], branch, now, 0.0);
            }
            // A contact, shut or open (C-19). Shut it is a piece of wire, which
            // MNA spells as a zero-volt source; open it is a gap, which it
            // spells by saying nothing at all.
            //
            // Ideal both ways, as §5 asks. An on-resistance would keep every
            // matrix well conditioned and would also be a number nobody can
            // justify — and the case it would paper over, a closed switch
            // straight across an ideal battery, is a real short circuit whose
            // honest answer is the unbounded current C-17 and C-25 already have
            // words for.
            ElementKind::Switch { .. }
            | ElementKind::PushButton { .. }
            | ElementKind::TiltSwitch { .. }
            | ElementKind::Reed { .. } => {
                if element.kind.contact() != Some(true) {
                    system.unstamped.push(Unstamped {
                        element: id,
                        reason: UnstampedReason::ContactOpen,
                    });
                    continue;
                }
                let branch = system
                    .row_of_source(id)
                    .expect("every wired closed contact was collected above");
                system.stamp_voltage_source(rows[0], rows[1], branch, 0.0, 0.0);
            }
            // An ammeter is the same piece of wire a shut contact is, and the
            // branch unknown a zero-volt source needs is exactly the number the
            // instrument exists to report (C-21). Measuring a current by
            // inserting a wire and asking what goes through it is also what a
            // real one does.
            ElementKind::Ammeter => {
                let branch = system
                    .row_of_source(id)
                    .expect("every wired ammeter was collected above");
                system.stamp_voltage_source(rows[0], rows[1], branch, 0.0, 0.0);
            }
            // A voltmeter is an open circuit, so it is not stamped — and
            // because it is not stamped, putting one across a component cannot
            // change what that component is doing.
            ElementKind::Voltmeter => system.unstamped.push(Unstamped {
                element: id,
                reason: UnstampedReason::DrawsNoCurrent,
            }),
            // Nonlinear: the linear system carries no contribution from it, and
            // Newton stamps it about the operating point on every iteration.
            ElementKind::Led { .. }
            | ElementKind::Diode
            | ElementKind::Schottky
            | ElementKind::Zener { .. }
            | ElementKind::Phototransistor { .. }
            | ElementKind::SolarCell { .. } => system.nonlinear.push(NonlinearPlacement {
                element: id,
                plus: rows[0],
                minus: rows[1],
                junction: 0,
                drive: 0.0,
            }),
            // Nonlinear and coupled: Newton stamps it about a moving operating
            // point every iteration, across all its terminals at once.
            //
            // An op-amp is the same: its five pins are the saturating
            // source's five terminals, in the same order (C-55). So is a
            // comparator, which has the op-amp's pins (C-56).
            ElementKind::Bjt { .. }
            | ElementKind::Mosfet { .. }
            | ElementKind::OpAmp
            | ElementKind::Comparator
            | ElementKind::Timer
            | ElementKind::Optocoupler => {
                let mut terminals = [None; MAX_TERMINALS];
                for (index, row) in rows.iter().take(MAX_TERMINALS).enumerate() {
                    terminals[index] = *row;
                }
                let latched = matches!(element.kind, ElementKind::Timer)
                    && step
                        .and_then(|step| step.previous.get(id))
                        .is_some_and(|carried| carried.latched);
                system.coupled.push(CoupledPlacement {
                    element: id,
                    rows: terminals,
                    latched,
                });
            }
            // Coupled too, through the saturating source it is modelled as
            // (C-53). Three pins, five terminals: the rows go to the terminals
            // in the source's order, each on the pin the part's wiring says,
            // so the solver sees an op-amp shape whatever the part is.
            ElementKind::Regulator { .. } | ElementKind::Adjustable | ElementKind::Tmp36 { .. } => {
                // An LM317 is a regulator (C-113). A TMP36 is wired the
                // regulator's way: a fixed target above
                // its GND, its supply the high rail (C-87).
                let wiring = if matches!(element.kind, ElementKind::Tmp36 { .. }) {
                    crate::solver::device::tmp36::WIRING
                } else {
                    crate::solver::device::regulator::WIRING
                };
                let mut terminals = [None; MAX_TERMINALS];
                for (terminal, &pin) in wiring.iter().enumerate() {
                    terminals[terminal] = rows[pin];
                }
                system.coupled.push(CoupledPlacement {
                    element: id,
                    rows: terminals,
                    latched: false,
                });
            }
            // Stamped above, before its pins were required to be wired.
            ElementKind::Relay { .. }
            | ElementKind::SevenSegment { .. }
            | ElementKind::RgbLed { .. }
            | ElementKind::LcdDigit
            | ElementKind::Decoder
            | ElementKind::Servo
            | ElementKind::Counter
            | ElementKind::MotorDriver
            | ElementKind::ShiftRegister
            | ElementKind::Gate { .. }
            | ElementKind::Inverter
            | ElementKind::Schmitt
            | ElementKind::Decade
            | ElementKind::FlipFlop
            | ElementKind::Hall { .. }
            | ElementKind::Pir { .. }
            | ElementKind::Uno
            | ElementKind::Transformer { .. }
            | ElementKind::Bridge
            | ElementKind::Lm3914
            | ElementKind::Bargraph => {
                unreachable!("stamped above")
            }
            // Ground carries no current of its own; it only named the reference.
            ElementKind::Ground => {}
        }
    }

    system
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::netlist::{ElementKind, LedColour, NetId, Netlist};

    const R_TOP: f64 = 1_000.0;
    const R_BOTTOM: f64 = 2_000.0;
    const SUPPLY: f64 = 9.0;

    struct Divider {
        netlist: Netlist,
        source: ElementId,
        top: NetId,
        middle: NetId,
    }

    /// A 9 V source across two resistors: 1 kΩ from the top node to the middle,
    /// 2 kΩ from the middle to the reference.
    fn divider() -> Divider {
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts: SUPPLY,
            series_ohms: 0.0,
        });
        let r_top = netlist.add_element(ElementKind::Resistor { ohms: R_TOP });
        let r_bottom = netlist.add_element(ElementKind::Resistor { ohms: R_BOTTOM });

        let top = netlist.add_net();
        let middle = netlist.add_net();
        let bottom = netlist.add_net();

        netlist.connect_named(source, "+", top).unwrap();
        netlist.connect_named(r_top, "a", top).unwrap();
        netlist.connect_named(r_top, "b", middle).unwrap();
        netlist.connect_named(r_bottom, "a", middle).unwrap();
        netlist.connect_named(r_bottom, "b", bottom).unwrap();
        netlist.connect_named(source, "-", bottom).unwrap();

        Divider {
            netlist,
            source,
            top,
            middle,
        }
    }

    #[test]
    fn a_divider_stamps_the_hand_computed_matrix() {
        let Divider {
            netlist,
            source,
            top,
            middle,
        } = divider();
        let map = netlist.compile().unwrap();
        let system = assemble(&netlist, &map);

        // Unknowns: v(top), v(middle), i(source). The reference node has no row.
        assert_eq!(system.size(), 3);

        let t = system.row_of_node(map.index_of(top).unwrap()).unwrap();
        let m = system.row_of_node(map.index_of(middle).unwrap()).unwrap();
        let i = system.row_of_source(source).unwrap();

        let g_top = 1.0 / R_TOP; // 1e-3
        let g_bottom = 1.0 / R_BOTTOM; // 5e-4

        // Worked out by hand from the two KCL equations and the source branch:
        //
        //   [  g_top          -g_top             1 ] [ v_top    ]   [ 0      ]
        //   [ -g_top   g_top + g_bottom          0 ] [ v_middle ] = [ 0      ]
        //   [      1                0            0 ] [ i_source ]   [ SUPPLY ]
        assert_eq!(system.at(t, t), g_top);
        assert_eq!(system.at(t, m), -g_top);
        assert_eq!(system.at(m, t), -g_top);
        assert_eq!(system.at(m, m), g_top + g_bottom);

        assert_eq!(system.at(t, i), 1.0);
        assert_eq!(system.at(i, t), 1.0);
        assert_eq!(system.at(m, i), 0.0);
        assert_eq!(system.at(i, m), 0.0);
        assert_eq!(system.at(i, i), 0.0);

        assert_eq!(system.z(), &[0.0, 0.0, SUPPLY][..]);
        assert!(system.unstamped().is_empty());
    }

    #[test]
    fn the_conductance_block_is_symmetric() {
        let Divider { netlist, .. } = divider();
        let map = netlist.compile().unwrap();
        let system = assemble(&netlist, &map);

        // Resistors alone give a symmetric G; losing that means a stamp went in
        // one direction only.
        for row in 0..2 {
            for col in 0..2 {
                assert_eq!(system.at(row, col), system.at(col, row));
            }
        }
    }

    #[test]
    fn the_reference_node_has_no_row() {
        let Divider { netlist, .. } = divider();
        let map = netlist.compile().unwrap();
        let system = assemble(&netlist, &map);

        assert_eq!(system.row_of_node(REFERENCE_NODE), None);
        // Three nets, one of them the reference, plus one source branch.
        assert_eq!(map.len(), 3);
        assert_eq!(system.size(), 3);
    }

    #[test]
    fn a_resistor_to_the_reference_only_lands_on_its_diagonal() {
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts: SUPPLY,
            series_ohms: 0.0,
        });
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: R_TOP });
        let top = netlist.add_net();
        let bottom = netlist.add_net();

        netlist.connect_named(source, "+", top).unwrap();
        netlist.connect_named(resistor, "a", top).unwrap();
        netlist.connect_named(resistor, "b", bottom).unwrap();
        netlist.connect_named(source, "-", bottom).unwrap();

        let map = netlist.compile().unwrap();
        let system = assemble(&netlist, &map);

        let t = system.row_of_node(map.index_of(top).unwrap()).unwrap();
        assert_eq!(system.at(t, t), 1.0 / R_TOP);
        assert_eq!(system.size(), 2);
    }

    #[test]
    fn a_half_wired_element_is_reported_not_swallowed() {
        let Divider { mut netlist, .. } = divider();
        let dangling = netlist.add_element(ElementKind::Resistor { ohms: 470.0 });
        let net = netlist.add_net();
        netlist.connect_named(dangling, "a", net).unwrap();

        let map = netlist.compile().unwrap();
        let system = assemble(&netlist, &map);

        // The live loop keeps solving while the user is mid-wire, but the app
        // can still say what it left out.
        assert_eq!(
            system.unstamped(),
            &[Unstamped {
                element: dangling,
                reason: UnstampedReason::PinNotConnected
            }][..]
        );
    }

    #[test]
    fn a_zero_ohm_resistor_is_refused_rather_than_stamped_as_infinity() {
        let Divider {
            mut netlist, top, ..
        } = divider();
        let short = netlist.add_element(ElementKind::Resistor { ohms: 0.0 });
        let other = netlist.add_net();
        netlist.connect_named(short, "a", top).unwrap();
        netlist.connect_named(short, "b", other).unwrap();

        let map = netlist.compile().unwrap();
        let system = assemble(&netlist, &map);

        assert_eq!(system.unstamped().len(), 1);
        assert_eq!(
            system.unstamped()[0].reason,
            UnstampedReason::DegenerateValue
        );
        assert!(system.z().iter().all(|value| value.is_finite()));
        for row in 0..system.size() {
            for col in 0..system.size() {
                assert!(system.at(row, col).is_finite());
            }
        }
    }

    #[test]
    fn a_nonlinear_element_is_placed_rather_than_skipped() {
        let Divider {
            mut netlist,
            middle,
            ..
        } = divider();
        let led = netlist.add_element(ElementKind::Led {
            colour: LedColour::Red,
        });
        let other = netlist.add_net();
        netlist.connect_named(led, "anode", middle).unwrap();
        netlist.connect_named(led, "cathode", other).unwrap();

        let map = netlist.compile().unwrap();
        let system = assemble(&netlist, &map);

        // Nothing linear to contribute, but it is not an omission either: Newton
        // restamps it about the operating point on every iteration.
        assert!(system.unstamped().is_empty());
        assert_eq!(system.nonlinear().len(), 1);
        assert_eq!(system.nonlinear()[0].element, led);
        assert_eq!(
            system.nonlinear()[0].plus,
            system.row_of_node(map.index_of(middle).unwrap())
        );
    }
}
