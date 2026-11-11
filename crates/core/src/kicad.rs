// SPDX-License-Identifier: MIT OR Apache-2.0

//! The circuit as a KiCad netlist (B-16, B-17, design doc §6.4).
//!
//! "The circuit graduates rather than being trapped in the app." A KiCad
//! netlist is what KiCad's PCB editor imports (File → Import → Netlist): each
//! part with its reference, its value, a KiCad symbol and a through-hole
//! footprint, and each net by the pins it joins. Imported, the footprints
//! arrive joined by their ratsnest, ready to lay out. (KiCad's schematic
//! editor does not read a netlist; a schematic file is a ticket of its own —
//! Denis, 2026-10-03.)
//!
//! **Every name is one KiCad's own libraries have**, symbol and footprint
//! alike, checked against them (see [`mapping`]), and every pin is numbered as
//! the footprint's pads are — an LED's cathode is pad 1, a 2N3904's emitter
//! is. Every part the palette offers has one (B-22); a kind with none would
//! **stop the export with a sentence naming it** (B-17), rather than write a
//! file that imports with a hole in it.
//!
//! What the board has and the schematic leaves out is written in as on the
//! breadboard (B-15): a 555's RESET on VCC's net. Every ground is one net,
//! `GND` (ADR 0003). The bench is not on a PCB: a battery or a signal
//! generator is the two-pin connector it plugs into, an ammeter the two pins
//! its leads go in, and a voltmeter, which joins nothing, is left out.

use crate::breadboard;
use crate::netlist::{ElementId, ElementKind, NetId, Netlist};

/// How one kind of part goes into KiCad.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mapping {
    /// The reference's letters: "R", "C", "U".
    pub prefix: &'static str,
    /// The symbol, `library:name`.
    pub symbol: &'static str,
    /// The footprint, `library:name`.
    pub footprint: &'static str,
    /// Each of the model's pins, in its order, as the footprint numbers its
    /// pad; `None` for a chip, whose pins are its package's (B-13).
    pub pads: Option<&'static [&'static str]>,
    /// Pads the package joins inside to one of the model's pins, each with
    /// that pin's index: a digit's second common pin, an L293D's other GNDs.
    /// They go on its net, so nothing reads as free that is not.
    pub also: &'static [(&'static str, usize)],
}

/// A part the export cannot write, by what it is.
#[derive(Debug, Clone, PartialEq)]
pub struct Unmapped {
    pub kinds: Vec<&'static str>,
}

/// What a part is in KiCad, or `None` for one that is the bench's alone.
///
/// Every symbol and footprint here was read off KiCad 9's own libraries (tag
/// 9.0.9.1 of kicad-symbols and kicad-footprints, 2026-10-03), and every pad
/// number with it — they are not the model's pin order: an LED's cathode is
/// pad 1, a 2N3904 is E-B-C on pads 1-2-3, a BS250 is D-G-S where a 2N7000 is
/// S-G-D, an LM317 is ADJ-OUT-IN. B-17 checked the v1 set (§5), B-22 the
/// parts added since.
///
/// The `match` names every kind, so a part added later does not compile
/// until it has a mapping. Where KiCad has no symbol of the part's own it
/// stands in for, the nearest one with the same pins is used and the comment
/// says so; where the part is a module or something wired off the board — a
/// PIR module, a servo, a motor, a transformer — it is the header its leads
/// go in, as a battery is.
pub fn mapping(kind: &ElementKind) -> Option<Mapping> {
    use crate::solver::device::digit::Common;
    use crate::solver::device::fuse::FusePart;
    use crate::solver::device::gate::Logic;
    use crate::solver::device::{Channel, Polarity, RegulatorPart};
    const TO92: &str = "Package_TO_SOT_THT:TO-92_Inline";
    const TO220: &str = "Package_TO_SOT_THT:TO-220-3_Vertical";
    const DIP8: &str = "Package_DIP:DIP-8_W7.62mm";
    const DIP14: &str = "Package_DIP:DIP-14_W7.62mm";
    const DIP16: &str = "Package_DIP:DIP-16_W7.62mm";
    const HEADER: &str = "Connector_PinHeader_2.54mm:PinHeader_1x02_P2.54mm_Vertical";
    const HEADER3: &str = "Connector_PinHeader_2.54mm:PinHeader_1x03_P2.54mm_Vertical";
    let map = |prefix, symbol, footprint, pads: &'static [&'static str]| Mapping {
        prefix,
        symbol,
        footprint,
        pads: Some(pads),
        also: &[],
    };
    let chip = |symbol, footprint| Mapping {
        prefix: "U",
        symbol,
        footprint,
        pads: None,
        also: &[],
    };
    Some(match *kind {
        ElementKind::Resistor { .. } => map(
            "R",
            "Device:R",
            "Resistor_THT:R_Axial_DIN0207_L6.3mm_D2.5mm_P10.16mm_Horizontal",
            &["1", "2"],
        ),
        ElementKind::Capacitor { .. } => map(
            "C",
            "Device:C",
            "Capacitor_THT:C_Disc_D5.0mm_W2.5mm_P5.00mm",
            &["1", "2"],
        ),
        // + is pad 1.
        ElementKind::Electrolytic { .. } => map(
            "C",
            "Device:C_Polarized",
            "Capacitor_THT:CP_Radial_D5.0mm_P2.00mm",
            &["1", "2"],
        ),
        ElementKind::Inductor { .. } => map(
            "L",
            "Device:L",
            "Inductor_THT:L_Axial_L5.3mm_D2.2mm_P10.16mm_Horizontal_Vishay_IM-1",
            &["1", "2"],
        ),
        // Anode, cathode: K is pad 1.
        ElementKind::Led { .. } => map("D", "Device:LED", "LED_THT:LED_D5.0mm", &["2", "1"]),
        ElementKind::Diode => map(
            "D",
            "Diode:1N4148",
            "Diode_THT:D_DO-35_SOD27_P7.62mm_Horizontal",
            &["2", "1"],
        ),
        ElementKind::Schottky => map(
            "D",
            "Diode:1N5819",
            "Diode_THT:D_DO-41_SOD81_P10.16mm_Horizontal",
            &["2", "1"],
        ),
        // The 1N4728A, 1N4733A and 1N4739A are one KiCad symbol, the
        // family's; K is pad 1.
        ElementKind::Zener { .. } => map(
            "D",
            "Diode:1N47xxA",
            "Diode_THT:D_DO-41_SOD81_P10.16mm_Horizontal",
            &["2", "1"],
        ),
        // AC, AC, +, −. KiCad has no DB107; the DF10M is the same 1 A,
        // 1000 V bridge in the same four-pin DIP, + on pad 1 and − on 2.
        ElementKind::Bridge => map(
            "D",
            "Diode_Bridge:DF10M",
            "Diode_THT:Diode_Bridge_DIP-4_W7.62mm_P5.08mm",
            &["3", "4", "1", "2"],
        ),
        // Base, collector, emitter.
        ElementKind::Bjt { polarity } => match polarity {
            Polarity::Npn => map("Q", "Transistor_BJT:2N3904", TO92, &["2", "3", "1"]),
            Polarity::Pnp => map("Q", "Transistor_BJT:2N3906", TO92, &["2", "3", "1"]),
            Polarity::Darlington => map("Q", "Transistor_BJT:TIP120", TO220, &["1", "2", "3"]),
        },
        // Gate, drain, source.
        ElementKind::Mosfet { channel } => match channel {
            Channel::N => map("Q", "Transistor_FET:2N7000", TO92, &["2", "3", "1"]),
            Channel::P => map("Q", "Transistor_FET:BS250", TO92, &["2", "1", "3"]),
            Channel::Power => map("Q", "Transistor_FET:IRLZ44N", TO220, &["1", "2", "3"]),
        },
        // Collector, emitter: a 5 mm clear case, C on pad 1 by the flat.
        ElementKind::Phototransistor { .. } => {
            map("Q", "Device:Q_Photo_NPN", "LED_THT:LED_D5.0mm", &["1", "2"])
        }
        // In, ground, out.
        ElementKind::Regulator { part } => match part {
            RegulatorPart::L7805 => map("U", "Regulator_Linear:L7805", TO220, &["1", "2", "3"]),
            RegulatorPart::L7812 => map("U", "Regulator_Linear:L7812", TO220, &["1", "2", "3"]),
            // The through-hole one, in a TO-220: GND, VO, VI on 1-2-3.
            RegulatorPart::Lm1117 => {
                map("U", "Regulator_Linear:LM1117T-3.3", TO220, &["3", "1", "2"])
            }
            // Not offered as a fixed regulator's choice (C-113), but its pins
            // are the adjustable one's: in, adjust, out.
            RegulatorPart::Lm317 => map(
                "U",
                "Regulator_Linear:LM317_TO-220",
                TO220,
                &["3", "1", "2"],
            ),
        },
        // In, adjust, out.
        ElementKind::Adjustable => map(
            "U",
            "Regulator_Linear:LM317_TO-220",
            TO220,
            &["3", "1", "2"],
        ),
        // +Vs, Vout, GND. KiCad has the TMP36 only in SOIC; the LM35 in a
        // TO-92 is its symbol with the same three pins in the same order.
        ElementKind::Tmp36 { .. } => map("U", "Sensor_Temperature:LM35-LP", TO92, &["1", "2", "3"]),
        // VCC, GND, OUT. KiCad has no A3144; the A1101 is Allegro's
        // unipolar Hall switch in the same three-pin case, pinned the same.
        ElementKind::Hall { .. } => map(
            "U",
            "Sensor_Magnetic:A1101xUA",
            "Package_TO_SOT_THT:TO-92S",
            &["1", "2", "3"],
        ),
        ElementKind::Timer => chip("Timer:NE555P", DIP8),
        ElementKind::OpAmp => chip("Amplifier_Operational:LM358", DIP8),
        ElementKind::Comparator => chip("Comparator:LM393", DIP8),
        ElementKind::Optocoupler => chip("Isolator:4N35", "Package_DIP:DIP-6_W7.62mm"),
        // Its four GND pins are one inside, as its heat sink: all on GND.
        ElementKind::MotorDriver => Mapping {
            also: &[("5", 12), ("12", 12), ("13", 12)],
            ..chip("Driver_Motor:L293D", DIP16)
        },
        // KiCad has the 74HC08 and 74HC32 only as their 74LS twins, whose
        // pins are the same.
        ElementKind::Gate { logic } => match logic {
            Logic::Nand => chip("74xx:74HC00", DIP14),
            Logic::And => chip("74xx:74LS08", DIP14),
            Logic::Or => chip("74xx:74LS32", DIP14),
            Logic::Xor => chip("74xx:74HC86", DIP14),
        },
        ElementKind::Inverter => chip("74xx:74HC04", DIP14),
        ElementKind::Schmitt => chip("74xx:74HC14", DIP14),
        ElementKind::ShiftRegister => chip("74xx:74HC595", DIP16),
        ElementKind::Decade => chip("4xxx:4017", DIP16),
        ElementKind::FlipFlop => chip("4xxx:4013", DIP14),
        ElementKind::Decoder => chip("4xxx_IEEE:4511", DIP16),
        // KiCad has no 4026: a sixteen-pin DIP numbered as one.
        ElementKind::Counter => chip("Connector_Generic:Conn_02x08_Counter_Clockwise", DIP16),
        ElementKind::Lm3914 => chip("Driver_LED:LM3914N", "Package_DIP:DIP-18_W7.62mm"),
        ElementKind::Switch { .. } => map(
            "SW",
            "Switch:SW_SPST",
            "Button_Switch_THT:SW_DIP_SPSTx01_Slide_9.78x4.72mm_W7.62mm_P2.54mm",
            &["1", "2"],
        ),
        ElementKind::PushButton { .. } => map(
            "SW",
            "Switch:SW_Push",
            "Button_Switch_THT:SW_PUSH_6mm",
            &["1", "2"],
        ),
        // KiCad has neither a tilt switch's symbol nor its footprint: a
        // contact, its two legs in holes 2.54 mm apart.
        ElementKind::TiltSwitch { .. } => map("SW", "Switch:SW_SPST", HEADER, &["1", "2"]),
        // No through-hole reed footprint either: an axial body as long as
        // its glass, its leads bent down.
        ElementKind::Reed { .. } => map(
            "SW",
            "Switch:SW_Reed",
            "Resistor_THT:R_Axial_DIN0414_L11.9mm_D4.5mm_P20.32mm_Horizontal",
            &["1", "2"],
        ),
        // Coil, coil, COM, NO, NC: the coil between pads 2 and 5, the blade
        // on 1 resting on 4.
        ElementKind::Relay { .. } => map(
            "K",
            "Relay:SANYOU_SRD_Form_C",
            "Relay_THT:Relay_SPDT_SANYOU_SRD_Series_Form_C",
            &["5", "2", "1", "3", "4"],
        ),
        // One end, the wiper, the other.
        ElementKind::Potentiometer { .. } => map(
            "RV",
            "Device:R_Potentiometer",
            "Potentiometer_THT:Potentiometer_Bourns_3386P_Vertical",
            &["1", "2", "3"],
        ),
        // A 5 mm cell, as the GL5528 is.
        ElementKind::Ldr { .. } => map(
            "R",
            "Device:R_Photo",
            "OptoDevice:R_LDR_5.1x4.3mm_P3.4mm_Vertical",
            &["1", "2"],
        ),
        // A bead on two legs: no thermistor footprint, a standing resistor's.
        ElementKind::Thermistor { .. } => map(
            "TH",
            "Device:Thermistor_NTC",
            "Resistor_THT:R_Axial_DIN0204_L3.6mm_D1.6mm_P2.54mm_Vertical",
            &["1", "2"],
        ),
        // A 5 × 20 mm cartridge in its clips, or a radial polyfuse.
        ElementKind::Fuse { part, .. } => match part {
            FusePart::Fast => map(
                "F",
                "Device:Fuse",
                "Fuse:Fuseholder_Clip-5x20mm_Littelfuse_111_Inline_P20.00x5.00mm_D1.05mm_Horizontal",
                &["1", "2"],
            ),
            FusePart::Polyfuse => map(
                "F",
                "Device:Polyfuse",
                "Fuse:Fuse_Bourns_MF-RG300",
                &["1", "2"],
            ),
        },
        // + is pad 1, active or passive.
        ElementKind::Buzzer | ElementKind::PassiveBuzzer => map(
            "BZ",
            "Device:Buzzer",
            "Buzzer_Beeper:Buzzer_12x9.5RM7.6",
            &["1", "2"],
        ),
        // R, G, B, the common pin, in a 5 mm four-legged case.
        ElementKind::RgbLed { common } => match common {
            Common::Cathode => map(
                "D",
                "Device:LED_RGBK",
                "LED_THT:LED_D5.0mm-4_RGB",
                &["1", "2", "3", "4"],
            ),
            Common::Anode => map(
                "D",
                "Device:LED_ARGB",
                "LED_THT:LED_D5.0mm-4_RGB",
                &["2", "3", "4", "1"],
            ),
        },
        // a–g, dp, common: a 0.39" red digit, its common on pads 3 and 8.
        ElementKind::SevenSegment { common } => Mapping {
            also: &[("8", 8)],
            ..map(
                "U",
                match common {
                    Common::Cathode => "Display_Character:SC39-11EWA",
                    Common::Anode => "Display_Character:SA39-11EWA",
                },
                "Display_7Segment:Sx39-1xxxxx",
                &["10", "9", "7", "5", "4", "2", "1", "6", "3"],
            )
        },
        // Ten red bars: each anode on pad k, its cathode on 21 − k.
        ElementKind::Bargraph => map(
            "BAR",
            "LED:HDSP-4830",
            "Display:HDSP-4830",
            &[
                "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "20", "19", "18", "17", "16",
                "15", "14", "13", "12", "11",
            ],
        ),
        // Off the board, by the header their leads go in. A motor and a
        // servo by their own symbols; a solar panel by its.
        ElementKind::Motor => map("M", "Motor:Motor_DC", HEADER, &["1", "2"]),
        // Signal, +, −, as a servo's plug runs.
        ElementKind::Servo => map("M", "Motor:Motor_Servo", HEADER3, &["1", "2", "3"]),
        ElementKind::SolarCell { .. } => map("SC", "Device:Solar_Cell", HEADER, &["1", "2"]),
        // A PIR module's own header: VCC, OUT, GND.
        ElementKind::Pir { .. } => map(
            "J",
            "Connector_Generic:Conn_01x03",
            HEADER3,
            &["1", "2", "3"],
        ),
        // Primary, then secondary; its four leads to a header.
        ElementKind::Transformer { .. } => map(
            "J",
            "Connector_Generic:Conn_01x04",
            "Connector_PinHeader_2.54mm:PinHeader_1x04_P2.54mm_Vertical",
            &["1", "2", "3", "4"],
        ),
        // A bare liquid-crystal digit has no KiCad symbol: its nine pins, in
        // the model's order, to the header it sits in.
        ElementKind::LcdDigit => map(
            "J",
            "Connector_Generic:Conn_01x09",
            "Connector_PinHeader_2.54mm:PinHeader_1x09_P2.54mm_Vertical",
            &["1", "2", "3", "4", "5", "6", "7", "8", "9"],
        ),
        // A shield on the Uno's headers: D0–D13 on pads 15–28, A0–A3 on
        // 9–12, A4 and A5 on 13 and 14 (SDA, SCL), 5V on 5, 3V3 on 4, VIN
        // on 8, GND on 6 — `MCU_Module:Arduino_UNO_R3` in KiCad 9.0.9.1,
        // checked 2026-10-04.
        ElementKind::Uno => map(
            "A",
            "MCU_Module:Arduino_UNO_R3",
            "Module:Arduino_UNO_R3",
            &[
                "15", "16", "17", "18", "19", "20", "21", "22", "23", "24", "25", "26", "27", "28",
                "9", "10", "11", "12", "13", "14", "5", "4", "8", "6",
            ],
        ),
        // The bench on a PCB: the two pins it plugs into.
        ElementKind::DcSource { .. }
        | ElementKind::Cell { .. }
        | ElementKind::SignalGenerator { .. }
        | ElementKind::Ammeter => map("J", "Connector_Generic:Conn_01x02", HEADER, &["1", "2"]),
        ElementKind::Ground | ElementKind::Voltmeter => return None,
    })
}

/// Parts that are the bench's, not the circuit's: left off a PCB.
fn bench_only(kind: &ElementKind) -> bool {
    matches!(kind, ElementKind::Ground | ElementKind::Voltmeter)
}

/// One part as both exports write it: its reference, value, mapping, and
/// every pin of its footprint with its name and net.
#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    pub reference: String,
    pub value: String,
    pub mapping: Mapping,
    /// Every pad, in the order a symbol lists them: a chip's by package
    /// number, 1 to n, its free pins included; anything else in the model's
    /// pin order.
    pub pins: Vec<Pin>,
}

/// One pin of a part.
#[derive(Debug, Clone, PartialEq)]
pub struct Pin {
    /// The pad number, as the footprint has it.
    pub number: String,
    /// Its name: the model's, or the package's for a pin the model leaves out.
    pub name: String,
    /// The net it is on: `GND` for every ground, `N1`, `N2` … otherwise.
    pub net: Option<String>,
}

/// What both exports write: every part that goes on a PCB, in the order it
/// was placed, with its pins and their nets — or the kinds that stop it.
pub fn plan(
    netlist: &Netlist,
    value: impl Fn(&ElementKind) -> String,
) -> Result<Vec<Part>, Unmapped> {
    let parts: Vec<(ElementId, &crate::netlist::Element)> = netlist
        .elements()
        .filter(|(_, element)| !bench_only(&element.kind))
        .collect();
    let mut unmapped: Vec<&'static str> = parts
        .iter()
        .filter(|(_, element)| mapping(&element.kind).is_none())
        .map(|(_, element)| element.kind.name())
        .collect();
    unmapped.sort_unstable();
    unmapped.dedup();
    if !unmapped.is_empty() {
        return Err(Unmapped { kinds: unmapped });
    }

    // Every ground is one net; the rest are numbered as they are met.
    let grounded = |net: NetId| {
        netlist.net(net).is_some_and(|net| {
            net.pins().iter().any(|pin| {
                netlist
                    .element(pin.element)
                    .is_some_and(|element| matches!(element.kind, ElementKind::Ground))
            })
        })
    };
    let mut names: Vec<(NetId, String)> = Vec::new();
    let mut name_of = |net: NetId| -> String {
        if grounded(net) {
            return "GND".to_string();
        }
        if let Some((_, name)) = names.iter().find(|(id, _)| *id == net) {
            return name.clone();
        }
        let name = format!("N{}", names.len() + 1);
        names.push((net, name.clone()));
        name
    };

    let mut counts: Vec<(&str, u32)> = Vec::new();
    let mut out = Vec::new();
    for (_, element) in parts {
        let mapping = mapping(&element.kind).expect("checked above");
        let count = match counts
            .iter_mut()
            .find(|(prefix, _)| *prefix == mapping.prefix)
        {
            Some((_, count)) => {
                *count += 1;
                *count
            }
            None => {
                counts.push((mapping.prefix, 1));
                1
            }
        };
        let net_of = |index: usize| element.pins.get(index).and_then(|pin| pin.net);
        let pins: Vec<Pin> = match (mapping.pads, breadboard::dip(&element.kind)) {
            (Some(pads), _) => pads
                .iter()
                .enumerate()
                .chain(mapping.also.iter().map(|(pad, index)| (*index, pad)))
                .map(|(index, pad)| Pin {
                    number: pad.to_string(),
                    name: element
                        .pins
                        .get(index)
                        .map_or("", |pin| pin.name)
                        .to_string(),
                    net: net_of(index).map(&mut name_of),
                })
                .collect(),
            (None, Some((count, numbers))) => (1..=count)
                .map(|number| {
                    let model = numbers
                        .iter()
                        .take(element.pins.len())
                        .position(|&other| other == number);
                    // What the board ties and the drawing leaves out (B-15).
                    let tie = breadboard::ties(&element.kind)
                        .iter()
                        .find(|&&(pin, _, _)| pin == number);
                    // A pad the package joins inside to a model pin.
                    let joined = mapping
                        .also
                        .iter()
                        .find(|(pad, _)| *pad == number.to_string())
                        .map(|&(_, index)| index);
                    let (name, net) = match (model.or(joined), tie) {
                        (Some(index), _) => (element.pins[index].name, net_of(index)),
                        (None, Some(&(_, name, to))) => (name, net_of(to)),
                        (None, None) => ("NC", None),
                    };
                    Pin {
                        number: number.to_string(),
                        name: name.to_string(),
                        net: net.map(&mut name_of),
                    }
                })
                .collect(),
            (None, None) => Vec::new(),
        };
        out.push(Part {
            reference: format!("{}{count}", mapping.prefix),
            value: value(&element.kind),
            mapping,
            pins,
        });
    }
    Ok(out)
}

fn quote(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The circuit as a KiCad netlist, or the kinds of part it cannot write.
///
/// `value` says a part's value as the schematic writes it beside its symbol;
/// `source` names the file the netlist came from, for KiCad to show.
pub fn netlist(
    netlist: &Netlist,
    value: impl Fn(&ElementKind) -> String,
    source: &str,
) -> Result<String, Unmapped> {
    let parts = plan(netlist, value)?;
    // Each net, the pads it joins, in the order they are met.
    let mut nets: Vec<(String, Vec<(String, String)>)> = Vec::new();
    for part in &parts {
        for pin in &part.pins {
            let Some(name) = &pin.net else { continue };
            let node = (part.reference.clone(), pin.number.clone());
            match nets.iter_mut().find(|(other, _)| other == name) {
                Some((_, nodes)) if !nodes.contains(&node) => nodes.push(node),
                Some(_) => {}
                None => nets.push((name.clone(), vec![node])),
            }
        }
    }
    let stamp = |index: usize| format!("00000000-0000-4000-8000-{:012x}", index + 1);
    let mut out = String::new();
    out.push_str("(export (version \"E\")\n");
    out.push_str(&format!(
        "  (design\n    (source {})\n    (tool \"Pastille\"))\n",
        quote(source)
    ));
    out.push_str("  (components");
    for (index, part) in parts.iter().enumerate() {
        let (lib, name) = part
            .mapping
            .symbol
            .split_once(':')
            .unwrap_or(("", part.mapping.symbol));
        out.push_str(&format!(
            "\n    (comp (ref {})\n      (value {})\n      (footprint {})\n      (libsource (lib {}) (part {}) (description \"\"))\n      (sheetpath (names \"/\") (tstamps \"/\"))\n      (tstamps {}))",
            quote(&part.reference),
            quote(&part.value),
            quote(part.mapping.footprint),
            quote(lib),
            quote(name),
            quote(&stamp(index)),
        ));
    }
    out.push_str(")\n  (nets");
    for (code, (name, nodes)) in nets.iter().enumerate() {
        out.push_str(&format!(
            "\n    (net (code \"{}\") (name {})",
            code + 1,
            quote(name)
        ));
        for (reference, pad) in nodes {
            out.push_str(&format!(
                "\n      (node (ref {}) (pin {}))",
                quote(reference),
                quote(pad)
            ));
        }
        out.push(')');
    }
    out.push_str("))\n");
    Ok(out)
}

/// A symbol's pins, sided as it draws them: the left side top to bottom,
/// then the right side top to bottom. A chip's go round it as on its
/// package, 1 to n/2 down the left and the rest back up the right; anything
/// else's first half on the left.
fn sides(part: &Part) -> (Vec<&Pin>, Vec<&Pin>) {
    let count = part.pins.len();
    let half = count.div_ceil(2);
    if part.mapping.pads.is_none() {
        let left = part.pins[..count / 2].iter().collect();
        let right = part.pins[count / 2..].iter().rev().collect();
        return (left, right);
    }
    (
        part.pins[..half].iter().collect(),
        part.pins[half..].iter().collect(),
    )
}

/// KiCad's grid and a pin's length, in millimetres.
const GRID: f64 = 1.27;
const PIN: f64 = 2.54;

/// Where a symbol's pins end, in its own coordinates (library Y up): each
/// side's pins a grid step of two apart, centred on the symbol's origin.
fn pin_ends(part: &Part) -> Vec<(&Pin, f64, f64, bool)> {
    let (left, right) = sides(part);
    let rows = left.len().max(right.len());
    let half_width = if part.pins.len() == 2 { 2.54 } else { 5.08 };
    let y = |row: usize| (rows as f64 - 1.0) * GRID - row as f64 * 2.0 * GRID;
    let mut ends = Vec::new();
    for (row, pin) in left.into_iter().enumerate() {
        ends.push((pin, -(half_width + PIN), y(row), true));
    }
    for (row, pin) in right.into_iter().enumerate() {
        ends.push((pin, half_width + PIN, y(row), false));
    }
    ends
}

/// The symbol a part is drawn as, for `lib_symbols`: a box with its pins,
/// named for the KiCad symbol it stands in for, numbered as its footprint.
/// The drawing is the app's own: KiCad's are not copied in (ADR 0001).
fn symbol_definition(name: &str, part: &Part) -> String {
    let (left, right) = sides(part);
    let rows = left.len().max(right.len()) as f64;
    let half_width = if part.pins.len() == 2 { 2.54 } else { 5.08 };
    let half_height = rows * GRID;
    let font = "(effects (font (size 1.27 1.27)))";
    let hidden = "(effects (font (size 1.27 1.27)) (hide yes))";
    let mut out = format!(
        "    (symbol {} (pin_names (offset 0.254)) (exclude_from_sim no) (in_bom yes) (on_board yes)\n",
        quote(&format!("pastille:{name}"))
    );
    out += &format!(
        "      (property \"Reference\" {} (at 0 {:.2} 0) {font})\n",
        quote(part.mapping.prefix),
        half_height + GRID
    );
    out += &format!(
        "      (property \"Value\" {} (at 0 {:.2} 0) {font})\n",
        quote(name),
        -(half_height + GRID)
    );
    out += &format!(
        "      (property \"Footprint\" {} (at 0 0 0) {hidden})\n",
        quote(part.mapping.footprint)
    );
    out += &format!("      (property \"Datasheet\" \"\" (at 0 0 0) {hidden})\n");
    out += &format!("      (property \"Description\" \"\" (at 0 0 0) {hidden})\n");
    out += &format!(
        "      (symbol {} (rectangle (start {:.2} {:.2}) (end {:.2} {:.2}) (stroke (width 0.254) (type default)) (fill (type background))))\n",
        quote(&format!("{name}_0_1")),
        -half_width,
        half_height,
        half_width,
        -half_height
    );
    out += &format!("      (symbol {}", quote(&format!("{name}_1_1")));
    for (pin, x, y, on_left) in pin_ends(part) {
        out += &format!(
            "\n        (pin passive line (at {x:.2} {y:.2} {}) (length {PIN:.2}) (name {} {font}) (number {} {font}))",
            if on_left { 0 } else { 180 },
            quote(&pin.name),
            quote(&pin.number)
        );
    }
    out += "))\n";
    out
}

/// The circuit as a KiCad schematic (B-21), for KiCad's schematic editor to
/// open — or the kinds of part that stop it, as for the netlist.
///
/// Written to the grammar of KiCad 8's files (version 20231120), which KiCad
/// 9 also opens, as read off both versions' parser and writer (2026-10-03):
/// field `hide` as `(hide yes)` inside `effects`, which both read; no
/// `embedded_fonts`, which 8 rejects; every placed symbol with its
/// `instances`, without which its reference reads "R?"; a pin's `at` its
/// connection point, library Y up and sheet Y down. The library nickname,
/// `pastille`, is in nobody's library table: KiCad opens the symbols from the
/// copy in the file, and its rules check notes the missing library.
///
/// Each part is a symbol on a grid, six to a row, numbered and footprinted as
/// the netlist has it, and every pin carries its net's label on its end:
/// labels of one name on a sheet are one net in KiCad, so the connections
/// are the circuit's without a wire drawn. A pin on no net is marked as left
/// free, so the electrical rules check does not ask about it. The drawing is
/// a starting point for schematic capture, not a laid-out schematic.
pub fn schematic(
    netlist: &Netlist,
    value: impl Fn(&ElementKind) -> String,
    title: &str,
) -> Result<String, Unmapped> {
    let parts = plan(netlist, value)?;
    let mut serial = 0u64;
    let mut uuid = || {
        serial += 1;
        format!("00000000-0000-4000-8000-{serial:012x}")
    };
    let root = uuid();
    // One symbol per kind of part, by the KiCad symbol it stands in for.
    let name_of = |part: &Part| {
        part.mapping
            .symbol
            .split_once(':')
            .map_or(part.mapping.symbol, |(_, name)| name)
            .to_string()
    };
    let mut library: Vec<(String, String)> = Vec::new();
    for part in &parts {
        let name = name_of(part);
        if !library.iter().any(|(other, _)| *other == name) {
            let definition = symbol_definition(&name, part);
            library.push((name, definition));
        }
    }

    let mut out = format!(
        "(kicad_sch (version 20231120) (generator \"pastille\") (generator_version \"1.0\")\n  (uuid {})\n  (paper \"A3\")\n  (title_block (title {}))\n  (lib_symbols\n",
        quote(&root),
        quote(title)
    );
    for (_, definition) in &library {
        out += definition;
    }
    out += "  )\n";

    // Six to a row, each row as tall as its tallest part needs.
    let font = "(effects (font (size 1.27 1.27)))";
    let mut row_top = 25.4;
    for row in parts.chunks(6) {
        let tallest = row
            .iter()
            .map(|part| part.pins.len().div_ceil(2) as f64 * 2.0 * GRID)
            .fold(0.0, f64::max);
        let centre_y = row_top + tallest / 2.0 + 2.0 * PIN;
        // On the grid: a symbol's origin on a multiple of 2.54.
        let y = (centre_y / PIN).round() * PIN;
        for (column, part) in row.iter().enumerate() {
            let x = 25.4 + column as f64 * 38.1;
            let rows = part.pins.len().div_ceil(2) as f64;
            let half_height = rows * GRID;
            out += &format!(
                "  (symbol (lib_id {}) (at {x:.2} {y:.2} 0) (unit 1) (exclude_from_sim no) (in_bom yes) (on_board yes) (dnp no)\n    (uuid {})\n",
                quote(&format!("pastille:{}", name_of(part))),
                quote(&uuid())
            );
            out += &format!(
                "    (property \"Reference\" {} (at {x:.2} {:.2} 0) {font})\n",
                quote(&part.reference),
                y - half_height - GRID
            );
            out += &format!(
                "    (property \"Value\" {} (at {x:.2} {:.2} 0) {font})\n",
                quote(&part.value),
                y + half_height + GRID
            );
            out += &format!(
                "    (property \"Footprint\" {} (at {x:.2} {y:.2} 0) (effects (font (size 1.27 1.27)) (hide yes)))\n",
                quote(part.mapping.footprint)
            );
            for field in ["Datasheet", "Description"] {
                out += &format!(
                    "    (property \"{field}\" \"\" (at {x:.2} {y:.2} 0) (effects (font (size 1.27 1.27)) (hide yes)))\n"
                );
            }
            for pin in &part.pins {
                out += &format!(
                    "    (pin {} (uuid {}))\n",
                    quote(&pin.number),
                    quote(&uuid())
                );
            }
            out += &format!(
                "    (instances (project \"circuit\" (path {} (reference {}) (unit 1)))))\n",
                quote(&format!("/{root}")),
                quote(&part.reference)
            );
            // Each pin's net, on its end; a free pin marked free.
            for (pin, px, py, on_left) in pin_ends(part) {
                let (at_x, at_y) = (x + px, y - py);
                match &pin.net {
                    Some(net) => {
                        out += &format!(
                            "  (label {} (at {at_x:.2} {at_y:.2} {}) (effects (font (size 1.27 1.27)) (justify {} bottom)) (uuid {}))\n",
                            quote(net),
                            if on_left { 180 } else { 0 },
                            if on_left { "right" } else { "left" },
                            quote(&uuid())
                        );
                    }
                    None => {
                        out += &format!(
                            "  (no_connect (at {at_x:.2} {at_y:.2}) (uuid {}))\n",
                            quote(&uuid())
                        );
                    }
                }
            }
        }
        row_top = y + tallest / 2.0 + 2.0 * PIN;
    }
    out += "  (sheet_instances (path \"/\" (page \"1\")))\n)\n";
    Ok(out)
}

#[cfg(test)]
mod tests {
    use crate::circuit::Circuit;

    fn exported(text: &str) -> super::super::circuit::KicadView {
        let mut circuit = Circuit::new();
        circuit.try_open(text).unwrap();
        circuit.kicad()
    }

    /// Every bracket opened is closed, outside quoted text, and the file is
    /// one list: what any s-expression reader needs first.
    fn balanced(text: &str) -> bool {
        let (mut depth, mut quoted, mut escaped) = (0i32, false, false);
        for c in text.chars() {
            match (quoted, escaped, c) {
                (true, true, _) => escaped = false,
                (true, false, '\\') => escaped = true,
                (true, false, '"') => quoted = false,
                (false, _, '"') => quoted = true,
                (false, _, '(') => depth += 1,
                (false, _, ')') => {
                    depth -= 1;
                    if depth < 0 {
                        return false;
                    }
                }
                _ => {}
            }
        }
        depth == 0 && !quoted
    }

    #[test]
    fn the_milestone_is_a_netlist_kicad_reads() {
        let view = exported(include_str!("../../../app/src/examples/milestone.json"));
        let text = view.netlist.expect("every part maps");
        assert!(balanced(&text), "{text}");
        assert!(text.starts_with("(export (version \"E\")"), "{text}");
        for line in [
            "(comp (ref \"J1\")",
            "(comp (ref \"R1\")",
            "(comp (ref \"D1\")",
            "(footprint \"LED_THT:LED_D5.0mm\")",
            "(libsource (lib \"Device\") (part \"LED\")",
            "(sheetpath (names \"/\") (tstamps \"/\"))",
        ] {
            assert!(text.contains(line), "{line} in {text}");
        }
        // Three nets: + to the resistor, the resistor to the LED's anode
        // (pad 2), its cathode (pad 1) to −.
        assert_eq!(text.matches("(net (code").count(), 3, "{text}");
        assert!(text.contains("(node (ref \"D1\") (pin \"2\"))"), "{text}");
        assert!(text.contains("(node (ref \"D1\") (pin \"1\"))"), "{text}");
    }

    #[test]
    fn a_555_has_its_reset_on_vcc() {
        let view = exported(include_str!("../../../app/src/examples/astable.json"));
        let text = view.netlist.expect("every part maps");
        assert!(balanced(&text));
        // VCC is pin 8; pin 4 is on the same net.
        let net = text
            .split("(net ")
            .find(|net| net.contains("(node (ref \"U1\") (pin \"8\"))"))
            .unwrap();
        assert!(net.contains("(node (ref \"U1\") (pin \"4\"))"), "{net}");
        assert!(text.contains("(footprint \"Package_DIP:DIP-8_W7.62mm\")"));
    }

    #[test]
    fn every_ground_is_one_net() {
        let mut circuit = Circuit::new();
        let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
        let resistor = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
        let here = circuit.try_add("Ground", 0.0, 80.0).unwrap();
        let there = circuit.try_add("Ground", 80.0, 80.0).unwrap();
        circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
        circuit.try_connect(&battery, "-", &here, "gnd").unwrap();
        circuit.try_connect(&resistor, "b", &there, "gnd").unwrap();
        let text = circuit.kicad().netlist.unwrap();
        assert_eq!(text.matches("(name \"GND\")").count(), 1, "{text}");
        assert!(!text.contains("Ground"), "{text}");
    }

    #[test]
    fn every_part_the_palette_offers_maps_in_each_of_its_choices() {
        for component in crate::registry::REGISTRY.iter().filter(|c| c.offered) {
            let blank = (component.blank)();
            let mut kinds = vec![blank];
            kinds.extend(
                (component.choices)()
                    .into_iter()
                    .map(|choice| blank.with_choice(choice).unwrap()),
            );
            for kind in kinds {
                if super::bench_only(&kind) {
                    continue;
                }
                let mapping = super::mapping(&kind).unwrap_or_else(|| panic!("{kind:?}"));
                // One pad a pin, none twice; or a chip, numbered by its package.
                match mapping.pads {
                    Some(pads) => {
                        assert_eq!(pads.len(), component.pins.len(), "{kind:?}");
                        let mut all: Vec<&str> = pads
                            .iter()
                            .copied()
                            .chain(mapping.also.iter().map(|(pad, _)| *pad))
                            .collect();
                        all.sort_unstable();
                        all.dedup();
                        assert_eq!(all.len(), pads.len() + mapping.also.len(), "{kind:?}");
                    }
                    None => assert!(crate::breadboard::dip(&kind).is_some(), "{kind:?}"),
                }
                assert!(mapping.symbol.contains(':') && mapping.footprint.contains(':'));
            }
        }
    }

    #[test]
    fn every_example_exports() {
        for entry in std::fs::read_dir("../../app/src/examples").unwrap() {
            let path = entry.unwrap().path();
            let view = exported(&std::fs::read_to_string(&path).unwrap());
            assert_eq!(view.unmapped, Vec::<String>::new(), "{}", path.display());
            let netlist = view.netlist.expect("every part maps");
            let schematic = view.schematic.expect("every part maps");
            assert!(balanced(&netlist), "{}", path.display());
            assert!(balanced(&schematic), "{}", path.display());
        }
    }

    #[test]
    fn a_pad_the_package_joins_inside_is_on_its_pins_net() {
        // A digit's common is pads 3 and 8; an L293D's GND pins 4, 5, 12, 13.
        let mut circuit = Circuit::new();
        let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
        let digit = circuit.try_add("SevenSegment", 80.0, 0.0).unwrap();
        let driver = circuit.try_add("MotorDriver", 160.0, 0.0).unwrap();
        circuit.try_connect(&battery, "-", &digit, "com").unwrap();
        circuit.try_connect(&battery, "-", &driver, "GND").unwrap();
        let text = circuit.kicad().netlist.unwrap();
        let net = text
            .split("(net ")
            .find(|net| net.contains("(node (ref \"J1\") (pin \"2\"))"))
            .unwrap();
        for (part, pad) in [
            ("U1", "3"),
            ("U1", "8"),
            ("U2", "4"),
            ("U2", "5"),
            ("U2", "12"),
            ("U2", "13"),
        ] {
            let node = format!("(node (ref \"{part}\") (pin \"{pad}\"))");
            assert!(net.contains(&node), "{node} in {net}");
        }
    }

    /// Every "(label NAME (at X Y" and "(no_connect (at X Y" in a schematic.
    /// A place on the sheet, in hundredths of a millimetre.
    type Place = (i64, i64);

    fn marks(text: &str) -> (Vec<(String, Place)>, Vec<Place>) {
        let at = |rest: &str| {
            let numbers: Vec<f64> = rest
                .split_whitespace()
                .take(2)
                .map(|n| n.trim_end_matches(')').parse().unwrap())
                .collect();
            (
                (numbers[0] * 100.0).round() as i64,
                (numbers[1] * 100.0).round() as i64,
            )
        };
        let mut labels = Vec::new();
        let mut free = Vec::new();
        for line in text.lines().map(str::trim) {
            if let Some(rest) = line.strip_prefix("(label \"") {
                let (name, rest) = rest.split_once('"').unwrap();
                let rest = rest.trim_start().strip_prefix("(at ").unwrap();
                labels.push((name.to_string(), at(rest)));
            } else if let Some(rest) = line.strip_prefix("(no_connect (at ") {
                free.push(at(rest));
            }
        }
        (labels, free)
    }

    #[test]
    fn every_pin_of_the_schematic_carries_its_net_or_is_marked_free() {
        for text in [
            include_str!("../../../app/src/examples/milestone.json"),
            include_str!("../../../app/src/examples/astable.json"),
            include_str!("../../../app/src/examples/transistor.json"),
        ] {
            let mut circuit = Circuit::new();
            circuit.try_open(text).unwrap();
            let parts = super::plan(circuit.netlist(), |kind| kind.name().to_string()).unwrap();
            let schematic =
                super::schematic(circuit.netlist(), |kind| kind.name().to_string(), "t").unwrap();
            assert!(balanced(&schematic), "{schematic}");
            let (labels, free) = marks(&schematic);
            let pins: usize = parts.iter().map(|part| part.pins.len()).sum();
            // One mark a pin, and no two in one place.
            assert_eq!(labels.len() + free.len(), pins);
            let mut places: Vec<_> = labels
                .iter()
                .map(|(_, at)| *at)
                .chain(free.iter().copied())
                .collect();
            places.sort_unstable();
            places.dedup();
            assert_eq!(places.len(), pins, "two marks share a place");
            // Every place on KiCad's 1.27 mm grid.
            assert!(
                places.iter().all(|(x, y)| x % 127 == 0 && y % 127 == 0),
                "{places:?}"
            );
            // Each net is labelled as many times as it has pins, by name.
            for part in &parts {
                for pin in &part.pins {
                    if let Some(net) = &pin.net {
                        assert!(labels.iter().any(|(name, _)| name == net), "{net}");
                    }
                }
            }
        }
    }

    #[test]
    fn a_label_sits_on_the_end_of_its_pin() {
        // The milestone's LED, D1: its cathode (pad 1) and anode (pad 2) are
        // on two nets; each label is where that pin of the placed symbol ends.
        let mut circuit = Circuit::new();
        circuit
            .try_open(include_str!("../../../app/src/examples/milestone.json"))
            .unwrap();
        let parts = super::plan(circuit.netlist(), |kind| kind.name().to_string()).unwrap();
        let schematic =
            super::schematic(circuit.netlist(), |kind| kind.name().to_string(), "t").unwrap();
        let (labels, _) = marks(&schematic);
        let index = parts
            .iter()
            .position(|part| part.reference == "D1")
            .unwrap();
        // Third in its row: x = 25.4 + 2 × 38.1; a two-pin box's pins end
        // 5.08 either side of it.
        let x = 25.4 + 2.0 * 38.1;
        let led = &parts[index];
        let cathode = led.pins.iter().find(|pin| pin.number == "1").unwrap();
        let anode = led.pins.iter().find(|pin| pin.number == "2").unwrap();
        let at = |x: f64| (x * 100.0).round() as i64;
        let label_x = |net: &str| {
            labels
                .iter()
                .filter(|(name, _)| name == net)
                .map(|(_, (lx, _))| *lx)
                .collect::<Vec<_>>()
        };
        // The model lists the anode first, so it is the left pin.
        assert!(label_x(anode.net.as_deref().unwrap()).contains(&at(x - 5.08)));
        assert!(label_x(cathode.net.as_deref().unwrap()).contains(&at(x + 5.08)));
    }
}
