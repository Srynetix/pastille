// SPDX-License-Identifier: MIT OR Apache-2.0

//! Changing the circuit: the `try_*` edits, undo, and the contacts and
//! fuses the run itself changes.

use super::*;

impl Circuit {
    /// The engine's clock: where the run has stepped to.
    pub(super) fn run_time(&self) -> f64 {
        self.simulation
            .as_ref()
            .map_or(0.0, |simulation| simulation.engine().time())
    }

    /// A bouncing button's contact, flipped by its chatter rather than by a
    /// finger: no step of history, no new chatter.
    pub(super) fn bounce(&mut self, id: ElementId) {
        let Some(element) = self.netlist.element(id) else {
            return;
        };
        let kind = element.kind;
        let Some(flipped) = kind.contact().and_then(|closed| kind.with_contact(!closed)) else {
            return;
        };
        if self.netlist.set_kind(id, flipped).is_ok() {
            self.revision += 1;
        }
    }

    /// Every fast-blow fuse the run has just melted, blown on the part, so it
    /// is drawn blown and saved blown until it is replaced (C-120). Not a
    /// step of undo: the circuit did it, not the person.
    pub(super) fn blow(&mut self) {
        let Some(simulation) = self.simulation.as_ref() else {
            return;
        };
        let engine = simulation.engine();
        let melted: Vec<(ElementId, ElementKind)> = self
            .netlist
            .elements()
            .filter_map(|(id, element)| match element.kind {
                ElementKind::Fuse {
                    amps,
                    part: crate::solver::device::fuse::FusePart::Fast,
                    blown: false,
                } if engine.carried(id).is_some_and(|carried| carried.latched) => Some((
                    id,
                    ElementKind::Fuse {
                        amps,
                        part: crate::solver::device::fuse::FusePart::Fast,
                        blown: true,
                    },
                )),
                _ => None,
            })
            .collect();
        for (id, blown) in melted {
            if self.netlist.set_kind(id, blown).is_ok() {
                self.revision += 1;
            }
        }
    }

    /// [`Circuit::replace_fuse`] without the boundary's error type.
    pub fn try_replace_fuse(&mut self, id: &str) -> Result<(), CircuitError> {
        let id = decode(id)?;
        let kind = self
            .netlist
            .element(id)
            .ok_or(CircuitError::Netlist(NetlistError::UnknownElement(id)))?
            .kind;
        let ElementKind::Fuse { amps, part, blown } = kind else {
            return Err(CircuitError::NotEditable(kind.name().to_string()));
        };
        if !blown {
            return Ok(());
        }
        self.remember();
        self.netlist
            .set_kind(
                id,
                ElementKind::Fuse {
                    amps,
                    part,
                    blown: false,
                },
            )
            .map_err(CircuitError::Netlist)?;
        self.revision += 1;
        Ok(())
    }

    /// One frame's worth of running, with no chatter in it.
    pub(super) fn advance_by(&mut self, elapsed: f64) -> Result<(), JsError> {
        if self.simulated != self.revision {
            self.rebuild();
        }

        if elapsed.is_finite() && elapsed > 0.0 {
            self.elapsed += elapsed;
        }

        // A circuit that failed is not asked again until something about it
        // changes. Half a wire is not a circuit and half a wire is the normal
        // state of one being built, so this is the common case — and retrying
        // it sixty times a second would run the whole convergence ladder of
        // C-11 for an answer that is not there.
        if self.failure.is_some() {
            return Ok(());
        }

        if let Some(simulation) = self.simulation.as_mut()
            && let Err(error) = simulation.advance(elapsed)
        {
            // Remember why and carry on; the next edit usually fixes it. The
            // engine stays, because what it is holding outlives a circuit that
            // is momentarily unsolvable (C-48).
            self.failure = Some(Failure {
                code: error.code(),
                detail: error.to_string(),
            });
        }

        Ok(())
    }

    /// Remembers the current state so the next change can be undone.
    ///
    /// Every mutation calls this. Inside a gesture the first call takes the
    /// snapshot and the rest do nothing — a drag of sixty frames, or a delete
    /// of four components, is one thing the user did and should be one press
    /// of undo.
    pub(super) fn remember(&mut self) {
        match self.gesture {
            Gesture::Recorded => return,
            Gesture::Open => self.gesture = Gesture::Recorded,
            Gesture::Closed => {}
        }
        self.past.push(Snapshot {
            netlist: self.netlist.clone(),
            layout: self.layout.clone(),
            board: self.board.clone(),
            wires: self.wires.clone(),
            description: self.description.clone(),
        });
        if self.past.len() > HISTORY_LIMIT {
            self.past.remove(0);
        }
        // An edit ends whatever a button was still doing: its chatter was
        // the circuit before the edit's.
        self.chatter.clear();
        // A new edit is a new branch: whatever was undone is no longer ahead.
        self.future.clear();
    }

    pub(super) fn restore(&mut self, snapshot: Snapshot) -> Snapshot {
        let current = Snapshot {
            netlist: core::mem::replace(&mut self.netlist, snapshot.netlist),
            layout: core::mem::replace(&mut self.layout, snapshot.layout),
            board: core::mem::replace(&mut self.board, snapshot.board),
            wires: core::mem::replace(&mut self.wires, snapshot.wires),
            description: core::mem::replace(&mut self.description, snapshot.description),
        };
        self.chatter.clear();
        self.revision += 1;
        self.board_edits += 1;
        current
    }

    /// [`Circuit::add`] without the boundary's error type.
    pub fn try_add(&mut self, kind: &str, x: f64, y: f64) -> Result<String, CircuitError> {
        let kind = parse_kind(kind)?;
        self.remember();
        let id = self.netlist.add_element(kind);
        self.layout.insert(id, Placement { x, y, turns: 0 });
        self.revision += 1;
        Ok(encode(id))
    }

    /// [`Circuit::remove`] without the boundary's error type.
    pub fn try_remove(&mut self, id: &str) -> Result<(), CircuitError> {
        let id = decode(id)?;
        self.remember();
        self.netlist
            .remove_element(id)
            .map_err(CircuitError::Netlist)?;
        self.layout.remove(id);
        self.board.remove(id);
        self.prune_wires();
        self.revision += 1;
        Ok(())
    }

    /// [`Circuit::set_initial_condition`] without the boundary's error type.
    pub fn try_set_initial_condition(&mut self, id: &str, text: &str) -> Result<(), CircuitError> {
        let id = decode(id)?;
        let kind = self
            .netlist
            .element(id)
            .ok_or(CircuitError::Netlist(NetlistError::UnknownElement(id)))?
            .kind;

        let magnitude = if text.trim().is_empty() {
            0.0
        } else {
            value::parse(text).map_err(CircuitError::Value)?
        };

        let updated = match kind {
            ElementKind::Capacitor { farads, .. } => ElementKind::Capacitor {
                farads,
                initial_volts: magnitude,
            },
            ElementKind::Electrolytic { farads, rated, .. } => ElementKind::Electrolytic {
                farads,
                initial_volts: magnitude,
                rated,
            },
            ElementKind::Inductor { henries, .. } => ElementKind::Inductor {
                henries,
                initial_amps: magnitude,
            },
            other => return Err(CircuitError::NotEditable(other.name().to_string())),
        };
        checked(&updated)?;

        self.remember();
        self.netlist
            .set_kind(id, updated)
            .map_err(CircuitError::Netlist)?;
        self.revision += 1;
        Ok(())
    }

    /// [`Circuit::set_program`] without the boundary's error type.
    pub fn try_set_program(&mut self, id: &str, name: &str, hex: &str) -> Result<(), CircuitError> {
        let id = decode(id)?;
        let kind = self
            .netlist
            .element(id)
            .ok_or(CircuitError::Netlist(NetlistError::UnknownElement(id)))?
            .kind;
        if !matches!(kind, ElementKind::Uno) {
            return Err(CircuitError::NotEditable(kind.name().to_string()));
        }
        let program = crate::netlist::Program::from_hex(name, hex).map_err(|error| {
            CircuitError::Refused(format!(
                "{name} is not a program the Uno can run: {error}. Export it from the Arduino \
                 IDE with Sketch → Export Compiled Binary, and load the .hex without the \
                 bootloader."
            ))
        })?;
        self.remember();
        self.netlist.set_program(id, std::sync::Arc::new(program));
        self.revision += 1;
        Ok(())
    }

    /// [`Circuit::take_serial`] without the boundary's error type.
    pub fn try_take_serial(&mut self, id: &str) -> Result<Vec<u8>, CircuitError> {
        let id = decode(id)?;
        Ok(self
            .simulation
            .as_mut()
            .map(|run| run.engine_mut().take_serial(id))
            .unwrap_or_default())
    }

    /// [`Circuit::send_serial`] without the boundary's error type.
    pub fn try_send_serial(&mut self, id: &str, text: &str) -> Result<(), CircuitError> {
        let id = decode(id)?;
        if let Some(run) = self.simulation.as_mut() {
            run.engine_mut().send_serial(id, text.as_bytes());
        }
        Ok(())
    }

    /// [`Circuit::set_monitor_baud`] without the boundary's error type.
    pub fn try_set_monitor_baud(&mut self, id: &str, baud: u32) -> Result<(), CircuitError> {
        let id = decode(id)?;
        if let Some(run) = self.simulation.as_mut() {
            run.engine_mut().set_monitor_baud(id, baud);
        }
        Ok(())
    }

    /// [`Circuit::reset_program`] without the boundary's error type.
    pub fn try_reset_program(&mut self, id: &str) -> Result<(), CircuitError> {
        let id = decode(id)?;
        let Some(program) = self.netlist.program(id) else {
            return Ok(());
        };
        // The same program as a new one: the run tells them apart by
        // identity, so this one starts from reset.
        let again = crate::netlist::Program {
            name: program.name.clone(),
            hex: program.hex.clone(),
            words: program.words.clone(),
        };
        self.netlist.set_program(id, std::sync::Arc::new(again));
        self.revision += 1;
        Ok(())
    }

    /// [`Circuit::set_choice`] without the boundary's error type.
    pub fn try_set_choice(&mut self, id: &str, choice: &str) -> Result<(), CircuitError> {
        let id = decode(id)?;
        let kind = self
            .netlist
            .element(id)
            .ok_or(CircuitError::Netlist(NetlistError::UnknownElement(id)))?
            .kind;

        if kind.choice().is_none() {
            return Err(CircuitError::NotEditable(kind.name().to_string()));
        }
        let updated = kind
            .with_choice(choice)
            .ok_or_else(|| CircuitError::UnknownChoice {
                kind: kind.name().to_string(),
                choice: choice.to_string(),
            })?;

        if updated == kind {
            return Ok(());
        }

        self.remember();
        self.netlist
            .set_kind(id, updated)
            .map_err(CircuitError::Netlist)?;
        self.revision += 1;
        Ok(())
    }

    /// [`Circuit::set_wiper`] without the boundary's error type.
    pub fn try_set_wiper(&mut self, id: &str, wiper: f64) -> Result<(), CircuitError> {
        let id = decode(id)?;
        let kind = self
            .netlist
            .element(id)
            .ok_or(CircuitError::Netlist(NetlistError::UnknownElement(id)))?
            .kind;

        if kind.wiper().is_none() {
            return Err(CircuitError::NotEditable(kind.name().to_string()));
        }
        if !wiper.is_finite() {
            return Err(CircuitError::Value(value::ValueError::NotANumber(
                wiper.to_string(),
            )));
        }

        // Clamped rather than refused: a knob dragged past its stop has not
        // been misused, it has reached the end.
        let updated = kind
            .with_wiper(wiper)
            .expect("a kind with a dial can be turned");
        if updated == kind {
            return Ok(());
        }

        self.remember();
        self.netlist
            .set_kind(id, updated)
            .map_err(CircuitError::Netlist)?;
        self.revision += 1;
        Ok(())
    }

    /// [`Circuit::set_closed`] without the boundary's error type.
    pub fn try_set_closed(&mut self, id: &str, closed: bool) -> Result<(), CircuitError> {
        let id = decode(id)?;
        let kind = self
            .netlist
            .element(id)
            .ok_or(CircuitError::Netlist(NetlistError::UnknownElement(id)))?
            .kind;

        let updated = kind
            .with_contact(closed)
            .ok_or_else(|| CircuitError::NotEditable(kind.name().to_string()))?;

        // Flipping a switch to where it already is is not something the user
        // did: a push-button reports a release on every pointer up, including
        // the ones that were never a press, and each of those would otherwise
        // be a step of undo to walk back through.
        if updated == kind {
            return Ok(());
        }
        // A PIR, or a push-button, let go sooner than a finger can be keeps
        // its press until it has lasted that long (C-116, C-123).
        let pressed = kind.moving() == Some(true)
            || matches!(kind, ElementKind::PushButton { closed: true, .. });
        if !closed && pressed && !self.pressed_long_enough(id) {
            if !self.unseen.contains(&id) {
                self.unseen.push(id);
            }
            return Ok(());
        }

        self.remember();
        self.netlist
            .set_kind(id, updated)
            .map_err(CircuitError::Netlist)?;
        self.revision += 1;
        if closed {
            self.pressed_at.insert(id, self.run_time());
        }
        self.unseen.retain(|&held| held != id);
        self.chatter_after(id, updated);
        Ok(())
    }

    /// Whether a press has lasted as long as a finger's shortest (C-123).
    pub(super) fn pressed_long_enough(&self, id: ElementId) -> bool {
        self.pressed_at
            .get(id)
            .is_none_or(|&at| self.run_time() - at >= SHORTEST_PRESS - 1e-9)
    }

    /// A bouncing button chatters after every press and every release —
    /// from when it actually made or broke, so a release held back a frame
    /// (C-123) chatters from then.
    pub(super) fn chatter_after(&mut self, id: ElementId, kind: ElementKind) {
        if let ElementKind::PushButton { bounces: true, .. } = kind {
            let now = self.run_time();
            self.chatter.retain(|chatter| chatter.element != id);
            self.chatter.extend(BOUNCES.iter().map(|after| Chatter {
                element: id,
                at: now + after,
            }));
            self.chatter.sort_by(|a, b| a.at.total_cmp(&b.at));
        }
    }

    /// [`Circuit::set_series_resistance`] without the boundary's error type.
    pub fn try_set_frequency(&mut self, id: &str, text: &str) -> Result<(), CircuitError> {
        let id = decode(id)?;
        let kind = self
            .netlist
            .element(id)
            .ok_or(CircuitError::Netlist(NetlistError::UnknownElement(id)))?
            .kind;

        let ElementKind::SignalGenerator {
            waveform, volts, ..
        } = kind
        else {
            return Err(CircuitError::NotEditable(kind.name().to_string()));
        };

        // A frequency of nothing is a battery, and a negative one is a
        // frequency of the same size played backwards: neither is a setting.
        let hertz = value::parse(text).map_err(CircuitError::Value)?;
        checked(&ElementKind::SignalGenerator {
            waveform,
            volts,
            hertz,
        })?;

        self.remember();
        self.netlist
            .set_kind(
                id,
                ElementKind::SignalGenerator {
                    waveform,
                    volts,
                    hertz,
                },
            )
            .map_err(CircuitError::Netlist)?;
        self.revision += 1;
        Ok(())
    }

    /// [`Circuit::open`] without the boundary's error type.
    pub fn try_open(&mut self, text: &str) -> Result<(), CircuitError> {
        let document: crate::document::Document =
            crate::json::from_str(text).map_err(CircuitError::Malformed)?;
        let opened = Circuit::from_document(&document)?;
        self.remember();
        self.netlist = opened.netlist;
        self.layout = opened.layout;
        self.board = opened.board;
        self.board_edits += 1;
        self.wires = opened.wires;
        self.description = opened.description;
        self.revision += 1;
        Ok(())
    }

    /// [`Circuit::set_description`], saying whether anything changed.
    pub fn try_set_description(&mut self, text: &str) -> bool {
        let text = text.trim();
        if text == self.description {
            return false;
        }
        self.remember();
        self.description = text.to_string();
        // No revision bump: a description is the drawing's, not the circuit's.
        true
    }

    /// [`Circuit::clear`], saying whether anything changed.
    pub fn try_clear(&mut self) -> bool {
        if self.netlist.element_count() == 0
            && self.netlist.net_count() == 0
            && self.description.is_empty()
        {
            return false;
        }
        self.remember();
        let room = self.netlist.room_celsius();
        self.netlist = Netlist::new();
        self.netlist.set_room_celsius(room);
        self.layout.clear();
        self.board.clear();
        self.board_edits += 1;
        self.wires.clear();
        self.description.clear();
        self.revision += 1;
        true
    }

    /// Gives a circuit being loaded the description it was saved with. Not a
    /// step of history, as nothing else a load does is.
    pub(crate) fn describe_as_loaded(&mut self, text: &str) {
        self.description = text.trim().to_string();
    }

    /// [`Circuit::set_room`] without the boundary's error type.
    pub fn try_set_room(&mut self, text: &str) -> Result<(), CircuitError> {
        // The unit is the field's, but typing it as well is not a mistake.
        let bare = text.trim().trim_end_matches("°C").trim_end_matches('°');
        let celsius = value::parse(bare).map_err(CircuitError::Value)?;
        if !(ROOM_RANGE.0..=ROOM_RANGE.1).contains(&celsius) {
            return Err(CircuitError::RoomOutOfRange(bare.to_string()));
        }
        if celsius == self.netlist.room_celsius() {
            return Ok(());
        }

        self.remember();
        self.netlist.set_room_celsius(celsius);
        self.revision += 1;
        Ok(())
    }

    pub fn try_set_series_resistance(&mut self, id: &str, text: &str) -> Result<(), CircuitError> {
        let id = decode(id)?;
        let kind = self
            .netlist
            .element(id)
            .ok_or(CircuitError::Netlist(NetlistError::UnknownElement(id)))?
            .kind;

        let ElementKind::DcSource { volts, .. } = kind else {
            return Err(CircuitError::NotEditable(kind.name().to_string()));
        };

        let series_ohms = if text.trim().is_empty() {
            0.0
        } else {
            value::parse(text).map_err(CircuitError::Value)?
        };
        checked(&ElementKind::DcSource { volts, series_ohms })?;

        self.remember();
        self.netlist
            .set_kind(id, ElementKind::DcSource { volts, series_ohms })
            .map_err(CircuitError::Netlist)?;
        self.revision += 1;
        Ok(())
    }

    /// [`Circuit::set_waypoints`] without the boundary's error type.
    pub fn try_set_waypoints(
        &mut self,
        id: &str,
        points: Vec<PointView>,
    ) -> Result<(), CircuitError> {
        let id = decode_net(id)?;
        if self.netlist.net(id).is_none() {
            return Err(CircuitError::Netlist(NetlistError::UnknownNet(id)));
        }

        self.remember();
        // An empty list is "straighten it", and storing one would leave an
        // entry behind that says nothing.
        if points.is_empty() {
            self.wires.remove(id);
        } else {
            self.wires.insert(id, points);
        }
        // No revision bump: where a wire runs is the drawing, not the circuit.
        Ok(())
    }

    /// [`Circuit::rotate`] without the boundary's error type.
    pub fn try_rotate(&mut self, id: &str) -> Result<u8, CircuitError> {
        let id = decode(id)?;
        let at = self
            .layout
            .get(id)
            .copied()
            .ok_or(CircuitError::Netlist(NetlistError::UnknownElement(id)))?;

        self.remember();
        let turns = (at.turns + 1) % TURNS;
        self.layout.insert(id, Placement { turns, ..at });
        // No revision bump, for the same reason a move does not bump it: which
        // way a component faces changes the drawing, not the circuit. The nets
        // are the same nets and the solve is the same solve.
        Ok(turns)
    }

    /// Adds an element of a kind already decided, at a placement already
    /// decided. What loading a file does, where `try_add` is what a palette
    /// does.
    ///
    /// `turns` comes from a file, so it is taken modulo a full turn: 5 is 1,
    /// as the drawing reads it, and a 255 cannot overflow the next rotation.
    pub(crate) fn place(&mut self, kind: ElementKind, x: f64, y: f64, turns: u8) -> ElementId {
        let id = self.netlist.add_element(kind);
        let turns = turns % TURNS;
        self.layout.insert(id, Placement { x, y, turns });
        self.revision += 1;
        id
    }

    /// Puts a circuit being loaded in the room it was saved in. Not a step
    /// of history, as nothing else a load does is.
    pub(crate) fn place_in_room(&mut self, celsius: f64) {
        self.netlist.set_room_celsius(celsius);
        self.revision += 1;
    }

    /// Recreates one saved net, joining the pins it names.
    pub(crate) fn rejoin(
        &mut self,
        pins: &[(ElementId, String)],
        waypoints: &[PointView],
    ) -> Result<(), CircuitError> {
        let net = self.netlist.add_net();
        for (element, name) in pins {
            self.netlist
                .connect_named(*element, name, net)
                .map_err(CircuitError::Netlist)?;
        }
        if !waypoints.is_empty() {
            self.wires.insert(net, waypoints.to_vec());
        }
        self.revision += 1;
        Ok(())
    }

    /// [`Circuit::remove_net`] without the boundary's error type.
    pub fn try_remove_net(&mut self, id: &str) -> Result<(), CircuitError> {
        let id = decode_net(id)?;
        self.remember();
        self.netlist.remove_net(id).map_err(CircuitError::Netlist)?;
        self.wires.remove(id);
        self.revision += 1;
        Ok(())
    }

    /// [`Circuit::move_element`] without the boundary's error type.
    pub fn try_move(&mut self, id: &str, x: f64, y: f64) -> Result<(), CircuitError> {
        let id = decode(id)?;
        if !self.layout.contains_key(id) {
            return Err(CircuitError::Netlist(NetlistError::UnknownElement(id)));
        }
        // The one mutation that records only inside a gesture: a drag is sixty
        // moves a second, and outside a gesture there is no step to fold them
        // into — so a lone move is not one either, as it never was.
        if self.gesture != Gesture::Closed {
            self.remember();
        }
        // Keeps whichever way it was facing: a drag moves a component, it does
        // not straighten it.
        let turns = self.layout.get(id).map_or(0, |at| at.turns);
        self.layout.insert(id, Placement { x, y, turns });
        // Deliberately no revision bump: moving changes the drawing, not the
        // circuit, and rebuilding the engine for it would restart the solve
        // every frame of a drag.
        Ok(())
    }

    /// Drops wires that no longer join two things.
    ///
    /// Deleting a component leaves the wires that reached it dangling from one
    /// end. On screen they are stubs pointing at nothing, so they go with it —
    /// which is what "delete removes the element and its nets" means.
    pub(super) fn prune_wires(&mut self) {
        let orphaned: Vec<NetId> = self
            .netlist
            .nets()
            .filter(|(_, net)| net.pins().len() < 2)
            .map(|(id, _)| id)
            .collect();

        for net in orphaned {
            let _ = self.netlist.remove_net(net);
            // A route with no wire left to draw is debris, the same way a
            // position with no element is (ADR 0006).
            self.wires.remove(net);
        }
    }

    /// [`Circuit::connect`] without the boundary's error type.
    pub fn try_connect(
        &mut self,
        from_element: &str,
        from_pin: &str,
        to_element: &str,
        to_pin: &str,
    ) -> Result<String, CircuitError> {
        let from = self.locate(from_element, from_pin)?;
        let to = self.locate(to_element, to_pin)?;

        if from == to {
            return Err(CircuitError::SamePin);
        }

        self.remember();

        let net = match (self.net_of(from), self.net_of(to)) {
            (Some(left), Some(right)) => {
                self.netlist
                    .merge_nets(left, right)
                    .map_err(CircuitError::Netlist)?;
                left
            }
            (Some(existing), None) => {
                self.attach(to, existing)?;
                existing
            }
            (None, Some(existing)) => {
                self.attach(from, existing)?;
                existing
            }
            (None, None) => {
                let fresh = self.netlist.add_net();
                self.attach(from, fresh)?;
                self.attach(to, fresh)?;
                fresh
            }
        };

        self.revision += 1;
        Ok(encode_net(net))
    }

    /// [`Circuit::apply_fix`] without the boundary's error type.
    pub fn try_apply_fix(&mut self, fix: &Fix) -> Result<(), CircuitError> {
        // Inside a gesture of its own, unless the caller has one open: the
        // fix is one thing the user did.
        let outer = !matches!(self.gesture, Gesture::Closed);
        if !outer {
            self.gesture = Gesture::Open;
        }
        let done = match fix {
            Fix::SetValue { element, value } => self.try_set_value(element, value),
            Fix::AddResistor { element, value } => self.splice_resistors(element, value),
        };
        if !outer {
            self.gesture = Gesture::Closed;
        }
        done
    }

    /// Puts a resistor of this value between a part's LED pins and whatever
    /// they were wired to (B-02's fix): an LED's anode, or each own pin of a
    /// digit or an RGB LED — the pin that is not the shared one; or between a
    /// transistor's base and its wire (B-04's).
    ///
    /// Each resistor is placed above and to the left of the part, in a column,
    /// where it can be moved from; the wires are drawn from where the pins
    /// are, so they follow (ADR 0006).
    pub(super) fn splice_resistors(
        &mut self,
        element: &str,
        value: &str,
    ) -> Result<(), CircuitError> {
        let id = decode(element)?;
        let part = self
            .netlist
            .element(id)
            .ok_or(CircuitError::Netlist(NetlistError::UnknownElement(id)))?;
        let own: Vec<usize> = match part.kind {
            ElementKind::Led { .. } => vec![0],
            // A transistor's base (B-04).
            ElementKind::Bjt { .. } => vec![crate::solver::device::BASE],
            _ => match part.kind.shared_leds() {
                Some((_, colours)) => (0..colours.len()).collect(),
                None => {
                    return Err(CircuitError::Netlist(NetlistError::KindMismatch {
                        element: id,
                    }));
                }
            },
        };
        // Only the pins that are wired: a resistor on a free pin limits
        // nothing and is one more part to delete.
        let wired: Vec<(usize, NetId, &'static str)> = own
            .into_iter()
            .filter_map(|index| {
                let pin = part.pins.get(index)?;
                Some((index, pin.net?, pin.name))
            })
            .collect();
        let at = self.layout.get(id).copied().unwrap_or(Placement {
            x: 0.0,
            y: 0.0,
            turns: 0,
        });

        for (row, (index, net, name)) in wired.into_iter().enumerate() {
            let resistor =
                self.try_add("Resistor", at.x - 60.0, at.y - 40.0 - 30.0 * row as f64)?;
            self.try_set_value(&resistor, value)?;
            self.remember();
            self.netlist
                .disconnect(PinRef::new(id, index))
                .map_err(CircuitError::Netlist)?;
            let resistor_id = decode(&resistor)?;
            self.attach(PinRef::new(resistor_id, 0), net)?;
            self.revision += 1;
            self.try_connect(&resistor, "b", element, name)?;
        }
        Ok(())
    }

    /// [`Circuit::set_value`] without the boundary's error type.
    pub fn try_set_value(&mut self, id: &str, text: &str) -> Result<(), CircuitError> {
        let id = decode(id)?;
        let kind = self
            .netlist
            .element(id)
            .ok_or(CircuitError::Netlist(NetlistError::UnknownElement(id)))?
            .kind;

        let magnitude = value::parse(text).map_err(CircuitError::Value)?;
        let updated = match kind {
            ElementKind::Resistor { .. } => ElementKind::Resistor { ohms: magnitude },
            // The value and the initial condition are two properties of one
            // component, as a source's voltage and internal resistance are.
            ElementKind::Capacitor { initial_volts, .. } => ElementKind::Capacitor {
                farads: magnitude,
                initial_volts,
            },
            ElementKind::Electrolytic {
                initial_volts,
                rated,
                ..
            } => ElementKind::Electrolytic {
                farads: magnitude,
                initial_volts,
                rated,
            },
            ElementKind::Inductor { initial_amps, .. } => ElementKind::Inductor {
                henries: magnitude,
                initial_amps,
            },
            // Editing the voltage leaves the internal resistance alone: they
            // are two properties of one component, not one field.
            ElementKind::DcSource { series_ohms, .. } => ElementKind::DcSource {
                volts: magnitude,
                series_ohms,
            },
            // The amplitude, which leaves the shape and the frequency alone.
            ElementKind::SignalGenerator {
                waveform, hertz, ..
            } => ElementKind::SignalGenerator {
                waveform,
                volts: magnitude,
                hertz,
            },
            // Retuning the track leaves the wiper where it is: a fraction still
            // means the same thing about a different total, which is the reason
            // it is a fraction (C-20).
            ElementKind::Potentiometer { wiper, .. } => ElementKind::Potentiometer {
                ohms: magnitude,
                wiper,
            },
            // Its delay knob (C-116).
            ElementKind::Transformer { .. } => ElementKind::Transformer { ratio: magnitude },
            // A new rating is a new fuse, whole.
            ElementKind::Fuse { part, .. } => ElementKind::Fuse {
                amps: magnitude,
                part,
                blown: false,
            },
            ElementKind::Pir {
                moving, retriggers, ..
            } => ElementKind::Pir {
                moving,
                hold: magnitude,
                retriggers,
            },
            other => return Err(CircuitError::NotEditable(kind_name(&other).to_string())),
        };
        // Checked before anything is remembered: a refused edit is not a step
        // of history.
        checked(&updated)?;

        self.remember();
        self.netlist
            .set_kind(id, updated)
            .map_err(CircuitError::Netlist)?;
        self.revision += 1;
        Ok(())
    }
}
