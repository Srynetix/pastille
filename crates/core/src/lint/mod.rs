// SPDX-License-Identifier: MIT OR Apache-2.0

//! The design linter's engine (B-01, design doc §6.1).
//!
//! Rules run on every solution, over the netlist and the solver's latest
//! state, and each says something about a part or says nothing. What they say
//! is a [`Finding`]: how much it matters, a sentence a beginner can act on,
//! the parts it is about, and, where there is one, the fix.
//!
//! The rules themselves are in `rules.rs`, one function each and one list
//! that orders them, so adding a rule touches that file alone. This one holds
//! what every rule shares: the [`Context`] it reads — including the one piece
//! of analysis that is the circuit's rather than a part's, which circuits are
//! alternating — and the loop that runs them.
//!
//! They were written into `Circuit::review` one at a time from C-32 on, each
//! because a part needed it before this engine existed; the shape they
//! produce is the one C-32 chose, extended rather than replaced.

mod rules;

use serde::{Deserialize, Serialize};
use slotmap::{Key, SecondaryMap, SparseSecondaryMap};
use tsify::Tsify;

use crate::engine::Engine;
use crate::netlist::{Element, ElementId, ElementKind, NetId, Netlist};

/// How much a finding matters.
///
/// Neither is an error: the circuit is the user's to get wrong, and the app's
/// part is to say so (§14's reactive help).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Something that will not work, or will not last.
    Warning,
    /// Something worth knowing, which works as the part's sheet says it does.
    Note,
}

impl PartialEq<&str> for Severity {
    fn eq(&self, other: &&str) -> bool {
        matches!(
            (self, *other),
            (Severity::Warning, "warning") | (Severity::Note, "note")
        )
    }
}

/// What would make a finding go, when there is one thing that would.
///
/// For the app to offer as a button beside the finding (C-135), so each one
/// is an edit the app already knows how to make. A rule offers one only when
/// it is the fix rather than a fix: "the LED is the wrong way round — or the
/// battery is" has two, and offers neither.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Fix {
    /// Give a part this value, written as the Inspector would take it.
    SetValue { element: String, value: String },
    /// Put a resistor of this value in series with a part, written as the
    /// Inspector would take it (B-02): what an LED with nothing to limit its
    /// current needs.
    AddResistor { element: String, value: String },
}

/// Something the app noticed and should say.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    /// Which rule said it, by the name in `rules.rs`: stable, for a test to
    /// name and for the app to tell two findings about one part apart.
    pub rule: String,
    pub severity: Severity,
    pub message: String,
    /// Handles of the elements this is about, so they can be pointed at.
    pub elements: Vec<String>,
    /// The one edit that would make it go, if there is one.
    pub fix: Option<Fix>,
}

/// What a rule says about one part; the engine adds the rule's name and the
/// part's handle.
pub(crate) struct Said {
    pub severity: Severity,
    pub message: String,
    pub fix: Option<Fix>,
}

impl Said {
    pub fn warning(message: impl Into<String>) -> Self {
        Said {
            severity: Severity::Warning,
            message: message.into(),
            fix: None,
        }
    }

    pub fn note(message: impl Into<String>) -> Self {
        Said {
            severity: Severity::Note,
            message: message.into(),
            fix: None,
        }
    }
}

/// A rule: a name, and what it says about a part — usually nothing.
pub(crate) struct Rule {
    pub name: &'static str,
    pub check: Check,
}

/// What a rule needs to read.
pub(crate) enum Check {
    /// The solved circuit: voltages, and what each part's model carries.
    Solved(fn(&Context, ElementId, &Element) -> Option<Said>),
    /// The wiring alone. Such a rule also runs when the circuit did not solve
    /// — which is when the commonest mistake of all, an LED straight across a
    /// battery, leaves the solver nothing to bound (B-02).
    Wiring(fn(&Netlist, ElementId, &Element) -> Option<Said>),
}

/// The current each part carries at the solved point, as the solution reads
/// it for the canvas (B-03).
///
/// Worked out where the dots and the arrows are, and handed over rather than
/// worked out again: what the rule holds to a rating and what the canvas
/// shows flowing are then the same number.
#[derive(Default)]
pub struct Flows {
    /// A two-pin part's, into its first pin and out of its second.
    through: SparseSecondaryMap<ElementId, f64>,
    /// A part with more pins, into each of them.
    into: SecondaryMap<ElementId, Vec<f64>>,
}

impl Flows {
    pub fn new(
        through: impl IntoIterator<Item = (ElementId, f64)>,
        into: SecondaryMap<ElementId, Vec<f64>>,
    ) -> Self {
        Flows {
            through: through.into_iter().collect(),
            into,
        }
    }
}

/// What every rule reads: the circuit, its solution, and what has been worked
/// out about it once rather than by each rule.
pub(crate) struct Context<'a> {
    pub netlist: &'a Netlist,
    pub engine: &'a dyn Engine,
    flows: &'a Flows,
    circuit_of: SparseSecondaryMap<NetId, usize>,
    swinging: Vec<bool>,
}

impl<'a> Context<'a> {
    fn new(netlist: &'a Netlist, engine: &'a dyn Engine, flows: &'a Flows) -> Self {
        // On an alternating source an LED is reversed for half of every cycle,
        // and that is what the circuit is for, not a mistake in it: two LEDs
        // taking turns would be told one of them is backwards, whichever the
        // moment. "The wrong way round" only means something against a supply
        // that has a right way (ADR 0011).
        //
        // Judged circuit by circuit (C-102): a signal generator in one
        // experiment says nothing about a battery in the one beside it. And a
        // part that drives a pin both high and low — an L293D's outputs, a
        // 74HC595's — swings an LED between two of them both ways on purpose.
        let circuits = netlist.circuits();
        let mut circuit_of: SparseSecondaryMap<NetId, usize> = SparseSecondaryMap::new();
        for (index, nets) in circuits.iter().enumerate() {
            for &net in nets {
                circuit_of.insert(net, index);
            }
        }
        let mut swinging = vec![false; circuits.len()];
        for (_, element) in netlist.elements() {
            if matches!(
                element.kind,
                ElementKind::SignalGenerator { .. }
                    | ElementKind::MotorDriver
                    | ElementKind::ShiftRegister
                    | ElementKind::Gate { .. }
                    | ElementKind::Inverter
                    | ElementKind::Schmitt
                    | ElementKind::Decade
                    | ElementKind::FlipFlop
                    | ElementKind::Hall { .. }
                    | ElementKind::Pir { .. }
                    // A program drives its pins high and low as it likes
                    // (A-10).
                    | ElementKind::Uno
                    // Nothing but a changing current crosses one, so both
                    // its sides are alternating (C-118).
                    | ElementKind::Transformer { .. }
            ) {
                for net in element.pins.iter().filter_map(|pin| pin.net) {
                    if let Some(&index) = circuit_of.get(net) {
                        swinging[index] = true;
                    }
                }
            }
        }
        Context {
            netlist,
            engine,
            flows,
            circuit_of,
            swinging,
        }
    }

    /// Whether the circuit this net is in swings both ways on purpose.
    pub fn alternating(&self, net: Option<NetId>) -> bool {
        net.and_then(|net| self.circuit_of.get(net))
            .is_some_and(|&index| self.swinging[index])
    }

    /// The voltage on a part's pin, by index, if it is wired to a solved net.
    pub fn pin_volts(&self, element: &Element, pin: usize) -> Option<f64> {
        self.engine.node_voltage(element.pins.get(pin)?.net?)
    }

    /// The current through a two-pin part, into its first pin.
    pub fn through(&self, id: ElementId) -> Option<f64> {
        self.flows.through.get(id).copied()
    }

    /// The current into each pin of a part with more than two.
    pub fn pin_currents(&self, id: ElementId) -> Option<&[f64]> {
        self.flows.into.get(id).map(Vec::as_slice)
    }

    /// The voltage on a part's pin, by name.
    pub fn named_volts(&self, element: &Element, name: &str) -> Option<f64> {
        self.pin_volts(element, element.pin_index(name)?)
    }
}

fn encode(id: ElementId) -> String {
    id.data().as_ffi().to_string()
}

/// Whether there is anything to have an opinion about. An empty breadboard,
/// or one being built, has nothing wrong with it yet, and a list of parts
/// "with no power" would be the app scolding somebody for not having finished.
/// An Uno is a source: USB powers it (A-10).
fn has_source(netlist: &Netlist) -> bool {
    netlist.elements().any(|(_, element)| {
        matches!(
            element.kind,
            ElementKind::DcSource { .. }
                | ElementKind::SignalGenerator { .. }
                | ElementKind::Cell { .. }
                | ElementKind::Uno
        )
    })
}

/// Every rule over every part, in the order `rules.rs` lists them, for a
/// circuit that solved.
pub fn review(netlist: &Netlist, engine: &dyn Engine, flows: &Flows) -> Vec<Finding> {
    if !has_source(netlist) {
        return Vec::new();
    }
    let context = Context::new(netlist, engine, flows);
    run(netlist, |check, id, element| match check {
        Check::Solved(check) => check(&context, id, element),
        Check::Wiring(check) => check(netlist, id, element),
    })
}

/// The rules that read the wiring alone, for a circuit that did not solve:
/// what can still be said about it, and often why it did not.
pub fn review_wiring(netlist: &Netlist) -> Vec<Finding> {
    if !has_source(netlist) {
        return Vec::new();
    }
    run(netlist, |check, id, element| match check {
        Check::Solved(_) => None,
        Check::Wiring(check) => check(netlist, id, element),
    })
}

fn run(
    netlist: &Netlist,
    check: impl Fn(&Check, ElementId, &Element) -> Option<Said>,
) -> Vec<Finding> {
    rules::RULES
        .iter()
        .flat_map(|rule| {
            let check = &check;
            netlist.elements().filter_map(move |(id, element)| {
                let said = check(&rule.check, id, element)?;
                Some(Finding {
                    rule: rule.name.to_string(),
                    severity: said.severity,
                    message: said.message,
                    elements: vec![encode(id)],
                    fix: said.fix,
                })
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_rule_has_a_name_of_its_own() {
        let mut names: Vec<_> = rules::RULES.iter().map(|rule| rule.name).collect();
        let count = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), count, "two rules share a name");
        assert!(
            names
                .iter()
                .all(|name| !name.is_empty() && !name.contains(' '))
        );
    }

    #[test]
    fn severity_is_written_as_the_app_reads_it() {
        assert_eq!(
            serde_json::to_string(&Severity::Warning).unwrap(),
            "\"warning\""
        );
        assert_eq!(serde_json::to_string(&Severity::Note).unwrap(), "\"note\"");
        assert_eq!(Severity::Note, "note");
    }

    #[test]
    fn a_fix_says_what_kind_it_is() {
        let fix = Fix::SetValue {
            element: "1".into(),
            value: "390".into(),
        };
        assert_eq!(
            serde_json::to_string(&fix).unwrap(),
            r#"{"kind":"setValue","element":"1","value":"390"}"#
        );
    }
}
