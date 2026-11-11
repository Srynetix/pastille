// SPDX-License-Identifier: MIT OR Apache-2.0

//! The circuit in other forms: a KiCad schematic, a breadboard layout, a
//! bill of materials, what to buy, what to swap, and the steps to build it.

use super::*;

impl Circuit {
    /// [`Circuit::kicad_netlist`] without the boundary's wrapper.
    pub fn kicad(&self) -> KicadView {
        let name = self
            .description
            .lines()
            .next()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .unwrap_or("Pastille circuit");
        match crate::kicad::netlist(&self.netlist, label_for, name) {
            Ok(text) => KicadView {
                netlist: Some(text),
                schematic: crate::kicad::schematic(&self.netlist, label_for, name).ok(),
                unmapped: Vec::new(),
            },
            Err(unmapped) => KicadView {
                netlist: None,
                schematic: None,
                unmapped: unmapped.kinds.into_iter().map(str::to_string).collect(),
            },
        }
    }

    /// The parts someone has moved, for the placer.
    pub(super) fn pinned(&self) -> Vec<breadboard::Pinned> {
        self.board
            .iter()
            .filter(|(id, _)| self.netlist.element(*id).is_some())
            .map(|(id, holes)| (id, holes.clone()))
            .collect()
    }

    pub(super) fn placed(&self) -> Result<breadboard::Placement, CircuitError> {
        breadboard::place_with(&self.netlist, &self.pinned()).map_err(|_| {
            CircuitError::Refused(
                "This circuit has more parts than a full breadboard holds.".to_string(),
            )
        })
    }

    /// [`Circuit::breadboard`] without the boundary's error type.
    pub fn try_breadboard(&self) -> Result<BoardView, CircuitError> {
        let placement = self.placed()?;
        let board = placement.board;
        let hole_view = |hole: Hole| {
            let (x, y) = board.position(hole).unwrap_or((0.0, 0.0));
            BoardHole {
                name: hole.name(),
                x,
                y,
                strip: strip_key(hole.strip()),
            }
        };
        let holes = board.holes().map(hole_view).collect();
        let mut parts: Vec<BoardPart> = Vec::new();
        for (id, element) in self.netlist.elements() {
            let legs: Vec<BoardLeg> = placement
                .wiring
                .pins
                .iter()
                .filter(|(pin, _)| pin.element == id)
                .map(|&(pin, hole)| {
                    let view = hole_view(hole);
                    BoardLeg {
                        pin: element
                            .pins
                            .get(pin.index)
                            .map_or("", |pin| pin.name)
                            .to_string(),
                        hole: view.name,
                        x: view.x,
                        y: view.y,
                        number: breadboard::dip(&element.kind)
                            .and_then(|(_, numbers)| numbers.get(pin.index).copied()),
                    }
                })
                .collect();
            if legs.is_empty() {
                continue;
            }
            parts.push(BoardPart {
                element_id: encode(id),
                kind: element.kind.name().to_string(),
                label: label_for(&element.kind),
                dip: breadboard::dip(&element.kind).map(|(count, _)| count),
                moved: self.board.contains_key(id),
                movable: !breadboard::off_board(&element.kind),
                legs,
            });
        }
        let jumpers = placement
            .wiring
            .jumpers
            .iter()
            .map(|&(from, to)| BoardJumper {
                from: hole_view(from),
                to: hole_view(to),
            })
            .collect();
        let findings = placement.wiring.check(board, &self.netlist);
        let strip_holes = |strips: &[breadboard::Strip]| {
            strips
                .iter()
                .flat_map(|&strip| board.holes_of(strip))
                .map(|hole| hole.name())
                .collect::<Vec<_>>()
        };
        let mut problems: Vec<BoardProblem> = findings
            .shorts
            .iter()
            .map(|(strips, _)| BoardProblem {
                message: format!(
                    "{} joins wires the schematic keeps apart — on this board they are shorted. Move one of them.",
                    strips_named(strips)
                ),
                holes: strip_holes(strips),
            })
            .collect();
        problems.extend(findings.splits.iter().map(|(_, groups)| BoardProblem {
            message: "One wire of the schematic is on rows nothing joins here — add a jumper between them.".to_string(),
            holes: groups.iter().flat_map(|group| strip_holes(group)).collect(),
        }));
        problems.extend(findings.crowded.iter().map(|&hole| BoardProblem {
            message: format!("Two leads are in {} — one hole holds one.", hole.name()),
            holes: vec![hole.name()],
        }));
        let steps = crate::build::steps(&self.netlist, &placement, label_for)
            .into_iter()
            .map(|step| BoardStep {
                text: step.text,
                holes: step
                    .holes
                    .into_iter()
                    .filter_map(|hole| board.position(hole))
                    .map(|(x, y)| BoardSpot { x, y })
                    .collect(),
                elements: step.elements.into_iter().map(encode).collect(),
            })
            .collect();
        Ok(BoardView {
            steps,
            size: match board.size {
                breadboard::Size::Half => "half",
                breadboard::Size::Full => "full",
            }
            .to_string(),
            rows: board.rows(),
            holes,
            parts,
            jumpers,
            problems,
        })
    }

    /// [`Circuit::move_on_board`] without the boundary's error type.
    ///
    /// Every leg moves by the same holes, so the part keeps its shape; a
    /// move that would take a leg off the board, or into a hole another
    /// part's leg is in, is refused and nothing changes. A chip only slides
    /// along the channel, which it has to straddle. One step of undo.
    pub fn try_move_on_board(&mut self, id: &str, dx: i32, dy: i32) -> Result<(), CircuitError> {
        let element_id = decode(id)?;
        let element = self
            .netlist
            .element(element_id)
            .ok_or(CircuitError::Netlist(NetlistError::UnknownElement(
                element_id,
            )))?;
        if breadboard::off_board(&element.kind) {
            return Err(CircuitError::Refused(
                "A battery's leads go in the rails, and stay there.".to_string(),
            ));
        }
        if breadboard::dip(&element.kind).is_some() && dy != 0 {
            return Err(CircuitError::Refused(
                "A chip only slides along the channel: it has to straddle it.".to_string(),
            ));
        }
        if dx == 0 && dy == 0 {
            return Ok(());
        }
        let pin_count = element.pins.len();
        let placement = self.placed()?;
        let board = placement.board;
        let mut holes: Vec<Option<Hole>> = vec![None; pin_count];
        for &(pin, hole) in &placement.wiring.pins {
            if pin.element == element_id && pin.index < pin_count {
                holes[pin.index] = Some(hole);
            }
        }
        let others: Vec<Hole> = placement
            .wiring
            .pins
            .iter()
            .filter(|(pin, _)| pin.element != element_id)
            .map(|&(_, hole)| hole)
            .collect();
        let mut moved: Vec<Option<Hole>> = Vec::with_capacity(pin_count);
        for hole in holes {
            let Some(hole) = hole else {
                moved.push(None);
                continue;
            };
            let (x, y) = board.position(hole).unwrap_or((0.0, 0.0));
            // A full board's rows are there to move onto, even from a half.
            let full = breadboard::Board::new(breadboard::Size::Full);
            let (to_x, to_y) = (x + f64::from(dx), y + f64::from(dy));
            // A lead in a rail stays in it, in the rail's free hole nearest
            // where it is taken: a rail's holes come in fives, and a move by
            // whole rows would otherwise land it in the gap between two.
            let target = match hole {
                Hole::Rail { rail, .. } => {
                    let along = |hole: &Hole| {
                        (full.position(*hole).map_or(f64::MAX, |at| at.0) - to_x).abs()
                    };
                    full.holes_of(breadboard::Strip::Rail(rail))
                        .into_iter()
                        .filter(|candidate| !others.contains(candidate))
                        .min_by(|a, b| along(a).total_cmp(&along(b)))
                }
                Hole::Row { .. } => full.hole_at(to_x, to_y),
            }
            .ok_or_else(|| {
                CircuitError::Refused("That would take a leg off the board.".to_string())
            })?;
            if others.contains(&target) {
                return Err(CircuitError::Refused(format!(
                    "Another part is already in {}.",
                    target.name()
                )));
            }
            moved.push(Some(target));
        }
        self.remember();
        self.board.insert(element_id, moved);
        self.board_edits += 1;
        Ok(())
    }

    /// A moved part's holes, as a file is read back (B-14).
    pub(crate) fn pin_on_board(&mut self, id: ElementId, holes: Vec<Option<Hole>>) {
        self.board.insert(id, holes);
    }

    /// A program from a file, onto the part it was saved with (A-03): not an
    /// edit, so no history — the circuit is being built, not changed. A file
    /// whose program cannot be read is refused, as one whose part cannot be.
    pub(crate) fn load_program(
        &mut self,
        id: ElementId,
        name: &str,
        hex: &str,
    ) -> Result<(), CircuitError> {
        let program = crate::netlist::Program::from_hex(name, hex).map_err(|error| {
            CircuitError::Refused(format!(
                "This file cannot be opened: the program {name} in it is not one the Uno can \
                 run ({error})."
            ))
        })?;
        self.netlist.set_program(id, std::sync::Arc::new(program));
        Ok(())
    }

    /// A moved part's holes, for a file to save.
    pub(crate) fn board_of(&self, id: ElementId) -> Option<Vec<Option<Hole>>> {
        self.board.get(id).cloned()
    }

    /// [`Circuit::bill`] without the boundary's error type.
    ///
    /// Every part on the canvas that is bought to build it — not a meter, a
    /// signal generator or a ground symbol, which are the bench's — at the
    /// value it can be bought in (B-09: a computed 217 Ω is bought as 220 Ω,
    /// and the line says so), by the part number the catalogue names for it
    /// (B-10) or the one its model is based on. Parts alike are one line. The
    /// notes are the linter's findings about them now, and any the rounding
    /// would add.
    pub fn try_bill(&self) -> Result<Vec<BillLine>, CircuitError> {
        let findings = self.solution().findings;
        let mut lines: Vec<BillLine> = Vec::new();
        for (id, element) in self.netlist.elements() {
            if matches!(
                element.kind,
                ElementKind::Ground
                    | ElementKind::Voltmeter
                    | ElementKind::Ammeter
                    | ElementKind::SignalGenerator { .. }
            ) {
                continue;
            }
            let handle = encode(id);
            let mut notes: Vec<String> = Vec::new();
            let value = match self.try_purchase(&handle)? {
                Some(purchase) => {
                    notes.push(format!(
                        "Set to {}; {} is the nearest {}, {:.1} % {}.",
                        label_for(&element.kind),
                        purchase.label,
                        purchase.series,
                        purchase.error_percent.abs(),
                        if purchase.error_percent > 0.0 {
                            "more"
                        } else {
                            "less"
                        }
                    ));
                    notes.extend(
                        purchase
                            .findings
                            .iter()
                            .map(|finding| format!("On {}: {}", purchase.label, finding.message)),
                    );
                    match element.kind {
                        // Its rating is printed on the can and bought with it.
                        ElementKind::Electrolytic { rated, .. } => {
                            format!("{} {}", purchase.label, rated.name())
                        }
                        _ => purchase.label,
                    }
                }
                None => label_for(&element.kind),
            };
            notes.extend(
                findings
                    .iter()
                    .filter(|finding| finding.elements.contains(&handle))
                    .map(|finding| finding.message.clone()),
            );
            let (part, datasheet) = part_number(&element.kind);
            let kind = element.kind.name().to_string();
            match lines
                .iter_mut()
                .find(|line| line.kind == kind && line.value == value && line.part == part)
            {
                Some(line) => {
                    line.quantity += 1;
                    line.elements.push(handle);
                    for note in notes {
                        if !line.notes.contains(&note) {
                            line.notes.push(note);
                        }
                    }
                }
                None => {
                    let mut unique: Vec<String> = Vec::new();
                    for note in notes {
                        if !unique.contains(&note) {
                            unique.push(note);
                        }
                    }
                    lines.push(BillLine {
                        kind,
                        value,
                        part,
                        datasheet,
                        quantity: 1,
                        elements: vec![handle],
                        notes: unique,
                    });
                }
            }
        }
        Ok(lines)
    }

    /// [`Circuit::purchase`] without the boundary's error type.
    ///
    /// A resistor is bought in E12 and a capacitor in E6, the series an
    /// assortment box holds (B-08). What the circuit does on the bought value
    /// is asked of two copies of it, run for a frame from the start — one on
    /// the bought value, one on the exact one — rather than of the live run,
    /// which cannot be copied: a copy starts as the circuit does when it is
    /// switched on, so what is compared is the same moment both ways, and a
    /// warning one has and the other does not is the rounding's. For a
    /// resistor the current is the same as the live run's once nothing is
    /// charging; a capacitor's would be its inrush, and is not given.
    pub fn try_purchase(&self, id: &str) -> Result<Option<Purchase>, CircuitError> {
        use crate::series::{self, Series};
        let element_id = decode(id)?;
        let element = self
            .netlist
            .element(element_id)
            .ok_or(CircuitError::Netlist(NetlistError::UnknownElement(
                element_id,
            )))?;
        let (value, unit, series, resistor) = match element.kind {
            ElementKind::Resistor { ohms } => (ohms, "Ω", Series::E12, true),
            ElementKind::Capacitor { farads, .. } | ElementKind::Electrolytic { farads, .. } => {
                (farads, "F", Series::E6, false)
            }
            _ => return Ok(None),
        };
        let Some(pick) = series::nearest(value, series) else {
            return Ok(None);
        };
        if pick.error_percent.abs() < 1e-6 {
            return Ok(None);
        }
        let written = value::format_editable(pick.value);

        let run = |text: Option<&str>| -> Result<SolutionView, CircuitError> {
            let mut twin = Circuit::new();
            twin.netlist = self.netlist.clone();
            twin.layout = self.layout.clone();
            if let Some(text) = text {
                twin.try_set_value(id, text)?;
            }
            // A failed solve is recorded, not raised, and is a finding like
            // any other.
            let _ = twin.advance_by(1.0 / 60.0);
            Ok(twin.solution())
        };
        let exact = run(None)?;
        let bought = run(Some(&written))?;
        let findings = bought
            .findings
            .into_iter()
            .filter(|finding| {
                !exact
                    .findings
                    .iter()
                    .any(|other| other.rule == finding.rule && other.elements == finding.elements)
            })
            .collect();
        let amps = resistor
            .then(|| {
                bought
                    .element_currents
                    .iter()
                    .find(|current| current.element_id == id)
                    .map(|current| current.amps.abs())
            })
            .flatten();
        Ok(Some(Purchase {
            element_id: id.to_string(),
            value: written,
            label: value::format(pick.value, unit),
            series: series.name().to_string(),
            error_percent: pick.error_percent,
            amps,
            findings,
        }))
    }

    /// [`Circuit::replace`] without the boundary's error type.
    ///
    /// The part keeps its id, so it stays where it is and stays selected;
    /// which pins keep their wires is [`crate::replace::carry`]'s. Its value
    /// comes across when the new kind's is in the same unit — a resistor's
    /// ohms to a potentiometer's track — and is the new kind's own
    /// otherwise. Its place on the breadboard is worked out afresh.
    pub fn try_replace(&mut self, id: &str, kind: &str) -> Result<Replacement, CircuitError> {
        let element_id = decode(id)?;
        let to = parse_kind(kind)?;
        let element = self
            .netlist
            .element(element_id)
            .ok_or(CircuitError::Netlist(NetlistError::UnknownElement(
                element_id,
            )))?;
        let from = element.kind;
        if core::mem::discriminant(&from) == core::mem::discriminant(&to) {
            return Ok(Replacement::default());
        }
        let carry = crate::replace::carry(&from, &to);
        let wired: Vec<(usize, &'static str)> = element
            .pins
            .iter()
            .enumerate()
            .filter(|(_, pin)| pin.net.is_some())
            .map(|(index, pin)| (index, pin.name))
            .collect();
        let replacement = Replacement {
            kept: carry
                .iter()
                .filter(|(old, _)| wired.iter().any(|(index, _)| index == old))
                .map(|&(old, new)| {
                    (
                        from.pin_names()[old].to_string(),
                        to.pin_names()[new].to_string(),
                    )
                })
                .collect(),
            dropped: wired
                .iter()
                .filter(|(index, _)| !carry.iter().any(|(old, _)| old == index))
                .map(|(_, name)| name.to_string())
                .collect(),
        };
        let unit = |kind: &ElementKind| {
            crate::registry::definition(kind.name()).map_or("", |component| component.unit)
        };
        let value = value_of(&from).filter(|_| {
            !unit(&from).is_empty() && unit(&from) == unit(&to) && value_of(&to).is_some()
        });

        let outer = !matches!(self.gesture, Gesture::Closed);
        if !outer {
            self.gesture = Gesture::Open;
        }
        self.remember();
        let done = self
            .netlist
            .replace(element_id, to, &carry)
            .map_err(CircuitError::Netlist)
            .map(|()| {
                self.board.remove(element_id);
                self.prune_wires();
                self.revision += 1;
                // A value the new kind will not take (a pot's track has its
                // own limits) leaves it at its own: the swap is still made.
                if let Some(value) = value {
                    let _ = self.try_set_value(id, &value::format_editable(value));
                }
            });
        if !outer {
            self.gesture = Gesture::Closed;
        }
        done.map(|()| replacement)
    }

    /// [`Circuit::steps`] without the boundary's error type.
    ///
    /// In the series each is bought in, as the purchase (B-09) says: E12
    /// for a resistor, what an assortment box holds, and E6 for a
    /// capacitor. A value between two steps to the nearer one on that side,
    /// never to itself (B-08's `above` and `below`), so − on 217 Ω is 180 Ω
    /// and + is 220 Ω.
    pub fn try_steps(&self, id: &str) -> Result<Steps, CircuitError> {
        use crate::series::{self, Series};
        /// How many either side: the chips under the field.
        const EACH: usize = 2;
        let element_id = decode(id)?;
        let element = self
            .netlist
            .element(element_id)
            .ok_or(CircuitError::Netlist(NetlistError::UnknownElement(
                element_id,
            )))?;
        let (value, unit, series) = match element.kind {
            ElementKind::Resistor { ohms } => (ohms, "Ω", Series::E12),
            ElementKind::Capacitor { farads, .. } | ElementKind::Electrolytic { farads, .. } => {
                (farads, "F", Series::E6)
            }
            _ => return Ok(Steps::default()),
        };
        let walk = |next: fn(f64, Series) -> Option<f64>| {
            std::iter::successors(next(value, series), |&at| next(at, series))
                .take(EACH)
                .map(|at| Step {
                    value: value::format_editable(at),
                    label: value::format(at, unit),
                })
                .collect()
        };
        Ok(Steps {
            below: walk(series::below),
            above: walk(series::above),
        })
    }
}
