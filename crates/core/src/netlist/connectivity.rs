// SPDX-License-Identifier: MIT OR Apache-2.0

//! What is joined to what, and what is not.
//!
//! C-05. Facts about a netlist's shape, with no opinion about any of them: a
//! dangling wire is reported, not complained about. Turning these into
//! something a user is told is B-01's rules engine, and keeping the two apart
//! is what lets a rule change its mind about severity or wording without the
//! analysis changing at all.
//!
//! Every query is over the netlist alone. None of them needs a solution, which
//! is the point — a circuit that cannot be solved is exactly when the linter
//! has the most to say, and C-25's failures are the cases where the solver had
//! nothing to offer.

use super::{ElementId, ElementKind, NetId, Netlist, PinRef};

/// Everything the analysis found.
///
/// Empty vectors rather than an `Option` or a bool: "nothing is floating" and
/// "nothing was checked" should not look the same to a rule, and a rule that
/// wants a count should not have to ask twice.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Connectivity {
    /// Pins joined to nothing at all.
    ///
    /// Every pin of a component just dropped on the canvas is one, so this is
    /// a fact about a circuit being unfinished rather than wrong.
    pub unconnected_pins: Vec<PinRef>,
    /// Nets joining fewer than two pins: a wire with one end loose.
    ///
    /// Distinct from a floating net, and more definite. This is a wire that
    /// reaches nothing; a floating one may be perfectly well wired and simply
    /// have nothing behind it.
    pub dangling_nets: Vec<NetId>,
    /// Nets in a part of the canvas where nothing defines a potential.
    ///
    /// No source and no ground anywhere among the elements they join, so every
    /// voltage there is measured from a zero the solver picked itself (ADR
    /// 0003, and the `Arbitrary` origin C-35 added). Legal, solvable, and
    /// almost always half a circuit.
    pub floating_nets: Vec<NetId>,
    /// Sources with both terminals on the same net: a short through wire alone.
    ///
    /// The one short this can be certain about. A source shorted through a
    /// path that happens to contain no resistance is a question about values
    /// rather than about connectivity, and belongs to a rule that can see
    /// them (B-03).
    pub shorted_sources: Vec<ElementId>,
}

impl Connectivity {
    /// Whether the netlist is joined up well enough to be worth solving.
    pub fn is_clean(&self) -> bool {
        self.unconnected_pins.is_empty()
            && self.dangling_nets.is_empty()
            && self.floating_nets.is_empty()
            && self.shorted_sources.is_empty()
    }
}

impl Netlist {
    /// Looks at what is joined to what.
    pub fn connectivity(&self) -> Connectivity {
        Connectivity {
            unconnected_pins: self.unconnected_pins(),
            dangling_nets: self.dangling_nets(),
            floating_nets: self.floating_nets(),
            shorted_sources: self.shorted_sources(),
        }
    }

    fn unconnected_pins(&self) -> Vec<PinRef> {
        self.elements()
            .flat_map(|(id, element)| {
                element
                    .pins
                    .iter()
                    .enumerate()
                    .filter(|(_, pin)| pin.net.is_none())
                    .map(move |(index, _)| PinRef { element: id, index })
            })
            .collect()
    }

    fn dangling_nets(&self) -> Vec<NetId> {
        self.nets()
            .filter(|(_, net)| net.is_floating())
            .map(|(id, _)| id)
            .collect()
    }

    fn floating_nets(&self) -> Vec<NetId> {
        // The same grouping the compiler uses to give each circuit on the
        // canvas its own reference, asked a different question.
        self.circuits()
            .into_iter()
            .filter(|circuit| !self.circuit_defines_a_potential(circuit))
            .flatten()
            .collect()
    }

    /// Whether anything in this part of the canvas says what zero volts means.
    fn circuit_defines_a_potential(&self, circuit: &[NetId]) -> bool {
        circuit.iter().any(|&id| {
            self.net(id).is_some_and(|net| {
                net.pins().iter().any(|pin| {
                    self.element(pin.element).is_some_and(|element| {
                        matches!(
                            element.kind,
                            ElementKind::DcSource { .. } | ElementKind::Ground
                        )
                    })
                })
            })
        })
    }

    fn shorted_sources(&self) -> Vec<ElementId> {
        self.elements()
            .filter(|(_, element)| matches!(element.kind, ElementKind::DcSource { .. }))
            .filter(|(_, element)| {
                let nets: Vec<NetId> = element.pins.iter().filter_map(|pin| pin.net).collect();
                nets.len() == element.pins.len() && nets.iter().all(|&net| net == nets[0])
            })
            .map(|(id, _)| id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::netlist::LedColour;

    /// The milestone circuit: battery, resistor, LED, wired into a loop.
    fn milestone() -> Netlist {
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
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
        netlist.connect_named(source, "+", top).unwrap();
        netlist.connect_named(resistor, "a", top).unwrap();
        netlist.connect_named(resistor, "b", middle).unwrap();
        netlist.connect_named(led, "anode", middle).unwrap();
        netlist.connect_named(led, "cathode", bottom).unwrap();
        netlist.connect_named(source, "-", bottom).unwrap();
        netlist
    }

    #[test]
    fn a_finished_circuit_has_nothing_to_report() {
        assert!(milestone().connectivity().is_clean());
    }

    #[test]
    fn a_component_just_dropped_has_both_pins_loose() {
        let mut netlist = Netlist::new();
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: 220.0 });

        let found = netlist.connectivity();

        assert_eq!(
            found.unconnected_pins,
            vec![
                PinRef {
                    element: resistor,
                    index: 0
                },
                PinRef {
                    element: resistor,
                    index: 1
                },
            ]
        );
    }

    #[test]
    fn a_wire_with_one_end_loose_is_dangling() {
        let mut netlist = Netlist::new();
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: 220.0 });
        let stub = netlist.add_net();
        netlist.connect_named(resistor, "a", stub).unwrap();

        assert_eq!(netlist.connectivity().dangling_nets, vec![stub]);
    }

    #[test]
    fn two_components_with_nothing_driving_them_are_floating() {
        // Legal, solvable — C-35 gives this its own zero — and almost always
        // half a circuit.
        let mut netlist = Netlist::new();
        let one = netlist.add_element(ElementKind::Resistor { ohms: 220.0 });
        let two = netlist.add_element(ElementKind::Resistor { ohms: 470.0 });
        let left = netlist.add_net();
        let right = netlist.add_net();
        netlist.connect_named(one, "a", left).unwrap();
        netlist.connect_named(two, "a", left).unwrap();
        netlist.connect_named(one, "b", right).unwrap();
        netlist.connect_named(two, "b", right).unwrap();

        let found = netlist.connectivity();

        assert_eq!(found.floating_nets.len(), 2);
        assert!(found.dangling_nets.is_empty(), "both nets join two pins");
    }

    #[test]
    fn a_battery_makes_its_own_part_of_the_canvas_solid() {
        assert!(milestone().connectivity().floating_nets.is_empty());
    }

    #[test]
    fn a_ground_is_enough_on_its_own() {
        // It defines a potential without driving anything, which is the whole
        // of what a ground is for (ADR 0003).
        let mut netlist = Netlist::new();
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: 220.0 });
        let ground = netlist.add_element(ElementKind::Ground);
        let left = netlist.add_net();
        let right = netlist.add_net();
        netlist.connect_named(resistor, "a", left).unwrap();
        netlist.connect_named(ground, "gnd", left).unwrap();
        netlist.connect_named(resistor, "b", right).unwrap();
        netlist.connect_named(ground, "gnd", right).unwrap_err();

        assert!(netlist.connectivity().floating_nets.is_empty());
    }

    #[test]
    fn one_half_of_a_canvas_can_float_while_the_other_does_not() {
        // Two experiments side by side, one of them unfinished. The finished
        // one should not be reported.
        let mut netlist = milestone();
        let stray = netlist.add_element(ElementKind::Resistor { ohms: 1_000.0 });
        let other = netlist.add_element(ElementKind::Resistor { ohms: 1_000.0 });
        let left = netlist.add_net();
        let right = netlist.add_net();
        netlist.connect_named(stray, "a", left).unwrap();
        netlist.connect_named(other, "a", left).unwrap();
        netlist.connect_named(stray, "b", right).unwrap();
        netlist.connect_named(other, "b", right).unwrap();

        let found = netlist.connectivity();

        assert_eq!(found.floating_nets.len(), 2, "only the stray pair");
    }

    #[test]
    fn a_battery_wired_to_itself_is_shorted() {
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts: 9.0,
            series_ohms: 0.0,
        });
        let net = netlist.add_net();
        netlist.connect_named(source, "+", net).unwrap();
        netlist.connect_named(source, "-", net).unwrap();

        assert_eq!(netlist.connectivity().shorted_sources, vec![source]);
    }

    #[test]
    fn a_battery_with_a_load_is_not_shorted() {
        assert!(milestone().connectivity().shorted_sources.is_empty());
    }

    #[test]
    fn a_battery_with_one_terminal_loose_is_not_shorted() {
        // Unfinished, not wrong. Reporting it as a short would be the app
        // guessing at what somebody is about to do.
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts: 9.0,
            series_ohms: 0.0,
        });
        let net = netlist.add_net();
        netlist.connect_named(source, "+", net).unwrap();

        assert!(netlist.connectivity().shorted_sources.is_empty());
    }

    #[test]
    fn an_empty_netlist_reports_nothing_rather_than_everything() {
        assert!(Netlist::new().connectivity().is_clean());
    }
}
