// SPDX-License-Identifier: MIT OR Apache-2.0

//! Which wires a part keeps when it is replaced by another kind (C-137).
//!
//! A resistor that should have been a potentiometer, an NPN that should be a
//! MOSFET: swapped in place, each pin that has a counterpart on the new part
//! keeps its wires, and the rest are let go and named. **Never by position**:
//! a pin is carried only to one that means the same, which is
//!
//! 1. **the same name, meaning the same** — `in+`, `VCC`, a gate's `1A`, a
//!    digit's `dp`;
//! 2. **the same role**, from the table in [`role`]: a two-pin part's first
//!    and second ends (a resistor's `a`, an electrolytic's `+`, a diode's
//!    `anode`), a transistor's control, top and bottom (a BJT's b-c-e, a
//!    MOSFET's g-d-s), a supply, a ground, an output, whatever each part
//!    calls it. A part with a way round now has one, and the app says so.
//!
//! Two parts with nothing in common — a resistor for a 555 — keep nothing,
//! and say so, rather than guess.

use crate::netlist::ElementKind;

/// What a pin is for, on this kind of part, across the names different
/// parts give it — `None` for a pin that is only itself (a digit's segment,
/// an op-amp's input), which keeps its wire only to a pin of its own name
/// that is not something else either.
///
/// A name alone is not enough: a BJT's `b` is its base, a resistor's `b` is
/// its second end.
fn role(kind: &ElementKind, name: &str) -> Option<&'static str> {
    let ends = kind.pin_names().len() == 2 || matches!(kind, ElementKind::Potentiometer { .. });
    let transistor = matches!(
        kind,
        ElementKind::Bjt { .. } | ElementKind::Mosfet { .. } | ElementKind::Phototransistor { .. }
    );
    Some(match name {
        // A two-pin part's ends, and a pot's: the first and the second, a
        // way round for the parts that have one.
        "a" | "+" | "anode" if ends => "first end",
        "b" | "-" | "cathode" if ends => "second end",
        // A transistor's: what steers it, the end current comes in by, the
        // end it leaves by (an NPN's and an N-channel's way round).
        "b" | "g" if transistor => "control",
        "c" | "d" if transistor => "top",
        "e" | "s" if transistor => "bottom",
        // A part's supply, its ground and its output, as a sensor or a chip
        // names them.
        "VCC" | "VDD" | "V+" | "+Vs" | "+5V" => "supply",
        "GND" | "VSS" | "V-" | "gnd" => "ground",
        "OUT" | "Vout" => "output",
        _ => return None,
    })
}

/// One way two pins can mean the same.
type Rule<'a> = dyn Fn(&str, &str) -> bool + 'a;

/// Each of `from`'s pins that has a counterpart on `to`, as `(from, to)` pin
/// indices; each pin of `to` taken at most once, by name (and role) before
/// role alone.
pub fn carry(from: &ElementKind, to: &ElementKind) -> Vec<(usize, usize)> {
    let old = from.pin_names();
    let new = to.pin_names();
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    let same_name = |a: &str, b: &str| a == b && role(from, a) == role(to, b);
    let same_role = |a: &str, b: &str| role(from, a).is_some() && role(from, a) == role(to, b);
    let rules: [&Rule; 2] = [&same_name, &same_role];
    for rule in rules {
        for (i, a) in old.iter().enumerate() {
            if pairs.iter().any(|&(from, _)| from == i) {
                continue;
            }
            if let Some(j) = new
                .iter()
                .enumerate()
                .position(|(j, b)| rule(a, b) && !pairs.iter().any(|&(_, to)| to == j))
            {
                pairs.push((i, j));
            }
        }
    }
    pairs.sort_unstable();
    pairs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blank(kind: &str) -> ElementKind {
        (crate::registry::definition(kind).unwrap().blank)()
    }

    /// The pairs as pin names, for reading.
    fn named(from: &str, to: &str) -> Vec<(&'static str, &'static str)> {
        let (from, to) = (blank(from), blank(to));
        carry(&from, &to)
            .into_iter()
            .map(|(i, j)| (from.pin_names()[i], to.pin_names()[j]))
            .collect()
    }

    #[test]
    fn a_pin_of_the_same_name_keeps_its_wire() {
        assert_eq!(named("Resistor", "Potentiometer"), [("a", "a"), ("b", "b")]);
        assert_eq!(
            named("Diode", "Led"),
            [("anode", "anode"), ("cathode", "cathode")]
        );
        assert_eq!(
            named("OpAmp", "Comparator"),
            [
                ("in+", "in+"),
                ("in-", "in-"),
                ("out", "out"),
                ("V+", "V+"),
                ("V-", "V-")
            ]
        );
    }

    #[test]
    fn a_transistor_keeps_its_wires_by_what_each_pin_does() {
        assert_eq!(named("Bjt", "Mosfet"), [("b", "g"), ("c", "d"), ("e", "s")]);
        // A phototransistor has no base: its light is.
        assert_eq!(named("Bjt", "Phototransistor"), [("c", "c"), ("e", "e")]);
    }

    #[test]
    fn two_pin_parts_keep_their_ends_and_a_way_round_is_given() {
        assert_eq!(named("Capacitor", "Electrolytic"), [("a", "+"), ("b", "-")]);
        assert_eq!(named("Resistor", "Led"), [("a", "anode"), ("b", "cathode")]);
    }

    #[test]
    fn a_sensor_keeps_its_supply_ground_and_output() {
        assert_eq!(
            named("Hall", "Tmp36"),
            [("VCC", "+Vs"), ("GND", "GND"), ("OUT", "Vout")]
        );
        // The adjustable regulator's ADJ is not a ground: it is let go.
        assert_eq!(
            named("Regulator", "Adjustable"),
            [("in", "in"), ("out", "out")]
        );
    }

    #[test]
    fn parts_with_nothing_in_common_keep_nothing() {
        // A 555's VCC and GND are a supply and a ground; a resistor has neither.
        assert!(named("Resistor", "Timer").is_empty());
        // A pot's ends keep theirs; its wiper has nowhere to go.
        assert_eq!(
            named("Potentiometer", "Capacitor"),
            [("a", "a"), ("b", "b")]
        );
        assert!(named("Bjt", "Resistor").is_empty());
        // A digit's `a` is a segment, a resistor's an end.
        assert!(named("SevenSegment", "Resistor").is_empty());
        // Between two digits, every segment keeps its wire.
        assert_eq!(named("SevenSegment", "LcdDigit").len(), 9);
    }
}
