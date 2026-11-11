// SPDX-License-Identifier: MIT OR Apache-2.0

//! DC operating point: where the circuit settles with nothing changing.
//!
//! Compile the netlist, stamp it, solve once. Nonlinear elements need Newton
//! around this point (M-09); for now they are reported as unstamped and the
//! linear part of the circuit still answers.

use crate::netlist::{
    CompileError, ElementId, ElementKind, NetId, Netlist, NodeMap, REFERENCE_NODE,
};

use super::device::{Bjt, Comparator, Diode, Led, OpAmp, Regulator, SaturatingSource, Timer};
use super::{
    Companion, Coupled, CoupledCompanion, CoupledDevice, Device, MAX_TERMINALS, MnaSystem,
    NewtonError, NewtonOptions, Nonlinear, Timestep, Unstamped, assemble, assemble_at,
    solve_nonlinear_coupled,
};

/// Why an operating point could not be found.
// Not Eq: NewtonError carries the floats that describe how it failed.
#[derive(Debug, Clone, PartialEq)]
pub enum DcError {
    /// The netlist could not be numbered — no reference node, typically.
    Compile(CompileError),
    /// No operating point could be found.
    Solve(NewtonError),
}

impl From<CompileError> for DcError {
    fn from(error: CompileError) -> Self {
        DcError::Compile(error)
    }
}

impl From<NewtonError> for DcError {
    fn from(error: NewtonError) -> Self {
        DcError::Solve(error)
    }
}

impl core::fmt::Display for DcError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DcError::Compile(error) => write!(f, "{error}"),
            DcError::Solve(error) => write!(f, "{error}"),
        }
    }
}

impl core::error::Error for DcError {}

/// The solved state of a circuit.
#[derive(Debug, Clone)]
pub struct OperatingPoint {
    map: NodeMap,
    /// Indexed by node index. Entry 0 is the reference, always exactly zero.
    voltages: Vec<f64>,
    source_currents: Vec<(ElementId, f64)>,
    device_voltages: Vec<(ElementId, f64)>,
    unstamped: Vec<Unstamped>,
    iterations: usize,
    /// Every unknown of the system this was solved from, in row order: the
    /// starting point for the next solve of the same circuit, which a
    /// millisecond later is nearly always nearly the answer.
    solution: Vec<f64>,
}

impl OperatingPoint {
    /// The unknowns as Newton left them, for starting the next solve here.
    pub(super) fn solution(&self) -> &[f64] {
        &self.solution
    }

    /// Potential of a net, relative to the reference node.
    ///
    /// `None` for a net that carries no pins, which is not a node at all.
    pub fn voltage(&self, net: NetId) -> Option<f64> {
        let index = self.map.index_of(net)?;
        self.voltages.get(index as usize).copied()
    }

    /// Node voltages by node index, reference first.
    pub fn node_voltages(&self) -> &[f64] {
        &self.voltages
    }

    /// Current through a voltage source, positive when it flows out of the `+`
    /// terminal into the rest of the circuit — the direction a beginner draws.
    pub fn source_current(&self, element: ElementId) -> Option<f64> {
        self.source_currents
            .iter()
            .find(|(id, _)| *id == element)
            .map(|(_, current)| *current)
    }

    /// Voltage across a nonlinear element, `+` terminal minus `-` terminal.
    pub fn voltage_across(&self, element: ElementId) -> Option<f64> {
        self.device_voltages
            .iter()
            .find(|(id, _)| *id == element)
            .map(|(_, volts)| *volts)
    }

    /// Newton iterations the answer took. Zero for a circuit with nothing
    /// nonlinear in it.
    pub fn iterations(&self) -> usize {
        self.iterations
    }

    /// Elements the solve ignored, and why.
    pub fn unstamped(&self) -> &[Unstamped] {
        &self.unstamped
    }

    pub fn node_map(&self) -> &NodeMap {
        &self.map
    }
}

/// Solves a netlist for its DC operating point.
pub fn operating_point(netlist: &Netlist) -> Result<OperatingPoint, DcError> {
    operating_point_with(netlist, &NewtonOptions::default())
}

/// The same solve, started from where a previous one of the same circuit
/// ended rather than from nothing.
///
/// The live loop solves the same circuit a thousand times a second, and from
/// a cold start every one of them walks every junction up from zero again: a
/// dozen Newton iterations for a CD4511 and its digit, where the last answer
/// needs one or two. A start that does not fit — the circuit was edited and
/// has another size — is a cold start; one that fits and does not converge
/// is retried cold before anything else is tried, so the answer is the one a
/// cold start finds whenever the warm one cannot.
pub fn operating_point_near(
    netlist: &Netlist,
    near: &OperatingPoint,
) -> Result<OperatingPoint, DcError> {
    let map = netlist.compile()?;
    let system = assemble(netlist, &map);
    solve_system_from(
        netlist,
        map,
        &system,
        &NewtonOptions::default(),
        near.solution(),
    )
}

/// Solves one step of a transient: the same solve, with the reactive elements
/// stamped as the companions the step makes of them (C-06).
///
/// One step, not a loop. Owning the sequence of steps — choosing their length,
/// rejecting one that moved too far, carrying the state forward — is C-09 and
/// C-10; this is the call they will make.
pub fn step(netlist: &Netlist, step: &Timestep) -> Result<OperatingPoint, DcError> {
    let map = netlist.compile()?;
    let system = assemble_at(netlist, &map, Some(step));
    solve_system(netlist, map, &system, &NewtonOptions::default())
}

/// Same, with the iteration limits and tolerances spelled out.
pub fn operating_point_with(
    netlist: &Netlist,
    options: &NewtonOptions,
) -> Result<OperatingPoint, DcError> {
    let map = netlist.compile()?;
    let system = assemble(netlist, &map);
    solve_system(netlist, map, &system, options)
}

/// How far gmin is stepped down each time, and where it starts and stops.
///
/// Ten at a time: enough that a handful of solves covers twelve orders of
/// magnitude, small enough that each answer is a usable guess for the next.
/// Starting at a millisiemens makes even an all-off circuit well conditioned;
/// a picosiemens is below anything a circuit of this size can notice, so the
/// last real step is the one that matters.
const GMIN_START: f64 = 1e-3;
const GMIN_FLOOR: f64 = 1e-12;
const GMIN_RATIO: f64 = 0.1;

/// How the sources are ramped, when gmin stepping was not enough.
///
/// A circuit with every supply at a tenth of itself has every junction far from
/// its exponential, which is a circuit Newton can always solve. Walking the
/// supplies up in tenths carries that answer to the real one.
const SOURCE_STEPS: usize = 10;

/// Solves a nonlinear system, reaching for a convergence aid only if it has to.
///
/// Plain Newton first, because almost every circuit converges cold and the aids
/// cost solves. Then gmin stepping, then source stepping — the order SPICE uses,
/// and for the same reason: gmin fixes a badly conditioned matrix, which is the
/// commoner failure and the cheaper fix, while source stepping fixes a bad
/// starting guess and costs a solve per step.
///
/// §11 of the design doc calls this the place projects of this kind die. What
/// makes it tractable here is that both aids are the same loop with one number
/// changed, and both hand their answer to the next attempt as its starting
/// guess.
fn attempt(
    system: &MnaSystem,
    devices: &[Device<'_>],
    coupled: &[CoupledDevice<'_>],
    options: &NewtonOptions,
    start: &[f64],
) -> Result<super::NewtonSolution, NewtonError> {
    if start.len() == system.size()
        && let Ok(solution) = solve_nonlinear_coupled(system, devices, coupled, options, start)
    {
        return Ok(solution);
    }
    let plain = solve_nonlinear_coupled(system, devices, coupled, options, &[]);
    let Err(first) = plain else {
        return plain;
    };

    if let Ok(solution) = with_gmin_stepping(system, devices, coupled, options) {
        return Ok(solution);
    }

    // Report the original failure rather than the aid's: what the user asked
    // for is the circuit, not the ramp, and the first error is the one that
    // describes it.
    with_source_stepping(system, devices, coupled, options).map_err(|_| first)
}

/// Leaks every node to ground, then closes the leak a decade at a time.
fn with_gmin_stepping(
    system: &MnaSystem,
    devices: &[Device<'_>],
    coupled: &[CoupledDevice<'_>],
    options: &NewtonOptions,
) -> Result<super::NewtonSolution, NewtonError> {
    let mut gmin = GMIN_START;
    let mut guess: Vec<f64> = Vec::new();
    let mut last: Option<super::NewtonSolution> = None;

    while gmin >= GMIN_FLOOR {
        let mut leaky = system.clone();
        leaky.add_gmin(gmin);

        let solution = solve_nonlinear_coupled(&leaky, devices, coupled, options, &guess)?;
        guess = solution.x.clone();
        last = Some(solution);
        gmin *= GMIN_RATIO;
    }

    // The real circuit, from the answer to the one nearly like it.
    match solve_nonlinear_coupled(system, devices, coupled, options, &guess) {
        Ok(solution) => Ok(solution),
        // Every leaky circuit solved and the dry one did not. The last answer
        // is within a picosiemens of the real one, which for a circuit this
        // size is closer than anything else on offer.
        Err(error) => last.ok_or(error),
    }
}

/// Turns every supply down, then walks it back up.
fn with_source_stepping(
    system: &MnaSystem,
    devices: &[Device<'_>],
    coupled: &[CoupledDevice<'_>],
    options: &NewtonOptions,
) -> Result<super::NewtonSolution, NewtonError> {
    let mut guess: Vec<f64> = Vec::new();
    let mut solution = None;

    for step in 1..=SOURCE_STEPS {
        let factor = step as f64 / SOURCE_STEPS as f64;
        let mut turned_down = system.clone();
        turned_down.scale_sources(factor);

        let found = solve_nonlinear_coupled(&turned_down, devices, coupled, options, &guess)?;
        guess = found.x.clone();
        solution = Some(found);
    }

    solution.ok_or(NewtonError::Solve(super::SolveError::Singular {
        column: 0,
    }))
}

/// A two-terminal nonlinear element, built for one solve.
///
/// Both are the same pn junction underneath — an LED is a diode whose
/// saturation current was chosen to put its forward drop where its colour says
/// (M-11). They are two variants rather than one model with a parameter
/// because only one of them has a brightness, and only one of them is a
/// component somebody chooses a colour for.
enum Junction {
    Diode(Diode),
    Led(Led),
    Zener(super::device::Zener),
    /// Not a lone junction but a transistor whose floating base makes it a
    /// two-terminal curve all the same (C-89).
    Photo(super::device::photo::Phototransistor),
    /// A junction with the light's current across it (C-117).
    Solar(super::device::solar::SolarCell),
    /// Not a junction at all but a current sink that saturates (C-119).
    Sink(super::device::lm3914::Sink),
}

impl Junction {
    /// The model for one of a kind's junctions, or nothing for a kind that is
    /// not a junction. `which` matters only to a part whose LEDs share a pin:
    /// every segment of a digit is the red LED, an RGB LED's dice are three
    /// colours (C-75, C-84).
    fn of(kind: ElementKind, which: usize, drive: f64) -> Option<Junction> {
        if let Some((_, colours)) = kind.shared_leds() {
            return colours
                .get(which)
                .map(|&colour| Junction::Led(Led::new(colour)));
        }
        match kind {
            // An L293D's clamps, two an output (C-91).
            // A bridge's four, all the same junction (C-118).
            ElementKind::Diode | ElementKind::MotorDriver | ElementKind::Bridge => {
                Some(Junction::Diode(Diode::default()))
            }
            ElementKind::Schottky => Some(Junction::Diode(super::device::SCHOTTKY)),
            ElementKind::Led { colour } => Some(Junction::Led(Led::new(colour))),
            ElementKind::Phototransistor { light } => Some(Junction::Photo(
                super::device::photo::Phototransistor::new(super::device::sensor::lux(light)),
            )),
            ElementKind::Zener { part } => Some(Junction::Zener(super::device::Zener::new(part))),
            // An LM3914's outputs, each sinking what its state says (C-119).
            ElementKind::Lm3914 => {
                Some(Junction::Sink(super::device::lm3914::Sink { amps: drive }))
            }
            // A bar's ten LEDs, all red (C-119).
            ElementKind::Bargraph => Some(Junction::Led(Led::new(crate::netlist::LedColour::Red))),
            ElementKind::SolarCell { sun } => {
                Some(Junction::Solar(super::device::solar::SolarCell::new(sun)))
            }
            _ => None,
        }
    }
}

impl Nonlinear for Junction {
    fn linearise(&self, voltage_across: f64, previous_point: f64) -> Companion {
        match self {
            Junction::Diode(diode) => diode.linearise(voltage_across, previous_point),
            Junction::Led(led) => led.linearise(voltage_across, previous_point),
            Junction::Zener(zener) => zener.linearise(voltage_across, previous_point),
            Junction::Photo(photo) => photo.linearise(voltage_across, previous_point),
            Junction::Solar(solar) => solar.linearise(voltage_across, previous_point),
            Junction::Sink(sink) => sink.linearise(voltage_across, previous_point),
        }
    }
}

/// A coupled device, as the solver builds one.
///
/// The same idea as [`Junction`] for the wider seam, and for the same reason:
/// the coupled placements are zipped with their models, so a kind the builder
/// did not know about would pair the next placement with the wrong one. With
/// only the transistor that could not happen; with the regulator (C-54) a
/// `filter_map` matching `Bjt` alone would have solved a regulator as its
/// neighbour's transistor.
enum Block {
    Transistor(Bjt),
    Mosfet(super::device::Mosfet),
    Saturating(SaturatingSource),
    Timer(Timer),
    Relay(super::device::Relay),
    Opto(super::device::opto::Optocoupler),
}

impl Block {
    /// The model for a kind, or nothing for a kind that is not coupled. A
    /// 555 is built with the bit it carries into this solve (ADR 0012).
    fn of(element: &crate::netlist::Element, latched: bool) -> Option<Block> {
        match element.kind {
            ElementKind::Timer => Some(Block::Timer(Timer { high: latched })),
            ElementKind::Relay { part } => {
                let mut wired = [false; super::device::relay::PIN_COUNT];
                for (slot, pin) in wired.iter_mut().zip(&element.pins) {
                    *slot = pin.net.is_some();
                }
                Some(Block::Relay(super::device::Relay {
                    part,
                    energised: latched,
                    wired,
                }))
            }
            ElementKind::Mosfet { channel } => {
                Some(Block::Mosfet(super::device::Mosfet::new(channel)))
            }
            ElementKind::Bjt { polarity } => Some(Block::Transistor(Bjt::new(polarity))),
            ElementKind::Regulator { part } => {
                Some(Block::Saturating(Regulator::new(part).model()))
            }
            ElementKind::Adjustable => Some(Block::Saturating(
                Regulator::new(super::device::RegulatorPart::Lm317).model(),
            )),
            ElementKind::OpAmp => Some(Block::Saturating(OpAmp.model())),
            ElementKind::Comparator => Some(Block::Saturating(Comparator.model())),
            ElementKind::Tmp36 { celsius } => {
                Some(Block::Saturating(super::device::tmp36::model(celsius)))
            }
            ElementKind::Optocoupler => Some(Block::Opto(super::device::opto::Optocoupler)),
            _ => None,
        }
    }
}

/// The current into each pin of a coupled part, with its pins at `volts`
/// (pin order, one a pin), or nothing for a kind that is not one (C-101).
///
/// The part is linearised at those potentials with nothing to limit — its
/// points settled there — so its companion's current into each terminal is
/// the part's own current at that point, exactly. A part whose terminals are
/// not its pins in order (a regulator, a TMP36) adds each terminal's current
/// to the pin it is wired to.
///
/// That rests on [`Coupled::settled`] being a history that holds nothing
/// back. A model left on the cold default is linearised where its bounds
/// would put it, not where it is: an op-amp or a comparator saturated against
/// a rail read as kiloamps that way, and a 555 at its current limit as more
/// than twice what its load took.
pub(crate) fn pin_currents_of(
    element: &crate::netlist::Element,
    latched: bool,
    volts: &[f64],
) -> Option<Vec<f64>> {
    let block = Block::of(element, latched)?;
    let wiring: Vec<usize> = match element.kind {
        ElementKind::Regulator { .. } | ElementKind::Adjustable => {
            super::device::regulator::WIRING.to_vec()
        }
        ElementKind::Tmp36 { .. } => super::device::tmp36::WIRING.to_vec(),
        _ => (0..element.pins.len().min(MAX_TERMINALS)).collect(),
    };
    let mut potentials = [0.0; MAX_TERMINALS];
    for (terminal, &pin) in wiring.iter().enumerate() {
        potentials[terminal] = *volts.get(pin)?;
    }
    let settled = block.settled(&potentials);
    let companion = block.linearise(&potentials, &settled);
    let mut into = vec![0.0; element.pins.len()];
    for (terminal, &pin) in wiring.iter().enumerate().take(companion.terminals) {
        let through: f64 = (0..companion.terminals)
            .map(|other| companion.conductances[terminal][other] * potentials[other])
            .sum();
        into[pin] += through + companion.currents[terminal];
    }
    Some(into)
}

impl Coupled for Block {
    fn settled(&self, potentials: &[f64; MAX_TERMINALS]) -> [f64; MAX_TERMINALS] {
        match self {
            Block::Transistor(bjt) => bjt.settled(potentials),
            Block::Mosfet(mosfet) => mosfet.settled(potentials),
            Block::Saturating(source) => source.settled(potentials),
            Block::Timer(timer) => timer.settled(potentials),
            Block::Relay(relay) => relay.settled(potentials),
            Block::Opto(opto) => opto.settled(potentials),
        }
    }

    fn linearise(
        &self,
        potentials: &[f64; MAX_TERMINALS],
        previous: &[f64; MAX_TERMINALS],
    ) -> CoupledCompanion {
        match self {
            Block::Transistor(bjt) => bjt.linearise(potentials, previous),
            Block::Mosfet(mosfet) => mosfet.linearise(potentials, previous),
            Block::Saturating(source) => source.linearise(potentials, previous),
            Block::Timer(timer) => timer.linearise(potentials, previous),
            Block::Relay(relay) => relay.linearise(potentials, previous),
            Block::Opto(opto) => opto.linearise(potentials, previous),
        }
    }
}

pub(super) fn solve_system(
    netlist: &Netlist,
    map: NodeMap,
    system: &MnaSystem,
    options: &NewtonOptions,
) -> Result<OperatingPoint, DcError> {
    solve_system_from(netlist, map, system, options, &[])
}

/// [`solve_system`], starting Newton at `start` when it fits the system.
pub(super) fn solve_system_from(
    netlist: &Netlist,
    map: NodeMap,
    system: &MnaSystem,
    options: &NewtonOptions,
    start: &[f64],
) -> Result<OperatingPoint, DcError> {
    // A circuit with a reference node and nothing else is already solved.
    if system.size() == 0 {
        return Ok(OperatingPoint {
            voltages: vec![0.0; map.len()],
            map,
            source_currents: Vec::new(),
            device_voltages: Vec::new(),
            unstamped: system.unstamped().to_vec(),
            iterations: 0,
            solution: Vec::new(),
        });
    }

    // Build a model per nonlinear element, then hand the placements to Newton.
    // Ownership has to outlive the borrow the devices take, hence the two steps.
    //
    // Zipped with the placements afterwards, so this has to produce one model
    // for every one of them and in the same order. A `filter_map` that quietly
    // dropped a kind would not fail to compile — it would pair the next
    // placement with the wrong model, which is a diode solved as somebody
    // else's LED.
    let models: Vec<(ElementId, Junction)> = system
        .nonlinear()
        .iter()
        .filter_map(|placement| {
            let element = netlist.element(placement.element)?;
            Some((
                placement.element,
                Junction::of(element.kind, placement.junction, placement.drive)?,
            ))
        })
        .collect();
    debug_assert_eq!(
        models.len(),
        system.nonlinear().len(),
        "every nonlinear placement needs a model of its own"
    );

    let devices: Vec<Device<'_>> = system
        .nonlinear()
        .iter()
        .zip(&models)
        .map(|(placement, (_, model))| Device {
            plus: placement.plus,
            minus: placement.minus,
            model,
        })
        .collect();

    // The coupled devices, built the same way and for the same reason: the
    // models have to outlive the borrows the placements take.
    let blocks: Vec<(ElementId, Block)> = system
        .coupled()
        .iter()
        .filter_map(|placement| {
            let element = netlist.element(placement.element)?;
            Some((placement.element, Block::of(element, placement.latched)?))
        })
        .collect();
    debug_assert_eq!(
        blocks.len(),
        system.coupled().len(),
        "every coupled placement needs a model of its own"
    );

    let coupled: Vec<CoupledDevice<'_>> = system
        .coupled()
        .iter()
        .zip(&blocks)
        .map(|(placement, (_, model))| CoupledDevice {
            rows: placement.rows,
            model,
        })
        .collect();

    let solution = attempt(system, &devices, &coupled, options, start)?;
    let x = solution.x;

    let mut voltages = vec![0.0; map.len()];
    for node in map.nodes() {
        if node.index == REFERENCE_NODE {
            continue;
        }
        let row = system
            .row_of_node(node.index)
            .expect("a non-reference node has a row");
        voltages[node.index as usize] = x[row];
    }

    // MNA's source unknown is the current flowing into the `+` terminal, so it
    // is the negation of the current the source delivers to the circuit.
    // Walked from the system's own list, row by row: asking each element for
    // its row was a search of that list per element, on every solve.
    let first_source_row = system.size() - system.sources().len();
    let source_currents = system
        .sources()
        .iter()
        .enumerate()
        .map(|(offset, &id)| (id, -x[first_source_row + offset]))
        .collect();

    let at = |row: Option<usize>| row.map_or(0.0, |row| x[row]);
    let device_voltages = system
        .nonlinear()
        .iter()
        .map(|placement| (placement.element, at(placement.plus) - at(placement.minus)))
        .collect();

    Ok(OperatingPoint {
        map,
        voltages,
        source_currents,
        device_voltages,
        unstamped: system.unstamped().to_vec(),
        iterations: solution.iterations,
        solution: x,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::netlist::{ElementKind, LedColour};

    /// The acceptance tolerance for M-08: analytic answers to within 1e-9.
    const TOLERANCE: f64 = 1e-9;

    #[test]
    fn a_divider_settles_at_its_analytic_voltages() {
        // 9 V across 1 kΩ + 2 kΩ. The junction sits at 9 × 2/3 = 6 V, and the
        // loop carries 9 / 3000 = 3 mA.
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts: 9.0,
            series_ohms: 0.0,
        });
        let upper = netlist.add_element(ElementKind::Resistor { ohms: 1_000.0 });
        let lower = netlist.add_element(ElementKind::Resistor { ohms: 2_000.0 });

        let top = netlist.add_net();
        let middle = netlist.add_net();
        let bottom = netlist.add_net();

        netlist.connect_named(source, "+", top).unwrap();
        netlist.connect_named(upper, "a", top).unwrap();
        netlist.connect_named(upper, "b", middle).unwrap();
        netlist.connect_named(lower, "a", middle).unwrap();
        netlist.connect_named(lower, "b", bottom).unwrap();
        netlist.connect_named(source, "-", bottom).unwrap();

        let point = operating_point(&netlist).unwrap();

        assert!((point.voltage(top).unwrap() - 9.0).abs() < TOLERANCE);
        assert!((point.voltage(middle).unwrap() - 6.0).abs() < TOLERANCE);
        assert!((point.voltage(bottom).unwrap() - 0.0).abs() < TOLERANCE);
        assert!((point.source_current(source).unwrap() - 3e-3).abs() < TOLERANCE);
    }

    #[test]
    fn a_series_chain_divides_in_proportion_to_its_resistors() {
        // 12 V across 1 k, 2 k and 3 k: 2 mA, so the taps sit at 10 V and 6 V.
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts: 12.0,
            series_ohms: 0.0,
        });
        let first = netlist.add_element(ElementKind::Resistor { ohms: 1_000.0 });
        let second = netlist.add_element(ElementKind::Resistor { ohms: 2_000.0 });
        let third = netlist.add_element(ElementKind::Resistor { ohms: 3_000.0 });

        let top = netlist.add_net();
        let tap_one = netlist.add_net();
        let tap_two = netlist.add_net();
        let bottom = netlist.add_net();

        netlist.connect_named(source, "+", top).unwrap();
        netlist.connect_named(first, "a", top).unwrap();
        netlist.connect_named(first, "b", tap_one).unwrap();
        netlist.connect_named(second, "a", tap_one).unwrap();
        netlist.connect_named(second, "b", tap_two).unwrap();
        netlist.connect_named(third, "a", tap_two).unwrap();
        netlist.connect_named(third, "b", bottom).unwrap();
        netlist.connect_named(source, "-", bottom).unwrap();

        let point = operating_point(&netlist).unwrap();

        assert!((point.voltage(tap_one).unwrap() - 10.0).abs() < TOLERANCE);
        assert!((point.voltage(tap_two).unwrap() - 6.0).abs() < TOLERANCE);
        assert!((point.source_current(source).unwrap() - 2e-3).abs() < TOLERANCE);
    }

    #[test]
    fn a_single_resistor_across_a_source_obeys_ohm() {
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts: 5.0,
            series_ohms: 0.0,
        });
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: 220.0 });

        let top = netlist.add_net();
        let bottom = netlist.add_net();
        netlist.connect_named(source, "+", top).unwrap();
        netlist.connect_named(resistor, "a", top).unwrap();
        netlist.connect_named(resistor, "b", bottom).unwrap();
        netlist.connect_named(source, "-", bottom).unwrap();

        let point = operating_point(&netlist).unwrap();

        assert!((point.voltage(top).unwrap() - 5.0).abs() < TOLERANCE);
        // Positive means out of the + terminal and into the circuit.
        assert!((point.source_current(source).unwrap() - 5.0 / 220.0).abs() < TOLERANCE);
    }

    #[test]
    fn parallel_resistors_share_the_current() {
        // 6 V across 1 kΩ ∥ 1 kΩ: 500 Ω, so 12 mA in total.
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts: 6.0,
            series_ohms: 0.0,
        });
        let left = netlist.add_element(ElementKind::Resistor { ohms: 1_000.0 });
        let right = netlist.add_element(ElementKind::Resistor { ohms: 1_000.0 });

        let top = netlist.add_net();
        let bottom = netlist.add_net();
        for (element, a, b) in [(left, "a", "b"), (right, "a", "b")] {
            netlist.connect_named(element, a, top).unwrap();
            netlist.connect_named(element, b, bottom).unwrap();
        }
        netlist.connect_named(source, "+", top).unwrap();
        netlist.connect_named(source, "-", bottom).unwrap();

        let point = operating_point(&netlist).unwrap();

        assert!((point.source_current(source).unwrap() - 12e-3).abs() < TOLERANCE);
    }

    #[test]
    fn the_reference_node_is_exactly_zero() {
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts: 9.0,
            series_ohms: 0.0,
        });
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: 1_000.0 });
        let top = netlist.add_net();
        let bottom = netlist.add_net();
        netlist.connect_named(source, "+", top).unwrap();
        netlist.connect_named(resistor, "a", top).unwrap();
        netlist.connect_named(resistor, "b", bottom).unwrap();
        netlist.connect_named(source, "-", bottom).unwrap();

        let point = operating_point(&netlist).unwrap();

        // Not "within tolerance": the reference is zero by construction, and a
        // drifting reference would mean the row was not really dropped.
        assert_eq!(point.voltage(bottom), Some(0.0));
        assert_eq!(point.node_voltages()[REFERENCE_NODE as usize], 0.0);
    }

    #[test]
    fn a_netlist_with_nothing_wired_reports_why() {
        // Not "no ground": there is nothing to reference because there is
        // nothing joined to anything.
        let mut netlist = Netlist::new();
        netlist.add_element(ElementKind::Resistor { ohms: 220.0 });

        assert_eq!(
            operating_point(&netlist).unwrap_err(),
            DcError::Compile(CompileError::NoReferenceNode)
        );
    }

    #[test]
    fn a_passive_fragment_settles_at_zero_rather_than_failing() {
        // Two resistors joined to each other and nothing else. No source means
        // no potential is defined, so one net is picked as the zero and
        // everything sits on it — which is true, and better than refusing to
        // solve the rest of the canvas because of it.
        let mut netlist = Netlist::new();
        let left = netlist.add_element(ElementKind::Resistor { ohms: 220.0 });
        let right = netlist.add_element(ElementKind::Resistor { ohms: 470.0 });
        let net = netlist.add_net();
        netlist.connect_named(left, "a", net).unwrap();
        netlist.connect_named(right, "a", net).unwrap();

        let point = operating_point(&netlist).unwrap();

        assert!(point.node_voltages().iter().all(|volts| *volts == 0.0));
    }

    #[test]
    fn a_stray_fragment_does_not_stop_the_rest_of_the_canvas_solving() {
        // A resistor off on its own, beside a working circuit. Before every
        // circuit had its own reference this made the whole matrix singular,
        // so one stray component stopped everything.
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts: 9.0,
            series_ohms: 0.0,
        });
        let load = netlist.add_element(ElementKind::Resistor { ohms: 1_000.0 });
        let stub = netlist.add_element(ElementKind::Resistor { ohms: 1_000.0 });

        let top = netlist.add_net();
        let bottom = netlist.add_net();
        let orphan = netlist.add_net();
        netlist.connect_named(source, "+", top).unwrap();
        netlist.connect_named(load, "a", top).unwrap();
        netlist.connect_named(load, "b", bottom).unwrap();
        netlist.connect_named(source, "-", bottom).unwrap();
        netlist.connect_named(stub, "a", orphan).unwrap();
        netlist.connect_named(stub, "b", orphan).unwrap();

        let point = operating_point(&netlist).unwrap();

        // The real circuit still answers.
        assert!((point.voltage(top).unwrap() - 9.0).abs() < 1e-9);
        // And the stray one sits at its own zero, carrying nothing.
        assert_eq!(point.voltage(orphan), Some(0.0));
    }

    #[test]
    fn an_led_is_a_nonlinear_device_rather_than_an_omission() {
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

        // The LED is not stamped into the linear system — it is handed to
        // Newton, which restamps it about the operating point every iteration.
        let map = netlist.compile().unwrap();
        let system = assemble(&netlist, &map);
        assert!(system.unstamped().is_empty());
        assert_eq!(system.nonlinear().len(), 1);
        assert_eq!(system.nonlinear()[0].element, led);
    }

    /// The milestone: a battery, 220 Ω and a red LED, and the node between.
    fn lit() -> (Netlist, NetId) {
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts: 9.0,
            series_ohms: 0.0,
        });
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: 220.0 });
        let led = netlist.add_element(ElementKind::Led {
            colour: LedColour::Red,
        });
        let (top, middle, bottom) = (netlist.add_net(), netlist.add_net(), netlist.add_net());
        netlist.connect_named(source, "+", top).unwrap();
        netlist.connect_named(resistor, "a", top).unwrap();
        netlist.connect_named(resistor, "b", middle).unwrap();
        netlist.connect_named(led, "anode", middle).unwrap();
        netlist.connect_named(led, "cathode", bottom).unwrap();
        netlist.connect_named(source, "-", bottom).unwrap();
        (netlist, middle)
    }

    #[test]
    fn a_solve_started_from_the_last_answer_finds_the_same_answer_sooner() {
        // The live loop solves the same circuit a thousand times a second;
        // starting each from the last answer is what keeps a CD4511 and its
        // digit inside a frame.
        let (netlist, middle) = lit();
        let cold = operating_point(&netlist).unwrap();
        let warm = operating_point_near(&netlist, &cold).unwrap();

        assert!(
            (warm.voltage(middle).unwrap() - cold.voltage(middle).unwrap()).abs() < 1e-6,
            "{:?} against {:?}",
            warm.voltage(middle),
            cold.voltage(middle)
        );
        assert!(
            warm.iterations() <= 2 && warm.iterations() < cold.iterations(),
            "{} iterations warm, {} cold",
            warm.iterations(),
            cold.iterations()
        );
    }

    #[test]
    fn a_conducting_transistor_started_from_its_own_answer_is_already_there() {
        // 5 V through 1 kΩ into an NPN's base, a load of 116 Ω on its
        // collector: switched hard on, as a kit drives a buzzer. Solved from
        // its own answer it must settle at once. Its junctions used to start
        // their limiting from zero whatever the start said, and walk back up
        // to 0.75 V a few tens of millivolts at a time: fourteen iterations a
        // step, every step, which was most of what a 555 at audio frequency
        // cost (C-94).
        let mut netlist = Netlist::new();
        let supply = netlist.add_element(ElementKind::DcSource {
            volts: 5.0,
            series_ohms: 0.0,
        });
        let base = netlist.add_element(ElementKind::Resistor { ohms: 1e3 });
        let load = netlist.add_element(ElementKind::Resistor { ohms: 116.0 });
        let transistor = netlist.add_element(ElementKind::Bjt {
            polarity: crate::solver::device::Polarity::Npn,
        });
        let (top, drive, collector, bottom) = (
            netlist.add_net(),
            netlist.add_net(),
            netlist.add_net(),
            netlist.add_net(),
        );
        netlist.connect_named(supply, "+", top).unwrap();
        netlist.connect_named(base, "a", top).unwrap();
        netlist.connect_named(base, "b", drive).unwrap();
        netlist.connect_named(transistor, "b", drive).unwrap();
        netlist.connect_named(load, "a", top).unwrap();
        netlist.connect_named(load, "b", collector).unwrap();
        netlist.connect_named(transistor, "c", collector).unwrap();
        netlist.connect_named(transistor, "e", bottom).unwrap();
        netlist.connect_named(supply, "-", bottom).unwrap();

        let cold = operating_point(&netlist).unwrap();
        assert!(cold.voltage(drive).unwrap() > 0.6, "it is on");
        let warm = operating_point_near(&netlist, &cold).unwrap();
        assert!(
            warm.iterations() <= 2,
            "{} iterations from its own answer",
            warm.iterations()
        );
        assert!((warm.voltage(collector).unwrap() - cold.voltage(collector).unwrap()).abs() < 1e-6);
    }

    #[test]
    fn a_start_from_another_circuit_is_a_cold_start() {
        // After an edit the last answer belongs to a circuit that is gone:
        // the wrong size is ignored, and the answer is the cold one.
        let (netlist, middle) = lit();
        let (mut bigger, _) = lit();
        let extra = bigger.add_element(ElementKind::Resistor { ohms: 1e3 });
        let net = bigger.add_net();
        bigger.connect_named(extra, "a", net).unwrap();
        let elsewhere = operating_point(&bigger).unwrap();

        let point = operating_point_near(&netlist, &elsewhere).unwrap();
        let cold = operating_point(&netlist).unwrap();
        assert!((point.voltage(middle).unwrap() - cold.voltage(middle).unwrap()).abs() < 1e-9);
    }
}

#[cfg(test)]
mod contact_tests {
    use super::*;
    use crate::netlist::{ElementId, ElementKind, NetId, Netlist};
    use crate::solver::UnstampedReason;

    const SUPPLY: f64 = 9.0;
    const OHMS: f64 = 220.0;
    const TOLERANCE: f64 = 1e-9;

    struct Lamp {
        netlist: Netlist,
        source: ElementId,
        contact: ElementId,
        /// Between the contact and the resistor: the node that says whether
        /// anything gets past the switch.
        middle: NetId,
    }

    /// A supply, a contact and a resistor in one loop — the circuit a beginner
    /// builds second, once the LED lights and they want to turn it off.
    fn switched(contact: ElementKind) -> Lamp {
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts: SUPPLY,
            series_ohms: 0.0,
        });
        let contact = netlist.add_element(contact);
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: OHMS });

        let top = netlist.add_net();
        let middle = netlist.add_net();
        let bottom = netlist.add_net();
        netlist.connect_named(source, "+", top).unwrap();
        netlist.connect_named(contact, "a", top).unwrap();
        netlist.connect_named(contact, "b", middle).unwrap();
        netlist.connect_named(resistor, "a", middle).unwrap();
        netlist.connect_named(resistor, "b", bottom).unwrap();
        netlist.connect_named(source, "-", bottom).unwrap();

        Lamp {
            netlist,
            source,
            contact,
            middle,
        }
    }

    #[test]
    fn a_closed_switch_is_a_piece_of_wire() {
        // Ideal means ideal: the loop answers exactly as it would with the
        // switch cut out and its ends joined. The whole supply reaches the
        // resistor, with nothing dropped across the contact.
        let lamp = switched(ElementKind::Switch { closed: true });

        let point = operating_point(&lamp.netlist).unwrap();

        assert!((point.voltage(lamp.middle).unwrap() - SUPPLY).abs() < TOLERANCE);
        assert!((point.source_current(lamp.source).unwrap() - SUPPLY / OHMS).abs() < TOLERANCE);
    }

    #[test]
    fn an_open_switch_stops_the_current() {
        let lamp = switched(ElementKind::Switch { closed: false });

        let point = operating_point(&lamp.netlist).unwrap();

        assert!(point.source_current(lamp.source).unwrap().abs() < TOLERANCE);
        // Nothing flows, so nothing is dropped across the resistor and the far
        // side of the gap sits at the reference rather than at the supply.
        assert!(point.voltage(lamp.middle).unwrap().abs() < TOLERANCE);
    }

    #[test]
    fn an_open_contact_is_reported_rather_than_quietly_skipped() {
        // The courtesy a capacitor at DC already gets. A rule (B-05) and a
        // failure message (C-25) both need to tell "this switch is open" from
        // "this switch is not wired up", and only the solver knows which.
        let lamp = switched(ElementKind::Switch { closed: false });
        let map = lamp.netlist.compile().unwrap();

        let system = assemble(&lamp.netlist, &map);

        let left_out: Vec<_> = system
            .unstamped()
            .iter()
            .filter(|entry| entry.element == lamp.contact)
            .collect();
        assert_eq!(left_out.len(), 1);
        assert_eq!(left_out[0].reason, UnstampedReason::ContactOpen);
    }

    #[test]
    fn a_button_held_down_is_the_same_contact_under_another_name() {
        // Said out loud rather than trusted: the two kinds share one stamp
        // today, and the day they stop sharing it this fails.
        let held = operating_point(
            &switched(ElementKind::PushButton {
                closed: true,
                bounces: false,
            })
            .netlist,
        )
        .unwrap();
        let flipped =
            operating_point(&switched(ElementKind::Switch { closed: true }).netlist).unwrap();

        assert_eq!(held.node_voltages(), flipped.node_voltages());
    }

    #[test]
    fn a_closed_contact_reports_the_current_it_passes() {
        // It is stamped as a zero-volt source, so it has a branch unknown of
        // its own — which is how the app draws current running *through* a
        // closed switch rather than stopping either side of it.
        let lamp = switched(ElementKind::Switch { closed: true });

        let point = operating_point(&lamp.netlist).unwrap();

        // `source_current` reports what leaves the first pin, and this loop
        // puts the current *in* at pin `a`, so it reads negative — the same
        // convention a battery's current follows.
        let amps = point.source_current(lamp.contact).unwrap();
        assert!((amps + SUPPLY / OHMS).abs() < TOLERANCE, "{amps} A");
    }

    #[test]
    fn a_contact_closed_across_an_ideal_battery_is_a_short_circuit() {
        // Not a gap in the model: an ideal source and an ideal contact really
        // do describe an unbounded current, and no convergence aid can find a
        // number that is not there. The answer is C-17's — give the cell the
        // milliohms a real one has — and C-25 is what says so in words.
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts: SUPPLY,
            series_ohms: 0.0,
        });
        let contact = netlist.add_element(ElementKind::Switch { closed: true });
        let top = netlist.add_net();
        let bottom = netlist.add_net();
        netlist.connect_named(source, "+", top).unwrap();
        netlist.connect_named(contact, "a", top).unwrap();
        netlist.connect_named(contact, "b", bottom).unwrap();
        netlist.connect_named(source, "-", bottom).unwrap();

        assert!(operating_point(&netlist).is_err());

        netlist
            .set_kind(
                source,
                ElementKind::DcSource {
                    volts: SUPPLY,
                    series_ohms: 0.05,
                },
            )
            .unwrap();
        let point = operating_point(&netlist).unwrap();

        assert!((point.source_current(source).unwrap() - SUPPLY / 0.05).abs() < 1e-6);
    }

    #[test]
    fn an_open_contact_does_not_take_the_rest_of_the_canvas_with_it() {
        // A switch is the one component that routinely leaves part of a circuit
        // with no path to anything, which is the condition that used to make
        // the whole matrix singular. Everything on the live side still answers.
        let lamp = switched(ElementKind::Switch { closed: false });
        let mut netlist = lamp.netlist;
        let orphan = netlist.add_element(ElementKind::Resistor { ohms: OHMS });
        let stub = netlist.add_net();
        netlist.connect_named(orphan, "a", lamp.middle).unwrap();
        netlist.connect_named(orphan, "b", stub).unwrap();

        let point = operating_point(&netlist).unwrap();

        assert!(point.voltage(stub).unwrap().abs() < 1e-6);
        assert!(point.source_current(lamp.source).unwrap().abs() < TOLERANCE);
    }
}

/// The instruments (C-21).
///
/// An ammeter and a voltmeter are the two ideals a measurement rests on: one
/// carries current without dropping any voltage, the other reads a voltage
/// without drawing any current. Both are tested by the same question — does
/// putting the meter in change what it was put in to look at.
#[cfg(test)]
mod probe_tests {
    use super::*;
    use crate::netlist::{ElementId, ElementKind, NetId, Netlist};
    use crate::solver::UnstampedReason;

    const SUPPLY: f64 = 9.0;
    const TOP: f64 = 1_000.0;
    const BOTTOM: f64 = 2_000.0;
    const TOLERANCE: f64 = 1e-9;

    struct Divider {
        netlist: Netlist,
        source: ElementId,
        /// Between the two resistors — the tap a voltmeter would be put across.
        middle: NetId,
        bottom: NetId,
    }

    /// 9 V across 1 kΩ and 2 kΩ: the tap sits at 6 V and 3 mA flows.
    fn divider() -> Divider {
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts: SUPPLY,
            series_ohms: 0.0,
        });
        let upper = netlist.add_element(ElementKind::Resistor { ohms: TOP });
        let lower = netlist.add_element(ElementKind::Resistor { ohms: BOTTOM });

        let top = netlist.add_net();
        let middle = netlist.add_net();
        let bottom = netlist.add_net();
        netlist.connect_named(source, "+", top).unwrap();
        netlist.connect_named(upper, "a", top).unwrap();
        netlist.connect_named(upper, "b", middle).unwrap();
        netlist.connect_named(lower, "a", middle).unwrap();
        netlist.connect_named(lower, "b", bottom).unwrap();
        netlist.connect_named(source, "-", bottom).unwrap();

        Divider {
            netlist,
            source,
            middle,
            bottom,
        }
    }

    #[test]
    fn a_voltmeter_does_not_change_what_it_measures() {
        // The property the whole instrument rests on. An ideal voltmeter is an
        // open circuit, so the divider answers exactly what it answered before
        // — not nearly, exactly, because nothing was stamped.
        let alone = divider();
        let before = operating_point(&alone.netlist).unwrap();

        let mut watched = divider();
        let meter = watched.netlist.add_element(ElementKind::Voltmeter);
        watched
            .netlist
            .connect_named(meter, "+", watched.middle)
            .unwrap();
        watched
            .netlist
            .connect_named(meter, "-", watched.bottom)
            .unwrap();
        let after = operating_point(&watched.netlist).unwrap();

        assert_eq!(before.node_voltages(), after.node_voltages());
        // And the number it is there to read is the one the divider gives.
        let across =
            after.voltage(watched.middle).unwrap() - after.voltage(watched.bottom).unwrap();
        assert!((across - SUPPLY * BOTTOM / (TOP + BOTTOM)).abs() < TOLERANCE);
    }

    #[test]
    fn a_voltmeter_is_left_out_of_the_matrix_and_says_so() {
        // Reported rather than silently skipped, so a rule can tell an
        // instrument that contributes nothing from an element that failed to.
        let mut circuit = divider();
        let meter = circuit.netlist.add_element(ElementKind::Voltmeter);
        circuit
            .netlist
            .connect_named(meter, "+", circuit.middle)
            .unwrap();
        circuit
            .netlist
            .connect_named(meter, "-", circuit.bottom)
            .unwrap();
        let map = circuit.netlist.compile().unwrap();

        let system = assemble(&circuit.netlist, &map);

        let left_out: Vec<_> = system
            .unstamped()
            .iter()
            .filter(|entry| entry.element == meter)
            .collect();
        assert_eq!(left_out.len(), 1);
        assert_eq!(left_out[0].reason, UnstampedReason::DrawsNoCurrent);
    }

    #[test]
    fn an_ammeter_reads_the_current_through_it_and_drops_nothing() {
        // In the branch rather than across it. Its branch unknown is the
        // reading, and the voltage either side of it is the same voltage —
        // which is what makes the reading the current that was already there.
        let mut circuit = divider();
        let meter = circuit.netlist.add_element(ElementKind::Ammeter);
        let cut = circuit.netlist.add_net();
        // Between the lower resistor and the supply's negative terminal.
        let lower = circuit
            .netlist
            .net(circuit.bottom)
            .unwrap()
            .pins()
            .iter()
            .find(|pin| pin.element != circuit.source)
            .copied()
            .unwrap();
        circuit.netlist.disconnect(lower).unwrap();
        circuit.netlist.connect(lower, cut).unwrap();
        circuit.netlist.connect_named(meter, "+", cut).unwrap();
        circuit
            .netlist
            .connect_named(meter, "-", circuit.bottom)
            .unwrap();

        let point = operating_point(&circuit.netlist).unwrap();

        let expected = SUPPLY / (TOP + BOTTOM);
        // Current goes in at `+`, and `source_current` reports what leaves it,
        // so the reading is the negation — the same convention a battery's is.
        assert!((point.source_current(meter).unwrap() + expected).abs() < TOLERANCE);
        // Nothing dropped across it: both its pins sit at the same potential.
        let across = point.voltage(cut).unwrap() - point.voltage(circuit.bottom).unwrap();
        assert!(across.abs() < TOLERANCE, "{across} V across a wire");
    }

    #[test]
    fn an_ammeter_in_the_loop_changes_nothing_else() {
        // The other half of the same property: inserting it leaves the rest of
        // the circuit reading exactly what it read before.
        let plain = divider();
        let before = operating_point(&plain.netlist).unwrap();
        let expected = before.source_current(plain.source).unwrap();

        let mut circuit = divider();
        let meter = circuit.netlist.add_element(ElementKind::Ammeter);
        let cut = circuit.netlist.add_net();
        let lower = circuit
            .netlist
            .net(circuit.bottom)
            .unwrap()
            .pins()
            .iter()
            .find(|pin| pin.element != circuit.source)
            .copied()
            .unwrap();
        circuit.netlist.disconnect(lower).unwrap();
        circuit.netlist.connect(lower, cut).unwrap();
        circuit.netlist.connect_named(meter, "+", cut).unwrap();
        circuit
            .netlist
            .connect_named(meter, "-", circuit.bottom)
            .unwrap();

        let after = operating_point(&circuit.netlist).unwrap();

        assert!((after.source_current(circuit.source).unwrap() - expected).abs() < TOLERANCE);
        assert!(
            (after.voltage(circuit.middle).unwrap() - before.voltage(plain.middle).unwrap()).abs()
                < TOLERANCE
        );
    }

    #[test]
    fn an_ammeter_straight_across_a_battery_is_a_short_circuit() {
        // What a real one does too, which is why it is the mistake every
        // multimeter's manual opens with. Not a modelling gap: an ideal supply
        // and an ideal wire describe an unbounded current, and C-25 is what
        // says so in words a beginner can act on.
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts: SUPPLY,
            series_ohms: 0.0,
        });
        let meter = netlist.add_element(ElementKind::Ammeter);
        let top = netlist.add_net();
        let bottom = netlist.add_net();
        netlist.connect_named(source, "+", top).unwrap();
        netlist.connect_named(meter, "+", top).unwrap();
        netlist.connect_named(meter, "-", bottom).unwrap();
        netlist.connect_named(source, "-", bottom).unwrap();

        assert!(operating_point(&netlist).is_err());
    }
}

/// The potentiometer (C-20): two resistors that always add up to one track.
#[cfg(test)]
mod potentiometer_tests {
    use super::*;
    use crate::netlist::{ElementId, ElementKind, NetId, Netlist};
    use crate::solver::UnstampedReason;

    const SUPPLY: f64 = 9.0;
    const TRACK: f64 = 10_000.0;
    const TOLERANCE: f64 = 1e-9;

    struct Divider {
        netlist: Netlist,
        pot: ElementId,
        /// The wiper's own net, which is the tap being measured.
        tap: NetId,
        bottom: NetId,
    }

    /// A pot straight across the supply, wiper loaded by nothing — the way one
    /// is used as a knob, and the case where the answer is arithmetic.
    fn across_the_supply(wiper: f64) -> Divider {
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts: SUPPLY,
            series_ohms: 0.0,
        });
        let pot = netlist.add_element(ElementKind::Potentiometer { ohms: TRACK, wiper });

        let top = netlist.add_net();
        let tap = netlist.add_net();
        let bottom = netlist.add_net();
        netlist.connect_named(source, "+", top).unwrap();
        netlist.connect_named(pot, "b", top).unwrap();
        netlist.connect_named(pot, "wiper", tap).unwrap();
        netlist.connect_named(pot, "a", bottom).unwrap();
        netlist.connect_named(source, "-", bottom).unwrap();

        Divider {
            netlist,
            pot,
            tap,
            bottom,
        }
    }

    #[test]
    fn the_wiper_taps_the_fraction_of_the_supply_it_sits_at() {
        // `a` is where 0 is, and `a` is on the bottom here, so a wiper at a
        // third of its travel sits at a third of the supply. Unloaded, so the
        // track's own value cannot matter and only the fraction can.
        for fraction in [0.25, 0.5, 0.75] {
            let circuit = across_the_supply(fraction);

            let point = operating_point(&circuit.netlist).unwrap();

            let tapped =
                point.voltage(circuit.tap).unwrap() - point.voltage(circuit.bottom).unwrap();
            let expected = SUPPLY * fraction;
            assert!(
                (tapped - expected).abs() < 1e-6,
                "at {fraction}: {tapped} V against {expected} V"
            );
        }
    }

    #[test]
    fn the_two_halves_always_add_up_to_the_track() {
        // The property that makes it one component rather than two resistors:
        // whatever the wiper does, end to end is unchanged. Measured as the
        // current a bare track draws, which is the supply over its resistance.
        for fraction in [0.0, 0.3, 0.5, 1.0] {
            let circuit = across_the_supply(fraction);

            let point = operating_point(&circuit.netlist).unwrap();

            let source = circuit
                .netlist
                .elements()
                .find(|(_, element)| matches!(element.kind, ElementKind::DcSource { .. }))
                .map(|(id, _)| id)
                .unwrap();
            let amps = point.source_current(source).unwrap();
            assert!(
                (amps - SUPPLY / TRACK).abs() < 1e-9,
                "at {fraction}: {amps} A against {} A",
                SUPPLY / TRACK
            );
        }
    }

    #[test]
    fn a_knob_at_the_stop_is_a_short_and_not_a_singularity() {
        // Turned all the way, one half of the track is nothing at all — which
        // MNA can only say as a voltage source, and which would make the stop
        // behave unlike a hair away from the stop. A real track has a little
        // metal left, and so does this one.
        for fraction in [0.0, 1.0] {
            let circuit = across_the_supply(fraction);

            let point = operating_point(&circuit.netlist)
                .unwrap_or_else(|error| panic!("at {fraction}: {error}"));

            let tapped =
                point.voltage(circuit.tap).unwrap() - point.voltage(circuit.bottom).unwrap();
            let expected = SUPPLY * fraction;
            // Within a millivolt of the rail it is pinned to: the end
            // resistance is a ten-thousandth of the track and shows as one.
            assert!(
                (tapped - expected).abs() < 1e-2,
                "at {fraction}: {tapped} V"
            );
        }
    }

    #[test]
    fn a_wiper_used_as_a_variable_resistor_limits_the_current() {
        // The other way one is used: `a` and the wiper in series with a load,
        // the far end left unwired. Turning it changes what gets through.
        let mut dim = 0.0;
        for fraction in [0.2, 0.8] {
            let mut netlist = Netlist::new();
            let source = netlist.add_element(ElementKind::DcSource {
                volts: SUPPLY,
                series_ohms: 0.0,
            });
            let pot = netlist.add_element(ElementKind::Potentiometer {
                ohms: TRACK,
                wiper: fraction,
            });
            let load = netlist.add_element(ElementKind::Resistor { ohms: 1_000.0 });

            let top = netlist.add_net();
            let middle = netlist.add_net();
            let bottom = netlist.add_net();
            netlist.connect_named(source, "+", top).unwrap();
            netlist.connect_named(pot, "a", top).unwrap();
            netlist.connect_named(pot, "wiper", middle).unwrap();
            netlist.connect_named(load, "a", middle).unwrap();
            netlist.connect_named(load, "b", bottom).unwrap();
            netlist.connect_named(source, "-", bottom).unwrap();

            let point = operating_point(&netlist).unwrap();
            let amps = point.source_current(source).unwrap();

            // Ohm's law over the track behind the wiper plus the load.
            let expected = SUPPLY / (TRACK * fraction + 1_000.0);
            assert!(
                (amps - expected).abs() / expected < 1e-6,
                "at {fraction}: {amps} A against {expected} A"
            );
            if fraction > 0.5 {
                assert!(amps < dim, "turning it up should have let less through");
            }
            dim = amps;
            let _ = pot;
        }
    }

    #[test]
    fn a_pot_wired_by_its_two_ends_is_a_resistor() {
        // Nothing on the wiper, both ends wired: people do use one as a fixed
        // resistor, and the track conducts whether or not anybody is tapping
        // it. Stamping nothing would make a wired component vanish.
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts: SUPPLY,
            series_ohms: 0.0,
        });
        let pot = netlist.add_element(ElementKind::Potentiometer {
            ohms: TRACK,
            wiper: 0.3,
        });

        let top = netlist.add_net();
        let bottom = netlist.add_net();
        netlist.connect_named(source, "+", top).unwrap();
        netlist.connect_named(pot, "a", top).unwrap();
        netlist.connect_named(pot, "b", bottom).unwrap();
        netlist.connect_named(source, "-", bottom).unwrap();

        let point = operating_point(&netlist).unwrap();

        // The whole track, and the wiper's position makes no difference to it.
        let amps = point.source_current(source).unwrap();
        assert!((amps - SUPPLY / TRACK).abs() < TOLERANCE, "{amps} A");
    }

    #[test]
    fn a_pot_joined_by_nothing_but_its_wiper_is_reported() {
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts: SUPPLY,
            series_ohms: 0.0,
        });
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: 1_000.0 });
        let pot = netlist.add_element(ElementKind::Potentiometer {
            ohms: TRACK,
            wiper: 0.5,
        });

        let top = netlist.add_net();
        let bottom = netlist.add_net();
        netlist.connect_named(source, "+", top).unwrap();
        netlist.connect_named(resistor, "a", top).unwrap();
        netlist.connect_named(resistor, "b", bottom).unwrap();
        netlist.connect_named(source, "-", bottom).unwrap();
        netlist.connect_named(pot, "wiper", top).unwrap();

        let map = netlist.compile().unwrap();
        let system = assemble(&netlist, &map);

        let left_out: Vec<_> = system
            .unstamped()
            .iter()
            .filter(|entry| entry.element == pot)
            .collect();
        assert_eq!(left_out.len(), 1);
        assert_eq!(left_out[0].reason, UnstampedReason::PinNotConnected);
        // And the circuit it was dropped into still answers.
        let point = operating_point(&netlist).unwrap();
        assert!((point.source_current(source).unwrap() - SUPPLY / 1_000.0).abs() < TOLERANCE);
    }

    #[test]
    fn a_track_of_nothing_is_reported_rather_than_divided_by() {
        let circuit = across_the_supply(0.5);
        let mut netlist = circuit.netlist;
        netlist
            .set_kind(
                circuit.pot,
                ElementKind::Potentiometer {
                    ohms: 0.0,
                    wiper: 0.5,
                },
            )
            .unwrap();
        let map = netlist.compile().unwrap();

        let system = assemble(&netlist, &map);

        let left_out: Vec<_> = system
            .unstamped()
            .iter()
            .filter(|entry| entry.element == circuit.pot)
            .collect();
        assert_eq!(left_out.len(), 1);
        assert_eq!(left_out[0].reason, UnstampedReason::DegenerateValue);
    }
}

#[cfg(test)]
mod transient_tests {
    use slotmap::SecondaryMap;

    use super::*;
    use crate::netlist::{ElementKind, Netlist};
    use crate::solver::UnstampedReason;

    const SUPPLY: f64 = 9.0;
    const OHMS: f64 = 1_000.0;
    const FARADS: f64 = 1e-6;
    /// τ = RC, the time an RC circuit takes to cover 1 − 1/e of the way there.
    const TAU: f64 = OHMS * FARADS;

    struct Rc {
        netlist: Netlist,
        capacitor: crate::netlist::ElementId,
        across: crate::netlist::NetId,
        bottom: crate::netlist::NetId,
    }

    /// A 9 V source charging a 1 µF capacitor through 1 kΩ.
    fn rc_circuit() -> Rc {
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts: SUPPLY,
            series_ohms: 0.0,
        });
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: OHMS });
        let capacitor = netlist.add_element(ElementKind::Capacitor {
            farads: FARADS,
            initial_volts: 0.0,
        });

        let top = netlist.add_net();
        let middle = netlist.add_net();
        let bottom = netlist.add_net();
        netlist.connect_named(source, "+", top).unwrap();
        netlist.connect_named(resistor, "a", top).unwrap();
        netlist.connect_named(resistor, "b", middle).unwrap();
        netlist.connect_named(capacitor, "a", middle).unwrap();
        netlist.connect_named(capacitor, "b", bottom).unwrap();
        netlist.connect_named(source, "-", bottom).unwrap();

        Rc {
            netlist,
            capacitor,
            across: middle,
            bottom,
        }
    }

    /// Steps the circuit to `seconds` and returns the voltage across the cap.
    ///
    /// The loop belongs to C-09; this is the hand-cranked version that proves
    /// the companion model is right before there is anything to crank it.
    fn charge_to(rc: &Rc, seconds: f64, step_seconds: f64) -> f64 {
        let mut previous: SecondaryMap<crate::netlist::ElementId, crate::solver::Charge> =
            SecondaryMap::new();
        let mut volts = 0.0;
        let steps = (seconds / step_seconds).round() as usize;

        for _ in 0..steps {
            let point = step(
                &rc.netlist,
                &Timestep {
                    seconds: step_seconds,
                    previous: &previous,
                    method: crate::solver::Integration::default(),
                    at: step_seconds,
                },
            )
            .expect("an RC circuit solves");

            let was = previous.get(rc.capacitor).copied().unwrap_or_default();
            volts = point.voltage(rc.across).unwrap() - point.voltage(rc.bottom).unwrap();
            previous.insert(
                rc.capacitor,
                crate::solver::device::Capacitor { farads: FARADS }.advance(
                    crate::solver::Integration::default(),
                    step_seconds,
                    volts,
                    was,
                ),
            );
        }

        volts
    }

    /// The closed form: v(t) = V·(1 − e^(−t/τ)).
    fn analytic(seconds: f64) -> f64 {
        SUPPLY * (1.0 - (-seconds / TAU).exp())
    }

    #[test]
    fn an_rc_circuit_charges_the_way_the_maths_says() {
        // The acceptance of C-06, at the three points that matter: one time
        // constant, two, and settled.
        let rc = rc_circuit();
        let step_seconds = TAU / 1000.0;

        for multiple in [1.0, 2.0, 5.0] {
            let seconds = multiple * TAU;
            let found = charge_to(&rc, seconds, step_seconds);
            let expected = analytic(seconds);

            let error = (found - expected).abs() / expected;
            assert!(
                error < 0.01,
                "at {multiple}τ: {found} V against {expected} V, {:.3}% out",
                error * 100.0
            );
        }
    }

    #[test]
    fn a_capacitor_starts_uncharged() {
        // Nothing in the map means nothing remembered, which is a capacitor
        // that has never been charged rather than one at an unknown voltage.
        let rc = rc_circuit();
        let previous = SecondaryMap::new();

        let point = step(
            &rc.netlist,
            &Timestep {
                seconds: TAU / 1000.0,
                previous: &previous,
                method: crate::solver::Integration::default(),
                at: TAU / 1000.0,
            },
        )
        .unwrap();

        let volts = point.voltage(rc.across).unwrap() - point.voltage(rc.bottom).unwrap();
        assert!(
            volts < SUPPLY / 100.0,
            "{volts} V after one thousandth of τ"
        );
    }

    #[test]
    fn a_capacitor_is_an_open_circuit_at_dc() {
        // Which is what it is once nothing is changing, and the operating point
        // a transient starts from.
        let rc = rc_circuit();

        let point = operating_point(&rc.netlist).unwrap();

        // No current flows, so the resistor drops nothing and the capacitor
        // holds the whole supply.
        let volts = point.voltage(rc.across).unwrap() - point.voltage(rc.bottom).unwrap();
        assert!((volts - SUPPLY).abs() < 1e-9, "{volts} V");
        assert!(
            point
                .unstamped()
                .iter()
                .any(|entry| entry.reason == UnstampedReason::OpenAtDc)
        );
    }

    #[test]
    fn a_smaller_step_tracks_the_curve_more_closely() {
        // Backward Euler's error is first order in the step, so halving the
        // step should roughly halve the error. If it does not, the companion is
        // wrong in a way the 1% test could still pass.
        let rc = rc_circuit();
        let expected = analytic(TAU);

        let coarse = (charge_to(&rc, TAU, TAU / 20.0) - expected).abs();
        let fine = (charge_to(&rc, TAU, TAU / 40.0) - expected).abs();

        assert!(fine < coarse * 0.75, "coarse {coarse}, fine {fine}");
    }

    #[test]
    fn a_capacitor_with_no_capacitance_is_not_stamped() {
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts: SUPPLY,
            series_ohms: 0.0,
        });
        let capacitor = netlist.add_element(ElementKind::Capacitor {
            farads: 0.0,
            initial_volts: 0.0,
        });
        let top = netlist.add_net();
        let bottom = netlist.add_net();
        netlist.connect_named(source, "+", top).unwrap();
        netlist.connect_named(capacitor, "a", top).unwrap();
        netlist.connect_named(capacitor, "b", bottom).unwrap();
        netlist.connect_named(source, "-", bottom).unwrap();

        let previous = SecondaryMap::new();
        let point = step(
            &netlist,
            &Timestep {
                seconds: 1e-6,
                previous: &previous,
                method: crate::solver::Integration::default(),
                at: 1e-6,
            },
        )
        .unwrap();

        assert!(
            point
                .unstamped()
                .iter()
                .any(|entry| entry.element == capacitor
                    && entry.reason == UnstampedReason::DegenerateValue)
        );
    }
}

#[cfg(test)]
mod convergence_tests {
    use super::*;
    use crate::netlist::{ElementKind, LedColour, NetId, Netlist};

    /// `leds` red LEDs in series, fed through `ohms` from `volts`.
    fn chain(ohms: f64, volts: f64, leds: usize) -> (Netlist, Vec<NetId>) {
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts,
            series_ohms: 0.0,
        });
        let resistor = netlist.add_element(ElementKind::Resistor { ohms });
        let top = netlist.add_net();
        let bottom = netlist.add_net();
        netlist.connect_named(source, "+", top).unwrap();
        netlist.connect_named(resistor, "a", top).unwrap();

        let mut node = netlist.add_net();
        let mut nodes = vec![node];
        netlist.connect_named(resistor, "b", node).unwrap();
        for index in 0..leds {
            let led = netlist.add_element(ElementKind::Led {
                colour: LedColour::Red,
            });
            netlist.connect_named(led, "anode", node).unwrap();
            let next = if index + 1 == leds {
                bottom
            } else {
                netlist.add_net()
            };
            netlist.connect_named(led, "cathode", next).unwrap();
            nodes.push(next);
            node = next;
        }
        netlist.connect_named(source, "-", bottom).unwrap();
        (netlist, nodes)
    }

    #[test]
    fn a_chain_of_leds_that_plain_newton_cannot_start_still_solves() {
        // The acceptance of C-11. Eight red LEDs want about 14 V and are given
        // nine, so every junction sits barely conducting and every node is
        // joined to the circuit only through devices that are all but off. The
        // matrix that produces is numerically zero in the node rows — reported
        // as singular, for a circuit with a perfectly good answer.
        let (netlist, nodes) = chain(220.0, 9.0, 8);

        let point = operating_point(&netlist).expect("gmin stepping finds it");

        // Eight identical LEDs in series share the supply evenly.
        let volts: Vec<f64> = nodes
            .iter()
            .map(|net| point.voltage(*net).unwrap())
            .collect();
        for pair in volts.windows(2) {
            let drop = pair[0] - pair[1];
            assert!((drop - 9.0 / 8.0).abs() < 0.01, "{drop} V across one LED");
        }
    }

    #[test]
    fn an_answer_is_not_refused_for_being_enormous() {
        // 9 V across a milliohm and an LED is thousands of amps: absurd to
        // build, and a perfectly well-defined answer. Judging its KCL error
        // against an absolute picoamp made the loop spend its hundred
        // iterations refusing a point it had already found, fifteen digits
        // down. Whether the circuit is a good idea is B-03's question.
        let (netlist, _) = chain(1e-3, 9.0, 1);

        let point = operating_point(&netlist).expect("a huge current is still an answer");

        assert!(point.iterations() > 0);
    }

    #[test]
    fn an_ordinary_circuit_is_unchanged_by_the_aids() {
        // The aids run only after a plain solve has failed, so the everyday
        // answer has to come back bit for bit the same.
        let (netlist, nodes) = chain(220.0, 9.0, 1);

        let point = operating_point(&netlist).unwrap();

        let across = point.voltage(nodes[0]).unwrap() - point.voltage(nodes[1]).unwrap();
        assert!((across - 1.825).abs() < 0.01, "{across} V across the LED");
    }

    #[test]
    fn a_source_straight_across_a_diode_is_still_refused() {
        // Not a convergence failure, and not something an aid can rescue: the
        // model's answer really is an unbounded current, because an ideal
        // source and an ideal diode leave nothing to limit it. C-17's optional
        // series resistance is what gives this circuit a finite answer; B-02
        // is what should catch it before the solver is asked.
        let (netlist, _) = chain(1e-9, 9.0, 1);

        assert!(operating_point(&netlist).is_err());
    }
}
