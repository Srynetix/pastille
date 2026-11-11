// SPDX-License-Identifier: MIT OR Apache-2.0

//! The linter's rules (B-01), one function each.
//!
//! To add one: write a function that looks at one part and returns what to
//! say about it, or `None`; then put it in [`RULES`], where its place is the
//! order findings come out in. Nothing else needs touching. Each says what is
//! wrong in a sentence short enough not to wrap (design doc §14) — what the
//! pins are called is the hover's job.

use super::{Check, Context, Fix, Rule, Said, encode};
use crate::netlist::{Element, ElementId, ElementKind, LedColour, NetId, Netlist};

/// Every rule, in the order their findings are listed: the loud and common
/// first.
pub(super) const RULES: &[Rule] = &[
    Rule {
        name: "source-shorted",
        check: Check::Wiring(source_shorted),
    },
    Rule {
        name: "reversed-led",
        check: Check::Solved(reversed_led),
    },
    Rule {
        name: "led-unlimited",
        check: Check::Wiring(led_unlimited),
    },
    Rule {
        name: "base-unlimited",
        check: Check::Wiring(base_unlimited),
    },
    Rule {
        name: "uno-coil",
        check: Check::Wiring(uno_coil),
    },
    Rule {
        name: "over-rated",
        check: Check::Solved(over_rated),
    },
    Rule {
        name: "uno-overvolted",
        check: Check::Solved(uno_overvolted),
    },
    Rule {
        name: "uno-overloaded",
        check: Check::Solved(uno_overloaded),
    },
    Rule {
        name: "flat-battery",
        check: Check::Solved(flat_battery),
    },
    Rule {
        name: "fuse-gone",
        check: Check::Solved(fuse_gone),
    },
    Rule {
        name: "wrong-common",
        check: Check::Solved(wrong_common),
    },
    Rule {
        name: "unpowered",
        check: Check::Solved(unpowered),
    },
    Rule {
        name: "regulator-hot",
        check: Check::Solved(regulator_hot),
    },
    Rule {
        name: "regulator-starved",
        check: Check::Solved(regulator_starved),
    },
    Rule {
        name: "lm317-cramped",
        check: Check::Solved(lm317_cramped),
    },
    Rule {
        name: "opamp-misused",
        check: Check::Solved(opamp_misused),
    },
    Rule {
        name: "hall-unpulled",
        check: Check::Solved(hall_unpulled),
    },
    Rule {
        name: "pir-settling",
        check: Check::Solved(pir_settling),
    },
    Rule {
        name: "l293d-unfed",
        check: Check::Solved(l293d_unfed),
    },
    Rule {
        name: "logic-input-free",
        check: Check::Solved(logic_input_free),
    },
    Rule {
        name: "chip-input-free",
        check: Check::Solved(chip_input_free),
    },
    Rule {
        name: "uno-input-floating",
        check: Check::Solved(uno_input_floating),
    },
    Rule {
        name: "tmp36-misfed",
        check: Check::Solved(tmp36_misfed),
    },
    Rule {
        name: "lcd-on-dc",
        check: Check::Solved(lcd_on_dc),
    },
    Rule {
        name: "electrolytic-strained",
        check: Check::Solved(electrolytic_strained),
    },
    Rule {
        name: "pin-free",
        check: Check::Solved(pin_free),
    },
];

/// A battery whose + is wired to its own − through nothing that takes the
/// current (B-05): wires, ammeters, switches as they are now — a shut one
/// conducts, an open one does not, so pressing a button across a battery is
/// when this speaks — and two ground symbols, which are one node (ADR 0003).
///
/// C-05's `shorted_sources` is the case of a wire alone, both terminals on
/// one net; this follows the same idea through the parts a short can hide
/// behind. A fuse in the way is not a short but the fuse's job, and the
/// examples that blow one say so.
fn source_shorted(netlist: &Netlist, id: ElementId, element: &Element) -> Option<Said> {
    let noun = match element.kind {
        ElementKind::DcSource { .. } | ElementKind::Cell { .. } => "battery",
        ElementKind::SignalGenerator { .. } => "signal generator",
        _ => return None,
    };
    let plus = element.pins.first()?.net?;
    let minus = element.pins.get(1)?.net?;
    let grounded_minus = grounded(netlist, id);
    let mut seen = vec![plus];
    let mut queue = vec![plus];
    let mut shorted = false;
    'search: while let Some(net) = queue.pop() {
        if net == minus {
            shorted = true;
            break;
        }
        for pin in netlist.net(net)?.pins() {
            if pin.element == id {
                continue;
            }
            let Some(other) = netlist.element(pin.element) else {
                continue;
            };
            let through = match other.kind {
                ElementKind::Ground if grounded_minus => {
                    shorted = true;
                    break 'search;
                }
                ElementKind::Ammeter => true,
                ref kind => kind.contact() == Some(true),
            };
            if !through {
                continue;
            }
            let Some(next) = other.pins.get(1 - pin.index).and_then(|pin| pin.net) else {
                continue;
            };
            if !seen.contains(&next) {
                seen.push(next);
                queue.push(next);
            }
        }
    }
    shorted.then(|| {
        Said::warning(format!(
            "This {noun}'s + is wired straight to its − — put a part between them, or take that wire out."
        ))
    })
}

/// Reverse bias deeper than this is a wire the wrong way round rather than a
/// rounding error.
const REVERSED_VOLTS: f64 = 0.1;

/// A volt is below anything the parts here run on and above any rounding.
const POWERED_VOLTS: f64 = 1.0;

/// An LED reversed against a supply that has a right way (C-32).
///
/// Which of the two is "backwards" is not something the circuit can know —
/// only that they disagree. Saying both is honest and is also the sentence
/// that teaches the rule.
fn reversed_led(cx: &Context, _: ElementId, element: &Element) -> Option<Said> {
    if !matches!(element.kind, ElementKind::Led { .. }) {
        return None;
    }
    let [anode, cathode] = element.pins.as_slice() else {
        return None;
    };
    if cx.alternating(anode.net) {
        return None;
    }
    let across = cx.engine.node_voltage(anode.net?)? - cx.engine.node_voltage(cathode.net?)?;
    if across > -REVERSED_VOLTS {
        return None;
    }
    Some(Said::warning(
        "This LED is the wrong way round — or the battery is.",
    ))
}

/// A digit or an RGB LED of the other common type, wired as if it were this
/// one, is dark with nothing to say why: every LED in it is reversed (C-102).
/// Said once for the part, with which way its common pin goes.
fn wrong_common(cx: &Context, _: ElementId, element: &Element) -> Option<Said> {
    let (common, colours) = element.kind.shared_leds()?;
    let shared = colours.len();
    if cx.alternating(element.pins.get(shared)?.net) {
        return None;
    }
    let backwards = (0..shared).any(|led| {
        let (anode, cathode) = common.junction(led, shared);
        cx.pin_volts(element, anode)
            .zip(cx.pin_volts(element, cathode))
            .is_some_and(|(anode, cathode)| anode - cathode < -REVERSED_VOLTS)
    });
    if !backwards {
        return None;
    }
    let noun = match element.kind {
        ElementKind::RgbLed { .. } => "RGB LED",
        _ => "digit",
    };
    let (kind, rail) = match common {
        crate::solver::device::digit::Common::Cathode => ("common cathode", "−"),
        crate::solver::device::digit::Common::Anode => ("common anode", "+"),
    };
    Some(Said::warning(format!(
        "This {noun} is wired the wrong way round: it is {kind}, so its common pin goes to {rail}."
    )))
}

/// A part with supply pins of its own and too little across them (C-55, C-56,
/// C-102, ADR 0010, ADR 0012).
///
/// An op-amp with nothing across its supply does nothing at all, and nothing
/// on the canvas says so: the circuit solves, the output just sits there. The
/// part's own supply is the thing a beginner leaves out, because textbook
/// drawings leave it out. With the least each needs, for a part whose model
/// stops below it: a servo under 4 V and an L293D's logic under 4.5 V do
/// nothing, and used to say nothing.
fn unpowered(cx: &Context, _: ElementId, element: &Element) -> Option<Said> {
    use crate::solver::device;
    let (part, high, low, least) = match element.kind {
        ElementKind::OpAmp => ("op-amp", "V+", "V-", POWERED_VOLTS),
        ElementKind::Comparator => ("comparator", "V+", "V-", POWERED_VOLTS),
        ElementKind::Timer => ("555", "VCC", "GND", POWERED_VOLTS),
        ElementKind::Decoder => ("CD4511", "VDD", "VSS", POWERED_VOLTS),
        ElementKind::Servo => ("servo", "+5V", "GND", device::servo::LEAST_SUPPLY),
        ElementKind::Counter => ("CD4026", "VDD", "VSS", POWERED_VOLTS),
        ElementKind::Decade => ("CD4017", "VDD", "VSS", POWERED_VOLTS),
        ElementKind::Lm3914 => ("LM3914", "V+", "V-", device::lm3914::LEAST_SUPPLY),
        ElementKind::FlipFlop => ("CD4013", "VDD", "VSS", POWERED_VOLTS),
        ElementKind::Pir { .. } => ("HC-SR501", "VCC", "GND", device::pir::LEAST_SUPPLY),
        ElementKind::Hall { .. } => ("A3144", "VCC", "GND", device::magnet::LEAST_SUPPLY),
        // Its logic's; the motor's is l293d_unfed's (C-91).
        ElementKind::MotorDriver => ("L293D", "VCC1", "GND", device::driver::LOGIC_FROM),
        ElementKind::ShiftRegister => ("74HC595", "VCC", "GND", POWERED_VOLTS),
        ElementKind::Gate { logic } => (logic.part_number(), "VCC", "GND", POWERED_VOLTS),
        ElementKind::Inverter => (device::gate::INVERTER_PART, "VCC", "GND", POWERED_VOLTS),
        ElementKind::Schmitt => (device::gate::SCHMITT_PART, "VCC", "GND", POWERED_VOLTS),
        _ => return None,
    };
    let supply = cx
        .named_volts(element, high)
        .zip(cx.named_volts(element, low))
        .map(|(high, low)| high - low);
    let (high_pin, low_pin) = (element.pin_index(high)?, element.pin_index(low)?);
    let low = low.replace('-', "−");
    let message = match supply {
        Some(volts) if volts >= least => return None,
        // Wired, and the wrong way round: "no power" would send a beginner to
        // wire what is already wired.
        Some(volts) if volts <= -POWERED_VOLTS => {
            format!("This {part} is powered the wrong way round — swap its {high} and {low}.")
        }
        Some(volts) if volts >= POWERED_VOLTS => {
            format!("This {part} needs at least {least} V across {high} and {low}.")
        }
        // One supply pin wired and the other free (B-20): the one to wire,
        // named, rather than both.
        _ => match (wired(cx, element, high_pin), wired(cx, element, low_pin)) {
            (true, false) => {
                format!("This {part}'s {low} is not wired — wire it to the battery's −.")
            }
            (false, true) => {
                format!("This {part}'s {high} is not wired — wire it to the battery's +.")
            }
            _ => format!("This {part} has no power — wire {high} and {low} to the battery."),
        },
    };
    Some(Said::warning(message))
}

/// Glass is damaged by DC: the current it drives through the film plates the
/// electrodes a little more every second. A digit wired straight to a battery
/// still shows — which is exactly why nobody notices — so it is the one thing
/// on it worth saying (C-76).
fn lcd_on_dc(cx: &Context, id: ElementId, element: &Element) -> Option<Said> {
    if !matches!(element.kind, ElementKind::LcdDigit) || !cx.engine.glass(id)?.driven_with_dc() {
        return None;
    }
    Some(Said::warning(
        "This LCD is driven with DC, which wears the glass out — drive it with AC, in antiphase with COM.",
    ))
}

/// An electrolytic wired backwards is the classic beginner bang, and one past
/// its rating the second: both said, neither breaks it, so turning it round or
/// choosing a higher rating makes the line go (C-86). Backwards first, the
/// commoner and the louder.
fn electrolytic_strained(cx: &Context, id: ElementId, element: &Element) -> Option<Said> {
    let ElementKind::Electrolytic { rated, .. } = element.kind else {
        return None;
    };
    let strain = cx.engine.strain(id)?;
    if strain.reversed() {
        Some(Said::warning(
            "This capacitor is the wrong way round — its + must face the higher voltage.",
        ))
    } else if strain.over() {
        Some(Said::warning(format!(
            "This capacitor has more than its {} across it — choose one rated higher.",
            rated.name()
        )))
    } else {
        None
    }
}

/// A TMP36 runs on 2.7 V to 5.5 V and is in every kit beside a 9 V battery:
/// outside that, it says so — unpowered, under or over (C-87).
fn tmp36_misfed(cx: &Context, _: ElementId, element: &Element) -> Option<Said> {
    if !matches!(element.kind, ElementKind::Tmp36 { .. }) {
        return None;
    }
    use crate::solver::device::tmp36::{GROUND_PIN, LEAST_SUPPLY, MOST_SUPPLY, SUPPLY_PIN};
    let supply = cx
        .pin_volts(element, SUPPLY_PIN)
        .zip(cx.pin_volts(element, GROUND_PIN))
        .map(|(high, low)| high - low);
    let message = match supply {
        Some(volts) if (LEAST_SUPPLY..=MOST_SUPPLY).contains(&volts) => return None,
        Some(volts) if volts > MOST_SUPPLY => {
            "This TMP36 has more than the 5.5 V it runs on — give it 5 V, not the battery."
        }
        Some(volts) if volts >= POWERED_VOLTS => {
            "This TMP36 needs at least 2.7 V across +Vs and GND."
        }
        Some(volts) if volts <= -POWERED_VOLTS => {
            "This TMP36 is powered the wrong way round — swap its +Vs and GND."
        }
        _ => "This TMP36 has no power — wire +Vs and GND to the battery.",
    };
    Some(Said::warning(message))
}

/// A regulator with nothing on its input gives nothing, and one fed less than
/// its output and its dropout follows its input down, a few volts short of what
/// it is for, without a word (C-102). The second is a note rather than a
/// warning: it works, as the sheet says it does — "Not enough volts" is there
/// to show exactly that.
fn regulator_starved(cx: &Context, _: ElementId, element: &Element) -> Option<Said> {
    let ElementKind::Regulator { part } = element.kind else {
        return None;
    };
    use crate::solver::device::regulator::{GROUND_PIN, INPUT_PIN, OUTPUT_PIN};
    let ground = cx.pin_volts(element, GROUND_PIN)?;
    let name = part.name();
    let sheet = part.datasheet();
    let needs = sheet.output_volts + sheet.dropout_volts;
    match cx.pin_volts(element, INPUT_PIN).map(|volts| volts - ground) {
        Some(input) if input >= needs => None,
        Some(input) if input >= POWERED_VOLTS => Some(Said::note(format!(
            "This {name} needs {needs} V in to give {} V out; on {input:.1} V it gives about {:.1} V.",
            sheet.output_volts,
            cx.pin_volts(element, OUTPUT_PIN)
                .map_or(0.0, |out| out - ground)
        ))),
        _ => Some(Said::warning(format!(
            "This {name} has nothing on its input — wire in to the battery's +."
        ))),
    }
}

/// A linear regulator turning more into heat than its package can shed with
/// nothing bolted to it (B-06, design doc §6.1).
///
/// What it drops, input to output, times what it passes is heat, and a
/// regulator's limit is the temperature that puts its die at rather than a
/// power: the room plus the watts times its package's θja (C-54, C-74), past
/// its maximum junction temperature. Judged by where it is heading, not where
/// it has got to — a TO-220 takes the best part of a minute to warm, and a
/// beginner should not have to wait to be told. Said with the watts it turns
/// into heat and the most it can shed, which is the one number a heatsink
/// changes; or the drop can go, which is what a switching regulator is for.
fn regulator_hot(cx: &Context, id: ElementId, element: &Element) -> Option<Said> {
    use crate::solver::device::RegulatorPart;
    use crate::solver::device::regulator::{INPUT_PIN, OUTPUT_PIN};
    let name = match element.kind {
        ElementKind::Regulator { part } => part.name(),
        ElementKind::Adjustable => RegulatorPart::Lm317.name(),
        _ => return None,
    };
    let thermal = crate::registry::rating_of(&element.kind).thermal?;
    let watts = cx.engine.heat(id)?.watts;
    let room = cx.netlist.room_celsius();
    if crate::solver::heat::settles_at(&thermal, room, watts) <= thermal.max_junction_celsius {
        return None;
    }
    let drop = cx.pin_volts(element, INPUT_PIN)? - cx.pin_volts(element, OUTPUT_PIN)?;
    let most = (thermal.max_junction_celsius - room) / thermal.junction_to_ambient;
    Some(Said::warning(format!(
        "This {name} drops {:.1} V and turns {} into heat, past the {} it can shed without a heatsink — give it one, or use a switching regulator.",
        drop,
        crate::value::format(two_figures(watts), "W"),
        crate::value::format(two_figures(most), "W"),
    )))
}

/// An LM317's output is whatever its two resistors ask for, so what it can run
/// short of is headroom: under its dropout between input and output, it
/// follows its input down instead (C-113). A note, as the fixed regulator's
/// is; nothing on its input, a warning.
fn lm317_cramped(cx: &Context, _: ElementId, element: &Element) -> Option<Said> {
    if !matches!(element.kind, ElementKind::Adjustable) {
        return None;
    }
    use crate::solver::device::RegulatorPart;
    use crate::solver::device::regulator::{GROUND_PIN, INPUT_PIN, OUTPUT_PIN};
    let adjust = cx.pin_volts(element, GROUND_PIN)?;
    let output = cx.pin_volts(element, OUTPUT_PIN)?;
    let sheet = RegulatorPart::Lm317.datasheet();
    // Holding, its output sits its 1.25 V above ADJ; dropped out, it cannot,
    // and sits its dropout under its input instead.
    match cx.pin_volts(element, INPUT_PIN) {
        Some(_) if output - adjust >= sheet.output_volts - 0.01 => None,
        Some(input) if input - adjust >= POWERED_VOLTS => Some(Said::note(format!(
            "This LM317 needs {} V more in than out; with {:.1} V across it, it gives only {:.1} V.",
            sheet.dropout_volts,
            input - output,
            output,
        ))),
        _ => Some(Said::warning(
            "This LM317 has nothing on its input — wire in to the battery's +.",
        )),
    }
}

/// An A3144's output only pulls down (C-115): with nothing on its wire that
/// could carry current up to it — a resistor, an LED, a coil — it floats when
/// the magnet is away, and what reads it reads nothing.
fn hall_unpulled(cx: &Context, id: ElementId, element: &Element) -> Option<Said> {
    if !matches!(element.kind, ElementKind::Hall { .. }) {
        return None;
    }
    use crate::solver::device::magnet::OUT;
    if pulled_up(cx, id, element.pins[OUT].net?)? {
        return None;
    }
    Some(Said::warning(
        "This A3144 can only pull OUT down — give OUT a resistor up to +, or it floats when the magnet is away.",
    ))
}

/// Whether anything on an open-collector output's wire could carry current up
/// to it — a resistor, an LED, a coil — so it has a high to be let go to:
/// `None` for a wire that is not there.
fn pulled_up(cx: &Context, id: ElementId, net: NetId) -> Option<bool> {
    let net = cx.netlist.net(net)?;
    Some(net.pins().iter().any(|pin| {
        pin.element != id
            && cx.netlist.element(pin.element).is_some_and(|other| {
                matches!(
                    other.kind,
                    ElementKind::Resistor { .. }
                        | ElementKind::Potentiometer { .. }
                        | ElementKind::Led { .. }
                        | ElementKind::RgbLed { .. }
                        | ElementKind::Buzzer
                        | ElementKind::PassiveBuzzer
                        | ElementKind::Motor
                        | ElementKind::Relay { .. }
                        | ElementKind::Ldr { .. }
                        | ElementKind::Thermistor { .. }
                        | ElementKind::DcSource { .. }
                        | ElementKind::Cell { .. }
                )
            })
    }))
}

/// Whether a part's pin is on a wire with something else on it.
fn wired(cx: &Context, element: &Element, pin: usize) -> bool {
    element.pins[pin]
        .net
        .and_then(|net| cx.netlist.net(net))
        .is_some_and(|net| net.pins().len() >= 2)
}

/// How far outside its supply an input may go before the real part's
/// protection conducts (the LM358's and LM393's −0.3 V absolute maximum).
const PAST_RAIL: f64 = 0.3;

/// How close to V+ an input can come and still be read: both sheets give a
/// common-mode range up to V+ − 1.5 V.
const COMMON_MODE_HEADROOM: f64 = 1.5;

/// An op-amp or a comparator wired as it cannot work, where the model shows
/// something the real part would not (B-20, design doc §6.1; C-55, C-56).
///
/// The model honours neither of what a real LM358's or LM393's inputs cannot
/// do, and says so here instead: an input outside the supply, where the
/// part's protection conducts and can be damaged; both inputs in the top
/// volt and a half, where it cannot compare them — one up there is fine, as
/// both sheets say, so a threshold set against the top of a pot is not
/// told. And an LM393's output with nothing to pull it up, which the model
/// shows at V+ and a real one leaves floating. A supply pin left free is
/// `unpowered`'s to say.
fn opamp_misused(cx: &Context, id: ElementId, element: &Element) -> Option<Said> {
    let part = match element.kind {
        ElementKind::OpAmp => crate::solver::device::opamp::PART,
        ElementKind::Comparator => crate::solver::device::comparator::PART,
        _ => return None,
    };
    let high = cx.named_volts(element, "V+")?;
    let low = cx.named_volts(element, "V-")?;
    if high - low < POWERED_VOLTS {
        return None;
    }
    let inputs = [
        ("in+", cx.named_volts(element, "in+")),
        ("in−", cx.named_volts(element, "in-")),
    ];
    for (name, volts) in inputs {
        let Some(volts) = volts else { continue };
        let (side, rail) = if volts > high + PAST_RAIL {
            ("above", "V+")
        } else if volts < low - PAST_RAIL {
            ("below", "V−")
        } else {
            continue;
        };
        return Some(Said::warning(format!(
            "This {part}'s {name} is at {volts:.1} V, {side} its {rail} — a real one can be damaged by an input outside its supply; keep both between V− and V+."
        )));
    }
    let ceiling = high - COMMON_MODE_HEADROOM;
    if inputs
        .iter()
        .all(|(_, volts)| volts.is_some_and(|volts| volts > ceiling))
    {
        return Some(Said::warning(format!(
            "Both of this {part}'s inputs are within 1.5 V of V+ — a real one cannot compare them there, whatever this shows; bring one lower."
        )));
    }
    if matches!(element.kind, ElementKind::Comparator)
        && !pulled_up(cx, id, element.pins[element.pin_index("out")?].net?)?
    {
        return Some(Said::warning(
            "This LM393 can only pull its output down — give out a resistor up to +, or a real one floats when it should be high.",
        ));
    }
    None
}

/// An HC-SR501 settles for a minute after power-up and answers nothing while
/// it does (C-116): said, or a beginner's first wave in front of it looks like
/// a part that does not work.
fn pir_settling(cx: &Context, id: ElementId, element: &Element) -> Option<Said> {
    if !matches!(element.kind, ElementKind::Pir { .. }) {
        return None;
    }
    use crate::solver::device::pir;
    if cx.pin_volts(element, pir::VCC)? - cx.pin_volts(element, pir::GND)? < pir::LEAST_SUPPLY {
        return None;
    }
    let state = pir::State::decode(cx.engine.carried(id));
    if state.warm() {
        return None;
    }
    Some(Said::note(format!(
        "This PIR is still settling after power-up — it ignores movement for {:.0} s more.",
        (pir::WARM_UP - state.powered_for).ceil()
    )))
}

/// An L293D has a second supply, and it is the one the motor runs on: its
/// logic powered and VCC2 left free, every input does what it should and
/// nothing turns (C-91). Said only once the logic is powered, so a chip with
/// neither gets one line, not two.
fn l293d_unfed(cx: &Context, _: ElementId, element: &Element) -> Option<Said> {
    use crate::solver::device::driver;
    if !matches!(element.kind, ElementKind::MotorDriver) {
        return None;
    }
    let ground = cx.pin_volts(element, driver::GND)?;
    if cx.pin_volts(element, driver::VCC1)? - ground < driver::LOGIC_FROM {
        return None;
    }
    if cx
        .pin_volts(element, driver::VCC2)
        .is_some_and(|volts| volts - ground >= POWERED_VOLTS)
    {
        return None;
    }
    Some(Said::warning(
        "This L293D has nothing on VCC2 for its motors — wire it to the battery’s +.",
    ))
}

/// A CMOS input left free reads whatever is near it — a hand, the wire beside
/// it — so a gate whose output is used and one of whose inputs is not wired
/// does something nobody chose. Here it reads low through its leak, which is a
/// guess, so it is said rather than passed off (C-108, B-05). A gate nothing is
/// taken from is left alone: three of four unused is the usual way to have a
/// 74HC00. And the family runs on 6 V at most, in every kit beside a 9 V
/// battery, as the TMP36 is.
fn logic_input_free(cx: &Context, _: ElementId, element: &Element) -> Option<Said> {
    use crate::solver::device::gate;
    let chip = gate::Chip::of(&element.kind)?;
    let part = chip.part();
    if let Some(volts) = cx
        .pin_volts(element, gate::VCC)
        .zip(cx.pin_volts(element, gate::GND))
        .map(|(high, low)| high - low)
        .filter(|&volts| volts > gate::MOST_SUPPLY + 0.05)
    {
        return Some(Said::warning(format!(
            "This {part} runs on 2 to 6 V and has {volts:.1} V — give it 5 V, not the battery."
        )));
    }
    let free: Vec<&str> = gate::wiring(chip)
        .iter()
        .filter(|wiring| element.pins[wiring.output].net.is_some())
        .flat_map(|wiring| wiring.inputs.iter())
        // On no net, or on one with nothing else: a wire to nowhere is as
        // free as no wire.
        .filter(|&&pin| {
            element.pins[pin]
                .net
                .and_then(|net| cx.netlist.net(net))
                .is_none_or(|net| net.pins().len() < 2)
        })
        .map(|&pin| element.pins[pin].name)
        .collect();
    said_free(part, &free)
}

/// The same for the other CMOS chips (B-05): a CD4017's reset, a 74HC595's
/// /OE, a CD4511's lamp test left free reads whatever is near it, and the chip
/// counts, clears or blanks when nobody asked it to. Every input of theirs is
/// one the sheet says to tie, so all are looked at — once the chip has power,
/// so a chip just placed is told it has none first, and only that. A CD4013
/// half nothing is taken from is left alone, as a gate is.
fn chip_input_free(cx: &Context, _: ElementId, element: &Element) -> Option<Said> {
    use crate::solver::device::{counter, decade, decoder, flipflop, shift};
    let (part, high, low, inputs): (&str, usize, usize, Vec<usize>) = match element.kind {
        ElementKind::Decade => (
            decade::PART,
            decade::VDD,
            decade::VSS,
            decade::INPUTS.to_vec(),
        ),
        ElementKind::ShiftRegister => (shift::PART, shift::VCC, shift::GND, shift::INPUTS.to_vec()),
        ElementKind::Decoder => (
            decoder::PART,
            decoder::VDD,
            decoder::VSS,
            decoder::BITS
                .iter()
                .copied()
                .chain([decoder::LAMP_TEST, decoder::BLANKING, decoder::LATCH_ENABLE])
                .collect(),
        ),
        ElementKind::Counter => (
            counter::PART,
            counter::VDD,
            counter::VSS,
            vec![
                counter::CLOCK,
                counter::INHIBIT,
                counter::RESET,
                counter::DISPLAY_ENABLE,
            ],
        ),
        ElementKind::FlipFlop => (
            flipflop::PART,
            flipflop::VDD,
            flipflop::VSS,
            flipflop::HALVES
                .iter()
                .filter(|half| {
                    element.pins[half.q].net.is_some() || element.pins[half.not_q].net.is_some()
                })
                .flat_map(|half| [half.data, half.clock, half.reset, half.set])
                .collect(),
        ),
        _ => return None,
    };
    if cx.pin_volts(element, high)? - cx.pin_volts(element, low)? < POWERED_VOLTS {
        return None;
    }
    let free: Vec<&str> = inputs
        .into_iter()
        .filter(|&pin| {
            element.pins[pin]
                .net
                .and_then(|net| cx.netlist.net(net))
                .is_none_or(|net| net.pins().len() < 2)
        })
        .map(|pin| element.pins[pin].name)
        .collect();
    said_free(part, &free)
}

/// A part that is wired, live, and does nothing, because of the one pin left
/// free (B-05's "unconnected pin", as Denis narrowed it on 2026-10-02): a
/// potentiometer with its ends across a supply and its wiper on nothing, so
/// turning it changes nothing; a transistor switched on with nothing on its
/// collector or drain, so it switches nothing.
///
/// Only these. A part half-wired is what every part is while it is being
/// wired, and the callout opens for any finding: a rule about free pins in
/// general would speak at every step of building (C-05: unfinished is not
/// wrong). Each of these needs the part live as well — volts across the
/// track, the base or gate driven — which is a circuit somebody has finished
/// enough to expect something of. A note: nothing is harmed.
fn pin_free(cx: &Context, _: ElementId, element: &Element) -> Option<Said> {
    use crate::solver::device::{Channel, Polarity};
    // On a net with something else: a wire to nowhere is as free as none.
    let wired = |name: &str| {
        element
            .pin_index(name)
            .and_then(|pin| element.pins[pin].net)
            .and_then(|net| cx.netlist.net(net))
            .is_some_and(|net| net.pins().len() >= 2)
    };
    let across = |high: &str, low: &str| {
        Some(cx.named_volts(element, high)? - cx.named_volts(element, low)?)
    };
    match element.kind {
        ElementKind::Potentiometer { .. } => {
            if wired("wiper") || across("a", "b")?.abs() < POWERED_VOLTS {
                return None;
            }
            Some(Said::note(
                "This potentiometer's wiper is not wired, so turning it changes nothing — wire its middle pin.",
            ))
        }
        ElementKind::Bjt { polarity } => {
            let on = across("b", "e")?;
            let on = if polarity == Polarity::Pnp { -on } else { on };
            if wired("c") || on < 0.5 {
                return None;
            }
            Some(Said::note(format!(
                "This {} is switched on with nothing on its collector — wire the load to c.",
                polarity.part_number()
            )))
        }
        ElementKind::Mosfet { channel } => {
            let on = across("g", "s")?;
            let on = if channel == Channel::P { -on } else { on };
            if wired("d") || on < POWERED_VOLTS {
                return None;
            }
            Some(Said::note(format!(
                "This {} is switched on with nothing on its drain — wire the load to d.",
                channel.part_number()
            )))
        }
        _ => None,
    }
}

/// "This 74HC00's input 1A is not wired — …", for the free inputs named.
fn said_free(part: &str, free: &[&str]) -> Option<Said> {
    let (last, rest) = free.split_last()?;
    let named = if rest.is_empty() {
        format!("input {last} is")
    } else {
        format!("inputs {} and {last} are", rest.join(", "))
    };
    Some(Said::warning(format!(
        "This {part}'s {named} not wired — a free input reads whatever is near it; tie it to + or −."
    )))
}

/// "D9", "D9 and D10", "D2, D3 and D4": an Uno's pins, by the names on its
/// headers.
fn uno_pins(pins: &[usize]) -> Option<String> {
    use crate::solver::device::uno::PINS;
    let (last, rest) = pins.split_last()?;
    Some(if rest.is_empty() {
        PINS[*last].to_string()
    } else {
        let rest: Vec<&str> = rest.iter().map(|&pin| PINS[pin]).collect();
        format!("{} and {}", rest.join(", "), PINS[*last])
    })
}

/// An Uno's pins as its program last left them, once it has run; `None`
/// before, when every pin is an input only because nothing has set it yet.
fn uno_drives(
    cx: &Context,
    id: ElementId,
) -> Option<[crate::solver::device::uno::Drive; crate::solver::device::uno::IO]> {
    cx.netlist.program(id)?;
    Some(crate::solver::device::uno::decode(
        cx.engine.carried(id)?.value,
    ))
}

/// What on a net is a coil a pin should not drive: a motor, or a relay's
/// coil.
fn coil_on(netlist: &Netlist, net: NetId) -> Option<&'static str> {
    use crate::solver::device::relay::{COIL_MINUS, COIL_PLUS};
    netlist
        .net(net)?
        .pins()
        .iter()
        .find_map(|pin| match netlist.element(pin.element)?.kind {
            ElementKind::Motor => Some("a motor"),
            ElementKind::Relay { .. } if matches!(pin.index, COIL_PLUS | COIL_MINUS) => {
                Some("a relay's coil")
            }
            _ => None,
        })
}

/// A motor or a relay's coil straight on an Uno's pin (A-10): it wants far
/// more than the pin's 20 mA, and kicks back when let go. Read off the
/// wiring, so it is said whatever the program does with the pin.
fn uno_coil(netlist: &Netlist, _: ElementId, element: &Element) -> Option<Said> {
    use crate::solver::device::uno;
    if !matches!(element.kind, ElementKind::Uno) {
        return None;
    }
    let (pin, coil) = (0..uno::IO).find_map(|pin| {
        let coil = coil_on(netlist, element.pins[pin].net?)?;
        Some((pin, coil))
    })?;
    Some(Said::warning(format!(
        "This Uno's {} drives {coil} straight — it wants more than a pin gives, and kicks back; switch it with a transistor, a diode across it.",
        uno::PINS[pin]
    )))
}

/// An Uno's pin above the 5.5 V the chip can take (A-10, §29.1): its
/// protection diodes conduct into its supply, and past what they take the
/// chip is damaged. Nothing in the model gives way, so it is said. VIN is
/// the barrel jack's, and takes up to 20 V.
fn uno_overvolted(cx: &Context, _: ElementId, element: &Element) -> Option<Said> {
    use crate::solver::device::uno;
    if !matches!(element.kind, ElementKind::Uno) {
        return None;
    }
    let ground = cx.pin_volts(element, uno::GND)?;
    let over: Vec<(usize, f64)> = (0..uno::IO)
        .chain([uno::FIVE_VOLTS, uno::THREE_VOLTS])
        .filter_map(|pin| Some((pin, cx.pin_volts(element, pin)? - ground)))
        .filter(|&(_, volts)| volts > uno::MOST_PIN_VOLTS + 0.05)
        .collect();
    let most = over.iter().map(|&(_, volts)| volts).reduce(f64::max)?;
    let pins: Vec<usize> = over.iter().map(|&(pin, _)| pin).collect();
    let is = if pins.len() == 1 {
        "is at"
    } else {
        "are at up to"
    };
    Some(Said::warning(format!(
        "This Uno's {} {is} {most:.1} V, past the 5.5 V a pin can take — bring it under 5 V, with a divider.",
        uno_pins(&pins)?
    )))
}

/// An Uno's pins asked for more than they can give (A-10): Arduino rates a
/// pin at 20 mA, 40 mA damages it, and the chip passes 200 mA at most through
/// its VCC and its GND whatever each pin carries. The model's pins give what
/// their resistance lets them, so the numbers are the ones a real board would
/// try for. A pin with a coil on it is `uno_coil`'s to say.
fn uno_overloaded(cx: &Context, id: ElementId, element: &Element) -> Option<Said> {
    use crate::solver::device::uno;
    if !matches!(element.kind, ElementKind::Uno) {
        return None;
    }
    let into = cx.pin_currents(id)?;
    let over: Vec<usize> = (0..uno::IO)
        .filter(|&pin| into[pin].abs() > uno::PIN_RATED_AMPS * PAST_RATING)
        .filter(|&pin| {
            element.pins[pin]
                .net
                .is_none_or(|net| coil_on(cx.netlist, net).is_none())
        })
        .collect();
    if let Some(&worst) = over
        .iter()
        .max_by(|&&a, &&b| into[a].abs().total_cmp(&into[b].abs()))
    {
        let amps = into[worst].abs();
        let carries = crate::value::format(two_figures(amps), "A");
        let named = if over.len() == 1 {
            format!("{} carries {carries}", uno::PINS[worst])
        } else {
            format!("{} carry up to {carries}", uno_pins(&over)?)
        };
        let damage = if amps > uno::PIN_MOST_AMPS {
            "past the 40 mA that damages it"
        } else {
            "and 40 mA damages it"
        };
        return Some(Said::warning(format!(
            "This Uno's {named}; a pin is rated for 20 mA, {damage} — limit it with a resistor."
        )));
    }
    // Out of the pins high, and into the pins low.
    let (out, sunk) = into[..uno::IO]
        .iter()
        .fold((0.0, 0.0), |(out, sunk), &amps| {
            if amps < 0.0 {
                (out - amps, sunk)
            } else {
                (out, sunk + amps)
            }
        });
    let total = f64::max(out, sunk);
    if total <= uno::CHIP_MOST_AMPS * PAST_RATING {
        return None;
    }
    Some(Said::warning(format!(
        "This Uno's pins carry {} between them, past the 200 mA the chip can pass — drive the heavier loads through transistors.",
        crate::value::format(two_figures(total), "A")
    )))
}

/// What an Uno's input reads when nothing sets it high or low (A-10): an
/// input whose wire has nothing on it but a button, a switch, or other
/// inputs. With the contact open it is on nothing — it reads whatever is
/// near it, and the program sees presses nobody made. Judged on the pins the
/// program has left as plain inputs: one it set to `INPUT_PULLUP` is held
/// high. D0 and D1 are the USB-serial chip's, and left alone.
fn uno_input_floating(cx: &Context, id: ElementId, element: &Element) -> Option<Said> {
    use crate::solver::device::uno::{self, Drive};
    if !matches!(element.kind, ElementKind::Uno) {
        return None;
    }
    let drives = uno_drives(cx, id)?;
    for pin in uno::D1 + 1..uno::IO {
        if drives[pin] != Drive::Input {
            continue;
        }
        let Some(net) = element.pins[pin].net.and_then(|net| cx.netlist.net(net)) else {
            continue;
        };
        if net.is_floating() {
            continue;
        }
        // The contact on its wire, if that is all there is, and which rail
        // its other side goes to.
        let mut contact: Option<(&str, Option<Direction>)> = None;
        let held = net.pins().iter().any(|on| {
            let Some(other) = cx.netlist.element(on.element) else {
                return false;
            };
            if on.element == id {
                // Another of the Uno's own pins: driven, or an input too.
                return on.index >= uno::IO || drives[on.index] != Drive::Input;
            }
            let noun = match other.kind {
                ElementKind::PushButton { .. } => "button",
                ElementKind::Switch { .. } => "switch",
                ElementKind::TiltSwitch { .. } => "tilt switch",
                ElementKind::Reed { .. } => "reed switch",
                _ => return true,
            };
            let rail = other
                .pins
                .get(1 - on.index)
                .and_then(|far| far.net)
                .and_then(|far| rail_of(cx.netlist, far));
            if contact.is_none_or(|(_, seen)| seen.is_none()) {
                contact = Some((noun, rail));
            }
            false
        });
        if held {
            continue;
        }
        let name = uno::PINS[pin];
        let message = match contact {
            Some((noun, Some(Direction::Up))) => format!(
                "This Uno's {name} floats while its {noun} is open — give {name} a 10 kΩ resistor to GND."
            ),
            Some((noun, Some(Direction::Down))) => format!(
                "This Uno's {name} floats while its {noun} is open — set it to INPUT_PULLUP, or give it a 10 kΩ resistor to 5V."
            ),
            _ => format!(
                "This Uno's {name} is an input on nothing that sets it high or low — it reads whatever is near it."
            ),
        };
        return Some(Said::warning(message));
    }
    None
}

/// Which rail a net is: up for an Uno's 5V or 3V3 or a source's +, down for
/// an Uno's GND, a source's − or a ground symbol.
fn rail_of(netlist: &Netlist, net: NetId) -> Option<Direction> {
    use crate::solver::device::uno;
    netlist
        .net(net)?
        .pins()
        .iter()
        .find_map(|pin| match netlist.element(pin.element)?.kind {
            ElementKind::Ground => Some(Direction::Down),
            ElementKind::Uno => match pin.index {
                uno::FIVE_VOLTS | uno::THREE_VOLTS => Some(Direction::Up),
                uno::GND => Some(Direction::Down),
                _ => None,
            },
            ElementKind::DcSource { .. } | ElementKind::Cell { .. } => match pin.index {
                0 => Some(Direction::Up),
                _ => Some(Direction::Down),
            },
            _ => None,
        })
}

/// A battery used up says so (C-121).
fn flat_battery(cx: &Context, id: ElementId, element: &Element) -> Option<Said> {
    let ElementKind::Cell { charge, .. } = element.kind else {
        return None;
    };
    use crate::solver::device::cell::State;
    let state = cx
        .engine
        .carried(id)
        .map_or_else(State::default, State::decode);
    if state.depth(charge) < 1.0 {
        return None;
    }
    Some(Said::warning("This battery is flat."))
}

/// A fuse that has done its job says so, and what to do next (C-120).
fn fuse_gone(cx: &Context, id: ElementId, element: &Element) -> Option<Said> {
    use crate::solver::device::fuse::FusePart;
    let ElementKind::Fuse { part, blown, .. } = element.kind else {
        return None;
    };
    let tripped = cx.engine.carried(id).is_some_and(|carried| carried.latched);
    match part {
        FusePart::Fast if blown => Some(Said::warning(
            "This fuse has blown. Find what drew too much, then replace it.",
        )),
        FusePart::Polyfuse if tripped => Some(Said::note(
            "This polyfuse has tripped. Once the fault is gone it cools, and conducts again.",
        )),
        _ => None,
    }
}

/// How far past its rating a part has to be before it is told: enough that
/// the two numbers in the sentence differ once rounded, and that a circuit
/// sitting on the line does not flicker in and out of a warning.
const PAST_RATING: f64 = 1.05;

/// A part carrying more current, or turning more power into heat, than it is
/// built for (B-03, design doc §6.1).
///
/// The ratings are the registry's (C-16), each read off the sheet of the part
/// the model is based on; a part with none is not judged. The current is the
/// one the canvas shows flowing — through a two-pin part, the largest at any
/// pin of one with more, the brightest LED of a digit — and the power, for a
/// part with a package, the one heating it (C-74). Current first: a beginner
/// knows what a milliamp is before a watt, and a part past both needs telling
/// once. Neither breaks it in the simulation, so making the change makes the
/// line go.
///
/// An LED whose resistor is too small is told the resistor it needs, and
/// offered it, as an LED with none is (B-02).
fn over_rated(cx: &Context, id: ElementId, element: &Element) -> Option<Said> {
    let rating = crate::registry::rating_of(&element.kind);
    let name = part_name(&element.kind);
    if let Some(rated) = rating.current {
        let amps = carried(cx, id, element).unwrap_or(0.0);
        if amps > rated * PAST_RATING {
            let (carries, rated) = (
                crate::value::format(two_figures(amps), "A"),
                crate::value::format(rated, "A"),
            );
            let message = format!(
                "This {name} carries {carries}; it is rated for {rated} and will burn out."
            );
            return Some(Said {
                fix: resistor_for(cx, id, element, amps),
                ..Said::warning(message)
            });
        }
    }
    let rated = rating.power?;
    let watts = cx
        .engine
        .heat(id)
        .map(|heat| heat.watts)
        .or_else(|| dissipated(cx, id, element))?;
    if watts <= rated * PAST_RATING {
        return None;
    }
    Some(Said::warning(format!(
        "This {name} turns {} into heat; it is rated for {} and will burn out.",
        crate::value::format(two_figures(watts), "W"),
        crate::value::format(rated, "W"),
    )))
}

/// What a beginner calls the part: a transistor by the number printed on it,
/// which is how the sheet beside it is found.
fn part_name(kind: &ElementKind) -> String {
    use crate::solver::device::RegulatorPart;
    match *kind {
        ElementKind::Led { colour } => format!("{} LED", colour.name().to_lowercase()),
        ElementKind::Bjt { polarity } => polarity.part_number().into(),
        ElementKind::Mosfet { channel } => channel.part_number().into(),
        ElementKind::Zener { part } => format!("{} Zener", part.part_number()),
        ElementKind::Regulator { part } => part.name().into(),
        ElementKind::Adjustable => RegulatorPart::Lm317.name().into(),
        ElementKind::RgbLed { .. } => "RGB LED".into(),
        ElementKind::SevenSegment { .. } => "digit".into(),
        ElementKind::Bargraph => "bar graph".into(),
        ElementKind::Schottky => "Schottky diode".into(),
        ElementKind::Bridge => "bridge rectifier".into(),
        ElementKind::Ldr { .. } => "LDR".into(),
        ElementKind::OpAmp => "op-amp".into(),
        ElementKind::Timer => "555".into(),
        _ => kind.name().to_lowercase(),
    }
}

/// The current a part carries, in amps, from what the solution read: through
/// a two-pin part; into the pin its rating is about, for one with more; the
/// most any one LED of a digit, an RGB LED or a bar graph carries, each being
/// rated as one.
fn carried(cx: &Context, id: ElementId, element: &Element) -> Option<f64> {
    use crate::solver::device::Led;
    let across = |anode: usize, cathode: usize| -> Option<f64> {
        Some(cx.pin_volts(element, anode)? - cx.pin_volts(element, cathode)?)
    };
    let brightest = |leds: Vec<(usize, usize, LedColour)>| {
        leds.into_iter()
            .filter_map(|(anode, cathode, colour)| {
                Some(Led::new(colour).forward_current(across(anode, cathode)?))
            })
            .reduce(f64::max)
    };
    if let ElementKind::Bargraph = element.kind {
        return brightest((0..10).map(|bar| (bar, 10 + bar, LedColour::Red)).collect());
    }
    if let Some((common, colours)) = element.kind.shared_leds() {
        let shared = colours.len();
        return brightest(
            colours
                .iter()
                .enumerate()
                .map(|(led, &colour)| {
                    let (anode, cathode) = common.junction(led, shared);
                    (anode, cathode, colour)
                })
                .collect(),
        );
    }
    if let Some(into) = cx.pin_currents(id) {
        return rated_pins(&element.kind)
            .iter()
            .filter_map(|&name| into.get(element.pin_index(name)?))
            .map(|amps| amps.abs())
            .reduce(f64::max);
    }
    cx.through(id).map(f64::abs)
}

/// The pins a part with more than two is rated at: where the current its
/// sheet limits flows.
fn rated_pins(kind: &ElementKind) -> &'static [&'static str] {
    match kind {
        ElementKind::Mosfet { .. } => &["d"],
        ElementKind::Bjt { .. } => &["c"],
        // Not an op-amp's or a comparator's output: each is rated at the
        // limit its own model holds it to, so it cannot be past it.
        ElementKind::Timer => &["OUT"],
        ElementKind::Regulator { .. } | ElementKind::Adjustable => &["out"],
        // Its contact; its coil is rated by its voltage.
        ElementKind::Relay { .. } => &["com"],
        // Either side of its gap.
        ElementKind::Optocoupler => &["A", "C"],
        // Its output, which all four diodes share.
        ElementKind::Bridge => &["+"],
        _ => &[],
    }
}

/// The power a part with no package of its own turns into heat: what flows in
/// at each pin times the potential it flows in at, which for two pins is the
/// drop times the current.
fn dissipated(cx: &Context, id: ElementId, element: &Element) -> Option<f64> {
    let watts = match cx.pin_currents(id) {
        Some(into) => into
            .iter()
            .enumerate()
            .map(|(pin, amps)| Some(cx.pin_volts(element, pin)? * amps))
            .sum::<Option<f64>>()?,
        None => (cx.pin_volts(element, 0)? - cx.pin_volts(element, 1)?) * cx.through(id)?,
    };
    Some(watts.max(0.0))
}

/// The value an LED's own resistor should have, when it has one in series
/// with it alone — on a wire that joins nothing else — and it is too small:
/// the drop across the two together is held, and the LED aimed at what B-02
/// aims one at.
fn resistor_for(cx: &Context, id: ElementId, element: &Element, amps: f64) -> Option<Fix> {
    let ElementKind::Led { colour } = element.kind else {
        return None;
    };
    let (resistor, ohms) = element.pins.iter().find_map(|pin| {
        let net = cx.netlist.net(pin.net?)?;
        let [first, second] = net.pins() else {
            return None;
        };
        let other = if first.element == id { second } else { first };
        match cx.netlist.element(other.element)?.kind {
            ElementKind::Resistor { ohms } if other.element != id => Some((other.element, ohms)),
            _ => None,
        }
    })?;
    let led_drop = cx.pin_volts(element, 0)? - cx.pin_volts(element, 1)?;
    let headroom =
        amps * ohms + led_drop - crate::solver::device::Led::new(colour).nominal_forward_voltage();
    let wanted = e12_at_least(headroom / AIMED_AMPS);
    (wanted > ohms).then(|| Fix::SetValue {
        element: encode(resistor),
        value: crate::value::format_editable(wanted),
    })
}

/// A reading rounded to two significant figures: what a sentence about it
/// needs, and no more than the model can stand behind.
fn two_figures(value: f64) -> f64 {
    if value == 0.0 || !value.is_finite() {
        return value;
    }
    let scale = 10f64.powf(value.abs().log10().floor() - 1.0);
    (value / scale).round() * scale
}

/// What an LED is aimed at when the rule works out its resistor: four fifths
/// of its 20 mA rating, room for a battery that reads a little high — and on
/// 9 V a red LED comes out at the 470 Ω every kit has.
const AIMED_AMPS: f64 = 16e-3;

/// Forward drops along a string, for a diode the path passes through.
const SILICON_DROP: f64 = 0.7;
const SCHOTTKY_DROP: f64 = 0.3;

/// An LED with nothing to limit its current (B-02, design doc §6.1).
///
/// Fires when there is a loop from a source's + through the LED to the same
/// source's − that passes only through parts that do not limit current:
/// wires, switches and buttons, fuses, ammeters, and other LEDs and diodes in
/// series. A switch counts whether it is open or shut — pressing it is the
/// moment the LED dies — and a source with real resistance of its own
/// (C-17) counts as the limit it is. Anything else in the loop, a chip's pin
/// or a transistor, is something this rule cannot judge, and it says nothing.
///
/// For a digit or an RGB LED, each of its LEDs is looked at; it is said once,
/// for the part. The resistor it suggests is worked out from the supply and
/// the drops along the loop, at [`AIMED_AMPS`], and rounded up to a value
/// that can be bought (E12, from B-08's tables).
fn led_unlimited(netlist: &Netlist, id: ElementId, element: &Element) -> Option<Said> {
    // Each LED in the part: its anode pin, its cathode pin, its colour.
    let junctions: Vec<(usize, usize, LedColour)> = match element.kind {
        ElementKind::Led { colour } => vec![(0, 1, colour)],
        _ => {
            let (common, colours) = element.kind.shared_leds()?;
            let shared = colours.len();
            colours
                .iter()
                .enumerate()
                .map(|(led, &colour)| {
                    let (anode, cathode) = common.junction(led, shared);
                    (anode, cathode, colour)
                })
                .collect()
        }
    };

    // The largest resistor any unlimited LED in it needs: the one that keeps
    // every one of them safe.
    let needed = junctions
        .iter()
        .filter_map(|&(anode, cathode, colour)| {
            let own = crate::solver::device::Led::new(colour).nominal_forward_voltage();
            unlimited_loop(
                netlist,
                id,
                element.pins.get(anode)?.net?,
                element.pins.get(cathode)?.net?,
                own,
            )
        })
        .fold(None, |most: Option<f64>, ohms| {
            Some(most.map_or(ohms, |most| most.max(ohms)))
        })?;

    let ohms = e12_at_least(needed);
    let written = crate::value::format(ohms, "Ω");
    let message = match element.kind {
        ElementKind::Led { colour } => format!(
            "This {} LED has nothing to limit its current — put a {written} resistor in series with it.",
            colour.name().to_lowercase()
        ),
        ElementKind::RgbLed { .. } => format!(
            "This RGB LED has nothing to limit its current — give each colour its own {written} resistor."
        ),
        _ => format!(
            "This digit has nothing to limit its current — give each segment its own {written} resistor."
        ),
    };
    Some(Said {
        fix: Some(Fix::AddResistor {
            element: encode(id),
            value: crate::value::format_editable(ohms),
        }),
        ..Said::warning(message)
    })
}

/// The resistor a loop needs, if there is a loop from a source's + to this
/// LED's anode and from its cathode back to the same source's − with nothing
/// in it that limits current; `None` if there is no such loop.
fn unlimited_loop(
    netlist: &Netlist,
    led: ElementId,
    anode: NetId,
    cathode: NetId,
    own_drop: f64,
) -> Option<f64> {
    // Up from the anode to a source's +, and down from the cathode to a −.
    let ups = reach(netlist, led, anode, Direction::Up);
    let downs = reach(netlist, led, cathode, Direction::Down);
    ups.found
        .iter()
        .filter_map(|&(source, up_drop)| {
            // Back to this source's −, or to ground with its − grounded too:
            // two ground symbols are one node (ADR 0003).
            let down_drop = downs
                .found
                .iter()
                .find(|&&(other, _)| other == source)
                .map(|&(_, drop)| drop)
                .or_else(|| downs.ground.filter(|_| grounded(netlist, source)))?;
            let (volts, series_ohms) = source_of(netlist, source)?;
            let headroom = volts - own_drop - up_drop - down_drop;
            // Not enough to light it at all: a resistor would be no answer.
            if headroom < 0.2 {
                return None;
            }
            let ohms = headroom / AIMED_AMPS;
            // A source that limits as much as half of it is the limit.
            if series_ohms >= ohms / 2.0 {
                return None;
            }
            Some(ohms - series_ohms)
        })
        .fold(None, |most: Option<f64>, ohms| {
            Some(most.map_or(ohms, |most| most.max(ohms)))
        })
}

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    /// Towards a source's +: a diode in the way is crossed cathode to anode.
    Up,
    /// Towards a source's −: a diode is crossed anode to cathode.
    Down,
}

/// What can be reached from a net through parts that do not limit current.
struct Reached {
    /// Each source whose + (going up) or − (going down) was reached, with the
    /// forward drop on the way.
    found: Vec<(ElementId, f64)>,
    /// A ground symbol, going down, and the drop on the way to it.
    ground: Option<f64>,
    /// Each logic chip whose output was reached, either way: an output drives
    /// both high and low, and what it drives is not limited by it (B-04).
    drivers: Vec<ElementId>,
}

/// Whether a source's pin is the one a search that way ends at: its + going
/// up and its − going down. An Uno's 5V going up and its GND going down, and
/// any of its I/O pins either way: the program drives each high or low as it
/// likes (A-10).
fn terminal(kind: &ElementKind, pin: usize, direction: Direction) -> bool {
    use crate::solver::device::uno;
    match (kind, direction) {
        (ElementKind::Uno, Direction::Up) => pin < uno::IO || pin == uno::FIVE_VOLTS,
        (ElementKind::Uno, Direction::Down) => pin < uno::IO || pin == uno::GND,
        (_, Direction::Up) => pin == 0,
        (_, Direction::Down) => pin == 1,
    }
}

/// Whether a source's − is on a net with a ground symbol.
fn grounded(netlist: &Netlist, source: ElementId) -> bool {
    netlist
        .element(source)
        .and_then(|element| {
            let minus = match element.kind {
                ElementKind::Uno => crate::solver::device::uno::GND,
                _ => 1,
            };
            element.pins.get(minus)?.net
        })
        .and_then(|net| netlist.net(net))
        .is_some_and(|net| {
            net.pins().iter().any(|pin| {
                netlist
                    .element(pin.element)
                    .is_some_and(|other| matches!(other.kind, ElementKind::Ground))
            })
        })
}

/// Every source whose + (going up) or − (going down) can be reached from a
/// net through parts that do not limit current, with the forward drop on the
/// way. The part being judged is not a way through.
fn reach(netlist: &Netlist, part: ElementId, from: NetId, direction: Direction) -> Reached {
    let mut found: Vec<(ElementId, f64)> = Vec::new();
    let mut ground: Option<f64> = None;
    let mut drivers: Vec<ElementId> = Vec::new();
    let mut seen: Vec<NetId> = vec![from];
    let mut queue: Vec<(NetId, f64)> = vec![(from, 0.0)];
    while let Some((net_id, drop)) = queue.pop() {
        let Some(net) = netlist.net(net_id) else {
            continue;
        };
        for pin in net.pins() {
            if pin.element == part {
                continue;
            }
            let Some(element) = netlist.element(pin.element) else {
                continue;
            };
            if matches!(element.kind, ElementKind::Ground) {
                if direction == Direction::Down && ground.is_none() {
                    ground = Some(drop);
                }
                continue;
            }
            // A source's terminal on this net ends the search that way.
            if source_of(netlist, pin.element).is_some() {
                if terminal(&element.kind, pin.index, direction)
                    && !found.iter().any(|&(source, _)| source == pin.element)
                {
                    found.push((pin.element, drop));
                }
                continue;
            }
            if logic_output(&element.kind, pin.index) {
                if !drivers.contains(&pin.element) {
                    drivers.push(pin.element);
                }
                continue;
            }
            // Through a part that does not limit current, to its other pin.
            let Some(step) = passes(&element.kind, pin.index, direction) else {
                continue;
            };
            let Some(next) = element.pins.get(step.to).and_then(|pin| pin.net) else {
                continue;
            };
            if !seen.contains(&next) {
                seen.push(next);
                queue.push((next, drop + step.drop));
            }
        }
    }
    Reached {
        found,
        ground,
        drivers,
    }
}

/// Whether a pin is a logic chip's output: a gate's, a 555's OUT, a
/// 74HC595's, a CD4017's or a CD4013's Q and /Q. Each drives its pin to a
/// rail through little more than a transistor's own resistance.
fn logic_output(kind: &ElementKind, pin: usize) -> bool {
    use crate::solver::device::{decade, flipflop, gate, shift, timer};
    if let Some(chip) = gate::Chip::of(kind) {
        return gate::wiring(chip).iter().any(|wiring| wiring.output == pin);
    }
    match kind {
        ElementKind::Timer => pin == timer::OUT,
        ElementKind::ShiftRegister => shift::OUTPUTS.contains(&pin),
        ElementKind::Decade => decade::OUTPUTS.contains(&pin),
        ElementKind::FlipFlop => flipflop::HALVES
            .iter()
            .any(|half| half.q == pin || half.not_q == pin),
        _ => false,
    }
}

/// What a logic chip is called, by the number printed on it.
fn chip_name(kind: &ElementKind) -> &'static str {
    use crate::solver::device::{flipflop, gate};
    if let Some(chip) = gate::Chip::of(kind) {
        return chip.part();
    }
    match kind {
        ElementKind::Timer => "555",
        ElementKind::ShiftRegister => "74HC595",
        ElementKind::Decade => "CD4017",
        ElementKind::FlipFlop => flipflop::PART,
        _ => "chip",
    }
}

struct Step {
    to: usize,
    drop: f64,
}

/// Where current goes on through a part that does not limit it, entering at
/// one pin: its other pin and the drop across it. `None` for anything that
/// limits current or that this rule cannot judge.
fn passes(kind: &ElementKind, at: usize, direction: Direction) -> Option<Step> {
    let free = |drop: f64| Some(Step { to: 1 - at, drop });
    // A diode is crossed only the way it conducts: going up, in at its
    // cathode (pin 1) to its anode; going down, the other way.
    let diode = |drop: f64| match (direction, at) {
        (Direction::Up, 1) | (Direction::Down, 0) => free(drop),
        _ => None,
    };
    match *kind {
        ElementKind::Switch { .. }
        | ElementKind::PushButton { .. }
        | ElementKind::TiltSwitch { .. }
        | ElementKind::Reed { .. }
        | ElementKind::Fuse { .. }
        | ElementKind::Ammeter => free(0.0),
        ElementKind::Led { colour } => {
            diode(crate::solver::device::Led::new(colour).nominal_forward_voltage())
        }
        ElementKind::Diode => diode(SILICON_DROP),
        ElementKind::Schottky => diode(SCHOTTKY_DROP),
        _ => None,
    }
}

/// A transistor whose base is wired straight to a battery or to a logic
/// output, with nothing to limit its current (B-04, design doc §6.1).
///
/// The base-emitter junction is a diode, and across a battery it is the LED
/// with no resistor all over again: it draws what the battery can give and
/// the transistor dies, or the chip driving it does. Fires when the base
/// reaches a source's + (a 2N3906's, its −) or a chip's output through
/// parts that do not limit current, and the emitter returns to the other side
/// the same way — so an emitter follower, whose emitter resistor is the
/// limit, is left alone.
///
/// The resistor it suggests is worked out from what the collector drives:
/// twice the base current the part's β needs to carry it, so a part with less
/// gain than the model's still saturates, rounded down to a value that can be
/// bought. A TIP120 has ten times the β of a 2N3904 and needs twice the
/// voltage, so its resistor comes out about ten times larger for the same
/// load. With a collector whose load it cannot read, it says so without a
/// value.
fn base_unlimited(netlist: &Netlist, id: ElementId, element: &Element) -> Option<Said> {
    use crate::solver::device::{BASE, COLLECTOR, EMITTER};
    use crate::solver::device::{Bjt, Polarity};
    let ElementKind::Bjt { polarity } = element.kind else {
        return None;
    };
    // An NPN's base is pulled up and its emitter down; a PNP's the other way.
    let (to_base, to_emitter) = match polarity {
        Polarity::Pnp => (Direction::Down, Direction::Up),
        _ => (Direction::Up, Direction::Down),
    };
    let base = reach(netlist, id, element.pins.get(BASE)?.net?, to_base);
    let emitter = reach(netlist, id, element.pins.get(EMITTER)?.net?, to_emitter);
    let (up, down) = match polarity {
        Polarity::Pnp => (&emitter, &base),
        _ => (&base, &emitter),
    };

    let (driver, volts, drop) = match same_loop(netlist, up, down) {
        Some((source, drop)) => {
            let noun = match netlist.element(source)?.kind {
                ElementKind::SignalGenerator { .. } => "the signal generator",
                ElementKind::Uno => "the Uno",
                _ => "the battery",
            };
            (noun.to_string(), source_of(netlist, source)?.0, drop)
        }
        None => {
            let &chip = base.drivers.first()?;
            // Returned to a rail, or the output has no loop to drive.
            let rail = emitter
                .found
                .first()
                .map(|&(source, _)| source)
                .or_else(|| {
                    emitter.ground?;
                    netlist
                        .elements()
                        .map(|(source, _)| source)
                        .find(|&source| {
                            source_of(netlist, source).is_some() && grounded(netlist, source)
                        })
                })?;
            let name = chip_name(&netlist.element(chip)?.kind);
            (
                format!("the {name}'s output"),
                source_of(netlist, rail)?.0,
                0.0,
            )
        }
    };

    let part = polarity.part_number();
    let model = Bjt::new(polarity);
    let (junction, saturated) = match polarity {
        Polarity::Darlington => (1.4, 0.8),
        _ => (0.7, 0.2),
    };
    let collector = element.pins.get(COLLECTOR)?.net;
    let ohms = collector
        .and_then(|net| load_on(netlist, id, net, to_base))
        .map(|(load_volts, load_drop, load_ohms)| (load_volts - load_drop - saturated) / load_ohms)
        .filter(|amps| *amps > 0.0)
        .map(|amps| (volts - junction - drop) / (2.0 * amps / model.forward_beta))
        .filter(|ohms| *ohms >= 1.0)
        .map(e12_at_most);
    let Some(ohms) = ohms else {
        return Some(Said::warning(format!(
            "This {part}'s base is wired straight to {driver} — it needs a resistor in series."
        )));
    };
    let written = crate::value::format(ohms, "Ω");
    Some(Said {
        fix: Some(Fix::AddResistor {
            element: encode(id),
            value: crate::value::format_editable(ohms),
        }),
        ..Said::warning(format!(
            "This {part}'s base is wired straight to {driver} — put a {written} resistor in series with it."
        ))
    })
}

/// A source whose + one search reached and whose − the other did — or, for
/// the −, a ground symbol, with the source's − grounded too (ADR 0003) — and
/// the drops along both ways.
fn same_loop(netlist: &Netlist, up: &Reached, down: &Reached) -> Option<(ElementId, f64)> {
    up.found.iter().find_map(|&(source, up_drop)| {
        let down_drop = down
            .found
            .iter()
            .find(|&&(other, _)| other == source)
            .map(|&(_, drop)| drop)
            .or_else(|| down.ground.filter(|_| grounded(netlist, source)))?;
        Some((source, up_drop + down_drop))
    })
}

/// What a transistor's collector drives: the first source reached from it,
/// through the parts a load is made of — a resistor, a motor, a buzzer, a
/// relay's coil — and those that limit nothing; with that source's voltage,
/// the forward drops on the way and the resistance. `None` for a load it
/// cannot read, or none at all.
fn load_on(
    netlist: &Netlist,
    part: ElementId,
    from: NetId,
    direction: Direction,
) -> Option<(f64, f64, f64)> {
    use crate::solver::device::{load, pitch, relay};
    let mut seen: Vec<NetId> = vec![from];
    let mut queue: std::collections::VecDeque<(NetId, f64, f64)> = [(from, 0.0, 0.0)].into();
    while let Some((net_id, drop, ohms)) = queue.pop_front() {
        for pin in netlist.net(net_id)?.pins() {
            if pin.element == part {
                continue;
            }
            let Some(element) = netlist.element(pin.element) else {
                continue;
            };
            if let Some((volts, _)) = source_of(netlist, pin.element) {
                if terminal(&element.kind, pin.index, direction) && ohms > 0.0 {
                    return Some((volts, drop, ohms));
                }
                continue;
            }
            let step = match element.kind {
                ElementKind::Resistor { ohms } => Some((1 - pin.index, 0.0, ohms)),
                ElementKind::Motor => Some((1 - pin.index, 0.0, load::MOTOR_OHMS)),
                ElementKind::Buzzer => Some((1 - pin.index, 0.0, load::BUZZER_OHMS)),
                ElementKind::PassiveBuzzer => Some((1 - pin.index, 0.0, pitch::OHMS)),
                ElementKind::Relay { part }
                    if matches!(pin.index, relay::COIL_PLUS | relay::COIL_MINUS) =>
                {
                    Some((1 - pin.index, 0.0, part.coil_ohms()))
                }
                ref kind => {
                    passes(kind, pin.index, direction).map(|step| (step.to, step.drop, 0.0))
                }
            };
            let Some((to, more_drop, more_ohms)) = step else {
                continue;
            };
            let Some(next) = element.pins.get(to).and_then(|pin| pin.net) else {
                continue;
            };
            if !seen.contains(&next) {
                seen.push(next);
                queue.push_back((next, drop + more_drop, ohms + more_ohms));
            }
        }
    }
    None
}

/// A source's voltage and its own series resistance: an Uno's, its USB
/// supply's.
fn source_of(netlist: &Netlist, id: ElementId) -> Option<(f64, f64)> {
    use crate::solver::device::uno;
    match netlist.element(id)?.kind {
        ElementKind::Uno => Some((uno::SUPPLY, uno::SUPPLY_OHMS)),
        ElementKind::DcSource { volts, series_ohms } => Some((volts.abs(), series_ohms)),
        ElementKind::SignalGenerator { volts, .. } => Some((volts.abs(), 0.0)),
        ElementKind::Cell { part, .. } => {
            let sheet = part.sheet();
            Some((sheet.open_volts(0.0), sheet.series_ohms()))
        }
        _ => None,
    }
}

/// The smallest E12 value at least this large: a resistor that can be bought,
/// and as much as was asked for (B-08's tables).
fn e12_at_least(ohms: f64) -> f64 {
    crate::series::at_least(ohms, crate::series::Series::E12).unwrap_or(ohms)
}

/// The largest E12 value no larger than this: for a base resistor, where less
/// is more drive and so the safe way to round.
fn e12_at_most(ohms: f64) -> f64 {
    crate::series::at_most(ohms, crate::series::Series::E12).unwrap_or(ohms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::Circuit;

    /// What the linter says about a circuit after a frame.
    fn said(circuit: &mut Circuit) -> Vec<crate::lint::Finding> {
        circuit.advance(0.016).unwrap();
        circuit.solution().findings
    }

    /// A 9 V battery and an LED, joined by whatever `between` puts between
    /// the battery's + and the LED's anode.
    fn led_from(between: &[&str]) -> (Circuit, String) {
        let mut circuit = Circuit::new();
        let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
        let led = circuit.try_add("Led", 200.0, 0.0).unwrap();
        let mut from = (battery.clone(), "+");
        for (index, kind) in between.iter().enumerate() {
            let part = circuit
                .try_add(kind, 40.0 * (index as f64 + 1.0), 0.0)
                .unwrap();
            let pins = crate::registry::definition(kind).unwrap().pins;
            circuit
                .try_connect(&from.0, from.1, &part, pins[0])
                .unwrap();
            from = (part, pins[1]);
        }
        circuit.try_connect(&from.0, from.1, &led, "anode").unwrap();
        circuit.try_connect(&led, "cathode", &battery, "-").unwrap();
        (circuit, led)
    }

    #[test]
    fn an_led_straight_across_a_battery_is_told_what_resistor_it_needs() {
        let (mut circuit, led) = led_from(&[]);
        let findings = said(&mut circuit);
        let finding = findings
            .iter()
            .find(|finding| finding.rule == "led-unlimited")
            .unwrap();
        assert_eq!(finding.elements, vec![led.clone()]);
        // (9 V − a red LED's 1.8 V) at 16 mA is 450 Ω, so 470 Ω.
        assert!(finding.message.contains("red LED"), "{}", finding.message);
        assert!(finding.message.contains("470 Ω"), "{}", finding.message);
        assert_eq!(
            finding.fix,
            Some(Fix::AddResistor {
                element: led,
                value: "470".into()
            })
        );
    }

    #[test]
    fn its_fix_puts_the_resistor_in_and_one_undo_takes_it_out() {
        // C-135: the button does what the sentence says, as one step.
        let (mut circuit, led) = led_from(&[]);
        let fix = said(&mut circuit)
            .into_iter()
            .find(|finding| finding.rule == "led-unlimited")
            .and_then(|finding| finding.fix)
            .unwrap();
        let before = circuit.size();
        circuit.try_apply_fix(&fix).unwrap();
        assert_eq!(circuit.size(), before + 1, "one resistor added");

        let findings = said(&mut circuit);
        assert!(findings.is_empty(), "{findings:?}");
        // And it lights, at about what the rule aimed for.
        let state = circuit.solution();
        let amps = state
            .element_currents
            .iter()
            .find(|current| current.element_id == led)
            .map(|current| current.amps.abs())
            .unwrap();
        assert!((0.010..0.017).contains(&amps), "{amps} A");

        assert!(circuit.undo());
        assert_eq!(circuit.size(), before, "one undo takes it all back");
        assert!(
            said(&mut circuit)
                .iter()
                .any(|finding| finding.rule == "led-unlimited")
        );
    }

    #[test]
    fn a_digit_gets_a_resistor_on_each_wired_segment() {
        let mut circuit = Circuit::new();
        let source = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
        let digit = circuit.try_add("SevenSegment", 120.0, 0.0).unwrap();
        let pins = crate::registry::definition("SevenSegment").unwrap().pins;
        circuit.try_connect(&source, "+", &digit, pins[0]).unwrap();
        circuit.try_connect(&source, "+", &digit, pins[1]).unwrap();
        circuit
            .try_connect(&digit, pins.last().unwrap(), &source, "-")
            .unwrap();
        let fix = said(&mut circuit)
            .into_iter()
            .find_map(|finding| finding.fix)
            .unwrap();
        circuit.try_apply_fix(&fix).unwrap();
        // Two segments wired, so two resistors: the free ones are left free.
        assert_eq!(circuit.size(), 4);
        assert!(
            !said(&mut circuit)
                .iter()
                .any(|finding| finding.rule == "led-unlimited")
        );
    }

    #[test]
    fn a_switch_or_a_fuse_in_the_way_limits_nothing() {
        for between in [&["Switch"][..], &["PushButton"], &["Fuse"], &["Ammeter"]] {
            let (mut circuit, _) = led_from(between);
            assert!(
                said(&mut circuit)
                    .iter()
                    .any(|finding| finding.rule == "led-unlimited"),
                "{between:?}"
            );
        }
    }

    #[test]
    fn a_resistor_anywhere_in_the_loop_is_the_limit() {
        let (mut circuit, _) = led_from(&["Resistor"]);
        assert!(
            !said(&mut circuit)
                .iter()
                .any(|finding| finding.rule == "led-unlimited")
        );
        let (mut circuit, _) = led_from(&["Switch", "Resistor"]);
        assert!(
            !said(&mut circuit)
                .iter()
                .any(|finding| finding.rule == "led-unlimited")
        );
    }

    #[test]
    fn two_leds_in_a_row_need_less_resistor_than_one() {
        let (mut circuit, _) = led_from(&["Led"]);
        let findings = said(&mut circuit);
        let messages: Vec<_> = findings
            .iter()
            .filter(|finding| finding.rule == "led-unlimited")
            .map(|finding| finding.message.clone())
            .collect();
        // Both are told, and each is told what the string needs: 9 − 1.8 −
        // 1.8 over 16 mA is 338 Ω, so 390 Ω.
        assert_eq!(messages.len(), 2, "{messages:?}");
        assert!(
            messages.iter().all(|message| message.contains("390 Ω")),
            "{messages:?}"
        );
    }

    #[test]
    fn a_battery_with_resistance_of_its_own_is_the_limit() {
        // C-17: a source given 470 Ω of its own limits the LED as a resistor
        // would.
        let mut circuit = Circuit::new();
        let source = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
        let led = circuit.try_add("Led", 80.0, 0.0).unwrap();
        circuit.try_set_series_resistance(&source, "470").unwrap();
        circuit.try_connect(&source, "+", &led, "anode").unwrap();
        circuit.try_connect(&led, "cathode", &source, "-").unwrap();
        assert!(
            !said(&mut circuit)
                .iter()
                .any(|finding| finding.rule == "led-unlimited")
        );
    }

    #[test]
    fn through_two_ground_symbols_is_still_one_loop() {
        // ADR 0003: two ground symbols are one node, so an LED whose cathode
        // goes to one and the battery's − to the other is across the battery.
        let mut circuit = Circuit::new();
        let source = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
        let led = circuit.try_add("Led", 80.0, 0.0).unwrap();
        let here = circuit.try_add("Ground", 80.0, 80.0).unwrap();
        let there = circuit.try_add("Ground", 0.0, 80.0).unwrap();
        circuit.try_connect(&source, "+", &led, "anode").unwrap();
        circuit.try_connect(&led, "cathode", &here, "gnd").unwrap();
        circuit.try_connect(&source, "-", &there, "gnd").unwrap();
        assert!(
            said(&mut circuit)
                .iter()
                .any(|finding| finding.rule == "led-unlimited")
        );
    }

    #[test]
    fn a_digit_straight_on_a_battery_is_told_once() {
        let mut circuit = Circuit::new();
        let source = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
        let digit = circuit.try_add("SevenSegment", 120.0, 0.0).unwrap();
        let pins = crate::registry::definition("SevenSegment").unwrap().pins;
        // Common cathode by default: segment a to +, the common pin to −.
        let common = *pins.last().unwrap();
        circuit.try_connect(&source, "+", &digit, pins[0]).unwrap();
        circuit.try_connect(&digit, common, &source, "-").unwrap();
        let findings: Vec<_> = said(&mut circuit)
            .into_iter()
            .filter(|finding| finding.rule == "led-unlimited")
            .collect();
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].elements, vec![digit]);
        assert!(
            findings[0].message.contains("each segment"),
            "{}",
            findings[0].message
        );
    }

    #[test]
    fn a_milestone_circuit_is_left_alone() {
        let (mut circuit, _) = led_from(&["Resistor"]);
        assert!(said(&mut circuit).is_empty());
    }

    /// The handle of the first part on the canvas that `is` picks out.
    fn handle_of(circuit: &Circuit, is: impl Fn(&ElementKind) -> bool) -> String {
        circuit
            .netlist()
            .elements()
            .find(|(_, element)| is(&element.kind))
            .map(|(id, _)| encode(id))
            .unwrap()
    }

    fn over_rated_in(circuit: &mut Circuit) -> Vec<crate::lint::Finding> {
        said(circuit)
            .into_iter()
            .filter(|finding| finding.rule == "over-rated")
            .collect()
    }

    #[test]
    fn a_2n7000_on_a_motor_is_told_what_it_carries_and_an_irlz44n_is_not() {
        // The ticket's case, the shipped example "Too much for a 2N7000":
        // two motors on 6 V, each switched by a MOSFET.
        use crate::solver::device::Channel;
        let mut circuit = Circuit::new();
        circuit
            .try_open(include_str!("../../../../app/src/examples/toohot.json"))
            .unwrap();
        assert!(over_rated_in(&mut circuit).is_empty(), "off, nothing flows");

        let switch = handle_of(&circuit, |kind| matches!(kind, ElementKind::Switch { .. }));
        circuit.try_set_closed(&switch, true).unwrap();
        let findings = over_rated_in(&mut circuit);
        let small = handle_of(&circuit, |kind| {
            *kind
                == ElementKind::Mosfet {
                    channel: Channel::N,
                }
        });
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].elements, vec![small]);
        assert_eq!(findings[0].severity, "warning");
        assert_eq!(
            findings[0].message,
            "This 2N7000 carries 270 mA; it is rated for 200 mA and will burn out."
        );
        assert_eq!(findings[0].fix, None);
    }

    /// A 9 V battery, a resistor of `ohms` and a red LED.
    fn led_behind(ohms: &str) -> (Circuit, String, String) {
        let (mut circuit, led) = led_from(&["Resistor"]);
        let resistor = handle_of(&circuit, |kind| {
            matches!(kind, ElementKind::Resistor { .. })
        });
        circuit.try_set_value(&resistor, ohms).unwrap();
        (circuit, led, resistor)
    }

    #[test]
    fn an_led_behind_too_small_a_resistor_is_told_its_current_and_the_resistor_it_needs() {
        // 9 V, 220 Ω — the old default — and a red LED: 33 mA in a 20 mA
        // part. (At 100 Ω the resistor is past its quarter watt too, and is
        // told so beside it.)
        let (mut circuit, led, resistor) = led_behind("220");
        let findings = over_rated_in(&mut circuit);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].elements, vec![led]);
        assert!(
            findings[0]
                .message
                .starts_with("This red LED carries 33 mA;"),
            "{}",
            findings[0].message
        );
        assert!(
            findings[0]
                .message
                .ends_with("it is rated for 20 mA and will burn out."),
            "{}",
            findings[0].message
        );
        // The resistor it needs is the one B-02 would have asked for.
        assert_eq!(
            findings[0].fix,
            Some(Fix::SetValue {
                element: resistor,
                value: "470".into()
            })
        );
    }

    #[test]
    fn its_fix_sets_the_resistor_and_one_undo_takes_it_back() {
        let (mut circuit, _, _) = led_behind("220");
        let fix = over_rated_in(&mut circuit)
            .into_iter()
            .find_map(|finding| finding.fix)
            .unwrap();
        circuit.try_apply_fix(&fix).unwrap();
        let findings = said(&mut circuit);
        assert!(findings.is_empty(), "{findings:?}");
        assert!(circuit.undo());
        assert_eq!(over_rated_in(&mut circuit).len(), 1);
    }

    #[test]
    fn an_led_with_more_than_its_resistor_in_the_way_is_told_but_offered_nothing() {
        // A switch between the resistor and the LED: the resistor is not on
        // a wire of the LED's alone, so which one to change is not certain.
        let (mut circuit, led) = led_from(&["Resistor", "Switch"]);
        let resistor = handle_of(&circuit, |kind| {
            matches!(kind, ElementKind::Resistor { .. })
        });
        let switch = handle_of(&circuit, |kind| matches!(kind, ElementKind::Switch { .. }));
        circuit.try_set_value(&resistor, "220").unwrap();
        circuit.try_set_closed(&switch, true).unwrap();
        let findings = over_rated_in(&mut circuit);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].elements, vec![led]);
        assert_eq!(findings[0].fix, None);
    }

    #[test]
    fn an_led_inside_its_rating_says_nothing() {
        // 390 Ω is some 18 mA; 470 Ω, the default, 15 mA.
        for ohms in ["390", "470"] {
            let (mut circuit, _, _) = led_behind(ohms);
            assert!(over_rated_in(&mut circuit).is_empty(), "{ohms}");
        }
    }

    #[test]
    fn a_resistor_past_a_quarter_watt_is_told_its_power() {
        // The shipped "A resistor running hot": 9 V across 150 Ω, 0.54 W.
        let mut circuit = Circuit::new();
        let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
        let resistor = circuit.try_add("Resistor", 80.0, 0.0).unwrap();
        circuit.try_set_value(&resistor, "150").unwrap();
        circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
        circuit.try_connect(&resistor, "b", &battery, "-").unwrap();
        let findings = over_rated_in(&mut circuit);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].elements, vec![resistor.clone()]);
        assert_eq!(
            findings[0].message,
            "This resistor turns 540 mW into heat; it is rated for 250 mW and will burn out."
        );
        // At 470 Ω it is 0.17 W, and nothing.
        circuit.try_set_value(&resistor, "470").unwrap();
        assert!(over_rated_in(&mut circuit).is_empty());
    }

    #[test]
    fn a_part_with_no_rating_is_not_judged() {
        // The registry has none for a 2N3904 (C-16): a beginner's transistor
        // straight on a battery is another rule's, not this one's.
        let rating = crate::registry::rating_of(&ElementKind::Bjt {
            polarity: crate::solver::device::Polarity::Npn,
        });
        assert_eq!((rating.current, rating.power), (None, None));
        let mut circuit = Circuit::new();
        let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
        let bjt = circuit.try_add("Bjt", 80.0, 0.0).unwrap();
        let resistor = circuit.try_add("Resistor", 40.0, 0.0).unwrap();
        circuit.try_set_value(&resistor, "10").unwrap();
        circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
        circuit.try_connect(&resistor, "b", &bjt, "c").unwrap();
        circuit.try_connect(&battery, "+", &bjt, "b").unwrap();
        circuit.try_connect(&bjt, "e", &battery, "-").unwrap();
        let findings = over_rated_in(&mut circuit);
        assert!(
            findings
                .iter()
                .all(|finding| finding.elements != vec![bjt.clone()]),
            "{findings:?}"
        );
    }

    fn base_unlimited_in(circuit: &mut Circuit) -> Vec<crate::lint::Finding> {
        said(circuit)
            .into_iter()
            .filter(|finding| finding.rule == "base-unlimited")
            .collect()
    }

    /// A 9 V battery and a transistor of `polarity` switching `load` — the
    /// kinds between its collector and the battery's +, in order — with its
    /// emitter on the −, and `base` between the + and its base.
    fn switching(polarity: &str, load: &[&str], base: &[&str]) -> (Circuit, String) {
        let mut circuit = Circuit::new();
        let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
        let bjt = circuit.try_add("Bjt", 200.0, 0.0).unwrap();
        circuit.try_set_choice(&bjt, polarity).unwrap();
        let chain = |circuit: &mut Circuit, kinds: &[&str], to: &str, row: f64| {
            let mut from = (battery.clone(), "+");
            for (index, kind) in kinds.iter().enumerate() {
                let part = circuit
                    .try_add(kind, 40.0 * (index as f64 + 1.0), row)
                    .unwrap();
                let pins = crate::registry::definition(kind).unwrap().pins;
                circuit
                    .try_connect(&from.0, from.1, &part, pins[0])
                    .unwrap();
                from = (part, pins[1]);
            }
            circuit.try_connect(&from.0, from.1, &bjt, to).unwrap();
        };
        chain(&mut circuit, load, "c", -80.0);
        chain(&mut circuit, base, "b", 80.0);
        circuit.try_connect(&bjt, "e", &battery, "-").unwrap();
        (circuit, bjt)
    }

    #[test]
    fn a_base_straight_on_the_battery_is_told_the_resistor_it_needs() {
        // An LED behind 470 Ω on the collector: some 15 mA, so 0.3 mA into
        // the base at twice what a β of 100 needs; 8.3 V over that is 27.9 kΩ,
        // rounded down to 27 kΩ.
        let (mut circuit, bjt) = switching("Npn", &["Resistor", "Led"], &[]);
        let findings = base_unlimited_in(&mut circuit);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].elements, vec![bjt.clone()]);
        assert_eq!(
            findings[0].message,
            "This 2N3904's base is wired straight to the battery — put a 27 kΩ resistor in series with it."
        );
        assert_eq!(
            findings[0].fix,
            Some(Fix::AddResistor {
                element: bjt,
                value: "27k".into()
            })
        );
    }

    #[test]
    fn its_fix_puts_the_resistor_on_the_base_and_the_led_lights() {
        let (mut circuit, _) = switching("Npn", &["Resistor", "Led"], &[]);
        let led = handle_of(&circuit, |kind| matches!(kind, ElementKind::Led { .. }));
        let fix = base_unlimited_in(&mut circuit)
            .into_iter()
            .find_map(|finding| finding.fix)
            .unwrap();
        let before = circuit.size();
        circuit.try_apply_fix(&fix).unwrap();
        assert_eq!(circuit.size(), before + 1);
        let findings = said(&mut circuit);
        assert!(findings.is_empty(), "{findings:?}");
        let amps = circuit
            .solution()
            .element_currents
            .iter()
            .find(|current| current.element_id == led)
            .map(|current| current.amps)
            .unwrap();
        // Saturated: as much as the load lets through.
        assert!((0.013..0.017).contains(&amps), "{amps} A");
        assert!(circuit.undo());
        assert_eq!(base_unlimited_in(&mut circuit).len(), 1);
    }

    #[test]
    fn a_darlington_needs_about_ten_times_the_resistor() {
        // The same motor: a 2N3904 wants 680 Ω, a TIP120 6.8 kΩ — twice the
        // volts across its base, a tenth of the current into it.
        let resistor = |polarity: &str| {
            let (mut circuit, _) = switching(polarity, &["Motor"], &[]);
            match base_unlimited_in(&mut circuit).remove(0).fix {
                Some(Fix::AddResistor { value, .. }) => value,
                other => panic!("{other:?}"),
            }
        };
        assert_eq!(resistor("Npn"), "680");
        assert_eq!(resistor("Darlington"), "6.8k");
    }

    #[test]
    fn a_darlington_on_a_motor_through_10k_is_left_alone() {
        // The shipped "A Darlington on a motor", held down.
        let mut circuit = Circuit::new();
        circuit
            .try_open(include_str!("../../../../app/src/examples/darlington.json"))
            .unwrap();
        let button = handle_of(&circuit, |kind| {
            matches!(kind, ElementKind::PushButton { .. })
        });
        circuit.try_set_closed(&button, true).unwrap();
        let findings = said(&mut circuit);
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn a_switch_on_the_base_limits_nothing() {
        let (mut circuit, _) = switching("Npn", &["Resistor", "Led"], &["Switch"]);
        assert_eq!(base_unlimited_in(&mut circuit).len(), 1);
        let (mut circuit, _) = switching("Npn", &["Resistor", "Led"], &["Resistor"]);
        assert!(base_unlimited_in(&mut circuit).is_empty());
    }

    #[test]
    fn an_emitter_follower_is_left_alone() {
        // Base on the +, the load under the emitter: the load is the limit.
        let mut circuit = Circuit::new();
        let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
        let bjt = circuit.try_add("Bjt", 80.0, 0.0).unwrap();
        let resistor = circuit.try_add("Resistor", 160.0, 0.0).unwrap();
        circuit.try_connect(&battery, "+", &bjt, "b").unwrap();
        circuit.try_connect(&battery, "+", &bjt, "c").unwrap();
        circuit.try_connect(&bjt, "e", &resistor, "a").unwrap();
        circuit.try_connect(&resistor, "b", &battery, "-").unwrap();
        assert!(base_unlimited_in(&mut circuit).is_empty());
    }

    #[test]
    fn a_pnp_with_its_base_on_the_minus_is_told() {
        let mut circuit = Circuit::new();
        let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
        let bjt = circuit.try_add("Bjt", 80.0, 0.0).unwrap();
        circuit.try_set_choice(&bjt, "Pnp").unwrap();
        let resistor = circuit.try_add("Resistor", 160.0, 0.0).unwrap();
        circuit.try_connect(&battery, "+", &bjt, "e").unwrap();
        circuit.try_connect(&bjt, "b", &battery, "-").unwrap();
        circuit.try_connect(&bjt, "c", &resistor, "a").unwrap();
        circuit.try_connect(&resistor, "b", &battery, "-").unwrap();
        let findings = base_unlimited_in(&mut circuit);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(
            findings[0].message.starts_with("This 2N3906's base"),
            "{}",
            findings[0].message
        );
    }

    #[test]
    fn a_base_straight_on_a_gate_output_names_the_chip() {
        // A 74HC00 on 5 V, its first gate's output on the base.
        let mut circuit = Circuit::new();
        let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
        circuit.try_set_value(&battery, "5").unwrap();
        let gate = circuit.try_add("Gate", 80.0, 0.0).unwrap();
        let bjt = circuit.try_add("Bjt", 200.0, 0.0).unwrap();
        let resistor = circuit.try_add("Resistor", 200.0, -80.0).unwrap();
        circuit.try_connect(&battery, "+", &gate, "VCC").unwrap();
        circuit.try_connect(&gate, "GND", &battery, "-").unwrap();
        circuit.try_connect(&gate, "1Y", &bjt, "b").unwrap();
        circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
        circuit.try_connect(&resistor, "b", &bjt, "c").unwrap();
        circuit.try_connect(&bjt, "e", &battery, "-").unwrap();
        let findings = base_unlimited_in(&mut circuit);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(
            findings[0]
                .message
                .starts_with("This 2N3904's base is wired straight to the 74HC00's output — put a"),
            "{}",
            findings[0].message
        );
    }

    #[test]
    fn a_collector_it_cannot_read_is_told_without_a_value() {
        // Nothing on the collector but the battery's +.
        let (mut circuit, _) = switching("Npn", &[], &[]);
        let findings = base_unlimited_in(&mut circuit);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(
            findings[0].message,
            "This 2N3904's base is wired straight to the battery — it needs a resistor in series."
        );
        assert_eq!(findings[0].fix, None);
    }

    fn by_rule(circuit: &mut Circuit, rule: &str) -> Vec<crate::lint::Finding> {
        said(circuit)
            .into_iter()
            .filter(|finding| finding.rule == rule)
            .collect()
    }

    #[test]
    fn a_wire_from_plus_to_minus_shorts_the_battery() {
        let mut circuit = Circuit::new();
        let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
        circuit.try_connect(&battery, "+", &battery, "-").unwrap();
        let findings = by_rule(&mut circuit, "source-shorted");
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].elements, vec![battery]);
        assert_eq!(
            findings[0].message,
            "This battery's + is wired straight to its − — put a part between them, or take that wire out."
        );
    }

    #[test]
    fn a_shut_switch_shorts_it_and_an_open_one_does_not() {
        // A button across the battery: nothing until it is pressed.
        let mut circuit = Circuit::new();
        let battery = circuit.try_add("Cell", 0.0, 0.0).unwrap();
        let button = circuit.try_add("PushButton", 80.0, 0.0).unwrap();
        let meter = circuit.try_add("Ammeter", 160.0, 0.0).unwrap();
        circuit.try_connect(&battery, "+", &button, "a").unwrap();
        circuit.try_connect(&button, "b", &meter, "+").unwrap();
        circuit.try_connect(&meter, "-", &battery, "-").unwrap();
        assert!(by_rule(&mut circuit, "source-shorted").is_empty());
        circuit.try_set_closed(&button, true).unwrap();
        assert_eq!(by_rule(&mut circuit, "source-shorted").len(), 1);
    }

    #[test]
    fn through_two_ground_symbols_it_is_still_shorted() {
        let mut circuit = Circuit::new();
        let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
        let here = circuit.try_add("Ground", 0.0, 80.0).unwrap();
        let there = circuit.try_add("Ground", 80.0, 80.0).unwrap();
        circuit.try_connect(&battery, "-", &here, "gnd").unwrap();
        circuit.try_connect(&battery, "+", &there, "gnd").unwrap();
        assert_eq!(by_rule(&mut circuit, "source-shorted").len(), 1);
    }

    #[test]
    fn a_fuse_or_a_resistor_across_it_is_no_short() {
        for between in ["Fuse", "Resistor"] {
            let mut circuit = Circuit::new();
            let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
            let part = circuit.try_add(between, 80.0, 0.0).unwrap();
            let pins = crate::registry::definition(between).unwrap().pins;
            circuit.try_connect(&battery, "+", &part, pins[0]).unwrap();
            circuit.try_connect(&part, pins[1], &battery, "-").unwrap();
            assert!(
                by_rule(&mut circuit, "source-shorted").is_empty(),
                "{between}"
            );
        }
    }

    /// A CD4017 on 9 V with everything tied but `free`.
    fn decade_with_free(free: &[&str]) -> Circuit {
        use crate::solver::device::decade;
        let mut circuit = Circuit::new();
        let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
        let chip = circuit.try_add("Decade", 120.0, 0.0).unwrap();
        let pin = |index: usize| decade::PINS[index];
        circuit
            .try_connect(&battery, "+", &chip, pin(decade::VDD))
            .unwrap();
        circuit
            .try_connect(&chip, pin(decade::VSS), &battery, "-")
            .unwrap();
        for &input in &decade::INPUTS {
            if !free.contains(&pin(input)) {
                circuit
                    .try_connect(&chip, pin(input), &battery, "-")
                    .unwrap();
            }
        }
        circuit
    }

    #[test]
    fn a_counters_free_reset_is_named() {
        use crate::solver::device::decade;
        let reset = decade::PINS[decade::RESET];
        let findings = by_rule(&mut decade_with_free(&[reset]), "chip-input-free");
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(
            findings[0].message,
            format!(
                "This CD4017's input {reset} is not wired — a free input reads whatever is near it; tie it to + or −."
            )
        );
        assert!(by_rule(&mut decade_with_free(&[]), "chip-input-free").is_empty());
    }

    #[test]
    fn a_chip_with_no_power_is_told_that_and_not_its_inputs() {
        // VDD left free, RST and the rest free too: one line, the power.
        use crate::solver::device::decade;
        let mut circuit = Circuit::new();
        let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
        let chip = circuit.try_add("Decade", 120.0, 0.0).unwrap();
        let resistor = circuit.try_add("Resistor", 60.0, 80.0).unwrap();
        circuit
            .try_connect(&chip, decade::PINS[decade::VSS], &battery, "-")
            .unwrap();
        circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
        circuit.try_connect(&resistor, "b", &battery, "-").unwrap();
        let findings = said(&mut circuit);
        let rules: Vec<_> = findings
            .iter()
            .map(|finding| finding.rule.as_str())
            .collect();
        assert_eq!(rules, ["unpowered"], "{findings:?}");
    }

    #[test]
    fn a_pot_across_the_battery_with_its_wiper_free_is_told() {
        let mut circuit = Circuit::new();
        let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
        let pot = circuit.try_add("Potentiometer", 80.0, 0.0).unwrap();
        circuit.try_connect(&battery, "+", &pot, "a").unwrap();
        circuit.try_connect(&pot, "b", &battery, "-").unwrap();
        let findings = by_rule(&mut circuit, "pin-free");
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].severity, "note");
        assert!(
            findings[0].message.contains("wiper"),
            "{}",
            findings[0].message
        );
        // Its wiper to a meter, and it is a divider.
        let meter = circuit.try_add("Voltmeter", 160.0, 0.0).unwrap();
        circuit.try_connect(&pot, "wiper", &meter, "+").unwrap();
        circuit.try_connect(&meter, "-", &battery, "-").unwrap();
        assert!(by_rule(&mut circuit, "pin-free").is_empty());
    }

    #[test]
    fn a_transistor_switched_on_into_nothing_is_told() {
        // Base through 10 kΩ, emitter to −, collector free.
        let mut circuit = Circuit::new();
        let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
        let resistor = circuit.try_add("Resistor", 40.0, 0.0).unwrap();
        let bjt = circuit.try_add("Bjt", 120.0, 0.0).unwrap();
        circuit.try_set_value(&resistor, "10k").unwrap();
        circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
        circuit.try_connect(&resistor, "b", &bjt, "b").unwrap();
        circuit.try_connect(&bjt, "e", &battery, "-").unwrap();
        let findings = by_rule(&mut circuit, "pin-free");
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(
            findings[0].message,
            "This 2N3904 is switched on with nothing on its collector — wire the load to c."
        );
    }

    #[test]
    fn a_part_being_wired_is_not_told_it_is_unfinished() {
        // A pot with one end wired, a transistor with only its emitter: half
        // way through building, and nothing to say.
        let mut circuit = Circuit::new();
        let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
        let pot = circuit.try_add("Potentiometer", 80.0, 0.0).unwrap();
        let bjt = circuit.try_add("Bjt", 160.0, 0.0).unwrap();
        let led = circuit.try_add("Led", 240.0, 0.0).unwrap();
        let resistor = circuit.try_add("Resistor", 240.0, 80.0).unwrap();
        circuit.try_connect(&battery, "+", &pot, "a").unwrap();
        circuit.try_connect(&bjt, "e", &battery, "-").unwrap();
        circuit.try_connect(&battery, "+", &resistor, "a").unwrap();
        circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
        circuit.try_connect(&led, "cathode", &battery, "-").unwrap();
        assert!(by_rule(&mut circuit, "pin-free").is_empty());
    }

    #[test]
    fn a_7805_dropping_7_v_into_a_motor_is_told_it_needs_a_heatsink() {
        // The shipped "A regulator too hot": 12 V in, 5 V out, a third of an
        // amp — some 2.3 W, and 50 K/W on 25 °C sheds 2 W at most.
        let mut circuit = Circuit::new();
        circuit
            .try_open(include_str!("../../../../app/src/examples/heatsink.json"))
            .unwrap();
        let regulator = handle_of(&circuit, |kind| {
            matches!(kind, ElementKind::Regulator { .. })
        });
        let findings = by_rule(&mut circuit, "regulator-hot");
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].elements, vec![regulator]);
        let message = &findings[0].message;
        assert!(
            message.starts_with("This 7805 drops 7.0 V and turns 2."),
            "{message}"
        );
        assert!(
            message.ends_with(
                "W into heat, past the 2 W it can shed without a heatsink — give it one, or use a switching regulator."
            ),
            "{message}"
        );
    }

    /// A 7805 on 12 V into a resistor of `ohms`, in a room at `room` °C.
    fn regulating_into(ohms: &str, room: &str) -> Circuit {
        let mut circuit = Circuit::new();
        let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
        circuit.try_set_value(&battery, "12").unwrap();
        let regulator = circuit.try_add("Regulator", 80.0, 0.0).unwrap();
        let load = circuit.try_add("Resistor", 160.0, 0.0).unwrap();
        circuit.try_set_value(&load, ohms).unwrap();
        circuit.try_set_room(room).unwrap();
        circuit
            .try_connect(&battery, "+", &regulator, "in")
            .unwrap();
        circuit
            .try_connect(&regulator, "gnd", &battery, "-")
            .unwrap();
        circuit.try_connect(&regulator, "out", &load, "a").unwrap();
        circuit.try_connect(&load, "b", &battery, "-").unwrap();
        circuit
    }

    #[test]
    fn a_light_load_is_left_alone() {
        // 10 mA: 70 mW, nothing to say.
        assert!(by_rule(&mut regulating_into("470", "25"), "regulator-hot").is_empty());
    }

    #[test]
    fn a_warm_room_leaves_less_to_shed() {
        // 150 mA through 33 Ω: about a watt, comfortable at 25 °C; at 75 °C
        // it may shed only (125 − 75) / 50 = 1 W, and is past it.
        assert!(by_rule(&mut regulating_into("33", "25"), "regulator-hot").is_empty());
        let findings = by_rule(&mut regulating_into("33", "75"), "regulator-hot");
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(
            findings[0].message.contains("past the 1 W"),
            "{}",
            findings[0].message
        );
    }

    /// An LM358 or LM393 on a 9 V battery, its inputs on `plus` and `minus`
    /// — the battery's "+" or "-", or a fresh voltmeter, which reads and
    /// passes nothing — and its output on a voltmeter.
    fn amplifier(kind: &str) -> (Circuit, String, String) {
        let mut circuit = Circuit::new();
        let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
        let part = circuit.try_add(kind, 120.0, 0.0).unwrap();
        circuit.try_connect(&battery, "+", &part, "V+").unwrap();
        circuit.try_connect(&part, "V-", &battery, "-").unwrap();
        (circuit, battery, part)
    }

    /// A divider of two resistors across the battery, its middle on `pin`.
    fn divide_into(
        circuit: &mut Circuit,
        battery: &str,
        part: &str,
        pin: &str,
        top: &str,
        bottom: &str,
    ) {
        let upper = circuit.try_add("Resistor", 60.0, 80.0).unwrap();
        let lower = circuit.try_add("Resistor", 60.0, 160.0).unwrap();
        circuit.try_set_value(&upper, top).unwrap();
        circuit.try_set_value(&lower, bottom).unwrap();
        circuit.try_connect(battery, "+", &upper, "a").unwrap();
        circuit.try_connect(&upper, "b", &lower, "a").unwrap();
        circuit.try_connect(&lower, "b", battery, "-").unwrap();
        circuit.try_connect(&upper, "b", part, pin).unwrap();
    }

    #[test]
    fn a_comparator_with_nothing_to_pull_its_output_up_is_told() {
        let (mut circuit, battery, part) = amplifier("Comparator");
        divide_into(&mut circuit, &battery, &part, "in+", "10k", "10k");
        divide_into(&mut circuit, &battery, &part, "in-", "22k", "10k");
        let meter = circuit.try_add("Voltmeter", 200.0, 0.0).unwrap();
        circuit.try_connect(&part, "out", &meter, "+").unwrap();
        circuit.try_connect(&meter, "-", &battery, "-").unwrap();
        let findings = by_rule(&mut circuit, "opamp-misused");
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(
            findings[0].message,
            "This LM393 can only pull its output down — give out a resistor up to +, or a real one floats when it should be high."
        );
        // A pull-up, and it says nothing.
        let pull = circuit.try_add("Resistor", 200.0, -80.0).unwrap();
        circuit.try_set_value(&pull, "10k").unwrap();
        circuit.try_connect(&battery, "+", &pull, "a").unwrap();
        circuit.try_connect(&pull, "b", &part, "out").unwrap();
        assert!(by_rule(&mut circuit, "opamp-misused").is_empty());
    }

    #[test]
    fn an_input_above_the_supply_is_told() {
        // The op-amp on 5 V, its in+ on a 9 V battery sharing the −.
        let (mut circuit, battery, part) = amplifier("OpAmp");
        circuit.try_set_value(&battery, "5").unwrap();
        let other = circuit.try_add("DcSource", 0.0, 160.0).unwrap();
        circuit.try_connect(&other, "-", &battery, "-").unwrap();
        circuit.try_connect(&other, "+", &part, "in+").unwrap();
        circuit.try_connect(&part, "in-", &part, "out").unwrap();
        let findings = by_rule(&mut circuit, "opamp-misused");
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(
            findings[0].message,
            "This LM358's in+ is at 9.0 V, above its V+ — a real one can be damaged by an input outside its supply; keep both between V− and V+."
        );
    }

    #[test]
    fn both_inputs_near_the_top_are_told_and_one_is_not() {
        // 10 kΩ over 100 kΩ puts an input at 8.2 V, within 1.5 V of 9 V.
        let (mut circuit, battery, part) = amplifier("OpAmp");
        divide_into(&mut circuit, &battery, &part, "in+", "10k", "100k");
        divide_into(&mut circuit, &battery, &part, "in-", "10k", "10k");
        assert!(
            by_rule(&mut circuit, "opamp-misused").is_empty(),
            "one is low"
        );
        let (mut circuit, battery, part) = amplifier("OpAmp");
        divide_into(&mut circuit, &battery, &part, "in+", "10k", "100k");
        divide_into(&mut circuit, &battery, &part, "in-", "4.7k", "100k");
        let findings = by_rule(&mut circuit, "opamp-misused");
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(
            findings[0]
                .message
                .starts_with("Both of this LM358's inputs")
        );
    }

    #[test]
    fn a_free_supply_pin_is_named() {
        let mut circuit = Circuit::new();
        let battery = circuit.try_add("DcSource", 0.0, 0.0).unwrap();
        let part = circuit.try_add("OpAmp", 120.0, 0.0).unwrap();
        circuit.try_connect(&battery, "+", &part, "V+").unwrap();
        divide_into(&mut circuit, &battery, &part, "in+", "10k", "10k");
        let messages: Vec<_> = said(&mut circuit)
            .into_iter()
            .map(|finding| finding.message)
            .collect();
        assert_eq!(
            messages,
            ["This op-amp's V− is not wired — wire it to the battery's −."]
        );
    }

    #[test]
    fn the_op_amp_examples_say_nothing_however_their_knobs_are_turned() {
        for text in [
            include_str!("../../../../app/src/examples/threshold.json"),
            include_str!("../../../../app/src/examples/hotfan.json"),
            include_str!("../../../../app/src/examples/twice.json"),
            include_str!("../../../../app/src/examples/buffer.json"),
        ] {
            let mut circuit = Circuit::new();
            circuit.try_open(text).unwrap();
            let pots: Vec<String> = circuit
                .netlist()
                .elements()
                .filter(|(_, element)| matches!(element.kind, ElementKind::Potentiometer { .. }))
                .map(|(id, _)| encode(id))
                .collect();
            for step in 0..=10 {
                for pot in &pots {
                    circuit.try_set_wiper(pot, step as f64 / 10.0).unwrap();
                }
                let findings = by_rule(&mut circuit, "opamp-misused");
                assert!(findings.is_empty(), "at {step}: {findings:?}");
            }
        }
    }

    #[test]
    fn rounds_a_base_resistor_down() {
        assert_eq!(e12_at_most(27_900.0), 27_000.0);
        assert_eq!(e12_at_most(470.0), 470.0);
        assert_eq!(e12_at_most(760.0), 680.0);
        assert!((e12_at_most(6_950.0) - 6_800.0).abs() < 1e-6);
    }

    #[test]
    fn a_reading_is_said_to_two_figures() {
        assert_eq!(two_figures(0.2956), 0.3);
        assert!((two_figures(0.06234) - 0.062).abs() < 1e-12);
        assert_eq!(two_figures(0.0), 0.0);
    }

    #[test]
    fn rounds_up_to_a_resistor_that_can_be_bought() {
        assert_eq!(e12_at_least(466.7), 470.0);
        assert_eq!(e12_at_least(470.0), 470.0);
        assert_eq!(e12_at_least(471.0), 560.0);
        assert_eq!(e12_at_least(85.0), 100.0);
        assert!((e12_at_least(1_150.0) - 1_200.0).abs() < 1e-6);
    }

    /// An Uno running `program` from `programs/`, or none.
    fn uno_running(program: Option<&str>) -> (Circuit, String) {
        let mut circuit = Circuit::new();
        let uno = circuit.try_add("Uno", 0.0, 0.0).unwrap();
        let hex = match program {
            Some("Blink") => include_str!("../../../../programs/Blink.hex"),
            Some("Button") => include_str!("../../../../programs/Button.hex"),
            Some(other) => panic!("no program {other} here"),
            None => return (circuit, uno),
        };
        circuit.try_set_program(&uno, "program.hex", hex).unwrap();
        (circuit, uno)
    }

    /// What the Uno rules say, after a frame.
    fn uno_said(circuit: &mut Circuit) -> Vec<crate::lint::Finding> {
        said(circuit)
            .into_iter()
            .filter(|finding| finding.rule.starts_with("uno-"))
            .collect()
    }

    #[test]
    fn an_led_straight_on_a_pin_is_told_the_resistor_it_needs() {
        // With no program at all: it is the wiring that is wrong.
        let (mut circuit, uno) = uno_running(None);
        let led = circuit.try_add("Led", 100.0, 0.0).unwrap();
        circuit.try_connect(&uno, "D13", &led, "anode").unwrap();
        circuit.try_connect(&led, "cathode", &uno, "GND").unwrap();
        let findings = by_rule(&mut circuit, "led-unlimited");
        assert_eq!(findings.len(), 1, "{findings:?}");
        // (5 V − a red LED's 1.8 V) at 16 mA is 200 Ω, so 220 Ω.
        assert_eq!(
            findings[0].message,
            "This red LED has nothing to limit its current — put a 220 Ω resistor in series with it."
        );
        assert_eq!(
            findings[0].fix,
            Some(Fix::AddResistor {
                element: led,
                value: "220".into()
            })
        );
    }

    #[test]
    fn an_led_behind_its_resistor_on_a_pin_is_left_alone() {
        let (mut circuit, uno) = uno_running(Some("Blink"));
        let resistor = circuit.try_add("Resistor", 100.0, 0.0).unwrap();
        let led = circuit.try_add("Led", 200.0, 0.0).unwrap();
        circuit.try_set_value(&resistor, "220").unwrap();
        circuit.try_connect(&uno, "D13", &resistor, "a").unwrap();
        circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
        circuit.try_connect(&led, "cathode", &uno, "GND").unwrap();
        assert!(said(&mut circuit).is_empty());
    }

    #[test]
    fn a_base_straight_on_a_pin_names_the_uno() {
        let (mut circuit, uno) = uno_running(None);
        let bjt = circuit.try_add("Bjt", 100.0, 0.0).unwrap();
        circuit.try_connect(&uno, "D9", &bjt, "b").unwrap();
        circuit.try_connect(&bjt, "e", &uno, "GND").unwrap();
        let findings = by_rule(&mut circuit, "base-unlimited");
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(
            findings[0]
                .message
                .starts_with("This 2N3904's base is wired straight to the Uno"),
            "{}",
            findings[0].message
        );
    }

    #[test]
    fn a_pin_past_20_ma_is_told_its_current() {
        // D13 high into 150 Ω: 5 V over 176 Ω with the pin's own, 28 mA.
        let (mut circuit, uno) = uno_running(Some("Blink"));
        let resistor = circuit.try_add("Resistor", 100.0, 0.0).unwrap();
        circuit.try_set_value(&resistor, "150").unwrap();
        circuit.try_connect(&uno, "D13", &resistor, "a").unwrap();
        circuit.try_connect(&resistor, "b", &uno, "GND").unwrap();
        let findings = uno_said(&mut circuit);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].rule, "uno-overloaded");
        assert_eq!(findings[0].elements, vec![uno]);
        assert_eq!(
            findings[0].message,
            "This Uno's D13 carries 28 mA; a pin is rated for 20 mA, and 40 mA damages it — limit it with a resistor."
        );
    }

    #[test]
    fn a_pin_past_40_ma_is_told_it_is_damaged() {
        let (mut circuit, uno) = uno_running(Some("Blink"));
        let resistor = circuit.try_add("Resistor", 100.0, 0.0).unwrap();
        circuit.try_set_value(&resistor, "47").unwrap();
        circuit.try_connect(&uno, "D13", &resistor, "a").unwrap();
        circuit.try_connect(&resistor, "b", &uno, "GND").unwrap();
        let findings = by_rule(&mut circuit, "uno-overloaded");
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(
            findings[0]
                .message
                .contains("past the 40 mA that damages it"),
            "{}",
            findings[0].message
        );
    }

    #[test]
    fn a_motor_on_a_pin_is_told_once_whatever_it_draws() {
        let (mut circuit, uno) = uno_running(Some("Blink"));
        let motor = circuit.try_add("Motor", 100.0, 0.0).unwrap();
        circuit.try_connect(&uno, "D13", &motor, "a").unwrap();
        circuit.try_connect(&motor, "b", &uno, "GND").unwrap();
        let findings = uno_said(&mut circuit);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].rule, "uno-coil");
        assert_eq!(
            findings[0].message,
            "This Uno's D13 drives a motor straight — it wants more than a pin gives, and kicks back; switch it with a transistor, a diode across it."
        );
    }

    #[test]
    fn a_relay_coil_on_a_pin_is_told() {
        let (mut circuit, uno) = uno_running(None);
        let relay = circuit.try_add("Relay", 100.0, 0.0).unwrap();
        let pins = crate::registry::definition("Relay").unwrap().pins;
        use crate::solver::device::relay::{COIL_MINUS, COIL_PLUS};
        circuit
            .try_connect(&uno, "D7", &relay, pins[COIL_PLUS])
            .unwrap();
        circuit
            .try_connect(&relay, pins[COIL_MINUS], &uno, "GND")
            .unwrap();
        let findings = by_rule(&mut circuit, "uno-coil");
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(
            findings[0].message.contains("D7 drives a relay's coil"),
            "{}",
            findings[0].message
        );
    }

    #[test]
    fn a_battery_on_a_pin_is_told_it_is_past_5_5_v() {
        let (mut circuit, uno) = uno_running(None);
        let battery = circuit.try_add("DcSource", 100.0, 0.0).unwrap();
        circuit.try_connect(&battery, "+", &uno, "A0").unwrap();
        circuit.try_connect(&battery, "-", &uno, "GND").unwrap();
        let findings = by_rule(&mut circuit, "uno-overvolted");
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(
            findings[0].message,
            "This Uno's A0 is at 9.0 V, past the 5.5 V a pin can take — bring it under 5 V, with a divider."
        );
    }

    /// An Uno running `program`, a button from D2 to `rail`, and whatever
    /// `pull` puts on D2 besides: a resistor to the other rail, or nothing.
    fn button_to(program: &str, rail: &str, pull: Option<&str>) -> Circuit {
        let (mut circuit, uno) = uno_running(Some(program));
        let button = circuit.try_add("PushButton", 100.0, 0.0).unwrap();
        circuit.try_connect(&uno, "D2", &button, "a").unwrap();
        circuit.try_connect(&button, "b", &uno, rail).unwrap();
        if let Some(other) = pull {
            let resistor = circuit.try_add("Resistor", 100.0, 80.0).unwrap();
            circuit.try_set_value(&resistor, "10k").unwrap();
            circuit.try_connect(&uno, "D2", &resistor, "a").unwrap();
            circuit.try_connect(&resistor, "b", &uno, other).unwrap();
        }
        circuit
    }

    #[test]
    fn a_button_to_5v_with_no_pull_down_is_told_to_get_one() {
        // Blink leaves D2 as it starts: an input, and nothing more.
        let mut circuit = button_to("Blink", "5V", None);
        let findings = uno_said(&mut circuit);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(
            findings[0].message,
            "This Uno's D2 floats while its button is open — give D2 a 10 kΩ resistor to GND."
        );
        let mut circuit = button_to("Blink", "5V", Some("GND"));
        assert!(uno_said(&mut circuit).is_empty());
    }

    #[test]
    fn a_button_to_gnd_needs_input_pullup_and_with_it_says_nothing() {
        let mut circuit = button_to("Blink", "GND", None);
        let findings = uno_said(&mut circuit);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(
            findings[0].message,
            "This Uno's D2 floats while its button is open — set it to INPUT_PULLUP, or give it a 10 kΩ resistor to 5V."
        );
        // Button sets D2 to INPUT_PULLUP.
        let mut circuit = button_to("Button", "GND", None);
        assert!(uno_said(&mut circuit).is_empty());
    }

    #[test]
    fn an_unprogrammed_uno_s_inputs_are_not_judged() {
        // Nothing has set a pin yet, so nothing has been left floating.
        let (mut circuit, uno) = uno_running(None);
        let button = circuit.try_add("PushButton", 100.0, 0.0).unwrap();
        circuit.try_connect(&uno, "D2", &button, "a").unwrap();
        circuit.try_connect(&button, "b", &uno, "5V").unwrap();
        assert!(uno_said(&mut circuit).is_empty());
    }

    #[test]
    fn the_uno_examples_say_nothing_about_the_uno() {
        for (name, file) in [
            (
                "uno-blink",
                include_str!("../../../../app/src/examples/uno-blink.json"),
            ),
            (
                "uno-button",
                include_str!("../../../../app/src/examples/uno-button.json"),
            ),
            (
                "uno-fade",
                include_str!("../../../../app/src/examples/uno-fade.json"),
            ),
            (
                "uno-dimmer",
                include_str!("../../../../app/src/examples/uno-dimmer.json"),
            ),
            (
                "uno-traffic",
                include_str!("../../../../app/src/examples/uno-traffic.json"),
            ),
            (
                "uno-sweep",
                include_str!("../../../../app/src/examples/uno-sweep.json"),
            ),
            (
                "uno-two-pots",
                include_str!("../../../../app/src/examples/uno-two-pots.json"),
            ),
            (
                "uno-smoothed",
                include_str!("../../../../app/src/examples/uno-smoothed.json"),
            ),
            (
                "uno-analog-input",
                include_str!("../../../../app/src/examples/uno-analog-input.json"),
            ),
            (
                "uno-analog-read-serial",
                include_str!("../../../../app/src/examples/uno-analog-read-serial.json"),
            ),
            (
                "uno-night-light",
                include_str!("../../../../app/src/examples/uno-night-light.json"),
            ),
            (
                "uno-thermometer",
                include_str!("../../../../app/src/examples/uno-thermometer.json"),
            ),
            (
                "uno-serial-led",
                include_str!("../../../../app/src/examples/uno-serial-led.json"),
            ),
            (
                "uno-serial-thermometer",
                include_str!("../../../../app/src/examples/uno-serial-thermometer.json"),
            ),
        ] {
            let mut circuit = Circuit::new();
            circuit.try_open(file).unwrap();
            // A second in: every program past its setup, Fade part way up.
            for _ in 0..60 {
                circuit.advance(0.016).unwrap();
            }
            let findings = circuit.solution().findings;
            assert!(findings.is_empty(), "{name}: {findings:?}");
        }
    }
}
