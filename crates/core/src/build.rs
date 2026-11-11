// SPDX-License-Identifier: MIT OR Apache-2.0

//! How to build it, one step at a time (B-15, design doc §6.3).
//!
//! "This is the feature, not the decoration": the board view (B-14) shows
//! where everything goes; these say what to push in next, in words a person
//! with a part in one hand can follow, and which holes to look at.
//!
//! **The order** is the one beginners are taught: chips first, because they
//! need both hands and a clear board; then the jumper wires, flat on the board
//! where the parts will cover them; then the parts with three legs or more,
//! then those with two; **the battery last**, so nothing is live while hands
//! are on the board; meters at the end, to look with. Within each, the order
//! the parts were placed in.
//!
//! **Each step says the thing a beginner gets wrong** about that part, where
//! there is one: which way a chip goes (its notch), an LED's long leg, a
//! diode's band, an electrolytic's stripe, which leg of a transistor is which.
//! And a step the schematic does not have: a 555's RESET tied to +, which the
//! drawing leaves out and a real one needs (see [`crate::breadboard::ties`]).

use crate::breadboard::{self, Hole, Placement, Strip};
use crate::netlist::{ElementId, ElementKind, Netlist};

/// A part on the board: which, what it is, and each leg's hole by pin index.
type Placed = (ElementId, ElementKind, Vec<(usize, Hole)>);

/// One step: what to do, the holes it is about, and the parts.
#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    pub text: String,
    pub holes: Vec<Hole>,
    pub elements: Vec<ElementId>,
}

/// The steps that build `placement` of `netlist`. `label` says a part's value
/// as the schematic writes it beside its symbol: "470 Ω", "Red", "NE555".
pub fn steps(
    netlist: &Netlist,
    placement: &Placement,
    label: impl Fn(&ElementKind) -> String,
) -> Vec<Step> {
    // Each part's legs, by pin index, as placed.
    let legs_of = |id: ElementId| {
        let mut legs: Vec<(usize, Hole)> = placement
            .wiring
            .pins
            .iter()
            .filter(|(pin, _)| pin.element == id)
            .map(|&(pin, hole)| (pin.index, hole))
            .collect();
        legs.sort_by_key(|&(index, _)| index);
        legs
    };
    let parts: Vec<Placed> = netlist
        .elements()
        .map(|(id, element)| (id, element.kind, legs_of(id)))
        .filter(|(_, _, legs)| !legs.is_empty())
        .collect();

    let mut steps = Vec::new();

    // Chips, and the rows their tied pins are in.
    let mut tied: Vec<(Strip, String)> = Vec::new();
    for (id, kind, legs) in &parts {
        let Some((count, _)) = breadboard::dip(kind) else {
            continue;
        };
        // Pin 1 and the last pin face each other across the channel in the
        // chip's first row.
        let first = legs
            .iter()
            .filter_map(|&(_, hole)| match hole {
                Hole::Row { row, .. } => Some(row),
                Hole::Rail { .. } => None,
            })
            .min()
            .unwrap_or(1);
        let half = u16::from(count / 2);
        let name = label(kind);
        let pin_one = Hole::Row {
            row: first,
            column: 4,
        };
        let pin_last = Hole::Row {
            row: first,
            column: 5,
        };
        steps.push(Step {
            text: format!(
                "Push the {name} across the channel, its notch towards row {first}: pin 1 in {}, pin {count} in {}.",
                pin_one.name(),
                pin_last.name()
            ),
            holes: legs.iter().map(|&(_, hole)| hole).collect(),
            elements: vec![*id],
        });
        for &(pin, pin_name, to) in breadboard::ties(kind) {
            let n = u16::from(pin);
            let hole = if n <= half {
                Hole::Row {
                    row: first + n - 1,
                    column: 4,
                }
            } else {
                Hole::Row {
                    row: first + 2 * half - n,
                    column: 5,
                }
            };
            let to = netlist
                .element(*id)
                .and_then(|element| element.pins.get(to))
                .map_or("", |pin| pin.name);
            tied.push((
                hole.strip(),
                format!("the {name}'s {pin_name} (pin {pin}) to {to}"),
            ));
        }
    }

    // Jumpers.
    for &(from, to) in &placement.wiring.jumpers {
        let why = tied
            .iter()
            .find(|(strip, _)| *strip == from.strip() || *strip == to.strip())
            .map(|(_, pin)| {
                format!(
                    " It ties {pin}: the drawing leaves that pin out, and a real one with it left free may not run."
                )
            })
            .unwrap_or_default();
        steps.push(Step {
            text: format!("A jumper wire from {} to {}.{why}", from.name(), to.name()),
            holes: vec![from, to],
            elements: Vec::new(),
        });
    }

    // Parts with three legs or more.
    for (id, kind, legs) in &parts {
        if breadboard::dip(kind).is_some() || breadboard::off_board(kind) {
            continue;
        }
        let Some(element) = netlist.element(*id) else {
            continue;
        };
        if element.pins.len() < 3 {
            continue;
        }
        let said: Vec<String> = legs
            .iter()
            .map(|&(index, hole)| {
                format!(
                    "{} in {}",
                    leg_name(kind, element.pins[index].name),
                    hole.name()
                )
            })
            .collect();
        steps.push(Step {
            text: format!(
                "Put the {} in: {}.",
                noun(kind, &label(kind)),
                said.join(", ")
            ),
            holes: legs.iter().map(|&(_, hole)| hole).collect(),
            elements: vec![*id],
        });
    }

    // Parts with two, a meter in the circuit among them.
    for (id, kind, legs) in &parts {
        if breadboard::off_board(kind) || matches!(kind, ElementKind::Voltmeter) {
            continue;
        }
        let Some(element) = netlist.element(*id) else {
            continue;
        };
        if element.pins.len() > 2 {
            continue;
        }
        let at = |index: usize| {
            legs.iter()
                .find(|&&(other, _)| other == index)
                .map(|&(_, hole)| hole.name())
        };
        let (Some(first), Some(second)) = (at(0), at(1)) else {
            // One leg on the board: say where that one goes.
            let (index, hole) = legs[0];
            steps.push(Step {
                text: format!(
                    "Put one leg of the {} in {}; its {} is not wired yet.",
                    noun(kind, &label(kind)),
                    hole.name(),
                    element.pins[1 - index.min(1)].name
                ),
                holes: vec![hole],
                elements: vec![*id],
            });
            continue;
        };
        let name = noun(kind, &label(kind));
        let text = match kind {
            ElementKind::Led { .. } => {
                format!(
                    "Put the {name}'s long leg in {first} and its short leg, by the flat side, in {second}."
                )
            }
            ElementKind::Diode | ElementKind::Schottky | ElementKind::Zener { .. } => {
                format!("Put the {name}'s plain end in {first} and its banded end in {second}.")
            }
            ElementKind::Electrolytic { .. } => format!(
                "Put the {name}'s long leg (+) in {first} and the leg by the stripe (−) in {second}."
            ),
            ElementKind::Ammeter => format!(
                "Put the meter into the circuit, set to measure current: red probe in {first}, black in {second}."
            ),
            _ if element.pins[0].name == "+" => {
                format!("Put the {name}'s + leg in {first} and its − leg in {second}.")
            }
            ElementKind::Phototransistor { .. } => {
                format!("Put the {name}'s collector in {first} and its emitter in {second}.")
            }
            _ => format!("Put the {name} from {first} to {second}."),
        };
        steps.push(Step {
            text,
            holes: legs.iter().map(|&(_, hole)| hole).collect(),
            elements: vec![*id],
        });
    }

    // Power, last.
    for (id, kind, legs) in &parts {
        if !breadboard::off_board(kind) {
            continue;
        }
        let at = |index: usize| {
            legs.iter()
                .find(|&&(other, _)| other == index)
                .map_or_else(|| "nothing".to_string(), |&(_, hole)| hole.name())
        };
        let text = match kind {
            ElementKind::SignalGenerator { .. } => format!(
                "Clip on the signal generator, {}: its + to {} and its − to {}.",
                label(kind),
                at(0),
                at(1)
            ),
            _ => format!(
                "Last, connect the {} battery: + to the {}, − to the {}.",
                label(kind),
                at(0).trim_start_matches("the "),
                at(1).trim_start_matches("the ")
            ),
        };
        steps.push(Step {
            text,
            holes: legs.iter().map(|&(_, hole)| hole).collect(),
            elements: vec![*id],
        });
    }

    // Meters, to look with.
    for (id, kind, legs) in &parts {
        if !matches!(kind, ElementKind::Voltmeter) {
            continue;
        }
        let at = |index: usize| {
            legs.iter()
                .find(|&&(other, _)| other == index)
                .map_or_else(|| "nothing".to_string(), |&(_, hole)| hole.name())
        };
        steps.push(Step {
            text: format!(
                "To read the voltage, touch the meter's red probe to {} and its black to {}.",
                at(0),
                at(1)
            ),
            holes: legs.iter().map(|&(_, hole)| hole).collect(),
            elements: vec![*id],
        });
    }

    // A hole on a row is named with its row; say "the" before a rail.
    for step in &mut steps {
        step.text = step
            .text
            .replace(" to top", " to the top")
            .replace(" to bottom", " to the bottom")
            .replace(" in top", " in the top")
            .replace(" in bottom", " in the bottom")
            .replace(" from top", " from the top")
            .replace(" from bottom", " from the bottom")
            .replace("the the ", "the ");
    }
    steps
}

/// What a part is called in a step: its value and what it is.
fn noun(kind: &ElementKind, label: &str) -> String {
    match kind {
        ElementKind::Resistor { .. } => format!("{label} resistor"),
        ElementKind::Capacitor { .. } => format!("{label} capacitor"),
        ElementKind::Electrolytic { .. } => format!("{label} electrolytic"),
        ElementKind::Inductor { .. } => format!("{label} inductor"),
        ElementKind::Led { .. } => format!("{} LED", label.to_lowercase()),
        // By the number printed on it, which is how its legs are looked up.
        ElementKind::Bjt { polarity } => polarity.part_number().to_string(),
        ElementKind::Mosfet { channel } => channel.part_number().to_string(),
        ElementKind::Potentiometer { .. } => format!("{label} potentiometer"),
        ElementKind::Diode => "diode".to_string(),
        ElementKind::Schottky => "Schottky diode".to_string(),
        ElementKind::Zener { .. } => format!("{label} Zener"),
        ElementKind::Switch { .. } => "switch".to_string(),
        ElementKind::PushButton { .. } => "push-button".to_string(),
        ElementKind::Fuse { .. } => "fuse".to_string(),
        ElementKind::Buzzer => "buzzer".to_string(),
        ElementKind::Motor => "motor".to_string(),
        ElementKind::Ldr { .. } => "LDR".to_string(),
        ElementKind::Thermistor { .. } => "thermistor".to_string(),
        ElementKind::Phototransistor { .. } => "phototransistor".to_string(),
        _ if label.is_empty() => kind.name().to_lowercase(),
        _ => label.to_string(),
    }
}

/// A leg, named as a person finds it on the part.
fn leg_name(kind: &ElementKind, pin: &str) -> String {
    let named = match (kind, pin) {
        (ElementKind::Bjt { .. }, "b") => "base",
        (ElementKind::Bjt { .. }, "c") => "collector",
        (ElementKind::Bjt { .. }, "e") => "emitter",
        (ElementKind::Mosfet { .. }, "g") => "gate",
        (ElementKind::Mosfet { .. }, "d") => "drain",
        (ElementKind::Mosfet { .. }, "s") => "source",
        (ElementKind::Potentiometer { .. }, "wiper") => "middle leg",
        (ElementKind::Potentiometer { .. }, "a") => "one end",
        (ElementKind::Potentiometer { .. }, "b") => "the other end",
        (ElementKind::Regulator { .. } | ElementKind::Adjustable, "in") => "IN",
        (ElementKind::Regulator { .. }, "gnd") => "GND",
        (ElementKind::Adjustable, "adj") => "ADJ",
        (ElementKind::Regulator { .. } | ElementKind::Adjustable, "out") => "OUT",
        _ => pin,
    };
    named.to_string()
}

#[cfg(test)]
mod tests {
    use crate::circuit::Circuit;

    fn steps_of(text: &str) -> Vec<String> {
        let mut circuit = Circuit::new();
        circuit.try_open(text).unwrap();
        circuit
            .try_breadboard()
            .unwrap()
            .steps
            .into_iter()
            .map(|step| step.text)
            .collect()
    }

    #[test]
    fn the_milestone_is_built_in_order_and_powered_last() {
        let steps = steps_of(include_str!("../../../app/src/examples/milestone.json"));
        // Jumpers to the rails, the resistor, the LED, the battery last.
        assert!(steps[0].starts_with("A jumper wire from"), "{steps:#?}");
        let resistor = steps
            .iter()
            .position(|step| step.contains("470 Ω resistor"))
            .unwrap();
        let led = steps
            .iter()
            .position(|step| step.contains("red LED's long leg"))
            .unwrap();
        let battery = steps
            .iter()
            .position(|step| step.starts_with("Last, connect the 9 V battery"))
            .unwrap();
        assert!(resistor < led && led < battery, "{steps:#?}");
        assert_eq!(battery, steps.len() - 1, "{steps:#?}");
        assert!(steps[battery].contains("top + rail"), "{}", steps[battery]);
    }

    #[test]
    fn a_chip_says_which_way_round_and_its_reset_is_tied() {
        let steps = steps_of(include_str!("../../../app/src/examples/astable.json"));
        assert!(
            steps[0].starts_with("Push the NE555 across the channel, its notch towards row"),
            "{steps:#?}"
        );
        assert!(
            steps
                .iter()
                .any(|step| step.contains("ties the NE555's RESET (pin 4) to VCC")),
            "{steps:#?}"
        );
    }

    #[test]
    fn a_transistor_says_which_leg_is_which() {
        let steps = steps_of(include_str!("../../../app/src/examples/transistor.json"));
        let step = steps.iter().find(|step| step.contains("2N3904")).unwrap();
        for leg in ["base in", "collector in", "emitter in"] {
            assert!(step.contains(leg), "{step}");
        }
    }

    #[test]
    fn every_example_has_a_step_for_every_part_and_every_jumper() {
        for entry in std::fs::read_dir("../../app/src/examples").unwrap() {
            let path = entry.unwrap().path();
            let mut circuit = Circuit::new();
            circuit
                .try_open(&std::fs::read_to_string(&path).unwrap())
                .unwrap();
            let view = circuit.try_breadboard().unwrap();
            let named = path.display();
            // Each part on the board is in a step, each jumper is one.
            for part in &view.parts {
                assert!(
                    view.steps
                        .iter()
                        .any(|step| step.elements.contains(&part.element_id)),
                    "{named}: {} has no step",
                    part.kind
                );
            }
            let jumpers = view
                .steps
                .iter()
                .filter(|step| step.text.starts_with("A jumper wire"))
                .count();
            assert_eq!(jumpers, view.jumpers.len(), "{named}");
            // Every step says where, in words a person reads.
            assert!(
                view.steps.iter().all(|step| !step.holes.is_empty()),
                "{named}"
            );
        }
    }
}
