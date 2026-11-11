// SPDX-License-Identifier: MIT OR Apache-2.0

//! The circuit's face to JavaScript. Each method is a thin wrapper over a
//! `try_*` one, so every path can be tested without a browser.

use super::*;

#[wasm_bindgen]
impl Circuit {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Circuit {
        Circuit {
            netlist: Netlist::new(),
            layout: SecondaryMap::new(),
            board: SecondaryMap::new(),
            board_edits: 0,
            wires: SparseSecondaryMap::new(),
            description: String::new(),
            simulation: None,
            failure: None,
            revision: 0,
            simulated: u64::MAX,
            past: Vec::new(),
            future: Vec::new(),
            gesture: Gesture::Closed,
            elapsed: 0.0,
            chatter: Vec::new(),
            unseen: Vec::new(),
            pressed_at: SecondaryMap::new(),
        }
    }

    /// Adds an element of the named kind at a point, and returns its handle.
    ///
    /// Defaults are the values a beginner would reach for first: a 9 V battery,
    /// a 220 Ω resistor, a red LED. C-16's registry is what will own them.
    pub fn add(&mut self, kind: &str, x: f64, y: f64) -> Result<String, JsError> {
        Ok(self.try_add(kind, x, y)?)
    }

    /// Removes an element and forgets where it was.
    pub fn remove(&mut self, id: &str) -> Result<(), JsError> {
        Ok(self.try_remove(id)?)
    }

    /// Everything to draw, in insertion order.
    pub fn elements(&self) -> Ts<ElementViews> {
        Ts::from_rust(&ElementViews(self.views())).expect("element views serialize")
    }

    /// Sets a component to one of its alternatives, by name (C-46).
    ///
    /// The second shape of property, and its own call for the same reason the
    /// wiper and the internal resistance have theirs: a choice from a closed
    /// set is not a number, and `setValue` parses numbers. The names are the
    /// core's, handed to the app through the element view, so nothing can offer
    /// a choice the core would refuse.
    #[wasm_bindgen(js_name = setChoice)]
    pub fn set_choice(&mut self, id: &str, choice: &str) -> Result<(), JsError> {
        Ok(self.try_set_choice(id, choice)?)
    }

    /// Gives a microcontroller a program, from the Intel HEX the Arduino IDE
    /// exports (A-03): it starts from reset. One step of undo.
    #[wasm_bindgen(js_name = setProgram)]
    pub fn set_program(&mut self, id: &str, name: &str, hex: &str) -> Result<(), JsError> {
        Ok(self.try_set_program(id, name, hex)?)
    }

    /// Starts a microcontroller's program again from the beginning, as the
    /// board's reset button does (A-03). Not an edit: the circuit is the
    /// same, so there is nothing to undo.
    #[wasm_bindgen(js_name = resetProgram)]
    pub fn reset_program(&mut self, id: &str) -> Result<(), JsError> {
        Ok(self.try_reset_program(id)?)
    }

    /// What a microcontroller's serial monitor has heard since this was last
    /// asked, as the bytes it heard (A-06): what the program printed, if the
    /// monitor's baud is the program's, and garbage if it is not. The app
    /// keeps the transcript; the core only hands over what is new.
    #[wasm_bindgen(js_name = takeSerial)]
    pub fn take_serial(&mut self, id: &str) -> Result<Vec<u8>, JsError> {
        Ok(self.try_take_serial(id)?)
    }

    /// Types a line into a microcontroller's serial monitor: sent at the
    /// monitor's baud, as it is given — the caller adds the line ending it
    /// wants, as the Arduino IDE's monitor does.
    #[wasm_bindgen(js_name = sendSerial)]
    pub fn send_serial(&mut self, id: &str, text: &str) -> Result<(), JsError> {
        Ok(self.try_send_serial(id, text)?)
    }

    /// Sets the baud a microcontroller's serial monitor listens and sends
    /// at. Not an edit: it is the monitor's, not the circuit's.
    #[wasm_bindgen(js_name = setMonitorBaud)]
    pub fn set_monitor_baud(&mut self, id: &str, baud: u32) -> Result<(), JsError> {
        Ok(self.try_set_monitor_baud(id, baud)?)
    }

    /// Moves a potentiometer's wiper, 0 at `a` and 1 at `b` (C-20).
    ///
    /// Its own call rather than `setValue`, like a source's internal resistance
    /// and a capacitor's initial condition: they are two properties of one
    /// component, not two spellings of the same one. A number rather than text,
    /// because what sends it is a knob rather than a keyboard, and nothing about
    /// a fraction needs parsing.
    #[wasm_bindgen(js_name = setWiper)]
    pub fn set_wiper(&mut self, id: &str, wiper: f64) -> Result<(), JsError> {
        Ok(self.try_set_wiper(id, wiper)?)
    }

    /// Makes or breaks a contact — a switch flipped, a button held (C-19).
    ///
    /// An ordinary edit, taking the same path a value change does: one step of
    /// history, one bump of the revision, and the next `advance` solves the
    /// circuit that is there now. That is the whole of "a state change forces a
    /// step re-solve" — nothing is carried across a flip that could be stale,
    /// because nothing is carried at all.
    #[wasm_bindgen(js_name = setClosed)]
    pub fn set_closed(&mut self, id: &str, closed: bool) -> Result<(), JsError> {
        Ok(self.try_set_closed(id, closed)?)
    }

    /// Puts a new fuse in place of a blown one (C-120).
    #[wasm_bindgen(js_name = replaceFuse)]
    pub fn replace_fuse(&mut self, id: &str) -> Result<(), JsError> {
        Ok(self.try_replace_fuse(id)?)
    }

    /// Sets what a reactive component starts a run holding (C-18).
    ///
    /// Volts for a capacitor, amps for an inductor — the quantity the element
    /// carries between steps, which is the only one it can be given a head
    /// start in. Empty text means zero: straight out of the palette.
    #[wasm_bindgen(js_name = setInitialCondition)]
    pub fn set_initial_condition(&mut self, id: &str, text: &str) -> Result<(), JsError> {
        Ok(self.try_set_initial_condition(id, text)?)
    }

    /// Sets a source's internal resistance from text the user typed.
    ///
    /// Apart from `setValue` because they are two properties of one component
    /// rather than two spellings of the same one, and because only a source
    /// has this one. Empty text, or zero, means ideal.
    #[wasm_bindgen(js_name = setSeriesResistance)]
    pub fn set_series_resistance(&mut self, id: &str, text: &str) -> Result<(), JsError> {
        Ok(self.try_set_series_resistance(id, text)?)
    }

    /// The temperature of the room the circuit is in, in °C (C-74).
    #[wasm_bindgen(js_name = roomCelsius)]
    pub fn room_celsius(&self) -> f64 {
        self.netlist.room_celsius()
    }

    /// Puts the circuit in a room at the temperature typed, in °C (C-74).
    ///
    /// The circuit's own setting rather than any part's: every part that
    /// warms starts from it and settles back to it. Saved with the circuit,
    /// and undone like any other edit.
    #[wasm_bindgen(js_name = setRoom)]
    pub fn set_room(&mut self, text: &str) -> Result<(), JsError> {
        Ok(self.try_set_room(text)?)
    }

    /// Sets a signal generator's frequency, parsed like any value ("2", "500m").
    #[wasm_bindgen(js_name = setFrequency)]
    pub fn set_frequency(&mut self, id: &str, text: &str) -> Result<(), JsError> {
        Ok(self.try_set_frequency(id, text)?)
    }

    /// Sets the points a wire is bent through (ADR 0007).
    ///
    /// The whole list at once rather than one bend at a time: a drag produces
    /// a new shape for the wire, and the app already knows what that shape is.
    #[wasm_bindgen(js_name = setWaypoints)]
    pub fn set_waypoints(&mut self, id: &str, points: Ts<Waypoints>) -> Result<(), JsError> {
        // `Ts` rather than tsify's own ABI, which leaks (ADR 0002 and the
        // deprecation on `into_wasm_abi`).
        let points = points.to_rust()?;
        Ok(self.try_set_waypoints(id, points.0)?)
    }

    /// Turns a component a quarter turn clockwise, and reports where it now
    /// faces.
    ///
    /// A quarter at a time rather than an angle to set: four presses return a
    /// component to where it started, and no press can leave a pin off the
    /// grid.
    pub fn rotate(&mut self, id: &str) -> Result<u8, JsError> {
        Ok(self.try_rotate(id)?)
    }

    /// What the circuit is, in its author's words; empty if nobody has said.
    pub fn description(&self) -> String {
        self.description.clone()
    }

    /// Describes the circuit — what it is, what to try on it.
    ///
    /// The circuit's own, as its room is: saved with it, and undone like any
    /// other edit. Not a change to the circuit the solver sees, so a run in
    /// progress carries on. Surrounding blank space is dropped, and text that
    /// is only that is no description.
    #[wasm_bindgen(js_name = setDescription)]
    pub fn set_description(&mut self, text: &str) {
        self.try_set_description(text);
    }

    /// Empties the canvas: every part, every wire and the description, in
    /// one step of history, so an undo puts it all back.
    ///
    /// The room stays, because it is where the bench is rather than what is
    /// on it. Clearing an empty canvas changes nothing and is not a step.
    pub fn clear(&mut self) {
        self.try_clear();
    }

    /// Writes the circuit out as JSON (C-04).
    pub fn save(&self) -> Result<String, JsError> {
        crate::json::to_string(&self.to_document())
            .map_err(|error| JsError::new(&format!("could not write the circuit: {error}")))
    }

    /// Opens a circuit over this one as one step of history, so an undo puts
    /// back what was there (C-98). What Open… and the examples use: a file
    /// opened over an hour of work must not lose it.
    pub fn open(&mut self, text: &str) -> Result<(), JsError> {
        Ok(self.try_open(text)?)
    }

    /// Reads a circuit in, replacing this one entirely — history and all.
    ///
    /// For bringing back the circuit from last time, where there is nothing
    /// before it to undo to; anything opened over a circuit is [`Circuit::open`].
    pub fn load(&mut self, text: &str) -> Result<(), JsError> {
        let document: crate::document::Document =
            crate::json::from_str(text).map_err(CircuitError::Malformed)?;
        *self = Circuit::from_document(&document)?;
        Ok(())
    }

    /// Removes a wire, leaving everything it joined exactly where it was.
    ///
    /// A wire is a net (C-41). Aiming at a drawn segment and deleting only
    /// that would mean the netlist remembering which drag joined which pins,
    /// and it does not: a net is the set of pins that share a potential, and
    /// the route between them is derived. So the unit a user can delete is the
    /// net, which for every two-pin wire — every wire in the milestone circuit
    /// — is exactly the one they drew.
    #[wasm_bindgen(js_name = removeNet)]
    pub fn remove_net(&mut self, id: &str) -> Result<(), JsError> {
        Ok(self.try_remove_net(id)?)
    }

    /// Wires two pins together, and returns the net they now share.
    ///
    /// Whatever the two pins were before — both loose, one already wired, or
    /// each on a net of its own — the result is one net. Two nets that merely
    /// touch would solve as two separate circuits.
    pub fn connect(
        &mut self,
        from_element: &str,
        from_pin: &str,
        to_element: &str,
        to_pin: &str,
    ) -> Result<String, JsError> {
        Ok(self.try_connect(from_element, from_pin, to_element, to_pin)?)
    }

    /// Sets a component's value from text the user typed.
    ///
    /// Rejecting bad input is the point: an unreadable value leaves the circuit
    /// exactly as it was, so a half-typed `4k` never becomes a real edit and the
    /// simulation never sees an intermediate state (M-18).
    #[wasm_bindgen(js_name = setValue)]
    pub fn set_value(&mut self, id: &str, text: &str) -> Result<(), JsError> {
        Ok(self.try_set_value(id, text)?)
    }

    /// Advances the simulation by `elapsed` seconds of wall time.
    ///
    /// There is no run button and never was one: the caller hands over the time
    /// that passed and the scheduler decides what that is worth (C-13). An edit
    /// takes effect here, on the next advance, without anything being started
    /// or stopped.
    pub fn advance(&mut self, elapsed: f64) -> Result<(), JsError> {
        // A bouncing button's chatter is cut into the frame: the run goes as
        // far as the next flip, the contact flips, and on — each flip a change
        // of the circuit, as a click is, landing on the step it is due at
        // rather than all of them at the frame's end (C-109).
        let mut left = if elapsed.is_finite() && elapsed > 0.0 {
            elapsed
        } else {
            0.0
        };
        while let Some(next) = self.chatter.first().map(|chatter| chatter.at) {
            let wait = (next - self.run_time()).max(0.0);
            if wait > left {
                break;
            }
            if wait > 0.0 {
                // A hair over, so a millisecond asked for is a millisecond
                // stepped and not one rounded down to none.
                self.advance_by(wait + 1e-9)?;
                left -= wait;
            }
            // Behind the clock — a slow circuit, or one that failed — it
            // waits for the next frame.
            if self.run_time() + 1e-6 < next {
                break;
            }
            let due = self.chatter.remove(0);
            // Late — the scheduler owed a step and took two — the rest of
            // its chatter is late by as much, so each make and break still
            // lasts a step and none is lost between two flips at once.
            let late = self.run_time() - due.at;
            if late > 1e-6 {
                for chatter in &mut self.chatter {
                    if chatter.element == due.element {
                        chatter.at += late;
                    }
                }
            }
            self.bounce(due.element);
        }
        self.advance_by(left)?;
        self.blow();
        // The presses that have now lasted long enough, let go (C-116,
        // C-123); the rest wait for a later frame.
        for id in std::mem::take(&mut self.unseen) {
            let Some(element) = self.netlist.element(id) else {
                continue;
            };
            if !self.pressed_long_enough(id) {
                self.unseen.push(id);
                continue;
            }
            let kind = element.kind;
            let still = kind.moving() == Some(true)
                || matches!(kind, ElementKind::PushButton { closed: true, .. });
            if let Some(released) = kind.with_contact(false)
                && still
                && self.netlist.set_kind(id, released).is_ok()
            {
                self.revision += 1;
                self.chatter_after(id, released);
            }
        }
        Ok(())
    }

    /// Reduces one probe's trace to something drawable, and reports how many
    /// numbers were written (C-22).
    ///
    /// Pairs — the lowest and the highest the trace reached in each column —
    /// because a trace is nearly always longer than the place it is drawn, and
    /// one sample per column would alias a ringing circuit into whatever the
    /// sampling happened to land on.
    ///
    /// The numbers stay in wasm memory and are read through [`Circuit::plot_ptr`] (`plotPtr`)
    /// rather than copied across: this runs once per trace per frame, which is
    /// the data plane's whole reason for existing (ADR 0002).
    #[wasm_bindgen(js_name = plotTrace)]
    pub fn plot_trace(&mut self, id: &str, span: f64, columns: usize) -> Result<usize, JsError> {
        let id = decode(id)?;
        let Some(simulation) = self.simulation.as_mut() else {
            return Ok(0);
        };
        Ok(simulation.engine_mut().traces_mut().plot(id, span, columns))
    }

    /// Start of the block [`Circuit::plot_trace`] (`plotTrace`) just filled.
    ///
    /// Read after every fill and never kept: filling can grow wasm memory,
    /// which detaches every view already taken over it.
    #[wasm_bindgen(getter, js_name = plotPtr)]
    pub fn plot_ptr(&self) -> *const f64 {
        match self.simulation.as_ref() {
            Some(simulation) => simulation.engine().traces().plotted().as_ptr(),
            None => core::ptr::null(),
        }
    }

    /// When the newest sample was taken, which is where the time axis ends.
    ///
    /// The traces' own clock rather than the circuit's: a trace is drawn
    /// against the moments it was recorded at, and those stop while a circuit
    /// that will not solve is left alone.
    #[wasm_bindgen(getter, js_name = traceTime)]
    pub fn trace_time(&self) -> f64 {
        self.simulation
            .as_ref()
            .and_then(|simulation| simulation.engine().traces().latest())
            .unwrap_or(0.0)
    }

    /// How many samples one probe's trace holds, so the app can tell a trace
    /// that is empty from one that is flat.
    #[wasm_bindgen(js_name = traceDepth)]
    pub fn trace_depth(&self, id: &str) -> Result<usize, JsError> {
        let id = decode(id)?;
        Ok(self
            .simulation
            .as_ref()
            .map_or(0, |simulation| simulation.engine().traces().depth(id)))
    }

    /// What the simulation currently says.
    pub fn state(&self) -> Ts<SolutionView> {
        Ts::from_rust(&self.solution()).expect("solution view serializes")
    }

    /// Moves an element. Wires follow, because a wire is drawn from where the
    /// pins are rather than remembered separately.
    #[wasm_bindgen(js_name = moveElement)]
    pub fn move_element(&mut self, id: &str, x: f64, y: f64) -> Result<(), JsError> {
        Ok(self.try_move(id, x, y)?)
    }

    /// Marks the start of a gesture, so a drag is one step of history rather
    /// than one per frame.
    ///
    /// Records nothing by itself: the first change inside the gesture records
    /// the state it changed. A gesture that changes nothing — the pointer down
    /// that only selected something — leaves the history exactly as it was.
    #[wasm_bindgen(js_name = beginGesture)]
    pub fn begin_gesture(&mut self) {
        self.gesture = Gesture::Open;
    }

    /// Closes a gesture. Changes after this are steps of their own again.
    #[wasm_bindgen(js_name = endGesture)]
    pub fn end_gesture(&mut self) {
        self.gesture = Gesture::Closed;
    }

    /// The breadboard, with this circuit on it (B-14).
    pub fn breadboard(&self) -> Result<Ts<BoardView>, JsError> {
        Ok(Ts::from_rust(&self.try_breadboard()?).expect("a board view serializes"))
    }

    /// Moves a part on the breadboard by whole holes, `dx` along the rows and
    /// `dy` across (B-14). The schematic is not touched.
    #[wasm_bindgen(js_name = moveOnBoard)]
    pub fn move_on_board(&mut self, id: &str, dx: i32, dy: i32) -> Result<(), JsError> {
        Ok(self.try_move_on_board(id, dx, dy)?)
    }

    /// Bumped by anything that can change what is on the board without
    /// changing the elements: a move on it, an undo, an open.
    #[wasm_bindgen(js_name = boardRevision)]
    pub fn board_revision(&self) -> f64 {
        (self.revision + self.board_edits) as f64
    }

    /// The circuit as a KiCad netlist (B-16), for KiCad's PCB editor to
    /// import — or which kinds of part stop it (B-17).
    #[wasm_bindgen(js_name = kicadNetlist)]
    pub fn kicad_netlist(&self) -> Ts<KicadView> {
        Ts::from_rust(&self.kicad()).expect("a KiCad view serializes")
    }

    /// The bill of materials (B-11): what to buy to build this circuit.
    pub fn bill(&self) -> Result<Ts<Bill>, JsError> {
        Ok(Ts::from_rust(&Bill(self.try_bill()?)).expect("a bill serializes"))
    }

    /// What buying a part would mean (B-09): the value nearest the one it
    /// has that it can be bought in, the current on it, and what the rounding
    /// makes the linter say. Null for a part already at such a value.
    pub fn purchase(&self, id: &str) -> Result<Ts<PurchaseView>, JsError> {
        Ok(Ts::from_rust(&PurchaseView(self.try_purchase(id)?)).expect("a purchase serializes"))
    }

    /// Replaces a part by another kind in place (C-137): the same position
    /// and turn, keeping the wires of each pin that has a counterpart on the
    /// new one. One step of undo.
    pub fn replace(&mut self, id: &str, kind: &str) -> Result<Ts<Replacement>, JsError> {
        Ok(Ts::from_rust(&self.try_replace(id, kind)?).expect("a replacement serializes"))
    }

    /// The values either side of a resistor's or capacitor's own, for
    /// its field's − and + and the chips under it (C-136).
    pub fn steps(&self, id: &str) -> Result<Ts<Steps>, JsError> {
        Ok(Ts::from_rust(&self.try_steps(id)?).expect("steps serialize"))
    }

    /// Makes the edit a finding offers as its fix (C-135): one step of undo,
    /// however many edits it takes.
    #[wasm_bindgen(js_name = applyFix)]
    pub fn apply_fix(&mut self, fix: Ts<Fix>) -> Result<(), JsError> {
        // `Ts` rather than tsify's own ABI, which leaks (ADR 0002).
        Ok(self.try_apply_fix(&fix.to_rust()?)?)
    }

    /// Steps back. Returns whether there was anything to step back to.
    pub fn undo(&mut self) -> bool {
        self.gesture = Gesture::Closed;
        let Some(previous) = self.past.pop() else {
            return false;
        };
        let current = self.restore(previous);
        self.future.push(current);
        true
    }

    /// Steps forward again.
    pub fn redo(&mut self) -> bool {
        self.gesture = Gesture::Closed;
        let Some(next) = self.future.pop() else {
            return false;
        };
        let current = self.restore(next);
        self.past.push(current);
        true
    }

    #[wasm_bindgen(getter, js_name = canUndo)]
    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }

    #[wasm_bindgen(getter, js_name = canRedo)]
    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    /// Every net and what it joins.
    pub fn nets(&self) -> Ts<NetViews> {
        let views: Vec<NetView> = self
            .netlist
            .nets()
            .map(|(id, net)| NetView {
                id: encode_net(id),
                waypoints: self.wires.get(id).cloned().unwrap_or_default(),
                pins: net
                    .pins()
                    .iter()
                    .filter_map(|pin| {
                        let element = self.netlist.element(pin.element)?;
                        Some(PinView {
                            element_id: encode(pin.element),
                            pin: element.pins.get(pin.index)?.name.to_string(),
                        })
                    })
                    .collect(),
            })
            .collect();

        Ts::from_rust(&NetViews(views)).expect("net views serialize")
    }

    #[wasm_bindgen(getter)]
    pub fn size(&self) -> usize {
        self.netlist.element_count()
    }

    // Named explicitly: wasm-bindgen does not rename snake_case to camelCase,
    // and a `net_count` sitting beside `size` in TypeScript reads as an
    // oversight.
    #[wasm_bindgen(getter, js_name = netCount)]
    pub fn net_count(&self) -> usize {
        self.netlist.net_count()
    }
}
