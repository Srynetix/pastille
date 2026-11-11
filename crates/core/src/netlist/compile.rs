// SPDX-License-Identifier: MIT OR Apache-2.0

//! Turning a netlist into the node-indexed form the solver wants.
//!
//! The netlist is what the user edits: elements joined by nets. The solver wants
//! numbered nodes, each measured from a reference at zero volts. This module is
//! the translation, and it is the only place that decides what "ground" means
//! (see `docs/adrs/0003-implicit-ground.md`).
//!
//! A canvas can hold more than one circuit. Two experiments side by side share
//! no wire, so a potential in one says nothing about a potential in the other,
//! and each needs a reference of its own — one reference for the whole canvas
//! leaves the other circuits floating and the matrix singular.

use slotmap::{SecondaryMap, SparseSecondaryMap};

use super::{ElementKind, NetId, Netlist, Node};

/// Index of the reference node. Not a magic zero: MNA drops this row and
/// column from the matrix, so the choice is structural.
///
/// Every circuit on the canvas shares it. They are not the same node — but
/// nothing joins them, so no current can flow between them and each is free to
/// call its own reference zero.
pub const REFERENCE_NODE: u32 = 0;

/// Why a netlist could not be compiled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    /// There is nothing to solve: no wire joins anything.
    NoReferenceNode,
}

impl core::fmt::Display for CompileError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CompileError::NoReferenceNode => {
                write!(f, "nothing is wired up yet")
            }
        }
    }
}

impl core::error::Error for CompileError {}

/// How a reference was arrived at.
///
/// Worth keeping, because an implicitly chosen ground is exactly the kind of
/// thing the linter should mention once the circuit misbehaves (C-05, C-25).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroundOrigin {
    /// The user placed a ground symbol.
    Explicit,
    /// Taken from the negative terminal of a DC source, because no ground was
    /// placed.
    ImpliedBySource,
    /// Nothing in this part of the canvas defines a potential, so one net was
    /// picked to be zero. Everything here is measured from it and nothing is
    /// driving it.
    Arbitrary,
}

/// One circuit's reference node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reference {
    pub net: NetId,
    pub origin: GroundOrigin,
}

/// A netlist numbered for the solver.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeMap {
    nodes: Vec<Node>,
    index_of_net: SecondaryMap<NetId, u32>,
    references: Vec<Reference>,
}

impl NodeMap {
    /// Every node, references first.
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// Node index of a net, or `None` if the net carries no pins and so is not
    /// a node at all.
    pub fn index_of(&self, net: NetId) -> Option<u32> {
        self.index_of_net.get(net).copied()
    }

    /// One per circuit on the canvas.
    pub fn references(&self) -> &[Reference] {
        &self.references
    }

    /// Node count including the references. The MNA matrix is smaller than
    /// this by the number of references, since those rows are dropped.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl Netlist {
    /// Numbers the nets for the solver, with one reference per circuit.
    ///
    /// Nets with no pins are skipped: a wire joined to nothing is not a node.
    pub fn compile(&self) -> Result<NodeMap, CompileError> {
        let circuits = self.circuits();
        if circuits.is_empty() {
            return Err(CompileError::NoReferenceNode);
        }

        let mut nodes = Vec::new();
        let mut index_of_net = SecondaryMap::new();
        let mut references = Vec::new();

        // Every reference first, all sharing index 0.
        for circuit in &circuits {
            let reference = self.pick_reference(circuit);
            references.push(reference);
            index_of_net.insert(reference.net, REFERENCE_NODE);
            nodes.push(Node {
                index: REFERENCE_NODE,
                net: reference.net,
            });
        }

        // Then everything else, numbered from one.
        for circuit in &circuits {
            for &net in circuit {
                if index_of_net.contains_key(net) {
                    continue;
                }
                let index = (nodes.len() - references.len() + 1) as u32;
                index_of_net.insert(net, index);
                nodes.push(Node { index, net });
            }
        }

        Ok(NodeMap {
            nodes,
            index_of_net,
            references,
        })
    }

    /// The nets of the netlist, grouped into the circuits they form.
    ///
    /// Two nets are in the same circuit when an element joins them. Nets with
    /// no pins belong to no circuit: a wire joined to nothing is not a node.
    pub(crate) fn circuits(&self) -> Vec<Vec<NetId>> {
        let mut neighbours: SparseSecondaryMap<NetId, Vec<NetId>> = SparseSecondaryMap::new();
        for (id, net) in self.nets() {
            if !net.pins().is_empty() {
                neighbours.entry(id).unwrap().or_default();
            }
        }

        // A part joins its nets — side by side, for a part whose sides
        // nothing conducts between (C-92).
        for (_, element) in self.elements() {
            let all: Vec<usize> = (0..element.pins.len()).collect();
            let sides: Vec<&[usize]> = match element.kind.sides() {
                Some(sides) => sides.to_vec(),
                None => vec![all.as_slice()],
            };
            for side in sides {
                let nets: Vec<NetId> = side
                    .iter()
                    .filter_map(|&pin| element.pins.get(pin)?.net)
                    .collect();
                for &left in &nets {
                    for &right in &nets {
                        if left != right
                            && let Some(list) = neighbours.get_mut(left)
                        {
                            list.push(right);
                        }
                    }
                }
            }
        }

        let mut seen: SparseSecondaryMap<NetId, ()> = SparseSecondaryMap::new();
        let mut circuits = Vec::new();

        for (start, _) in self.nets() {
            if !neighbours.contains_key(start) || seen.contains_key(start) {
                continue;
            }

            let mut circuit = Vec::new();
            let mut queue = vec![start];
            seen.insert(start, ());

            while let Some(net) = queue.pop() {
                circuit.push(net);
                for &next in neighbours.get(net).into_iter().flatten() {
                    if seen.insert(next, ()).is_none() {
                        queue.push(next);
                    }
                }
            }

            circuits.push(circuit);
        }

        circuits
    }

    /// Picks the reference for one circuit.
    ///
    /// An explicit ground wins. Failing that, the negative terminal of a DC
    /// source, or an Uno's GND — a beginner does not know what a ground is, and refusing to
    /// simulate until they place one teaches nothing (ADR 0003). Failing that,
    /// any net will do: nothing here defines a potential, so one is chosen and
    /// everything is measured from it.
    fn pick_reference(&self, circuit: &[NetId]) -> Reference {
        let mut source_negative = None;

        for (_, element) in self.elements() {
            let on_this_circuit =
                |net: Option<NetId>| net.is_some_and(|net| circuit.contains(&net));

            match element.kind {
                ElementKind::Ground => {
                    if let Some(net) = element.pins[0].net
                        && on_this_circuit(Some(net))
                    {
                        return Reference {
                            net,
                            origin: GroundOrigin::Explicit,
                        };
                    }
                }
                ElementKind::DcSource { .. } | ElementKind::SignalGenerator { .. }
                    if source_negative.is_none() =>
                {
                    let negative = element
                        .pin_index("-")
                        .and_then(|index| element.pins[index].net)
                        .filter(|&net| on_this_circuit(Some(net)));
                    source_negative = negative;
                }
                // An Uno is a supply as a battery is — USB powers it, and its
                // pins drive from it — so its GND is the circuit's 0 V (A-03):
                // a pin high reads 5 V, not some other net's −5 V.
                ElementKind::Uno if source_negative.is_none() => {
                    source_negative = element
                        .pin_index("GND")
                        .and_then(|index| element.pins[index].net)
                        .filter(|&net| on_this_circuit(Some(net)));
                }
                _ => {}
            }
        }

        match source_negative {
            Some(net) => Reference {
                net,
                origin: GroundOrigin::ImpliedBySource,
            },
            None => Reference {
                net: circuit[0],
                origin: GroundOrigin::Arbitrary,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::netlist::{ElementKind, LedColour, Netlist};

    struct Loop {
        netlist: Netlist,
        top: NetId,
        bottom: NetId,
    }

    /// Battery, resistor and LED in a loop, with no ground symbol anywhere.
    fn loop_without_ground() -> Loop {
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

        Loop {
            netlist,
            top,
            bottom,
        }
    }

    /// Adds a second, separate battery-and-resistor circuit.
    fn add_second_circuit(netlist: &mut Netlist) -> NetId {
        let battery = netlist.add_element(ElementKind::DcSource {
            volts: 5.0,
            series_ohms: 0.0,
        });
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: 1_000.0 });
        let top = netlist.add_net();
        let bottom = netlist.add_net();
        netlist.connect_named(battery, "+", top).unwrap();
        netlist.connect_named(resistor, "a", top).unwrap();
        netlist.connect_named(resistor, "b", bottom).unwrap();
        netlist.connect_named(battery, "-", bottom).unwrap();
        top
    }

    #[test]
    fn numbers_every_net_with_one_reference() {
        let Loop {
            netlist, bottom, ..
        } = loop_without_ground();

        let map = netlist.compile().unwrap();

        assert_eq!(map.len(), 3);
        assert_eq!(map.index_of(bottom), Some(REFERENCE_NODE));
        assert_eq!(map.references().len(), 1);
        assert_eq!(map.references()[0].net, bottom);
    }

    #[test]
    fn a_circuit_with_no_ground_symbol_still_solves() {
        let Loop {
            netlist, bottom, ..
        } = loop_without_ground();

        let map = netlist.compile().unwrap();

        assert_eq!(map.references()[0].origin, GroundOrigin::ImpliedBySource);
        assert_eq!(map.references()[0].net, bottom);
    }

    #[test]
    fn an_explicit_ground_wins_over_the_source() {
        let Loop {
            mut netlist,
            top,
            bottom,
        } = loop_without_ground();

        let ground = netlist.add_element(ElementKind::Ground);
        netlist.connect_named(ground, "gnd", top).unwrap();

        let map = netlist.compile().unwrap();

        assert_eq!(map.references()[0].origin, GroundOrigin::Explicit);
        assert_eq!(map.index_of(top), Some(REFERENCE_NODE));
        assert_ne!(map.index_of(bottom), Some(REFERENCE_NODE));
    }

    #[test]
    fn two_circuits_joined_by_an_optocoupler_each_get_a_reference() {
        // Its LED across one loop and its transistor across another: nothing
        // conducts across, so they are still two circuits, each its own zero
        // (C-92).
        let Loop {
            mut netlist,
            top,
            bottom,
        } = loop_without_ground();
        let battery = netlist.add_element(ElementKind::DcSource {
            volts: 5.0,
            series_ohms: 0.0,
        });
        let (high, low) = (netlist.add_net(), netlist.add_net());
        netlist.connect_named(battery, "+", high).unwrap();
        netlist.connect_named(battery, "-", low).unwrap();
        let opto = netlist.add_element(ElementKind::Optocoupler);
        netlist.connect_named(opto, "A", top).unwrap();
        netlist.connect_named(opto, "K", bottom).unwrap();
        netlist.connect_named(opto, "C", high).unwrap();
        netlist.connect_named(opto, "E", low).unwrap();

        let map = netlist.compile().unwrap();

        assert_eq!(map.references().len(), 2);
        assert_eq!(map.index_of(low), Some(REFERENCE_NODE));
        assert_eq!(map.index_of(bottom), Some(REFERENCE_NODE));
    }

    #[test]
    fn two_circuits_side_by_side_each_get_a_reference() {
        // The whole point of the ticket: a canvas holding two experiments is a
        // normal thing to build, and one reference for both leaves the second
        // floating and the matrix singular.
        let Loop { mut netlist, .. } = loop_without_ground();
        let second_top = add_second_circuit(&mut netlist);

        let map = netlist.compile().unwrap();

        assert_eq!(map.references().len(), 2);
        // Five nets, two of them references.
        assert_eq!(map.len(), 5);
        assert_ne!(map.index_of(second_top), Some(REFERENCE_NODE));

        // Every non-reference node has a distinct index, or two circuits would
        // be sharing a row of the matrix.
        let mut indices: Vec<u32> = map
            .nodes()
            .iter()
            .map(|node| node.index)
            .filter(|index| *index != REFERENCE_NODE)
            .collect();
        indices.sort_unstable();
        indices.dedup();
        assert_eq!(indices.len(), 3);
    }

    #[test]
    fn a_circuit_with_nothing_driving_it_picks_its_own_zero() {
        // No source and no ground: nothing defines a potential here, so one net
        // is chosen and everything is measured from it.
        let mut netlist = Netlist::new();
        let left = netlist.add_element(ElementKind::Resistor { ohms: 220.0 });
        let right = netlist.add_element(ElementKind::Resistor { ohms: 470.0 });
        let net = netlist.add_net();
        netlist.connect_named(left, "a", net).unwrap();
        netlist.connect_named(right, "a", net).unwrap();

        let map = netlist.compile().unwrap();

        assert_eq!(map.references().len(), 1);
        assert_eq!(map.references()[0].origin, GroundOrigin::Arbitrary);
    }

    #[test]
    fn a_netlist_with_nothing_wired_has_nothing_to_compile() {
        let mut netlist = Netlist::new();
        netlist.add_element(ElementKind::Resistor { ohms: 220.0 });
        netlist.add_net();

        assert_eq!(
            netlist.compile().unwrap_err(),
            CompileError::NoReferenceNode
        );
    }

    #[test]
    fn a_net_joining_nothing_is_not_a_node() {
        let Loop { mut netlist, .. } = loop_without_ground();
        let drawing = netlist.add_net();

        let map = netlist.compile().unwrap();

        assert_eq!(map.len(), 3);
        assert_eq!(map.index_of(drawing), None);
    }
}
