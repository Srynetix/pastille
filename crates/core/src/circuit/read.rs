// SPDX-License-Identifier: MIT OR Apache-2.0

//! Reading the circuit back: what to draw, and what the run says about it.

use super::*;

impl Circuit {
    pub(super) fn locate(&self, element: &str, pin: &str) -> Result<PinRef, CircuitError> {
        let id = decode(element)?;
        let index = self
            .netlist
            .element(id)
            .ok_or(CircuitError::Netlist(NetlistError::UnknownElement(id)))?
            .pin_index(pin)
            .ok_or_else(|| {
                CircuitError::Netlist(NetlistError::UnknownPinName {
                    element: id,
                    name: pin.to_string(),
                })
            })?;
        Ok(PinRef::new(id, index))
    }

    /// One pin, named the way the app names them.
    pub(super) fn pin_view(&self, pin: PinRef) -> Option<PinView> {
        Some(PinView {
            element_id: encode(pin.element),
            pin: self
                .netlist
                .element(pin.element)?
                .pins
                .get(pin.index)?
                .name
                .to_string(),
        })
    }

    pub(super) fn net_of(&self, pin: PinRef) -> Option<NetId> {
        self.netlist.element(pin.element)?.pins.get(pin.index)?.net
    }

    pub(super) fn attach(&mut self, pin: PinRef, net: NetId) -> Result<(), CircuitError> {
        self.netlist
            .connect(pin, net)
            .map_err(CircuitError::Netlist)
    }

    /// [`Circuit::state`] without the boundary's wrapper.
    ///
    /// A circuit that did not solve shows nothing, whether or not there is
    /// still an engine to ask. The last good answer belongs to a circuit that
    /// is no longer on the canvas, and drawing current along wires that have
    /// stopped carrying any would be the app contradicting its own status bar.
    pub fn solution(&self) -> SolutionView {
        match (&self.failure, self.simulation.as_ref()) {
            (None, Some(simulation)) => self.read(simulation.engine(), self.elapsed),
            _ => SolutionView {
                net_voltages: Vec::new(),
                element_currents: Vec::new(),
                net_currents: Vec::new(),
                pin_currents: Vec::new(),
                digits: Vec::new(),
                failure: self.failure.clone(),
                time: 0.0,
                // What the wiring alone says, which is often why it did
                // not solve: an LED straight across a battery (B-02).
                findings: crate::lint::review_wiring(&self.netlist),
                energised: Vec::new(),
                heat: Vec::new(),
                tones: Vec::new(),
                angles: Vec::new(),
                batteries: Vec::new(),
                reversed: Vec::new(),
                serial: Vec::new(),
            },
        }
    }

    /// Rebuilds the engine from the current netlist.
    ///
    /// Wholesale rather than incrementally: at this size it costs less than the
    /// bookkeeping would, and a rebuilt engine cannot disagree with the netlist.
    /// Incremental updates are an optimisation for when C-15 says so.
    pub(super) fn rebuild(&mut self) {
        self.simulated = self.revision;
        self.failure = None;

        // An engine that already exists is handed the edited circuit rather
        // than thrown away and built again. What it has accumulated about the
        // circuit survives the edit that way — a capacitor does not empty
        // itself because somebody changed a resistor (C-48) — and the first
        // build is the only one that can fail outright.
        if let Some(simulation) = self.simulation.as_mut() {
            simulation.engine_mut().adopt(self.netlist.clone());
            return;
        }

        match engine_for(self.netlist.clone()) {
            Ok(engine) => self.simulation = Some(Scheduler::new(engine)),
            Err(error) => {
                self.failure = Some(Failure {
                    code: error.code(),
                    detail: error.to_string(),
                });
            }
        }
    }

    /// Reads the engine's state into something the canvas can use.
    pub(super) fn read(&self, engine: &dyn Engine, time: f64) -> SolutionView {
        let net_voltages = self
            .netlist
            .nets()
            .filter_map(|(id, _)| {
                Some(NetVoltage {
                    net_id: encode_net(id),
                    volts: engine.node_voltage(id)?,
                })
            })
            .collect();

        // The drop across an element, from the potentials of the nets its pins
        // sit on. Computed here rather than asked of the engine, which only
        // tracks it for the elements Newton linearises — a resistor has no
        // entry there, which is why it reported no current at all at first.
        let across = |element: &crate::netlist::Element| -> Option<f64> {
            let [first, second] = element.pins.as_slice() else {
                return None;
            };
            Some(engine.node_voltage(first.net?)? - engine.node_voltage(second.net?)?)
        };

        // The current through each LED of a part whose LEDs share a pin — a
        // digit's segments, an RGB LED's dice — anode to cathode, from the
        // potentials at its two pins: nothing for one either of whose pins is
        // free, which is also one the solver did not stamp (C-75, C-84).
        let shared_amps = |element: &crate::netlist::Element| -> Option<Vec<f64>> {
            let (common, colours) = element.kind.shared_leds()?;
            let volts = |pin: usize| {
                element
                    .pins
                    .get(pin)?
                    .net
                    .and_then(|net| engine.node_voltage(net))
            };
            let amps = colours
                .iter()
                .enumerate()
                .map(|(led, &colour)| {
                    let (anode, cathode) = common.junction(led, colours.len());
                    match (volts(anode), volts(cathode)) {
                        (Some(high), Some(low)) => {
                            crate::solver::device::Led::new(colour).forward_current(high - low)
                        }
                        _ => 0.0,
                    }
                })
                .collect();
            Some(amps)
        };

        let currents: Vec<(ElementId, f64, Option<f64>)> = self
            .netlist
            .elements()
            .filter_map(|(id, element)| {
                let (amps, brightness) = match element.kind {
                    // The branch unknown is the current the source delivers out
                    // of its `+` terminal, which is the opposite of "into pin 0".
                    ElementKind::DcSource { .. } | ElementKind::SignalGenerator { .. } => {
                        (-engine.source_current(id)?, None)
                    }
                    ElementKind::Resistor { ohms } => (across(element)? / ohms, None),
                    // Into its +, so negative while it gives (C-121).
                    ElementKind::Cell { part, charge } => {
                        use crate::solver::device::cell::State;
                        let sheet = part.sheet();
                        let state = engine
                            .carried(id)
                            .map_or_else(State::default, State::decode);
                        let open = sheet.open_volts(state.depth(charge));
                        ((across(element)? - open) / sheet.series_ohms(), None)
                    }
                    ElementKind::Buzzer => (
                        across(element)? / crate::solver::device::load::BUZZER_OHMS,
                        None,
                    ),
                    ElementKind::Motor => (
                        across(element)? / crate::solver::device::load::MOTOR_OHMS,
                        None,
                    ),
                    ElementKind::PassiveBuzzer => {
                        (across(element)? / crate::solver::device::pitch::OHMS, None)
                    }
                    // Nine pins and eight currents: said per segment below, and
                    // per pin for the wires (C-75, C-76). Four and three on an
                    // RGB LED, said the same way (C-84).
                    ElementKind::SevenSegment { .. }
                    | ElementKind::RgbLed { .. }
                    | ElementKind::LcdDigit => return None,
                    // Sixteen pins, and its outputs' current comes from VDD.
                    ElementKind::Decoder
                    | ElementKind::Counter
                    | ElementKind::Servo
                    | ElementKind::MotorDriver => {
                        return None;
                    }
                    // Two currents, one either side of its gap (C-92).
                    ElementKind::Optocoupler => return None,
                    // Sixteen pins, its outputs' current from VCC (C-93).
                    ElementKind::ShiftRegister => return None,
                    // Fourteen, and the same (C-108).
                    ElementKind::Gate { .. } | ElementKind::Inverter | ElementKind::Schmitt => {
                        return None;
                    }
                    // Sixteen, and the same (C-110); fourteen (C-111).
                    ElementKind::Decade | ElementKind::FlipFlop => return None,
                    // Three pins: its supply and its output (C-115, C-116).
                    ElementKind::Hall { .. } | ElementKind::Pir { .. } => return None,
                    // Twenty-two pins, each its own (ADR 0017).
                    ElementKind::Uno => return None,
                    // Through its resistance as the run has it, or nothing
                    // once it has given way (C-120).
                    ElementKind::Fuse { amps, part, blown } => {
                        use crate::solver::device::fuse::{self, State};
                        let state = engine
                            .carried(id)
                            .map_or_else(State::default, State::decode);
                        let ohms = fuse::model(part, amps)
                            .and_then(|model| model.ohms_in(state))
                            .filter(|_| !blown);
                        (
                            ohms.map_or(Some(0.0), |ohms| Some(across(element)? / ohms))?,
                            None,
                        )
                    }
                    // Two windings, a current each; four diodes (C-118). Ten
                    // outputs and a reference; ten LEDs (C-119).
                    ElementKind::Transformer { .. }
                    | ElementKind::Bridge
                    | ElementKind::Lm3914
                    | ElementKind::Bargraph => return None,
                    ElementKind::Ldr { light } => (
                        across(element)? / crate::solver::device::sensor::ldr_ohms(light),
                        None,
                    ),
                    ElementKind::Thermistor { celsius } => (
                        across(element)? / crate::solver::device::sensor::thermistor_ohms(celsius),
                        None,
                    ),
                    // A shut contact is stamped as a zero-volt source, so it
                    // has a branch unknown and its current reads back like any
                    // other source's — which is what keeps the dots running
                    // through a closed switch instead of stopping at it. An
                    // open one carries nothing, and that is the whole point.
                    ElementKind::Switch { .. }
                    | ElementKind::PushButton { .. }
                    | ElementKind::TiltSwitch { .. }
                    | ElementKind::Reed { .. } => (
                        if element.kind.contact() == Some(true) {
                            -engine.source_current(id)?
                        } else {
                            0.0
                        },
                        None,
                    ),
                    // The reading (C-21). An ammeter is stamped as a zero-volt
                    // source, so its branch unknown is the current through it,
                    // and it reads back exactly as a supply's does.
                    ElementKind::Ammeter => (-engine.source_current(id)?, None),
                    // And an ideal voltmeter passes none, which is the property
                    // that makes it a measurement rather than a load.
                    ElementKind::Voltmeter => (0.0, None),

                    // A capacitor's current is a rate of change: C·dv/dt, the
                    // rate the run carries beside the voltage. Before a step
                    // has run, or at DC, nothing is changing and it is none.
                    // Its voltage is pin a over pin b, so a rising one is
                    // current into pin a — the direction every other element
                    // here is read in.
                    ElementKind::Capacitor { farads, .. } => (
                        engine
                            .carried(id)
                            .map_or(0.0, |carried| farads * carried.rate()),
                        None,
                    ),
                    // The same, and what its leak lets through beside it (C-86).
                    ElementKind::Electrolytic { farads, .. } => (
                        engine
                            .carried(id)
                            .map_or(0.0, |carried| farads * carried.rate())
                            + across(element).unwrap_or(0.0)
                                / crate::solver::device::electrolytic::leak_ohms(farads),
                        None,
                    ),
                    // An inductor's current is what it carries, into its first
                    // pin. At DC it is a wire, stamped as a zero-volt source
                    // whose branch unknown is that current, read like any
                    // source's; mid-run it has no branch and the run knows it.
                    ElementKind::Inductor { .. } => (
                        engine
                            .carried(id)
                            .map(|carried| carried.value)
                            .or_else(|| engine.source_current(id).map(|amps| -amps))
                            .unwrap_or(0.0),
                        None,
                    ),
                    // Three terminals, so "the current through it" is not one
                    // number: it divides at the wiper, and an arrow for the
                    // whole element would be a lie. C-21's ammeter reads a
                    // branch of one.
                    ElementKind::Bjt { .. }
                    | ElementKind::Mosfet { .. }
                    | ElementKind::Potentiometer { .. } => return None,
                    // Three pins and no one current: what comes in at the input
                    // leaves partly at the output and partly, as its quiescent
                    // current, at the ground pin (C-54).
                    ElementKind::Regulator { .. }
                    | ElementKind::Adjustable
                    | ElementKind::Tmp36 { .. } => return None,
                    // Five pins, and its output current comes from its supply.
                    ElementKind::OpAmp
                    | ElementKind::Comparator
                    | ElementKind::Timer
                    | ElementKind::Relay { .. } => {
                        return None;
                    }
                    ElementKind::Led { colour } => {
                        let led = crate::solver::device::Led::new(colour);
                        let amps = led.forward_current(across(element)?);
                        (amps, Some(led.brightness(amps)))
                    }
                    // The same junction read the same way, with nothing to
                    // light: what a diode does with its current is pass it or
                    // refuse it (C-50).
                    ElementKind::Diode => (
                        crate::solver::device::Diode::default().current(across(element)?),
                        None,
                    ),
                    ElementKind::Schottky => (
                        crate::solver::device::SCHOTTKY.current(across(element)?),
                        None,
                    ),
                    ElementKind::Zener { part } => (
                        crate::solver::device::Zener::new(part).current(across(element)?),
                        None,
                    ),
                    // Collector to emitter, as the light lets it (C-89).
                    ElementKind::Phototransistor { light } => (
                        crate::solver::device::photo::Phototransistor::new(
                            crate::solver::device::sensor::lux(light),
                        )
                        .current(across(element)?),
                        None,
                    ),
                    // Into its +, which is backwards while it gives power: the
                    // current leaves by its +, as a battery's does (C-117).
                    ElementKind::SolarCell { sun } => (
                        crate::solver::device::solar::SolarCell::new(sun).current(across(element)?),
                        None,
                    ),
                    ElementKind::Ground => return None,
                };
                Some((id, amps, brightness))
            })
            .collect();

        // What a part with more than two pins draws through each, where the
        // core can say (C-101): a coupled part from its own model at the
        // solved point, a relay's coil from the current the run carries, a
        // potentiometer from its two halves. Without these, a wire between
        // two such parts — a transistor and a relay's coil, a wiper and a
        // base — had no current to draw, though the battery was paying for it.
        let mut multi: SecondaryMap<ElementId, Vec<f64>> = SecondaryMap::new();
        for (id, element) in self.netlist.elements() {
            let volts: Vec<Option<f64>> = element
                .pins
                .iter()
                .map(|pin| pin.net.and_then(|net| engine.node_voltage(net)))
                .collect();
            let into = match element.kind {
                ElementKind::Potentiometer { ohms, wiper } => {
                    let along = wiper.clamp(
                        crate::solver::END_RESISTANCE,
                        1.0 - crate::solver::END_RESISTANCE,
                    );
                    let (a, w, b) = (volts[0], volts[1], volts[2]);
                    let half = |end: Option<f64>, share: f64| {
                        end.zip(w)
                            .map_or(0.0, |(end, w)| (end - w) / (ohms * share))
                    };
                    let (into_a, into_b) = (half(a, along), half(b, 1.0 - along));
                    Some(vec![into_a, -(into_a + into_b), into_b])
                }
                ElementKind::Relay { part } => {
                    // Unwired pins are no terminal of the model's, which
                    // stamps nothing that touches them, so any potential will
                    // do for them here.
                    let at: Vec<f64> = volts.iter().map(|volts| volts.unwrap_or(0.0)).collect();
                    let latched = engine.latched(id).unwrap_or(false);
                    crate::solver::pin_currents_of(element, latched, &at).map(|mut into| {
                        use crate::solver::device::relay::{COIL_MINUS, COIL_PLUS};
                        let coil = engine
                            .carried(id)
                            .map(|carried| carried.value)
                            .or_else(|| {
                                Some((volts[COIL_PLUS]? - volts[COIL_MINUS]?) / part.coil_ohms())
                            })
                            .unwrap_or(0.0);
                        into[COIL_PLUS] += coil;
                        into[COIL_MINUS] -= coil;
                        into
                    })
                }
                // Each winding's current, as the run carries it: in at its
                // first pin, out at its second (C-118).
                ElementKind::Transformer { .. } => engine.carried(id).map(|carried| {
                    let (primary, secondary) = (carried.value, carried.held[0]);
                    vec![primary, -primary, secondary, -secondary]
                }),
                // Each LED's current, in at its anode and out at its cathode
                // (C-119).
                ElementKind::Bargraph => {
                    let led = crate::solver::device::Led::new(crate::netlist::LedColour::Red);
                    let mut into = vec![0.0; 20];
                    for segment in 0..10 {
                        if let (Some(a), Some(k)) = (volts[segment], volts[10 + segment]) {
                            let amps = led.forward_current(a - k);
                            into[segment] += amps;
                            into[10 + segment] -= amps;
                        }
                    }
                    Some(into)
                }
                // Its outputs' sinks into V−, its supply's standing current,
                // its reference and its divider, as stamped (C-119).
                ElementKind::Lm3914 => {
                    use crate::solver::device::lm3914::{self, Sink, State};
                    let state = engine
                        .carried(id)
                        .map_or_else(State::default, State::decode);
                    let mut into = vec![0.0; 18];
                    let mut between = |a: usize, b: usize, amps: f64| {
                        into[a] += amps;
                        into[b] -= amps;
                    };
                    let across = |a: usize, b: usize| volts[a].zip(volts[b]).map(|(a, b)| a - b);
                    for k in 1..=10 {
                        let pin = lm3914::led_pin(k);
                        if let Some(v) = across(pin, lm3914::V_MINUS) {
                            let amps = if state.is_lit(k) { state.amps } else { 0.0 };
                            between(pin, lm3914::V_MINUS, Sink { amps }.current(v));
                        }
                    }
                    if let Some(v) = across(lm3914::V_PLUS, lm3914::V_MINUS) {
                        between(lm3914::V_PLUS, lm3914::V_MINUS, v / lm3914::STANDING_OHMS);
                    }
                    if let Some(v) = across(lm3914::RHI, lm3914::RLO) {
                        between(lm3914::RHI, lm3914::RLO, v / lm3914::DIVIDER_OHMS);
                    }
                    if state.powered {
                        if let Some(v) = across(lm3914::REF_OUT, lm3914::REF_ADJ) {
                            let out = (lm3914::REFERENCE_VOLTS - v) / lm3914::REFERENCE_OHMS;
                            between(lm3914::V_PLUS, lm3914::REF_OUT, out);
                        }
                        if across(lm3914::V_PLUS, lm3914::REF_ADJ).is_some() {
                            between(lm3914::V_PLUS, lm3914::REF_ADJ, lm3914::ADJUST_AMPS);
                        }
                    }
                    Some(into)
                }
                // Each pin's, as the program left it and as stamped (ADR
                // 0017): what the linter holds a pin to its 20 mA (A-10).
                ElementKind::Uno => {
                    use crate::solver::device::uno;
                    let drives = engine
                        .carried(id)
                        .map_or([uno::Drive::Input; uno::IO], |carried| {
                            uno::decode(carried.value)
                        });
                    volts[uno::GND].map(|ground| {
                        let above: Vec<Option<f64>> = volts
                            .iter()
                            .map(|volts| volts.map(|volts| volts - ground))
                            .collect();
                        uno::currents(&drives, &above)
                    })
                }
                // Its four diodes' currents, added up at each pin (C-118).
                ElementKind::Bridge => {
                    let junction = crate::solver::device::Diode::default();
                    let mut into = vec![0.0; 4];
                    for (anode, cathode) in [(0, 2), (1, 2), (3, 0), (3, 1)] {
                        if let (Some(a), Some(k)) = (volts[anode], volts[cathode]) {
                            let amps = junction.current(a - k);
                            into[anode] += amps;
                            into[cathode] -= amps;
                        }
                    }
                    Some(into)
                }
                ElementKind::Bjt { .. }
                | ElementKind::Mosfet { .. }
                | ElementKind::OpAmp
                | ElementKind::Comparator
                | ElementKind::Timer
                | ElementKind::Regulator { .. }
                | ElementKind::Adjustable
                | ElementKind::Tmp36 { .. }
                | ElementKind::Optocoupler => {
                    // Stamped only with every pin wired, so read only then.
                    let at: Option<Vec<f64>> = volts.iter().copied().collect();
                    let latched = engine.latched(id).unwrap_or(false);
                    at.and_then(|at| crate::solver::pin_currents_of(element, latched, &at))
                }
                _ => None,
            };
            if let Some(into) = into.filter(|into| into.iter().all(|amps| amps.is_finite())) {
                multi.insert(id, into);
            }
        }

        // Each two-pin part's current, by element: the wires below look one
        // up per pin, and a search of the list per pin was quadratic in the
        // circuit's size, every frame.
        let through: SecondaryMap<ElementId, f64> =
            currents.iter().map(|&(id, amps, _)| (id, amps)).collect();

        let net_currents = self
            .netlist
            .nets()
            .filter_map(|(net_id, net)| {
                // Only a wire joining exactly two pins carries one current. Where
                // three meet, it splits, and a single arrow would be a lie.
                //
                // A pin that cannot carry current is not one of the three. An
                // ideal voltmeter is an open circuit, so tapping one onto a
                // wire adds an end without dividing anything, and the arrow
                // that was there before should still be (C-21).
                let conducting: Vec<PinRef> = net
                    .pins()
                    .iter()
                    .copied()
                    .filter(|pin| {
                        self.netlist
                            .element(pin.element)
                            .is_some_and(|element| element.kind.carries_current())
                    })
                    .collect();
                let [first, second] = conducting[..] else {
                    return None;
                };
                // What flows out of a pin into the wire: a two-pin part's
                // current enters pin 0 and leaves pin 1; a part with more is
                // read pin by pin (C-101). Read off whichever end can say, so
                // whether a wire has an arrow does not depend on which end was
                // wired first.
                let leaving_at = |pin: PinRef| -> Option<f64> {
                    if let Some(into) = multi.get(pin.element) {
                        return Some(-*into.get(pin.index)?);
                    }
                    let amps = *through.get(pin.element)?;
                    match pin.index {
                        0 => Some(-amps),
                        1 => Some(amps),
                        _ => None,
                    }
                };
                let leaving = leaving_at(first).or_else(|| leaving_at(second).map(|amps| -amps))?;
                // Which end the current sets out from. Naming the pin rather
                // than the position, because the two ends may belong to
                // different elements and an id from one with a pin name from
                // the other is not a pin at all.
                let (source, sink) = if leaving >= 0.0 {
                    (first, second)
                } else {
                    (second, first)
                };

                Some(NetCurrent {
                    net_id: encode_net(net_id),
                    amps: leaving.abs(),
                    from: self.pin_view(source)?,
                    to: self.pin_view(sink)?,
                })
            })
            .collect();

        let pin_currents = self
            .netlist
            .nets()
            .flat_map(|(net_id, net)| {
                net.pins()
                    .iter()
                    .filter_map(|pin| {
                        let element = self.netlist.element(pin.element)?;
                        // A voltmeter passes nothing, and a ground symbol only
                        // names the reference: both are ends of a wire that
                        // nothing flows into.
                        let amps = if !element.kind.carries_current()
                            || matches!(element.kind, ElementKind::Ground)
                        {
                            0.0
                        } else if let ElementKind::LcdDigit = element.kind {
                            // Each segment leaks from its pin to COM, so what
                            // leaves the wire at a segment arrives at the
                            // common, and nothing else moves (C-76).
                            let volts =
                                |pin: usize| engine.node_voltage(element.pins.get(pin)?.net?);
                            let leak = |segment: usize| {
                                volts(segment)
                                    .zip(volts(crate::solver::device::digit::COMMON))
                                    .map_or(0.0, |(own, shared)| {
                                        (own - shared) / crate::solver::device::lcd::LEAK_OHMS
                                    })
                            };
                            let segments = crate::solver::device::digit::SEGMENTS.len();
                            if pin.index == crate::solver::device::digit::COMMON {
                                (0..segments).map(leak).sum()
                            } else {
                                -leak(pin.index)
                            }
                        } else if let Some((common, colours)) = element.kind.shared_leds() {
                            // Each LED's current leaves the wire at its anode
                            // and enters it at its cathode; the common pin is
                            // the anode or the cathode of all of them.
                            let amps = shared_amps(element)?;
                            let shared = colours.len();
                            let signed = |led: usize, at: usize| {
                                let (anode, _) = common.junction(led, shared);
                                if at == anode { -amps[led] } else { amps[led] }
                            };
                            if pin.index == shared {
                                (0..amps.len()).map(|led| signed(led, pin.index)).sum()
                            } else {
                                signed(pin.index, pin.index)
                            }
                        } else if let Some(into) = multi.get(pin.element) {
                            // What leaves the wire into the part at this pin.
                            -*into.get(pin.index)?
                        } else {
                            let amps = *through.get(pin.element)?;
                            // The current enters pin 0 and leaves pin 1.
                            match pin.index {
                                0 => -amps,
                                1 => amps,
                                _ => return None,
                            }
                        };
                        Some(PinCurrent {
                            net_id: encode_net(net_id),
                            pin: self.pin_view(*pin)?,
                            amps,
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        // What each part carries, for the rule that holds it to its rating
        // (B-03): read here once, rather than worked out again by the rule.
        let flows =
            crate::lint::Flows::new(currents.iter().map(|&(id, amps, _)| (id, amps)), multi);
        let element_currents: Vec<ElementCurrent> = currents
            .into_iter()
            .map(|(id, amps, brightness)| ElementCurrent {
                element_id: encode(id),
                amps,
                brightness,
            })
            .collect();

        let digits = self
            .netlist
            .elements()
            .filter_map(|(id, element)| {
                // Glass darkens rather than lights, and it is the run that
                // knows how far (C-76).
                if let ElementKind::LcdDigit = element.kind {
                    let glass = engine.glass(id)?;
                    return Some(DigitGlow {
                        element_id: encode(id),
                        segments: (0..crate::solver::device::digit::SEGMENTS.len())
                            .map(|segment| glass.contrast(segment))
                            .collect(),
                    });
                }
                // Ten LEDs of its own, each lit by what it carries (C-119).
                if let ElementKind::Bargraph = element.kind {
                    let led = crate::solver::device::Led::new(crate::netlist::LedColour::Red);
                    let volts = |pin: usize| engine.node_voltage(element.pins.get(pin)?.net?);
                    return Some(DigitGlow {
                        element_id: encode(id),
                        segments: (0..10)
                            .map(|segment| {
                                volts(segment)
                                    .zip(volts(10 + segment))
                                    .map_or(0.0, |(a, k)| {
                                        led.brightness(led.forward_current(a - k))
                                    })
                            })
                            .collect(),
                    });
                }
                let (_, colours) = element.kind.shared_leds()?;
                let amps = shared_amps(element)?;
                Some(DigitGlow {
                    element_id: encode(id),
                    segments: colours
                        .iter()
                        .zip(amps)
                        .map(|(&colour, amps)| {
                            crate::solver::device::Led::new(colour).brightness(amps)
                        })
                        .collect(),
                })
            })
            .collect();

        let findings = self.review(engine, &flows);
        // Each part that is doing what it is for: a relay pulled in, a
        // buzzer sounding, a motor turning (C-64, C-66).
        let volts_across = |element: &crate::netlist::Element| {
            let first = element.pins.first()?.net?;
            let second = element.pins.get(1)?.net?;
            Some(engine.node_voltage(first)? - engine.node_voltage(second)?)
        };
        let energised =
            self.netlist
                .elements()
                .filter(|(id, element)| match element.kind {
                    ElementKind::Relay { .. } => engine.latched(*id) == Some(true),
                    ElementKind::Buzzer => volts_across(element)
                        .is_some_and(crate::solver::device::load::buzzer_sounds),
                    ElementKind::Motor => {
                        volts_across(element).is_some_and(crate::solver::device::load::motor_turns)
                    }
                    // Sounding while it hears a note (C-85).
                    ElementKind::PassiveBuzzer => engine.pitch(*id).is_some(),
                    _ => false,
                })
                .map(|(id, _)| encode(id))
                .collect();

        // Which of those motors turns backwards (C-91): read the same way, so
        // a motor in here is always turning.
        let reversed = self
            .netlist
            .elements()
            .filter(|(_, element)| {
                matches!(element.kind, ElementKind::Motor)
                    && volts_across(element).is_some_and(|volts| {
                        crate::solver::device::load::motor_turns(volts) && volts < 0.0
                    })
            })
            .map(|(id, _)| encode(id))
            .collect();

        // Each running program's serial port (A-06).
        let serial = self
            .netlist
            .elements()
            .filter(|(_, element)| matches!(element.kind, ElementKind::Uno))
            .filter_map(|(id, _)| {
                let (monitor_baud, program_baud) = engine.serial_bauds(id)?;
                Some(ElementSerial {
                    element_id: encode(id),
                    monitor_baud,
                    program_baud,
                })
            })
            .collect();

        // Where each servo's horn is (C-90): the run's, so it moves with the
        // solution and is drawn from it.
        let angles = self
            .netlist
            .elements()
            .filter(|(_, element)| matches!(element.kind, ElementKind::Servo))
            .map(|(id, _)| ElementAngle {
                element_id: encode(id),
                degrees: crate::solver::device::servo::State::decode(engine.carried(id)).angle,
            })
            .collect();

        // How full each battery is, and how long it lasts at what it is
        // giving (C-121).
        let batteries = self
            .netlist
            .elements()
            .filter_map(|(id, element)| {
                let ElementKind::Cell { part, charge } = element.kind else {
                    return None;
                };
                use crate::solver::device::cell::State;
                let sheet = part.sheet();
                let depth = engine
                    .carried(id)
                    .map_or_else(State::default, State::decode)
                    .depth(charge);
                let volts = |pin: usize| engine.node_voltage(element.pins.get(pin)?.net?);
                let amps = volts(0).zip(volts(1)).map_or(0.0, |(a, b)| {
                    (sheet.open_volts(depth) - (a - b)) / sheet.series_ohms()
                });
                Some(ElementCharge {
                    element_id: encode(id),
                    left: (1.0 - depth).clamp(0.0, 1.0),
                    hours: sheet.hours_left(depth, amps),
                })
            })
            .collect();

        let tones = self
            .netlist
            .elements()
            .filter_map(|(id, _)| {
                Some(ElementTone {
                    element_id: encode(id),
                    hertz: engine.pitch(id)?,
                })
            })
            .collect();

        let room = self.netlist.room_celsius();
        let heat = self
            .netlist
            .elements()
            .filter_map(|(id, element)| {
                let now = engine.heat(id)?;
                let rating = crate::registry::rating_of(&element.kind);
                let thermal = rating.thermal?;
                Some(ElementHeat {
                    element_id: encode(id),
                    watts: now.watts,
                    celsius: now.celsius,
                    settles_at: crate::solver::heat::settles_at(&thermal, room, now.watts),
                    rated_watts: rating.power,
                    max_celsius: thermal.max_junction_celsius,
                })
            })
            .collect();

        SolutionView {
            net_voltages,
            element_currents,
            net_currents,
            pin_currents,
            digits,
            failure: None,
            time,
            findings,
            energised,
            heat,
            tones,
            angles,
            batteries,
            reversed,
            serial,
        }
    }

    /// Looks over the solved circuit for things worth saying: the design
    /// linter's rules, which live in `lint` (B-01).
    pub(super) fn review(&self, engine: &dyn Engine, flows: &crate::lint::Flows) -> Vec<Finding> {
        crate::lint::review(&self.netlist, engine, flows)
    }

    /// The netlist itself, for the parts of the core that solve it.
    pub fn netlist(&self) -> &Netlist {
        &self.netlist
    }

    /// Net handles, for tests that need one without going through `Ts`.
    #[cfg(test)]
    pub(crate) fn nets_for_test(&self) -> Vec<String> {
        self.netlist.nets().map(|(id, _)| encode_net(id)).collect()
    }

    /// Everything to draw, in insertion order.
    ///
    /// One description of an element, used by the boundary and by the tests
    /// that read what the boundary would have said. Two copies of this list
    /// meant a field could reach the canvas and not the tests, which is exactly
    /// the sort of thing the tests are there to catch.
    pub(super) fn views(&self) -> Vec<ElementView> {
        self.netlist
            .elements()
            .filter_map(|(id, element)| {
                let at = self.layout.get(id)?;
                Some(ElementView {
                    id: encode(id),
                    kind: kind_name(&element.kind).to_string(),
                    x: at.x,
                    y: at.y,
                    turns: at.turns,
                    label: label_for(&element.kind),
                    value: value_of(&element.kind).map(value::format_editable),
                    unit: unit_of(&element.kind).to_string(),
                    colour: match element.kind {
                        ElementKind::Led { colour } => Some(format!("{colour:?}")),
                        _ => None,
                    },
                    series_ohms: match element.kind {
                        ElementKind::DcSource { series_ohms, .. } => Some(series_ohms),
                        _ => None,
                    },
                    hertz: match element.kind {
                        ElementKind::SignalGenerator { hertz, .. } => Some(hertz),
                        _ => None,
                    },
                    initial_condition: element.kind.initial_condition(),
                    wiper: element.kind.wiper(),
                    choice: element.kind.choice().map(str::to_string),
                    program: self.netlist.program(id).map(|program| program.name.clone()),
                    based_on: crate::registry::definition(element.kind.name())
                        .and_then(|component| (component.basis)(&element.kind))
                        .map(|basis| BasisView {
                            part: basis.part.to_string(),
                            datasheet: basis.datasheet.map(str::to_string),
                            approximate: basis.approximate,
                        }),
                    choices: crate::registry::definition(element.kind.name())
                        .map(|component| {
                            (component.choices)()
                                .into_iter()
                                .map(str::to_string)
                                .collect()
                        })
                        .unwrap_or_default(),
                    initial_unit: match element.kind {
                        ElementKind::Capacitor { .. } | ElementKind::Electrolytic { .. } => {
                            "V".to_string()
                        }
                        ElementKind::Inductor { .. } => "A".to_string(),
                        _ => String::new(),
                    },
                    // A PIR module's "pressed" is someone moving (C-116).
                    closed: element.kind.contact().or(element.kind.moving()),
                    blown: match element.kind {
                        ElementKind::Fuse { blown, .. } => Some(blown),
                        _ => None,
                    },
                })
            })
            .collect()
    }

    /// The element views, for tests that need to read what crosses the
    /// boundary without going through `Ts`.
    #[cfg(test)]
    pub(crate) fn elements_for_test(&self) -> Vec<ElementView> {
        self.views()
    }

    pub fn placement_of(&self, id: ElementId) -> Option<Placement> {
        self.layout.get(id).copied()
    }

    /// Where a wire has been bent, if it has been.
    pub fn waypoints_of(&self, id: NetId) -> &[PointView] {
        self.wires.get(id).map_or(&[], Vec::as_slice)
    }
}
