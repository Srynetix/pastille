// SPDX-License-Identifier: MIT OR Apache-2.0

//! A circuit as a file.
//!
//! C-04. The whole `Circuit` rather than the netlist alone, because ADR 0006
//! put the layout beside the netlist for exactly this reason: a reload that
//! restores the connections while scattering the drawing is not a reload.
//!
//! The format names elements by their position in a list rather than by their
//! handle. A handle is a slotmap key — an index and a generation packed into
//! 64 bits — and while it would round-trip, it would put the core's internals
//! in a file people keep, and make a hand-edited or generated file nearly
//! impossible to write. Loading builds fresh handles; nothing outside the
//! running session ever sees one.

use serde::{Deserialize, Serialize};

use crate::ACCURACY;
use crate::circuit::{Circuit, CircuitError, Placement, PointView};
use crate::netlist::ElementKind;

/// The format this build writes, and the only one it reads.
///
/// An integer rather than a semantic version: a file either has a shape this
/// code knows or it does not, and there is no partial credit.
pub const SCHEMA: u32 = 1;

/// A saved circuit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub schema: u32,
    /// What produced the numbers in it, in the app's own words (C-26).
    ///
    /// The file is the one thing v1 makes that outlives the session and can be
    /// read somewhere the app's status bar is not — mailed to somebody, opened
    /// in an editor, fed to the runner. The claim that this is an educational
    /// simulation travels with it rather than staying on the screen it was
    /// made on.
    ///
    /// Written every time, read never: a file that says something else is a
    /// file somebody edited, and it is not for this build to argue with them
    /// about it. Defaulted so that every file written before this existed
    /// still opens.
    #[serde(default)]
    pub note: String,
    /// The temperature of the room the circuit was built in, in °C (C-74).
    ///
    /// Defaulted, so a file written before rooms existed opens in the room
    /// every circuit was in until then.
    #[serde(default = "default_room")]
    pub room_celsius: f64,
    /// What the circuit is, in its author's words. Absent from a file whose
    /// circuit nobody described, and from every file written before there
    /// was one to write.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    pub elements: Vec<SavedElement>,
    pub nets: Vec<SavedNet>,
}

fn default_room() -> f64 {
    crate::netlist::ROOM_CELSIUS
}

/// One component: what it is, and where it sits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedElement {
    pub kind: ElementKind,
    pub x: f64,
    pub y: f64,
    /// Quarter turns clockwise (C-40).
    #[serde(default)]
    pub turns: u8,
    /// Where it has been moved on the breadboard, each pin's hole (B-14).
    /// Absent for a part the placer puts, which is most of them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub board: Option<Vec<Option<crate::breadboard::Hole>>>,
    /// The program a microcontroller runs (A-03). Absent for every other
    /// part, and for an Uno with none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub program: Option<SavedProgram>,
}

/// A microcontroller's program as a file holds it (A-03): its name, and the
/// Intel HEX it was loaded as — text, so the file stays one a person can
/// read, and exactly what was loaded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedProgram {
    pub name: String,
    pub hex: String,
}

/// One net: what it joins, and where its wire was bent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedNet {
    pub pins: Vec<SavedPin>,
    /// Points the wire passes through (ADR 0007). Absent in a file that has
    /// none, which is most of them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub waypoints: Vec<PointView>,
}

/// One end of a net: which element, and which of its pins.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedPin {
    /// Position in the document's `elements`, not a handle.
    pub element: usize,
    pub pin: String,
}

impl Circuit {
    /// Writes the circuit out.
    pub fn to_document(&self) -> Document {
        let order: Vec<_> = self.netlist().elements().map(|(id, _)| id).collect();
        let position = |wanted| order.iter().position(|&id| id == wanted);

        let elements = order
            .iter()
            .map(|&id| {
                let element = self.netlist().element(id).expect("it was just listed");
                let at = self.placement_of(id).unwrap_or(Placement {
                    x: 0.0,
                    y: 0.0,
                    turns: 0,
                });
                SavedElement {
                    kind: element.kind,
                    x: at.x,
                    y: at.y,
                    turns: at.turns,
                    board: self.board_of(id),
                    program: self.netlist().program(id).map(|program| SavedProgram {
                        name: program.name.clone(),
                        hex: program.hex.clone(),
                    }),
                }
            })
            .collect();

        let nets = self
            .netlist()
            .nets()
            .map(|(id, net)| SavedNet {
                pins: net
                    .pins()
                    .iter()
                    .filter_map(|pin| {
                        let element = self.netlist().element(pin.element)?;
                        Some(SavedPin {
                            element: position(pin.element)?,
                            pin: element.pins.get(pin.index)?.name.to_string(),
                        })
                    })
                    .collect(),
                waypoints: self.waypoints_of(id).to_vec(),
            })
            .collect();

        Document {
            schema: SCHEMA,
            note: ACCURACY.to_string(),
            room_celsius: self.netlist().room_celsius(),
            description: self.description(),
            elements,
            nets,
        }
    }

    /// Reads a circuit in, replacing whatever was there.
    ///
    /// Everything, including the history: there is no undoing past an open,
    /// because what came before belonged to a different circuit.
    pub fn from_document(document: &Document) -> Result<Circuit, CircuitError> {
        if document.schema != SCHEMA {
            return Err(CircuitError::UnknownSchema(document.schema));
        }

        // What a file holds is checked as what is typed is (C-96): a part no
        // edit could make is not one a file can make either.
        let (coldest, warmest) = crate::circuit::ROOM_RANGE;
        if !(coldest..=warmest).contains(&document.room_celsius) {
            return Err(CircuitError::Refused(format!(
                "This file cannot be opened: its room is at {} °C, and a room is between \
                 {coldest} °C and {warmest} °C.",
                document.room_celsius
            )));
        }
        for (index, element) in document.elements.iter().enumerate() {
            crate::check::check(&element.kind).map_err(|sentence| {
                CircuitError::Refused(format!(
                    "This file cannot be opened: part {} in it is wrong. {sentence}",
                    index + 1
                ))
            })?;
        }

        let mut circuit = Circuit::new();
        circuit.place_in_room(document.room_celsius);
        circuit.describe_as_loaded(&document.description);
        let mut handles = Vec::with_capacity(document.elements.len());

        for element in &document.elements {
            let id = circuit.place(element.kind, element.x, element.y, element.turns);
            if let Some(holes) = &element.board {
                circuit.pin_on_board(id, holes.clone());
            }
            if let Some(saved) = &element.program {
                circuit.load_program(id, &saved.name, &saved.hex)?;
            }
            handles.push(id);
        }

        for net in &document.nets {
            let pins: Vec<_> = net
                .pins
                .iter()
                .map(|pin| {
                    handles
                        .get(pin.element)
                        .copied()
                        .map(|id| (id, pin.pin.clone()))
                        .ok_or_else(|| {
                            CircuitError::Malformed(format!(
                                "a wire joins element {}, and there are only {}",
                                pin.element,
                                document.elements.len()
                            ))
                        })
                })
                .collect::<Result<_, _>>()?;

            circuit.rejoin(&pins, &net.waypoints)?;
        }

        Ok(circuit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::netlist::LedColour;

    /// The milestone circuit, built the way the app builds one.
    fn milestone() -> Circuit {
        let mut circuit = Circuit::new();
        let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
        let resistor = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
        let led = circuit.try_add("Led", 160.0, 0.0).unwrap();
        circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
        circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
        circuit.try_connect(&led, "cathode", &battery, "-").unwrap();
        circuit
    }

    fn written(circuit: &Circuit) -> String {
        serde_json::to_string(&circuit.to_document()).unwrap()
    }

    fn read(text: &str) -> Result<Circuit, CircuitError> {
        let document: Document = serde_json::from_str(text)
            .map_err(|error| CircuitError::Malformed(error.to_string()))?;
        Circuit::from_document(&document)
    }

    #[test]
    fn a_file_the_browser_wrote_opens_here_too() {
        // In the browser a circuit is written by the browser's own JSON
        // (src/json.rs), which spells a whole number without its `.0`. The
        // runner and every native reader still have to open it.
        let browser = written(&milestone())
            .replace(".0,", ",")
            .replace(".0}", "}");
        assert!(browser.contains("\"ohms\":470}"), "{browser}");

        let opened = read(&browser).unwrap();

        assert_eq!(opened.to_document(), milestone().to_document());
    }

    #[test]
    fn a_saved_circuit_says_what_made_it() {
        // C-26. The file leaves the app, and the sentence in the status bar
        // does not go with it unless it is written down here.
        let text = written(&milestone());

        assert!(text.contains(ACCURACY), "{text}");
    }

    #[test]
    fn a_file_with_a_part_no_edit_could_make_is_refused_and_says_which() {
        // C-96: what a file holds is checked as what is typed is.
        let text = written(&milestone()).replace("\"ohms\":470.0", "\"ohms\":0.0");
        assert!(text.contains("\"ohms\":0.0"), "{text}");
        let refused = read(&text).err().unwrap().to_string();
        assert!(refused.contains("part 2"), "{refused}");
        assert!(refused.contains("more than 0 Ω"), "{refused}");
    }

    #[test]
    fn a_file_in_a_room_no_circuit_is_built_in_is_refused() {
        let text = written(&milestone()).replace("\"room_celsius\":25.0", "\"room_celsius\":1e9");
        assert!(text.contains("1e9"), "{text}");
        let refused = read(&text).err().unwrap().to_string();
        assert!(refused.contains("room"), "{refused}");
    }

    #[test]
    fn a_circuit_opens_with_the_description_it_was_saved_with() {
        let mut circuit = milestone();
        circuit.try_set_description("  The first circuit: watch the LED.\n");

        let reopened = read(&written(&circuit)).unwrap();

        assert_eq!(reopened.description(), "The first circuit: watch the LED.");
        assert!(!reopened.can_undo(), "opening it is not an edit");
    }

    #[test]
    fn a_circuit_nobody_described_writes_no_description() {
        // And a file from before descriptions existed opens with none.
        let text = written(&milestone());
        assert!(!text.contains("description"), "{text}");
        assert_eq!(read(&text).unwrap().description(), "");
    }

    #[test]
    fn a_circuit_opens_in_the_room_it_was_saved_in() {
        // C-74. The room is the circuit's, so it goes where the circuit goes.
        let mut circuit = milestone();
        circuit.try_set_room("35").unwrap();

        let reopened = read(&written(&circuit)).unwrap();

        assert_eq!(reopened.netlist().room_celsius(), 35.0);
    }

    #[test]
    fn a_file_written_before_rooms_existed_opens_at_25_degrees() {
        let text = r#"{"schema":1,"elements":[],"nets":[]}"#;

        let circuit = read(text).expect("a file without a room opens");

        assert_eq!(circuit.netlist().room_celsius(), 25.0);
    }

    #[test]
    fn a_file_written_before_the_note_existed_still_opens() {
        // Every circuit saved up to now. The field defaults rather than being
        // required, because a note is something the writer adds and never
        // something a reader should insist on.
        let text = r#"{"schema":1,"elements":[{"kind":{"Resistor":{"ohms":220.0}},"x":0.0,"y":0.0,"turns":0}],"nets":[]}"#;

        let circuit = read(text).expect("a file without a note opens");

        assert_eq!(circuit.netlist().element_count(), 1);
    }

    #[test]
    fn a_note_somebody_edited_is_not_argued_with() {
        // It is read into the document and then ignored: the file is theirs,
        // and what this build writes next is this build's claim, not theirs.
        let text = r#"{"schema":1,"note":"something else entirely","elements":[],"nets":[]}"#;

        let circuit = read(text).expect("a file with another note opens");

        assert!(written(&circuit).contains(ACCURACY));
    }

    #[test]
    fn a_circuit_survives_the_round_trip_byte_for_byte() {
        // The acceptance. Not "looks the same" — the same bytes, so a saved
        // file opened and saved again shows no diff.
        let before = written(&milestone());

        let after = written(&read(&before).unwrap());

        assert_eq!(before, after);
    }

    #[test]
    fn everything_comes_back() {
        let saved = written(&milestone());

        let reloaded = read(&saved).unwrap();

        assert_eq!(reloaded.size(), 3);
        assert_eq!(reloaded.net_count(), 3);
    }

    #[test]
    fn a_reloaded_circuit_still_solves() {
        // The point of saving a layout beside the netlist (ADR 0006): a reload
        // that restores the connections and scatters the drawing is not a
        // reload, and one that restores neither is not a circuit.
        let mut reloaded = read(&written(&milestone())).unwrap();

        reloaded.advance(0.016).unwrap();

        let state = reloaded.solution();
        assert!(state.failure.is_none(), "{:?}", state.failure);
        assert!(
            state.element_currents.iter().any(|c| c.amps.abs() > 1e-6),
            "current should be flowing"
        );
    }

    #[test]
    fn positions_and_orientations_come_back() {
        let mut circuit = Circuit::new();
        let resistor = circuit.try_add("Resistor", 120.0, -40.0).unwrap();
        circuit.try_rotate(&resistor).unwrap();

        let reloaded = read(&written(&circuit)).unwrap();

        let document = reloaded.to_document();
        assert_eq!(document.elements[0].x, 120.0);
        assert_eq!(document.elements[0].y, -40.0);
        assert_eq!(document.elements[0].turns, 1);
    }

    #[test]
    fn a_turn_count_past_a_full_turn_is_read_as_the_drawing_reads_it() {
        // Nothing writes 255, but a file is text anybody can edit, and a 255
        // used to overflow the next rotation.
        let text = r#"{"schema":1,"elements":[{"kind":{"Resistor":{"ohms":220.0}},"x":0.0,"y":0.0,"turns":255}],"nets":[]}"#;

        let mut circuit = read(text).expect("the file opens");
        assert_eq!(circuit.to_document().elements[0].turns, 3);

        let id = circuit.elements_for_test()[0].id.clone();
        assert_eq!(circuit.try_rotate(&id).unwrap(), 0);
    }

    #[test]
    fn a_bent_wire_comes_back_bent() {
        let mut circuit = milestone();
        let net = circuit.nets_for_test()[0].clone();
        circuit
            .try_set_waypoints(&net, vec![PointView { x: 40.0, y: -60.0 }])
            .unwrap();

        let reloaded = read(&written(&circuit)).unwrap();

        let document = reloaded.to_document();
        let bent: Vec<_> = document
            .nets
            .iter()
            .filter(|net| !net.waypoints.is_empty())
            .collect();
        assert_eq!(bent.len(), 1);
        assert_eq!(bent[0].waypoints, vec![PointView { x: 40.0, y: -60.0 }]);
    }

    #[test]
    fn a_straight_wire_writes_no_waypoints_at_all() {
        // Not an empty list in every net of every file: the common case should
        // cost nothing to store and nothing to read past.
        assert!(!written(&milestone()).contains("waypoints"));
    }

    #[test]
    fn values_come_back_exactly() {
        let mut circuit = Circuit::new();
        let led = circuit.try_add("Led", 0.0, 0.0).unwrap();
        let capacitor = circuit.try_add("Capacitor", 40.0, 0.0).unwrap();
        circuit.try_set_value(&capacitor, "4n7").unwrap();
        circuit.try_set_initial_condition(&capacitor, "3").unwrap();
        let _ = led;

        let reloaded = read(&written(&circuit)).unwrap();

        // Exactly as saved, which is not the same as exactly 4.7 nF: `4n7`
        // parses to 4.700000000000001e-9 and the file has to carry that back
        // unchanged rather than tidy it.
        let ElementKind::Capacitor {
            farads,
            initial_volts,
        } = reloaded.to_document().elements[1].kind
        else {
            panic!("not a capacitor")
        };
        assert_eq!(farads, crate::value::parse("4n7").unwrap());
        assert_eq!(initial_volts, 3.0);
    }

    #[test]
    fn an_led_keeps_its_colour() {
        let mut circuit = Circuit::new();
        circuit.try_add("Led", 0.0, 0.0).unwrap();

        let reloaded = read(&written(&circuit)).unwrap();

        assert_eq!(
            reloaded.to_document().elements[0].kind,
            ElementKind::Led {
                colour: LedColour::Red
            }
        );
    }

    #[test]
    fn a_file_from_a_newer_version_says_so() {
        // The acceptance's other half: a readable message rather than a
        // silent mis-load.
        let saved = written(&milestone()).replace("\"schema\":1", "\"schema\":2");

        let refused = read(&saved);

        assert_eq!(refused.err(), Some(CircuitError::UnknownSchema(2)));
    }

    #[test]
    fn the_message_says_which_format_it_reads() {
        let message = CircuitError::UnknownSchema(7).to_string();

        assert!(message.contains('7'), "{message}");
        assert!(message.contains('1'), "{message}");
    }

    #[test]
    fn a_wire_to_an_element_that_is_not_there_is_refused() {
        let saved = r#"{"schema":1,"elements":[],"nets":[{"pins":[{"element":3,"pin":"a"}]}]}"#;

        let refused = read(saved);

        assert!(matches!(refused.err(), Some(CircuitError::Malformed(_))));
    }

    #[test]
    fn a_wire_to_a_pin_that_is_not_there_is_refused() {
        let saved = r#"{"schema":1,"elements":[{"kind":{"Resistor":{"ohms":220.0}},"x":0.0,"y":0.0,"turns":0}],"nets":[{"pins":[{"element":0,"pin":"anode"}]}]}"#;

        assert!(read(saved).is_err());
    }

    #[test]
    fn something_that_is_not_a_circuit_at_all_is_refused() {
        assert!(matches!(
            read("not json").err(),
            Some(CircuitError::Malformed(_))
        ));
    }

    #[test]
    fn an_empty_canvas_saves_and_reloads() {
        let empty = written(&Circuit::new());

        let reloaded = read(&empty).unwrap();

        assert_eq!(reloaded.size(), 0);
        assert_eq!(written(&reloaded), empty);
    }
}
