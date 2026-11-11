// SPDX-License-Identifier: MIT OR Apache-2.0

//! Transient analysis: the circuit over time, one step after another.
//!
//! A DC operating point answers "where does this settle". A transient answers
//! "how does it get there", which is the question a capacitor makes it possible
//! to ask (C-06) and the one an oscilloscope displays (C-22).
//!
//! Fixed steps here. Choosing the length of each one from how fast the circuit
//! is actually moving — and throwing a step away when it moved too far — is
//! C-10, and this loop is what it will take the step size away from.

use slotmap::SecondaryMap;

use crate::netlist::{ElementId, ElementKind, NetId, Netlist, NodeMap};

use super::device::{Capacitor, Inductor};

use super::{Charge, DcError, Integration, NewtonOptions, Timestep, assemble_at, operating_point};

/// Where the first step starts from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InitialCondition {
    /// The DC operating point, where capacitors are open and the circuit has
    /// already settled. What SPICE does unless told otherwise, and the honest
    /// starting point for a circuit that has been powered for a while.
    #[default]
    OperatingPoint,
    /// Every reactive element starts at the initial condition it was given,
    /// which is nothing unless somebody said otherwise (C-18). SPICE spells
    /// this UIC, and it is what shows the charging curve rather than the flat
    /// line after it.
    AsPlaced,
}

/// What to run, and how finely.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransientOptions {
    /// Length of every step, in seconds.
    pub step: f64,
    /// How long to run for, in seconds.
    pub duration: f64,
    pub start: InitialCondition,
    /// Which difference stands in for the derivative — backward Euler unless
    /// asked otherwise (C-08).
    pub integration: Integration,
}

/// Why a transient could not be run.
#[derive(Debug, Clone, PartialEq)]
pub enum TransientError {
    /// A step that is zero, negative or not a number, or a duration that is
    /// not positive. Reported rather than clamped: a caller asking for zero
    /// steps has a bug, and running one step instead would hide it.
    BadSchedule,
    /// A step did not solve. Carries how far in it got, because "it failed at
    /// 3 µs" is a different problem from "it never started".
    Failed { at: f64, error: DcError },
}

impl core::fmt::Display for TransientError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TransientError::BadSchedule => {
                write!(f, "a transient needs a positive step and duration")
            }
            TransientError::Failed { at, error } => write!(f, "at {at} s: {error}"),
        }
    }
}

impl core::error::Error for TransientError {}

/// A circuit's node voltages over time.
///
/// Stored by sample rather than by node, because every step produces one
/// sample of every node at once and that is also the order C-22 draws them in.
#[derive(Debug, Clone)]
pub struct Transient {
    map: NodeMap,
    times: Vec<f64>,
    /// One entry per time, each indexed by node index with the reference first.
    samples: Vec<Vec<f64>>,
    rejections: usize,
    forced: usize,
}

impl Transient {
    /// When each sample was taken, in seconds. The first is one step in, not
    /// zero: every sample is a step that was solved.
    pub fn times(&self) -> &[f64] {
        &self.times
    }

    /// Every node's voltage at one sample, reference first.
    pub fn sample(&self, index: usize) -> Option<&[f64]> {
        self.samples.get(index).map(Vec::as_slice)
    }

    /// One net's voltage over the whole run — the time series C-22 plots and
    /// C-30 prints.
    ///
    /// `None` for a net that carries no pins, which is not a node at all.
    pub fn series(&self, net: NetId) -> Option<Vec<f64>> {
        let index = self.map.index_of(net)? as usize;
        Some(
            self.samples
                .iter()
                .map(|sample| sample.get(index).copied().unwrap_or(0.0))
                .collect(),
        )
    }

    /// How many samples there are — one per step.
    pub fn len(&self) -> usize {
        self.times.len()
    }

    pub fn is_empty(&self) -> bool {
        self.times.is_empty()
    }

    pub fn node_map(&self) -> &NodeMap {
        &self.map
    }

    /// Steps that were tried, found too coarse, and taken again shorter.
    ///
    /// Zero for a fixed-step run, which never rejects anything because it
    /// never looks.
    pub fn rejections(&self) -> usize {
        self.rejections
    }

    /// Steps accepted at the shortest length allowed while still too coarse.
    ///
    /// The alternative is refusing to finish, which for a circuit the user is
    /// watching is worse. Not silent: this is the count that says the answer
    /// is less accurate than it was asked to be.
    pub fn forced(&self) -> usize {
        self.forced
    }
}

/// Runs a transient analysis.
///
/// The netlist does not change while it runs, so it is compiled once and every
/// step reuses the numbering. Each step is the same solve a DC operating point
/// is, with the reactive elements stamped as the companions that step makes of
/// them (C-06).
pub fn transient(
    netlist: &Netlist,
    options: &TransientOptions,
) -> Result<Transient, TransientError> {
    // NaN falls out here through `is_finite`, which is the only reason the
    // check is written as two halves rather than one comparison.
    let unusable = !options.step.is_finite()
        || options.step <= 0.0
        || !options.duration.is_finite()
        || options.duration <= 0.0;
    if unusable {
        return Err(TransientError::BadSchedule);
    }

    let map = netlist.compile().map_err(|error| TransientError::Failed {
        at: 0.0,
        error: DcError::Compile(error),
    })?;

    let mut previous = starting_charge(netlist, options.start)
        .map_err(|error| TransientError::Failed { at: 0.0, error })?;

    // Every sample is a solved step, so the series starts at the first one
    // rather than at zero.
    //
    // A t = 0 sample would have to be invented. The reactive elements' part of
    // it is known — it is the initial condition — but the rest of the circuit
    // is not: getting it honestly means stamping each capacitor as an ideal
    // voltage source at its initial voltage and solving the resistive network
    // around it, which is a second assembly mode this ticket does not need. A
    // first step short enough to plot is short enough to stand in for it.
    let mut times = Vec::new();
    let mut samples = Vec::new();

    let steps = (options.duration / options.step).round() as usize;
    for index in 1..=steps {
        let at = index as f64 * options.step;
        // Fixed steps do not stop at a signal's edges: this run is the
        // textbook one C-14 checks the integration rules against, and a step
        // that moved would be a different run. `adaptive` and the live
        // stepper are the ones that land on edges (ADR 0011).
        let (point, charge) = take_step(
            netlist,
            &map,
            options.integration,
            at - options.step,
            options.step,
            &previous,
            &[],
        )
        .map_err(|error| TransientError::Failed { at, error })?;

        previous = charge;
        times.push(at);
        samples.push(point.node_voltages().to_vec());
    }

    Ok(Transient {
        map,
        times,
        samples,
        rejections: 0,
        forced: 0,
    })
}

/// What to run, and how much error to put up with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdaptiveOptions {
    /// Where to start. Wrong is fine: the first step is checked like every
    /// other, and a step that is too long is rejected rather than believed.
    pub initial_step: f64,
    /// Shortest step to try before giving up on the tolerance and taking the
    /// step anyway. Stops a discontinuity halving the step forever.
    pub min_step: f64,
    /// Longest step, however quiet the circuit goes. A run that grows its step
    /// without limit steps straight over whatever happens next.
    pub max_step: f64,
    pub duration: f64,
    /// Largest local truncation error tolerated at any node, in volts.
    pub tolerance: f64,
    pub start: InitialCondition,
    /// Which difference stands in for the derivative. It is also what the
    /// error estimate below divides by, since a second-order method's two
    /// answers converge four times faster than a first-order one's (C-08).
    pub integration: Integration,
}

impl Default for AdaptiveOptions {
    fn default() -> Self {
        AdaptiveOptions {
            initial_step: 1e-6,
            min_step: 1e-12,
            max_step: 1e-3,
            duration: 1e-3,
            // A millivolt: below what an LED's brightness or a probe's reading
            // can show, and far above the noise of a double.
            tolerance: 1e-3,
            start: InitialCondition::default(),
            integration: Integration::default(),
        }
    }
}

/// How much the step grows when the error is comfortably inside tolerance, and
/// shrinks when it is not.
///
/// Doubling and halving rather than the classic `(tol/err)^(1/order)` scaling:
/// with a first-order method that exponent is 1, which makes the ideal step a
/// straight ratio and turns one bad estimate into a wild overshoot. Powers of
/// two converge on the right size within a couple of steps and never overshoot.
const GROWTH: f64 = 2.0;
const SHRINK: f64 = 0.5;

/// How far inside tolerance an error has to be before the step grows.
///
/// Growing the moment the error fits means growing into a rejection on the
/// very next step, and a run that alternates accept and reject does twice the
/// work for the same answer.
const COMFORTABLE: f64 = 0.25;

/// Runs a transient, choosing each step from how far the circuit actually moves.
///
/// The error estimate is **step doubling**: take the step once at `h`, then
/// twice at `h/2` from the same state, and compare. A first-order method's
/// error halves with the step, so the gap between the two answers is the error
/// in the long one — and the short one is the better answer, so it is the one
/// kept.
///
/// Three solves per accepted step, where SPICE's divided-difference estimate
/// costs none. The trade is deliberate: divided differences need three accepted
/// points of history and a special case at the start of the run and after every
/// change of step, and this is the estimate that is obviously right. Nothing is
/// on this path at frame rate yet (C-09), and C-15's benchmark is where the
/// cost gets an argument rather than an opinion.
pub fn adaptive(netlist: &Netlist, options: &AdaptiveOptions) -> Result<Transient, TransientError> {
    let unusable = !options.initial_step.is_finite()
        || options.initial_step <= 0.0
        || !options.duration.is_finite()
        || options.duration <= 0.0
        || options.min_step <= 0.0
        || options.max_step < options.min_step
        || !options.tolerance.is_finite()
        || options.tolerance <= 0.0;
    if unusable {
        return Err(TransientError::BadSchedule);
    }

    let map = netlist.compile().map_err(|error| TransientError::Failed {
        at: 0.0,
        error: DcError::Compile(error),
    })?;

    let mut previous = starting_charge(netlist, options.start)
        .map_err(|error| TransientError::Failed { at: 0.0, error })?;

    let mut times = Vec::new();
    let mut samples = Vec::new();
    let mut rejections = 0;
    let mut forced = 0;

    let mut at = 0.0;
    let mut step = options.initial_step.min(options.max_step);

    while at < options.duration {
        // Never overshoot the end: the last step is whatever is left.
        step = step.min(options.duration - at).max(options.min_step);
        let shortest = step <= options.min_step;
        let length = up_to_edge(netlist, at, step);

        match measure(
            netlist,
            &map,
            options.integration,
            at,
            length,
            &previous,
            &[],
        ) {
            Ok(attempt) if attempt.error <= options.tolerance || shortest => {
                if attempt.error > options.tolerance {
                    forced += 1;
                }

                at += length;
                previous = attempt.charge;
                times.push(at);
                samples.push(attempt.point.node_voltages().to_vec());

                if attempt.error < options.tolerance * COMFORTABLE {
                    step = (step * GROWTH).min(options.max_step);
                }
            }
            // Too coarse, or it would not solve at all. Both mean the same
            // thing — the step asked the circuit to move too far in one go —
            // and both are answered by asking for less.
            Ok(_) | Err(_) if !shortest => {
                rejections += 1;
                step = (step * SHRINK).max(options.min_step);
            }
            // At the shortest step and still failing to solve. Shrinking
            // further is not allowed and pretending otherwise would loop.
            Err(error) => return Err(TransientError::Failed { at, error }),
            Ok(_) => unreachable!("an accepted attempt is handled above"),
        }
    }

    Ok(Transient {
        map,
        times,
        samples,
        rejections,
        forced,
    })
}

/// Most internal steps one call to [`Stepper::advance`] will take.
///
/// A frame's worth of wall time is a fixed span, and a circuit whose time
/// constant is a thousandth of it would want a thousandth of a step. Somewhere
/// the estimator has to be told it may not have what it is asking for, and this
/// is that place: the shortest step allowed is the span divided by this, which
/// bounds the work per call whatever is on the canvas.
///
/// The cost of the bound is accuracy, not stability — both rules are stable at
/// any step, so an overlong one damps or rings rather than blowing up (C-08) —
/// and it is counted rather than hidden, in the same `forced` the adaptive run
/// reports. C-22's time base is what will let a fast circuit be watched
/// properly, by running it slower than the clock on the wall.
const MOST_STEPS_PER_CALL: usize = 32;

/// A transient that can be advanced a bit at a time (C-48).
///
/// [`transient`] and [`adaptive`] run a circuit from one end to the other and
/// hand back every sample. The live loop cannot work that way: it is asked for
/// a frame's worth of time, sixty times a second, and has to pick up exactly
/// where it left off. What has to survive between calls is the charge in the
/// reactive elements, the step length the estimator had settled on, and the
/// node numbering — so they live here rather than in a loop that ends.
pub struct Stepper {
    map: NodeMap,
    charge: SecondaryMap<ElementId, Charge>,
    method: Integration,
    /// The step the estimator last settled on, carried so the next call starts
    /// where this one left off rather than re-learning it every frame.
    step: f64,
    tolerance: f64,
    max_step: f64,
    rejections: usize,
    forced: usize,
    /// Where the last accepted step's solve ended, to start the next one
    /// from. Empty until a step has been taken, which is a cold start.
    near: Vec<f64>,
    /// Each microcontroller's program, running (ADR 0017).
    mcus: SecondaryMap<ElementId, Mcu>,
    /// The last point solved, which is what a microcontroller's inputs read.
    last: Option<super::OperatingPoint>,
}

/// The longest a microcontroller runs before the circuit catches up with it,
/// in seconds: what its inputs can lag by when nothing it drives changes.
const MCU_QUANTUM: f64 = 1e-3;

/// Two moments this close are the same moment: what a sum of steps that
/// should land on an edge lands within.
const SAME_MOMENT: f64 = 1e-12;

/// A microcontroller running in the run (ADR 0017): its AVR, and where it is
/// against the run's clock.
///
/// The AVR runs ahead of the circuit, never behind it: to the end of the
/// slice the run is about to take, or to the instruction that changes one of
/// its pins, whichever comes first. The circuit is then solved up to that
/// moment with the pins as they were, and only then do they take their new
/// levels — so nothing is rolled back on either side, and a step the solver
/// rejects is retried shorter inside a slice in which nothing the AVR drives
/// changes.
#[derive(Clone)]
pub struct Mcu {
    avr: crate::avr::Avr,
    program: std::sync::Arc<crate::netlist::Program>,
    /// The run's time at the AVR's cycle zero; set by the first slice.
    epoch: Option<f64>,
    /// The run's time the AVR has run to.
    reached: f64,
    /// The pins' drives from `reached` on, when the AVR stopped there
    /// because one changed.
    pending: Option<[super::device::uno::Drive; super::device::uno::IO]>,
    /// What each input last read, for its hysteresis.
    levels: u32,
    /// The serial monitor on its USB port (A-06).
    monitor: crate::serial::Monitor,
}

impl Mcu {
    fn new(program: std::sync::Arc<crate::netlist::Program>) -> Mcu {
        Mcu {
            avr: crate::avr::Avr::atmega328p(program.words.to_vec()),
            program,
            epoch: None,
            reached: f64::NEG_INFINITY,
            pending: None,
            levels: 0,
            monitor: crate::serial::Monitor::default(),
        }
    }

    /// The run's time at a cycle count.
    fn time_of(&self, cycles: u64) -> f64 {
        self.epoch.unwrap_or(0.0) + cycles as f64 / crate::avr::ATMEGA328P_CLOCK_HZ as f64
    }

    /// Every I/O pin's drive, as the AVR has them now. While the USART's
    /// transmitter is on, D1 is its TXD, driven and idling high; while its
    /// receiver is, D0 is its RXD, an input whatever its DDR says, keeping
    /// its pull-up if PORTD0 asks for one (§20.3, §20.6, A-06).
    fn drives(&self) -> [super::device::uno::Drive; super::device::uno::IO] {
        use super::device::uno::{self, Drive};
        let mut drives: [Drive; uno::IO] = std::array::from_fn(|pin| {
            let (port, bit) = uno::port_bit(pin);
            self.avr.io.ports[port].pin_state(&self.avr.cpu, bit).into()
        });
        if let Some(usart) = &self.avr.io.usart {
            if usart.tx_enable(&self.avr.cpu) {
                drives[uno::D1] = Drive::High;
            }
            if usart.rx_enable(&self.avr.cpu) {
                drives[uno::D0] = match drives[uno::D0] {
                    Drive::High | Drive::PullUp => Drive::PullUp,
                    Drive::Low | Drive::Input => Drive::Input,
                };
            }
        }
        drives
    }

    /// Gives the AVR what its inputs read at `point`: each pin's voltage
    /// above the board's GND, through its thresholds. A pin on no net reads
    /// its pull-up, or low through its leak.
    fn feed(&mut self, element: &crate::netlist::Element, point: Option<&super::OperatingPoint>) {
        use super::device::uno::{self, Drive};
        let volts = |pin: usize| {
            let net = element.pins.get(pin)?.net?;
            point?.voltage(net)
        };
        let ground = volts(uno::GND).unwrap_or(0.0);
        let drives = self.drives();
        for (pin, drive) in drives.iter().enumerate() {
            let was = self.levels & (1 << pin) != 0;
            let high = match (drive, volts(pin)) {
                (Drive::High, _) => true,
                (Drive::Low, _) => false,
                (_, Some(at)) => uno::reads_high(at - ground, was),
                (Drive::PullUp, None) => true,
                (_, None) => false,
            };
            if high {
                self.levels |= 1 << pin;
            } else {
                self.levels &= !(1 << pin);
            }
            let (port, bit) = uno::port_bit(pin);
            self.avr.io.set_pin(&mut self.avr.cpu, port, bit, high);
        }
        // What the next conversion of each analog pin samples (A-04).
        if let Some(adc) = &mut self.avr.io.adc {
            adc.avcc = uno::SUPPLY;
            for channel in 0..uno::ANALOG {
                adc.channel_values[channel] =
                    volts(uno::A0 + channel).map_or(0.0, |at| at - ground);
            }
        }
    }

    /// The analog pin the ADC's sample-and-hold is on, as the run carries it:
    /// its channel plus one, or 0 for none (A-04).
    fn sampling(&self) -> f64 {
        use super::device::uno;
        self.avr
            .io
            .adc
            .as_ref()
            .and_then(|adc| adc.selected_channel(&self.avr.cpu))
            .filter(|&channel| (channel as usize) < uno::ANALOG)
            .map_or(0.0, |channel| f64::from(channel) + 1.0)
    }

    /// Types `bytes` into the monitor: sent at its baud from the program's
    /// now, and heard by the chip's receiver at the program's — so a
    /// mismatch is garbage this way too.
    fn send(&mut self, bytes: &[u8]) {
        let hz = crate::avr::ATMEGA328P_CLOCK_HZ as f64;
        let now = self.time_of(self.avr.cpu.cycles);
        let frames = self.monitor.send(now, bytes);
        let Some(usart) = &mut self.avr.io.usart else {
            return;
        };
        let shape = usart.shape(&self.avr.cpu);
        let mut receiver = crate::serial::Receiver::new(
            shape.cycles_per_bit as f64 / hz,
            shape.data_bits,
            shape.parity.is_some(),
        );
        for frame in frames {
            receiver.push(frame);
        }
        for heard in receiver.hear(f64::INFINITY) {
            let cycles = ((heard.at - now) * hz).ceil().max(1.0) as u64;
            usart.receive_at(&mut self.avr.cpu, heard.value, cycles);
        }
    }

    /// The baud the program's serial port runs at, while it is on.
    fn program_baud(&self) -> Option<u32> {
        let usart = self.avr.io.usart.as_ref()?;
        let cpu = &self.avr.cpu;
        (usart.tx_enable(cpu) || usart.rx_enable(cpu)).then(|| usart.baud_rate(cpu) as u32)
    }

    /// Runs the AVR from `now` to `until` at most. If it stopped because a
    /// pin it drives changed, the pins' new drives are pending from where it
    /// stopped.
    fn run(&mut self, now: f64, until: f64) {
        let epoch = *self.epoch.get_or_insert(now);
        let hz = crate::avr::ATMEGA328P_CLOCK_HZ as f64;
        let target = ((until - epoch) * hz).ceil().max(0.0) as u64;
        let changed = self.avr.run_until(target);
        for port in &mut self.avr.io.ports {
            port.take_change();
        }
        self.reached = self.time_of(self.avr.cpu.cycles).max(now);
        // What it sent, on its way to the monitor, which hears what has
        // arrived by now.
        if let Some(adc) = &mut self.avr.io.adc {
            adc.selection_changed = false;
        }
        if let Some(usart) = &mut self.avr.io.usart {
            usart.pins_changed = false;
            let hz = crate::avr::ATMEGA328P_CLOCK_HZ as f64;
            for sent in std::mem::take(&mut usart.transmitted) {
                let start = self.epoch.unwrap_or(0.0) + sent.cycles as f64 / hz;
                self.monitor.listen(crate::serial::Frame::sent(
                    start, sent.value, sent.shape, hz,
                ));
            }
        }
        self.monitor.hear(self.reached);
        if changed {
            self.pending = Some(self.drives());
        }
    }
}

impl Stepper {
    /// Starts a run, with the reactive elements holding what they were placed
    /// holding.
    pub fn start(
        netlist: &Netlist,
        start: InitialCondition,
        integration: Integration,
    ) -> Result<Stepper, DcError> {
        let defaults = AdaptiveOptions::default();
        Ok(Stepper {
            map: netlist.compile()?,
            charge: starting_charge(netlist, start)?,
            method: integration,
            step: defaults.initial_step,
            tolerance: defaults.tolerance,
            max_step: defaults.max_step,
            rejections: 0,
            forced: 0,
            near: Vec::new(),
            mcus: netlist
                .elements()
                .filter(|(_, element)| matches!(element.kind, ElementKind::Uno))
                .filter_map(|(id, _)| Some((id, Mcu::new(netlist.program(id)?.clone()))))
                .collect(),
            last: None,
        })
    }

    /// Advances the run by `seconds`, and reports where it ended up.
    ///
    /// The same accept-or-shorten loop [`adaptive`] runs, bounded twice over: it
    /// stops at `seconds` rather than at a duration, and it will not take more
    /// than [`MOST_STEPS_PER_CALL`] steps to get there.
    ///
    /// Every accepted step is offered to `record`, with how far into this span
    /// it landed. Only the last one is returned — the caller wanted the state —
    /// but a scope wants all of them, and this is the only place they exist
    /// (C-22). A caller with nothing to record passes a closure that drops them.
    ///
    /// `from` is how far into the run this call starts, in seconds, which is
    /// what a signal generator is read against (ADR 0011). The stepper does
    /// not keep its own clock: it is rebuilt whenever the circuit is edited,
    /// and the engine's clock is the one that keeps going.
    pub fn advance(
        &mut self,
        netlist: &Netlist,
        from: f64,
        seconds: f64,
        record: &mut dyn FnMut(f64, &super::OperatingPoint),
    ) -> Result<Option<super::OperatingPoint>, DcError> {
        let shortest = seconds / MOST_STEPS_PER_CALL as f64;
        let mut at = 0.0;
        let mut point = None;

        while at < seconds {
            self.step = self.step.min(seconds - at).max(shortest);
            let floored = self.step <= shortest;
            let mut length = up_to_edge(netlist, from + at, self.step);
            // Each microcontroller that the circuit has caught up with: its
            // pins take the levels it left them at, it reads its inputs, and
            // it runs on to the end of the next slice or to its next edge —
            // which this step does not cross (ADR 0017).
            let now = from + at;
            for (id, mcu) in self.mcus.iter_mut() {
                if mcu.reached <= now + SAME_MOMENT {
                    // What else the Uno carries — its sample-and-hold —
                    // survives its pins changing.
                    let carried = self.charge.get(id).copied().unwrap_or_default();
                    if let Some(drives) = mcu.pending.take() {
                        self.charge.insert(
                            id,
                            Charge {
                                held: carried.held,
                                ..Charge::settled(super::device::uno::encode(&drives))
                            },
                        );
                    }
                    if let Some(element) = netlist.element(id) {
                        mcu.feed(element, self.last.as_ref());
                    }
                    let sampling = mcu.sampling();
                    if sampling != carried.held[1] {
                        // Moved from one pin to another, the sample-and-hold
                        // takes the charge it held there with it, which is
                        // the crosstalk a high-impedance source shows. Put on
                        // a pin for the first time, it starts at that pin's
                        // voltage: what it held before the chip came up is
                        // nobody's, and one step of backward Euler would read
                        // its 35 ns of settling as microseconds (A-04).
                        let first = carried.held[1] == 0.0;
                        let entry = self.charge.entry(id).unwrap().or_default();
                        entry.held[1] = sampling;
                        if first
                            && let Some(pin) = super::device::uno::sampled_pin(sampling)
                            && let Some(element) = netlist.element(id)
                            && let Some(point) = self.last.as_ref()
                        {
                            let volts = |pin: usize| {
                                element
                                    .pins
                                    .get(pin)?
                                    .net
                                    .and_then(|net| point.voltage(net))
                            };
                            if let (Some(on), Some(ground)) =
                                (volts(pin), volts(super::device::uno::GND))
                            {
                                entry.held[0] = on - ground;
                            }
                        }
                    }
                    mcu.run(now, (now + MCU_QUANTUM).min(from + seconds));
                }
                if mcu.reached > now + SAME_MOMENT {
                    length = length.min(mcu.reached - now);
                }
            }

            match measure(
                netlist,
                &self.map,
                self.method,
                from + at,
                length,
                &self.charge,
                &self.near,
            ) {
                Ok(attempt) if attempt.error <= self.tolerance || floored => {
                    if attempt.error > self.tolerance {
                        self.forced += 1;
                    }
                    at += length;
                    self.charge = attempt.charge;
                    self.near.clear();
                    self.near.extend_from_slice(attempt.point.solution());
                    record(at, &attempt.point);
                    self.last = Some(attempt.point.clone());
                    point = Some(attempt.point);

                    if attempt.error < self.tolerance * COMFORTABLE {
                        self.step = (self.step * GROWTH).min(self.max_step);
                    }
                }
                // Too coarse, or it would not solve at all: both mean the step
                // asked the circuit to move too far in one go.
                Ok(_) | Err(_) if !floored => {
                    self.rejections += 1;
                    self.step = (self.step * SHRINK).max(shortest);
                }
                // At the shortest step allowed and still failing to solve.
                // Shrinking further is not permitted and pretending otherwise
                // would loop for ever.
                Err(error) => return Err(error),
                Ok(_) => unreachable!("an accepted attempt is handled above"),
            }
        }

        // `None` when `seconds` was zero or less: nothing was asked of the
        // circuit, so the caller keeps the state it already had.
        Ok(point)
    }

    /// Steps that were tried, found too coarse and taken again shorter.
    pub fn rejections(&self) -> usize {
        self.rejections
    }

    /// Steps taken at the shortest length allowed while still too coarse — the
    /// count that says the answer is less accurate than it was asked to be.
    pub fn forced(&self) -> usize {
        self.forced
    }

    /// What an element is carrying into the next step — a capacitor's charge,
    /// a 555's or a relay's bit — if anything.
    pub fn carried(&self, id: ElementId) -> Option<Charge> {
        self.charge.get(id).copied()
    }

    /// What the reactive elements are holding, for a run that is about to be
    /// rebuilt around an edited circuit.
    pub fn into_charge(self) -> SecondaryMap<ElementId, Charge> {
        self.charge
    }

    /// Puts back what a previous run was holding.
    ///
    /// Overlaid on what the elements were placed holding rather than replacing
    /// it, so a component added since keeps its own initial condition and one
    /// that was already there keeps what it has since accumulated (C-48).
    pub fn resume(&mut self, carried: SecondaryMap<ElementId, Charge>) {
        for (id, value) in carried {
            self.charge.insert(id, value);
        }
    }

    /// What a microcontroller's serial monitor has heard since this was last
    /// asked (A-06).
    pub fn take_serial(&mut self, id: ElementId) -> Vec<u8> {
        self.mcus
            .get_mut(id)
            .map(|mcu| mcu.monitor.take())
            .unwrap_or_default()
    }

    /// Types `bytes` into a microcontroller's serial monitor.
    pub fn send_serial(&mut self, id: ElementId, bytes: &[u8]) {
        if let Some(mcu) = self.mcus.get_mut(id) {
            mcu.send(bytes);
        }
    }

    /// Sets the baud a microcontroller's serial monitor listens and sends at.
    pub fn set_monitor_baud(&mut self, id: ElementId, baud: u32) {
        if let Some(mcu) = self.mcus.get_mut(id) {
            mcu.monitor.set_baud(baud);
        }
    }

    /// The monitor's baud, and the program's while its serial port is on.
    pub fn serial_bauds(&self, id: ElementId) -> Option<(u32, Option<u32>)> {
        let mcu = self.mcus.get(id)?;
        Some((mcu.monitor.baud(), mcu.program_baud()))
    }

    /// The microcontrollers, running, for a run about to be rebuilt around
    /// an edited circuit, with the last point their inputs read.
    pub fn take_mcus(&mut self) -> (SecondaryMap<ElementId, Mcu>, Option<super::OperatingPoint>) {
        (core::mem::take(&mut self.mcus), self.last.take())
    }

    /// Puts back the microcontrollers a previous run was running: each still
    /// an Uno with the same program carries on where it was; one whose
    /// program changed starts from reset (ADR 0017).
    pub fn resume_mcus(
        &mut self,
        carried: SecondaryMap<ElementId, Mcu>,
        last: Option<super::OperatingPoint>,
    ) {
        for (id, mcu) in carried {
            let same = self
                .mcus
                .get(id)
                .is_some_and(|fresh| std::sync::Arc::ptr_eq(&fresh.program, &mcu.program));
            if same {
                self.mcus.insert(id, mcu);
            } else if let Some(fresh) = self.mcus.get_mut(id) {
                // The monitor is the board's USB port, not the program's:
                // it keeps its baud and what it has not shown.
                fresh.monitor = mcu.monitor;
            }
        }
        self.last = last;
    }
}

/// The step to take from `now`: `step`, or less if a signal's edge comes
/// first (ADR 0011, term 5).
///
/// A step that straddles a square wave's jump smears it across the step and
/// leaves the error estimate to notice afterwards; one that ends on it takes
/// the jump cleanly between two steps. Clipping never lengthens a step, so
/// nothing the estimator decided is undone — the next step picks up the
/// length it had settled on.
fn up_to_edge(netlist: &Netlist, now: f64, step: f64) -> f64 {
    match super::device::signal::next_breakpoint(netlist, now) {
        Some(edge) if edge - now < step => edge - now,
        _ => step,
    }
}

/// One step taken twice: once whole, once in halves.
struct Attempt {
    /// Largest disagreement between the two, in volts, at any node.
    error: f64,
    /// The half-step answer, which is the better of the two.
    point: super::OperatingPoint,
    charge: SecondaryMap<ElementId, Charge>,
}

fn measure(
    netlist: &Netlist,
    map: &NodeMap,
    method: Integration,
    from: f64,
    step: f64,
    previous: &SecondaryMap<ElementId, Charge>,
    near: &[f64],
) -> Result<Attempt, DcError> {
    let half = step / 2.0;
    let (whole, _) = take_step(netlist, map, method, from, step, previous, near)?;
    let (first, midway) = take_step(netlist, map, method, from, half, previous, near)?;
    let (halves, charge) = take_step(
        netlist,
        map,
        method,
        from + half,
        half,
        &midway,
        first.solution(),
    )?;

    let gap = whole
        .node_voltages()
        .iter()
        .zip(halves.node_voltages())
        .map(|(one, two)| (one - two).abs())
        .fold(0.0, f64::max);

    // The gap between the two answers is not the error in the better of them:
    // halving the step divides the error by 2^order, so the gap is 2^order − 1
    // times the error that is left. One for backward Euler, which is why this
    // divisor only appears now — three for trapezoidal, and treating its gap as
    // the error would make it ask for steps three times shorter than the
    // tolerance wants.
    //
    // The first step of a run is the one exception, and it is one the other
    // way: every sub-step with nothing behind it is taken by backward Euler
    // whatever was asked for, so the gap is a first-order one divided by a
    // second-order divisor, and the estimate reads low. It is one step, and
    // the next one is measured properly.
    let error = gap / f64::from((1u32 << method.order()) - 1);

    Ok(Attempt {
        error,
        point: halves,
        charge,
    })
}

/// Takes one step of `seconds` from `previous`, without disturbing it.
///
/// Returns the solved point and the charge the step leaves behind. Branching
/// rather than advancing is what lets C-10 try the same step twice at two
/// different lengths and keep only one of the answers.
fn take_step(
    netlist: &Netlist,
    map: &NodeMap,
    method: Integration,
    from: f64,
    seconds: f64,
    previous: &SecondaryMap<ElementId, Charge>,
    near: &[f64],
) -> Result<(super::OperatingPoint, SecondaryMap<ElementId, Charge>), DcError> {
    let system = assemble_at(
        netlist,
        map,
        Some(&Timestep {
            seconds,
            previous,
            method,
            at: from + seconds,
        }),
    );
    // Started where the last solve of this run ended, when there was one:
    // a step is a small move, and Newton from nothing is a dozen iterations
    // spent walking every junction up from zero again.
    let point = super::dc::solve_system_from(
        netlist,
        map.clone(),
        &system,
        &NewtonOptions::default(),
        near,
    )?;

    let mut charge = previous.clone();
    remember_charge(netlist, &point, Some((method, seconds)), &mut charge);
    Ok((point, charge))
}

/// The voltage each reactive element starts the run holding.
fn starting_charge(
    netlist: &Netlist,
    start: InitialCondition,
) -> Result<SecondaryMap<ElementId, Charge>, DcError> {
    let mut charge = SecondaryMap::new();
    if start == InitialCondition::AsPlaced {
        for (id, element) in reactive(netlist) {
            let stated = match element.kind {
                ElementKind::Capacitor { initial_volts, .. }
                | ElementKind::Electrolytic { initial_volts, .. } => initial_volts,
                ElementKind::Inductor { initial_amps, .. } => initial_amps,
                _ => 0.0,
            };
            if stated != 0.0 {
                charge.insert(id, Charge::placed(stated));
            }
        }
        return Ok(charge);
    }

    let point = operating_point(netlist)?;
    remember_charge(netlist, &point, None, &mut charge);
    Ok(charge)
}

/// Records what this step left every reactive element carrying.
///
/// A capacitor's value is the voltage the step ended at, which the solved point
/// gives directly; what has to be worked out is how fast it was changing. An
/// inductor is the mirror: the rate is `v/L` and there for the reading, and the
/// current it adds up to is what the step has to be integrated for. Each device
/// does its own half, because which way round the integration rule is read is
/// the difference between them (C-08).
///
/// `at` is the method and the length of the step just taken, or `None` for a
/// DC operating point, which is not a step and where nothing is moving.
fn remember_charge(
    netlist: &Netlist,
    point: &super::OperatingPoint,
    at: Option<(Integration, f64)>,
    previous: &mut SecondaryMap<ElementId, Charge>,
) {
    // A 555's latch, read from where this step left its pins (ADR 0012). It
    // is carried in the same map as charge, as a bit beside it, so it rides through a
    // rejected step and an edit exactly as a capacitor's voltage does.
    // A relay's contact, likewise, from the current in its coil (C-64) — and
    // that current is carried too, as an inductor's is, because the coil is
    // one (C-67).
    for (id, element) in netlist.elements() {
        let ElementKind::Relay { part } = element.kind else {
            continue;
        };
        // Only the coil decides, so only the coil has to be wired.
        let [Some(plus), Some(minus)] = [
            super::device::relay::COIL_PLUS,
            super::device::relay::COIL_MINUS,
        ]
        .map(|pin| element.pins[pin].net.and_then(|net| point.voltage(net))) else {
            continue;
        };
        let was = previous.get(id).copied().unwrap_or_default();
        let mut carried = part.advance(at, plus - minus, was);
        carried.latched = super::device::relay::latch(part, was.latched, carried.value);
        previous.insert(id, carried);
    }
    // A transformer's two winding currents, and how fast each was changing
    // (C-118): what its companion needs from one step to build the next. A
    // winding off a net is taken as carrying nothing across it.
    for (id, element) in netlist.elements() {
        let ElementKind::Transformer { ratio } = element.kind else {
            continue;
        };
        use super::device::transformer::{P1, P2, S1, S2, Transformer};
        let volts = |plus: usize, minus: usize| {
            let at = |pin: usize| element.pins[pin].net.and_then(|net| point.voltage(net));
            at(plus)
                .zip(at(minus))
                .map_or(0.0, |(plus, minus)| plus - minus)
        };
        let was = previous.get(id).copied().unwrap_or_default();
        let carried = Transformer { ratio }.advance(at, [volts(P1, P2), volts(S1, S2)], was);
        previous.insert(id, carried);
    }
    // The voltage on an Uno's sample-and-hold, from the analog pin it is on
    // (A-04): what its companion needs to build the next step. A DC point
    // has nothing to say about it.
    if at.is_some() {
        use super::device::uno;
        for (id, element) in netlist.elements() {
            if !matches!(element.kind, ElementKind::Uno) {
                continue;
            }
            let Some(carried) = previous.get_mut(id) else {
                continue;
            };
            let Some(pin) = uno::sampled_pin(carried.held[1]) else {
                continue;
            };
            let volts = |pin: usize| element.pins[pin].net.and_then(|net| point.voltage(net));
            if let (Some(on), Some(ground)) = (volts(pin), volts(uno::GND)) {
                carried.held[0] = on - ground;
            }
        }
    }
    // What a battery has given this step, at the current it gave (C-121).
    for (id, element) in netlist.elements() {
        let ElementKind::Cell { part, charge } = element.kind else {
            continue;
        };
        use super::device::cell::State;
        let sheet = part.sheet();
        let volts = |pin: usize| element.pins[pin].net.and_then(|net| point.voltage(net));
        let was = previous
            .get(id)
            .map_or_else(State::default, |&carried| State::decode(carried));
        let Some(across) = volts(0).zip(volts(1)).map(|(a, b)| a - b) else {
            previous.insert(id, was.encode());
            continue;
        };
        let amps = (sheet.open_volts(was.depth(charge)) - across) / sheet.series_ohms();
        let seconds = at.map_or(0.0, |(_, seconds)| seconds);
        previous.insert(id, was.advance(&sheet, amps, seconds).encode());
    }
    // A fuse's heat, at the power this step had it taking (C-120); a blown
    // one is open whatever it is carrying.
    for (id, element) in netlist.elements() {
        let ElementKind::Fuse { amps, part, blown } = element.kind else {
            continue;
        };
        use super::device::fuse::{self, State};
        let Some(model) = fuse::model(part, amps) else {
            continue;
        };
        let volts = |pin: usize| element.pins[pin].net.and_then(|net| point.voltage(net));
        let across = volts(0).zip(volts(1)).map_or(0.0, |(a, b)| a - b);
        let was = previous
            .get(id)
            .map_or_else(State::default, |&carried| State::decode(carried));
        let seconds = at.map_or(0.0, |(_, seconds)| seconds);
        let mut now = model.advance(was, if blown { 0.0 } else { across }, seconds);
        now.open |= blown;
        previous.insert(id, now.encode());
    }
    // An LM3914's lit LEDs and their current (C-119), read from where this
    // step left its pins.
    for (id, element) in netlist.elements() {
        if !matches!(element.kind, ElementKind::Lm3914) {
            continue;
        }
        let mut volts = [None; 18];
        for (slot, pin) in volts.iter_mut().zip(&element.pins) {
            *slot = pin.net.and_then(|net| point.voltage(net));
        }
        previous.insert(id, super::device::lm3914::read(&volts).encode());
    }
    // A CD4511's latch and its two controls, likewise (C-77): read at half
    // its supply, and only when it has one. An input left free reads low.
    for (id, element) in netlist.elements() {
        if !matches!(element.kind, ElementKind::Decoder) {
            continue;
        }
        use super::device::decoder;
        let volts = |pin: usize| {
            element
                .pins
                .get(pin)?
                .net
                .and_then(|net| point.voltage(net))
        };
        if volts(decoder::VDD).is_none() || volts(decoder::VSS).is_none() {
            continue;
        }
        let low = volts(decoder::VSS).unwrap_or(0.0);
        let mut potentials = [low; 16];
        for (pin, slot) in potentials.iter_mut().enumerate() {
            if let Some(at) = volts(pin) {
                *slot = at;
            }
        }
        let was = decoder::State::decode(previous.get(id).map_or(0.0, |carried| carried.value));
        let now = decoder::latch(was, &potentials);
        previous.insert(id, Charge::settled(now.encode()));
    }
    // A CD4026's count and its clock's last level (C-78): an edge is the
    // level now against the level then, so both ride between steps.
    for (id, element) in netlist.elements() {
        if !matches!(element.kind, ElementKind::Counter) {
            continue;
        }
        use super::device::counter;
        let volts = |pin: usize| {
            element
                .pins
                .get(pin)?
                .net
                .and_then(|net| point.voltage(net))
        };
        let (Some(high), Some(low)) = (volts(counter::VDD), volts(counter::VSS)) else {
            continue;
        };
        let mut potentials = [low; 14];
        for (pin, slot) in potentials.iter_mut().enumerate() {
            if let Some(at) = volts(pin) {
                *slot = at;
            }
        }
        // Nothing carried yet is the first reading, which sees a level and
        // not an edge.
        let was = match previous.get(id) {
            Some(carried) => counter::State::decode(carried.value),
            None => counter::first(&potentials),
        };
        // Its supply beside its count, for its outputs' strength (C-125).
        previous.insert(
            id,
            Charge {
                held: [high - low, 0.0],
                ..Charge::settled(counter::advance(was, &potentials).encode())
            },
        );
    }
    // A 74HC595's two registers and two clocks' last levels (C-93), as the
    // CD4026's count and clock are.
    for (id, element) in netlist.elements() {
        if !matches!(element.kind, ElementKind::ShiftRegister) {
            continue;
        }
        use super::device::shift;
        let volts = |pin: usize| element.pins[pin].net.and_then(|net| point.voltage(net));
        let (Some(_), Some(low)) = (volts(shift::VCC), volts(shift::GND)) else {
            continue;
        };
        let mut potentials = [low; 16];
        for (pin, slot) in potentials.iter_mut().enumerate() {
            if let Some(at) = volts(pin) {
                *slot = at;
            }
        }
        let was = match previous.get(id) {
            Some(carried) => shift::State::decode(carried.value),
            None => shift::first(&potentials),
        };
        previous.insert(
            id,
            Charge::settled(shift::advance(was, &potentials).encode()),
        );
    }
    // What each gate's output should be (C-108), read as the 74HC595's
    // inputs are; nothing else is carried, so the first reading is as good
    // as any. An input wired to an earlier gate of the same chip reads that
    // gate's new output, so a latch settles (`gate::outputs`).
    for (id, element) in netlist.elements() {
        use super::device::gate;
        let Some(chip) = gate::Chip::of(&element.kind) else {
            continue;
        };
        let volts = |pin: usize| element.pins[pin].net.and_then(|net| point.voltage(net));
        let (Some(_), Some(low)) = (volts(gate::VCC), volts(gate::GND)) else {
            continue;
        };
        let mut potentials = [low; 14];
        for (pin, slot) in potentials.iter_mut().enumerate() {
            if let Some(at) = volts(pin) {
                *slot = at;
            }
        }
        let joined = |a: usize, b: usize| {
            element.pins[a].net.is_some() && element.pins[a].net == element.pins[b].net
        };
        // A 74HC14 reads each input against which way it last went.
        let was = previous
            .get(id)
            .map_or(gate::resting(chip), |carried| gate::decode(carried.value));
        previous.insert(
            id,
            Charge::settled(gate::encode(gate::outputs(chip, &potentials, &joined, was))),
        );
    }
    // A CD4017's count and its clock's last level (C-110), as the CD4026's.
    for (id, element) in netlist.elements() {
        if !matches!(element.kind, ElementKind::Decade) {
            continue;
        }
        use super::device::decade;
        let volts = |pin: usize| element.pins[pin].net.and_then(|net| point.voltage(net));
        let (Some(high), Some(low)) = (volts(decade::VDD), volts(decade::VSS)) else {
            continue;
        };
        let mut potentials = [low; 16];
        for (pin, slot) in potentials.iter_mut().enumerate() {
            if let Some(at) = volts(pin) {
                *slot = at;
            }
        }
        let was = match previous.get(id) {
            Some(carried) => decade::State::decode(carried.value),
            None => decade::first(&potentials),
        };
        // Its supply beside its count, for its outputs' strength (C-124).
        previous.insert(
            id,
            Charge {
                held: [high - low, 0.0],
                ..Charge::settled(decade::advance(was, &potentials).encode())
            },
        );
    }
    // A CD4013's two flip-flops and their clocks' last levels (C-111).
    for (id, element) in netlist.elements() {
        if !matches!(element.kind, ElementKind::FlipFlop) {
            continue;
        }
        use super::device::flipflop;
        let volts = |pin: usize| element.pins[pin].net.and_then(|net| point.voltage(net));
        let (Some(high), Some(low)) = (volts(flipflop::VDD), volts(flipflop::VSS)) else {
            continue;
        };
        let mut potentials = [low; 14];
        for (pin, slot) in potentials.iter_mut().enumerate() {
            if let Some(at) = volts(pin) {
                *slot = at;
            }
        }
        let was = match previous.get(id) {
            Some(carried) => flipflop::decode(carried.value),
            None => flipflop::first(&potentials),
        };
        // Its supply beside what it holds, for its outputs' strength (C-124).
        previous.insert(
            id,
            Charge {
                held: [high - low, 0.0],
                ..Charge::settled(flipflop::encode(flipflop::advance(was, &potentials)))
            },
        );
    }
    // Whether an A3144 is on (C-115): its field against its two points, as
    // it was in between, and off without its supply.
    for (id, element) in netlist.elements() {
        let ElementKind::Hall { magnet } = element.kind else {
            continue;
        };
        use super::device::magnet as hall;
        let volts = |pin: usize| element.pins[pin].net.and_then(|net| point.voltage(net));
        let (Some(high), Some(low)) = (volts(hall::VCC), volts(hall::GND)) else {
            continue;
        };
        let was = previous.get(id).is_some_and(|carried| carried.value > 0.5);
        let on = hall::hall_on(was, magnet, high - low);
        previous.insert(id, Charge::settled(f64::from(u8::from(on))));
    }
    // An HC-SR501's warm-up, delay and block (C-116), carried as the servo's
    // pulse is, and moved on by the step's length.
    for (id, element) in netlist.elements() {
        let ElementKind::Pir {
            moving,
            hold,
            retriggers,
        } = element.kind
        else {
            continue;
        };
        use super::device::pir;
        let was = pir::State::decode(previous.get(id).copied());
        let Some((_, seconds)) = at else {
            previous.insert(id, was.encode());
            continue;
        };
        let volts = |pin: usize| element.pins[pin].net.and_then(|net| point.voltage(net));
        let supply = volts(pir::VCC)
            .zip(volts(pir::GND))
            .map_or(0.0, |(high, low)| high - low);
        previous.insert(
            id,
            was.advance(moving, hold, retriggers, supply, seconds)
                .encode(),
        );
    }
    // An L293D's inputs, enables and logic supply (C-91): read against its
    // GND, and only when it has one. A pin left free reads low.
    for (id, element) in netlist.elements() {
        if !matches!(element.kind, ElementKind::MotorDriver) {
            continue;
        }
        use super::device::driver;
        let volts = |pin: usize| element.pins[pin].net.and_then(|net| point.voltage(net));
        let Some(ground) = volts(driver::GND) else {
            continue;
        };
        let mut potentials = [ground; 13];
        for (pin, slot) in potentials.iter_mut().enumerate() {
            if let Some(at) = volts(pin) {
                *slot = at;
            }
        }
        previous.insert(id, Charge::settled(driver::read(&potentials).encode()));
    }
    // A servo's horn, and the pulse it is reading (C-90): moved by every step
    // the run takes, so it turns at its own speed whatever the step, and only
    // during a step — at DC nothing moves.
    for (id, element) in netlist.elements() {
        if !matches!(element.kind, ElementKind::Servo) {
            continue;
        }
        use super::device::servo;
        let volts = |pin: usize| element.pins[pin].net.and_then(|net| point.voltage(net));
        let Some(ground) = volts(servo::GROUND) else {
            continue;
        };
        let was = servo::State::decode(previous.get(id).copied());
        let Some((_, seconds)) = at else {
            previous.insert(id, was.encode());
            continue;
        };
        let signal = volts(servo::SIGNAL).map_or(0.0, |volts| volts - ground);
        let supply = volts(servo::SUPPLY).map_or(0.0, |volts| volts - ground);
        previous.insert(id, was.advance(signal, supply, seconds).encode());
    }
    for (id, element) in netlist.elements() {
        if !matches!(element.kind, ElementKind::Timer) {
            continue;
        }
        let mut potentials = [0.0; 6];
        let mut wired = true;
        for (slot, pin) in potentials.iter_mut().zip(&element.pins) {
            match pin.net.and_then(|net| point.voltage(net)) {
                Some(volts) => *slot = volts,
                None => wired = false,
            }
        }
        if !wired {
            continue;
        }
        let was = previous.get(id).is_some_and(|carried| carried.latched);
        let now = super::device::timer::latch(was, &potentials);
        previous.insert(id, Charge::bit(now));
    }

    for (id, element) in reactive(netlist) {
        let Some(volts) = across(netlist, point, id) else {
            continue;
        };
        let was = previous.get(id).copied().unwrap_or_default();

        let carried = match (element.kind, at) {
            (
                ElementKind::Capacitor { farads, .. } | ElementKind::Electrolytic { farads, .. },
                Some((method, seconds)),
            ) => Capacitor { farads }.advance(method, seconds, volts, was),
            (ElementKind::Inductor { henries, .. }, Some((method, seconds))) => {
                Inductor { henries }.advance(method, seconds, volts, was)
            }
            // At DC an inductor is a wire, so its current is whatever the rest
            // of the circuit puts through it — the branch unknown the
            // zero-volt source left behind. Negated, because `source_current`
            // reports what a supply *delivers*, out of its `+` terminal, and
            // an inductor's companion wants what flows *into* its first pin.
            (ElementKind::Inductor { .. }, None) => {
                Charge::settled(-point.source_current(id).unwrap_or(0.0))
            }
            // A settled capacitor holds the voltage it is under and passes
            // nothing, so there is no rate to carry either.
            _ => Charge::settled(volts),
        };
        previous.insert(id, carried);
    }
}

/// Every element that carries state from one step to the next.
fn reactive(netlist: &Netlist) -> impl Iterator<Item = (ElementId, &crate::netlist::Element)> {
    netlist.elements().filter(|(_, element)| {
        matches!(
            element.kind,
            ElementKind::Capacitor { .. }
                | ElementKind::Electrolytic { .. }
                | ElementKind::Inductor { .. }
        )
    })
}

/// Voltage across a two-terminal element, first pin minus second.
fn across(netlist: &Netlist, point: &super::OperatingPoint, id: ElementId) -> Option<f64> {
    let element = netlist.element(id)?;
    let plus = point.voltage(element.pins.first()?.net?)?;
    let minus = point.voltage(element.pins.get(1)?.net?)?;
    Some(plus - minus)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::netlist::ElementKind;

    const SUPPLY: f64 = 9.0;
    const OHMS: f64 = 1_000.0;
    const FARADS: f64 = 1e-6;
    /// τ = RC: the time an RC circuit takes to cover 1 − 1/e of the way there.
    pub(super) const TAU: f64 = OHMS * FARADS;

    pub(super) struct Rc {
        pub netlist: Netlist,
        pub across: NetId,
        pub bottom: NetId,
    }

    /// A 9 V source charging a 1 µF capacitor through 1 kΩ.
    pub(super) fn rc_circuit() -> Rc {
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
            across: middle,
            bottom,
        }
    }

    fn charging(duration: f64, step: f64) -> (Rc, Transient) {
        let rc = rc_circuit();
        let run = transient(
            &rc.netlist,
            &TransientOptions {
                step,
                duration,
                start: InitialCondition::AsPlaced,
                integration: Integration::default(),
            },
        )
        .expect("an RC circuit runs");
        (rc, run)
    }

    /// Voltage across the capacitor at each sample.
    fn across_capacitor(rc: &Rc, run: &Transient) -> Vec<f64> {
        let high = run.series(rc.across).unwrap();
        let low = run.series(rc.bottom).unwrap();
        high.iter().zip(low).map(|(a, b)| a - b).collect()
    }

    #[test]
    fn an_rc_circuit_charges_the_way_the_maths_says() {
        // The same curve C-06's companion was checked against, now produced by
        // the loop rather than by a test cranking it by hand.
        let step = TAU / 1000.0;
        let (rc, run) = charging(5.0 * TAU, step);
        let volts = across_capacitor(&rc, &run);

        for multiple in [1.0, 2.0, 5.0] {
            let index = (multiple * TAU / step).round() as usize - 1;
            let found = volts[index];
            let expected = SUPPLY * (1.0 - (-multiple).exp());

            let error = (found - expected).abs() / expected;
            assert!(
                error < 0.01,
                "at {multiple}τ: {found} V against {expected} V"
            );
        }
    }

    #[test]
    fn it_reports_one_sample_per_step() {
        let (_, run) = charging(10.0 * TAU, TAU);

        assert_eq!(run.len(), 10);
        assert_eq!(run.times().len(), run.len());
    }

    #[test]
    fn the_times_are_the_steps() {
        let (_, run) = charging(3.0 * TAU, TAU);

        for (index, at) in run.times().iter().enumerate() {
            assert!((at - (index + 1) as f64 * TAU).abs() < 1e-15, "{at}");
        }
    }

    #[test]
    fn every_sample_covers_every_node() {
        let (_, run) = charging(3.0 * TAU, TAU);
        let nodes = run.node_map().len();

        for index in 0..run.len() {
            assert_eq!(run.sample(index).unwrap().len(), nodes);
        }
    }

    #[test]
    fn a_series_is_as_long_as_the_run() {
        let (rc, run) = charging(3.0 * TAU, TAU);

        assert_eq!(run.series(rc.across).unwrap().len(), run.len());
    }

    #[test]
    fn starting_from_the_operating_point_starts_settled() {
        // A circuit that has been powered for a while is not charging: the
        // capacitor is already full and the run is a flat line. SPICE's default
        // and, for once, the boring answer is the right one.
        let rc = rc_circuit();
        let run = transient(
            &rc.netlist,
            &TransientOptions {
                step: TAU / 10.0,
                duration: 2.0 * TAU,
                start: InitialCondition::OperatingPoint,
                integration: Integration::default(),
            },
        )
        .unwrap();

        for volts in across_capacitor(&rc, &run) {
            assert!((volts - SUPPLY).abs() < 1e-6, "{volts} V");
        }
    }

    #[test]
    fn starting_empty_starts_at_nothing() {
        let step = TAU / 1000.0;
        let (rc, run) = charging(TAU, step);

        // One thousandth of τ in, the capacitor has barely moved.
        assert!(across_capacitor(&rc, &run)[0] < SUPPLY / 100.0);
    }

    #[test]
    fn a_circuit_with_nothing_reactive_in_it_still_runs() {
        // A plain divider has no state to carry, so every step is the same
        // answer — which is correct, and worth not crashing over.
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts: SUPPLY,
            series_ohms: 0.0,
        });
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: OHMS });
        let top = netlist.add_net();
        let bottom = netlist.add_net();
        netlist.connect_named(source, "+", top).unwrap();
        netlist.connect_named(resistor, "a", top).unwrap();
        netlist.connect_named(resistor, "b", bottom).unwrap();
        netlist.connect_named(source, "-", bottom).unwrap();

        let run = transient(
            &netlist,
            &TransientOptions {
                step: 1e-3,
                duration: 3e-3,
                start: InitialCondition::AsPlaced,
                integration: Integration::default(),
            },
        )
        .unwrap();

        assert_eq!(run.len(), 3);
        for volts in run.series(top).unwrap() {
            assert!((volts - SUPPLY).abs() < 1e-9);
        }
    }

    #[test]
    fn a_schedule_that_makes_no_sense_is_refused() {
        // Clamping a zero step to something would hide the caller's bug behind
        // a run that looks like it worked.
        let rc = rc_circuit();
        for (step, duration) in [(0.0, 1e-3), (-1e-6, 1e-3), (1e-6, 0.0), (f64::NAN, 1e-3)] {
            let refused = transient(
                &rc.netlist,
                &TransientOptions {
                    step,
                    duration,
                    start: InitialCondition::AsPlaced,
                    integration: Integration::default(),
                },
            );
            assert_eq!(
                refused.err(),
                Some(TransientError::BadSchedule),
                "{step}, {duration}"
            );
        }
    }

    #[test]
    fn a_circuit_that_cannot_be_solved_says_where_it_stopped() {
        let netlist = Netlist::new();

        let refused = transient(
            &netlist,
            &TransientOptions {
                step: 1e-6,
                duration: 1e-3,
                start: InitialCondition::AsPlaced,
                integration: Integration::default(),
            },
        );

        assert!(matches!(refused, Err(TransientError::Failed { at, .. }) if at == 0.0));
    }
}

#[cfg(test)]
mod adaptive_tests {
    use super::tests::{TAU, rc_circuit};
    use super::*;
    use crate::netlist::ElementKind;

    const SUPPLY: f64 = 9.0;

    fn run(options: AdaptiveOptions) -> Transient {
        let rc = rc_circuit();
        adaptive(&rc.netlist, &options).expect("an RC circuit runs")
    }

    fn charging(options: AdaptiveOptions) -> (super::tests::Rc, Transient) {
        let rc = rc_circuit();
        let found = adaptive(&rc.netlist, &options).expect("an RC circuit runs");
        (rc, found)
    }

    fn settings() -> AdaptiveOptions {
        AdaptiveOptions {
            initial_step: TAU / 1000.0,
            min_step: TAU / 1e9,
            max_step: TAU,
            duration: 5.0 * TAU,
            tolerance: 1e-3,
            integration: Integration::default(),
            start: InitialCondition::AsPlaced,
        }
    }

    #[test]
    fn it_lands_on_the_analytic_curve() {
        // Adaptive or not, the answer is the answer: within 1% of
        // V(1 − e^(−t/τ)) wherever it happens to have put its samples.
        let (rc, found) = charging(settings());
        let high = found.series(rc.across).unwrap();
        let low = found.series(rc.bottom).unwrap();

        for (index, at) in found.times().iter().enumerate() {
            let volts = high[index] - low[index];
            let expected = SUPPLY * (1.0 - (-at / TAU).exp());
            assert!(
                (volts - expected).abs() < 0.01 * SUPPLY,
                "at {at} s: {volts} V against {expected} V"
            );
        }
    }

    #[test]
    fn it_grows_the_step_once_the_circuit_stops_moving() {
        // An RC circuit is all action at the start and nearly still by 5τ. A
        // run that kept its first step would take a thousand of them to say
        // nothing.
        let found = run(settings());
        let times = found.times();
        let first = times[0];
        let last = times[times.len() - 1] - times[times.len() - 2];

        assert!(last > first * 10.0, "first {first} s, last {last} s");
    }

    #[test]
    fn it_never_steps_past_the_end() {
        let found = run(settings());

        let end = found.times().last().copied().unwrap();
        assert!((end - 5.0 * TAU).abs() < 1e-15, "{end}");
    }

    #[test]
    fn it_never_steps_further_than_it_was_allowed() {
        let mut options = settings();
        options.max_step = TAU / 50.0;
        let found = run(options);

        let mut previous = 0.0;
        for at in found.times() {
            assert!(at - previous <= TAU / 50.0 + 1e-15, "{}", at - previous);
            previous = *at;
        }
    }

    #[test]
    fn a_step_that_is_too_long_is_rejected_rather_than_believed() {
        // Start with a step the size of the whole run. The first attempt
        // cannot be accurate, and what the loop does about it is the ticket.
        let mut options = settings();
        options.initial_step = 5.0 * TAU;
        let found = run(options);

        assert!(found.rejections() > 0, "nothing was rejected");
        assert_eq!(found.forced(), 0, "nothing should have needed forcing");
    }

    #[test]
    fn a_rejected_step_leaves_no_sample_behind() {
        // "Never silently accepted": a rejected step contributes nothing to
        // the series, so the times stay strictly increasing and every sample
        // is one the loop stood behind.
        let mut options = settings();
        options.initial_step = 5.0 * TAU;
        let found = run(options);

        let mut previous = 0.0;
        for at in found.times() {
            assert!(*at > previous, "{at} after {previous}");
            previous = *at;
        }
        assert_eq!(found.times().len(), found.len());
    }

    #[test]
    fn a_tighter_tolerance_costs_more_steps() {
        let loose = run(AdaptiveOptions {
            tolerance: 1e-2,
            ..settings()
        });
        let tight = run(AdaptiveOptions {
            tolerance: 1e-5,
            ..settings()
        });

        assert!(
            tight.len() > loose.len(),
            "loose {} steps, tight {}",
            loose.len(),
            tight.len()
        );
    }

    #[test]
    fn it_says_when_it_gave_up_on_the_tolerance() {
        // Held to an impossible tolerance with nowhere left to shrink, the
        // loop finishes rather than hanging — and counts every step it had to
        // force, because an answer less accurate than it was asked to be is
        // not something to keep quiet about.
        let found = run(AdaptiveOptions {
            tolerance: 1e-30,
            min_step: TAU / 100.0,
            initial_step: TAU / 100.0,
            duration: TAU,
            ..settings()
        });

        assert!(found.forced() > 0, "nothing was forced");
        assert!(!found.is_empty(), "the run should still have finished");
    }

    #[test]
    fn a_stiff_rc_and_diode_circuit_completes_without_tuning() {
        // The acceptance. A diode across the charging capacitor: nothing
        // happens until the cap reaches about 0.6 V, and then the diode turns
        // on over a few tens of millivolts and the circuit's time constant
        // collapses. A fixed step picked for the first part steps clean over
        // the second.
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts: SUPPLY,
            series_ohms: 0.0,
        });
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: 1_000.0 });
        let capacitor = netlist.add_element(ElementKind::Capacitor {
            farads: 1e-6,
            initial_volts: 0.0,
        });
        let diode = netlist.add_element(ElementKind::Led {
            colour: crate::netlist::LedColour::Red,
        });

        let top = netlist.add_net();
        let middle = netlist.add_net();
        let bottom = netlist.add_net();
        netlist.connect_named(source, "+", top).unwrap();
        netlist.connect_named(resistor, "a", top).unwrap();
        netlist.connect_named(resistor, "b", middle).unwrap();
        netlist.connect_named(capacitor, "a", middle).unwrap();
        netlist.connect_named(capacitor, "b", bottom).unwrap();
        netlist.connect_named(diode, "anode", middle).unwrap();
        netlist.connect_named(diode, "cathode", bottom).unwrap();
        netlist.connect_named(source, "-", bottom).unwrap();

        let found = adaptive(
            &netlist,
            &AdaptiveOptions {
                // Deliberately far too long for what happens at the knee.
                initial_step: TAU / 10.0,
                min_step: TAU / 1e9,
                max_step: TAU,
                duration: 5.0 * TAU,
                tolerance: 1e-3,
                start: InitialCondition::AsPlaced,
                integration: Integration::default(),
            },
        )
        .expect("a stiff circuit completes");

        assert!(found.rejections() > 0, "it never had to shrink");
        assert!(!found.is_empty());

        // It settles where the diode holds it, well below the supply.
        let high = found.series(middle).unwrap();
        let low = found.series(bottom).unwrap();
        let settled = high.last().unwrap() - low.last().unwrap();
        assert!(
            (0.5..2.5).contains(&settled),
            "clamped by the diode, not charged to the supply: {settled} V"
        );
    }

    #[test]
    fn a_schedule_that_makes_no_sense_is_refused() {
        let rc = rc_circuit();
        for options in [
            AdaptiveOptions {
                initial_step: 0.0,
                ..settings()
            },
            AdaptiveOptions {
                tolerance: 0.0,
                ..settings()
            },
            AdaptiveOptions {
                min_step: TAU,
                max_step: TAU / 10.0,
                ..settings()
            },
            AdaptiveOptions {
                duration: -1.0,
                ..settings()
            },
        ] {
            assert_eq!(
                adaptive(&rc.netlist, &options).err(),
                Some(TransientError::BadSchedule)
            );
        }
    }
}

#[cfg(test)]
mod inductor_tests {
    use super::*;
    use crate::netlist::ElementKind;

    pub(super) const SUPPLY: f64 = 9.0;
    pub(super) const OHMS: f64 = 100.0;
    const HENRIES: f64 = 1e-3;
    /// τ = L/R for an RL circuit, as RC is for an RC one.
    pub(super) const TAU: f64 = HENRIES / OHMS;

    pub(super) struct Rl {
        pub netlist: Netlist,
        /// Between the resistor and the inductor.
        middle: NetId,
        bottom: NetId,
    }

    /// A 9 V source driving 1 mH through 100 Ω.
    pub(super) fn rl_circuit() -> Rl {
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts: SUPPLY,
            series_ohms: 0.0,
        });
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: OHMS });
        let inductor = netlist.add_element(ElementKind::Inductor {
            henries: HENRIES,
            initial_amps: 0.0,
        });

        let top = netlist.add_net();
        let middle = netlist.add_net();
        let bottom = netlist.add_net();
        netlist.connect_named(source, "+", top).unwrap();
        netlist.connect_named(resistor, "a", top).unwrap();
        netlist.connect_named(resistor, "b", middle).unwrap();
        netlist.connect_named(inductor, "a", middle).unwrap();
        netlist.connect_named(inductor, "b", bottom).unwrap();
        netlist.connect_named(source, "-", bottom).unwrap();

        Rl {
            netlist,
            middle,
            bottom,
        }
    }

    /// Current through the circuit at each sample, from the resistor's drop.
    pub(super) fn current(rl: &Rl, run: &Transient) -> Vec<f64> {
        run.series(rl.middle)
            .unwrap()
            .iter()
            .zip(run.series(rl.bottom).unwrap())
            .map(|(middle, bottom)| (SUPPLY - (middle - bottom)) / OHMS)
            .collect()
    }

    #[test]
    fn an_rl_circuit_rises_the_way_the_maths_says() {
        // The acceptance of C-07: i(t) = (V/R)·(1 − e^(−t/τ)), at the three
        // points that matter.
        let rl = rl_circuit();
        let step = TAU / 1000.0;
        let run = transient(
            &rl.netlist,
            &TransientOptions {
                step,
                duration: 5.0 * TAU,
                start: InitialCondition::AsPlaced,
                integration: Integration::default(),
            },
        )
        .unwrap();
        let amps = current(&rl, &run);

        for multiple in [1.0, 2.0, 5.0] {
            let index = (multiple * TAU / step).round() as usize - 1;
            let found = amps[index];
            let expected = SUPPLY / OHMS * (1.0 - (-multiple).exp());

            let error = (found - expected).abs() / expected;
            assert!(
                error < 0.01,
                "at {multiple}τ: {found} A against {expected} A"
            );
        }
    }

    #[test]
    fn a_step_into_an_inductor_does_not_ring() {
        // Backward Euler damps rather than overshoots, which is the whole
        // reason C-06 chose it and the reason this circuit is worth checking:
        // trapezoidal on a step input is where the ringing would show.
        let rl = rl_circuit();
        let run = transient(
            &rl.netlist,
            &TransientOptions {
                step: TAU / 20.0,
                duration: 5.0 * TAU,
                start: InitialCondition::AsPlaced,
                integration: Integration::default(),
            },
        )
        .unwrap();
        let amps = current(&rl, &run);

        let settled = SUPPLY / OHMS;
        for pair in amps.windows(2) {
            assert!(pair[1] >= pair[0] - 1e-12, "{} then {}", pair[0], pair[1]);
        }
        for value in &amps {
            assert!(*value <= settled + 1e-9, "overshot to {value} A");
        }
    }

    #[test]
    fn an_inductor_is_a_wire_at_dc() {
        // Once nothing is changing it drops nothing, so the whole supply is
        // across the resistor and the current is V/R.
        let rl = rl_circuit();

        let point = crate::solver::operating_point(&rl.netlist).unwrap();

        let across = point.voltage(rl.middle).unwrap() - point.voltage(rl.bottom).unwrap();
        assert!(across.abs() < 1e-9, "{across} V across a wire");
    }

    #[test]
    fn starting_from_the_operating_point_starts_settled() {
        // A circuit that has been powered a while has its inductor already at
        // full current, so there is no rise left to watch.
        let rl = rl_circuit();
        let run = transient(
            &rl.netlist,
            &TransientOptions {
                step: TAU / 10.0,
                duration: 2.0 * TAU,
                start: InitialCondition::OperatingPoint,
                integration: Integration::default(),
            },
        )
        .unwrap();

        for amps in current(&rl, &run) {
            assert!((amps - SUPPLY / OHMS).abs() < 1e-6, "{amps} A");
        }
    }
}

/// What happens when the circuit changes shape between one step and the next.
///
/// C-19's acceptance, and the reason the switch is a solver ticket rather than
/// a palette one. Nothing is carried from step to step but the charge in the
/// reactive elements, so a flip cannot leave the solver holding a stale
/// factorisation or a stale operating point: the step after it is a fresh
/// assembly and a fresh solve of whatever is there now.
#[cfg(test)]
mod contact_tests {
    use super::*;
    use crate::netlist::{ElementId, ElementKind};

    const SUPPLY: f64 = 9.0;
    const OHMS: f64 = 1_000.0;
    const FARADS: f64 = 1e-6;
    const HENRIES: f64 = 1e-3;

    struct Switched {
        netlist: Netlist,
        contact: ElementId,
        reactive: ElementId,
        /// Across the reactive element, whose other pin is the reference.
        across: NetId,
    }

    /// A supply, a contact, a resistor and one reactive element in a loop.
    fn switched(reactive: ElementKind) -> Switched {
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts: SUPPLY,
            series_ohms: 0.0,
        });
        let contact = netlist.add_element(ElementKind::Switch { closed: true });
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: OHMS });
        let reactive = netlist.add_element(reactive);

        let top = netlist.add_net();
        let switched = netlist.add_net();
        let middle = netlist.add_net();
        let bottom = netlist.add_net();
        netlist.connect_named(source, "+", top).unwrap();
        netlist.connect_named(contact, "a", top).unwrap();
        netlist.connect_named(contact, "b", switched).unwrap();
        netlist.connect_named(resistor, "a", switched).unwrap();
        netlist.connect_named(resistor, "b", middle).unwrap();
        netlist.connect_named(reactive, "a", middle).unwrap();
        netlist.connect_named(reactive, "b", bottom).unwrap();
        netlist.connect_named(source, "-", bottom).unwrap();

        Switched {
            netlist,
            contact,
            reactive,
            across: middle,
        }
    }

    #[test]
    fn a_switch_opened_mid_run_leaves_the_charge_where_it_was() {
        // A capacitor does not forget what it holds because something upstream
        // opened, and the step that does the opening has to say so rather than
        // jumping. Continuity across the flip is the property; that the step
        // solves at all is the other half.
        let mut circuit = switched(ElementKind::Capacitor {
            farads: FARADS,
            initial_volts: 0.0,
        });
        let map = circuit.netlist.compile().unwrap();
        let step = OHMS * FARADS / 100.0;
        let mut previous = SecondaryMap::new();

        for _ in 0..100 {
            let (_, charge) = take_step(
                &circuit.netlist,
                &map,
                Integration::default(),
                0.0,
                step,
                &previous,
                &[],
            )
            .unwrap();
            previous = charge;
        }
        let charged = previous[circuit.reactive].value;
        assert!(charged > SUPPLY / 2.0, "only reached {charged} V");

        circuit
            .netlist
            .set_kind(circuit.contact, ElementKind::Switch { closed: false })
            .unwrap();
        let (point, after) = take_step(
            &circuit.netlist,
            &map,
            Integration::default(),
            0.0,
            step,
            &previous,
            &[],
        )
        .expect("the step after a flip still solves");

        assert!((after[circuit.reactive].value - charged).abs() < 1e-9);
        assert!((point.voltage(circuit.across).unwrap() - charged).abs() < 1e-9);
    }

    #[test]
    fn the_numbering_survives_a_flip() {
        // The step after the flip reuses the node map compiled before it, which
        // is what `transient` does and what makes a flip cheap. An open contact
        // still joins the two nets it sits between, so nothing is renumbered —
        // only the stamp changes.
        let mut circuit = switched(ElementKind::Capacitor {
            farads: FARADS,
            initial_volts: 0.0,
        });
        let before = circuit.netlist.compile().unwrap();

        circuit
            .netlist
            .set_kind(circuit.contact, ElementKind::Switch { closed: false })
            .unwrap();
        let after = circuit.netlist.compile().unwrap();

        assert_eq!(before.len(), after.len());
        assert_eq!(
            before.index_of(circuit.across),
            after.index_of(circuit.across)
        );
    }

    #[test]
    fn interrupting_an_inductor_kicks_and_then_settles() {
        // The case that kills a solver if anything is going to: an inductor
        // insists on the current it was passing, and opening the switch takes
        // its path away. The answer is a large reverse voltage — the spark
        // across a real switch's contacts — and it has to be a number rather
        // than an infinity or a failure to converge.
        let mut circuit = switched(ElementKind::Inductor {
            henries: HENRIES,
            initial_amps: 0.0,
        });
        let map = circuit.netlist.compile().unwrap();
        let step = HENRIES / OHMS / 100.0;
        let mut previous = SecondaryMap::new();

        for _ in 0..500 {
            let (_, charge) = take_step(
                &circuit.netlist,
                &map,
                Integration::default(),
                0.0,
                step,
                &previous,
                &[],
            )
            .unwrap();
            previous = charge;
        }
        let running = previous[circuit.reactive].value;
        assert!(running > SUPPLY / OHMS / 2.0, "only reached {running} A");

        circuit
            .netlist
            .set_kind(circuit.contact, ElementKind::Switch { closed: false })
            .unwrap();
        let (kick, after) = take_step(
            &circuit.netlist,
            &map,
            Integration::default(),
            0.0,
            step,
            &previous,
            &[],
        )
        .expect("the step that interrupts an inductor still solves");

        let volts = kick.voltage(circuit.across).unwrap();
        assert!(volts.is_finite(), "{volts} V");
        // The other way round and far bigger than the supply: that is what a
        // coil does when its current is taken away, not a numerical accident.
        assert!(volts < -SUPPLY, "{volts} V");

        // And it is over: with nowhere to go the current is gone in one step,
        // so the step after that is quiet rather than ringing or running away.
        let (settled, _) = take_step(
            &circuit.netlist,
            &map,
            Integration::default(),
            0.0,
            step,
            &after,
            &[],
        )
        .unwrap();
        assert!(after[circuit.reactive].value.abs() < 1e-12);
        assert!(settled.voltage(circuit.across).unwrap().abs() < 1e-9);
    }
}

/// The two integration rules, side by side, on the circuits that tell them
/// apart (C-08).
///
/// The trade-off is documented in `solver::integration`; this is where it is
/// demonstrated, because a trade-off nobody has measured is a preference.
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::netlist::ElementKind;

    const HENRIES: f64 = 1e-3;
    const FARADS: f64 = 1e-6;

    struct Tank {
        netlist: Netlist,
        top: NetId,
        bottom: NetId,
    }

    /// A charged capacitor and a coil, and nothing else: no resistance
    /// anywhere, so the energy has nowhere to go and the real circuit rings
    /// for ever at 1/2π√(LC).
    ///
    /// The circuit every integration rule is judged on, because whatever it
    /// loses or gains here is the rule's own doing and not the circuit's.
    fn lc_tank() -> Tank {
        let mut netlist = Netlist::new();
        let capacitor = netlist.add_element(ElementKind::Capacitor {
            farads: FARADS,
            initial_volts: 1.0,
        });
        let inductor = netlist.add_element(ElementKind::Inductor {
            henries: HENRIES,
            initial_amps: 0.0,
        });

        let top = netlist.add_net();
        let bottom = netlist.add_net();
        netlist.connect_named(capacitor, "a", top).unwrap();
        netlist.connect_named(capacitor, "b", bottom).unwrap();
        netlist.connect_named(inductor, "a", top).unwrap();
        netlist.connect_named(inductor, "b", bottom).unwrap();

        Tank {
            netlist,
            top,
            bottom,
        }
    }

    /// How long one swing of that tank takes: T = 2π√(LC), 199 µs here.
    fn period() -> f64 {
        2.0 * core::f64::consts::PI * (HENRIES * FARADS).sqrt()
    }

    /// The voltage across a pair of nets at each sample.
    fn between(run: &Transient, high: NetId, low: NetId) -> Vec<f64> {
        let top = run.series(high).unwrap();
        let bottom = run.series(low).unwrap();
        top.iter().zip(bottom).map(|(a, b)| a - b).collect()
    }

    fn largest(volts: &[f64]) -> f64 {
        volts.iter().fold(0.0f64, |most, v| most.max(v.abs()))
    }

    #[test]
    fn an_lc_tank_rings_on_under_trapezoidal_and_dies_under_backward_euler() {
        // The headline of C-08, and the reason trapezoidal is offered at all.
        // Nothing in this circuit dissipates anything, so any amplitude lost
        // over five swings was taken by the integration rule — backward
        // Euler's damping, which is exactly the error it trades accuracy for.
        let tank = lc_tank();
        let cycle = period();
        let per_cycle = 100;

        let mut amplitudes = Vec::new();
        for method in [Integration::BackwardEuler, Integration::Trapezoidal] {
            let run = transient(
                &tank.netlist,
                &TransientOptions {
                    step: cycle / per_cycle as f64,
                    duration: 5.0 * cycle,
                    start: InitialCondition::AsPlaced,
                    integration: method,
                },
            )
            .expect("a tank rings");
            let volts = between(&run, tank.top, tank.bottom);

            // It is a swing and not a circuit sitting still: the voltage
            // crosses zero twice a cycle, which also says the period is the
            // one √(LC) asks for and not one the rule invented.
            let crossings = volts
                .windows(2)
                .filter(|pair| pair[0].signum() != pair[1].signum())
                .count();
            assert!(
                (9..=11).contains(&crossings),
                "{method:?}: {crossings} zero crossings over five cycles"
            );

            amplitudes.push((
                largest(&volts[..per_cycle]),
                largest(&volts[volts.len() - per_cycle..]),
            ));
        }

        let (euler_first, euler_last) = amplitudes[0];
        let (trapezoid_first, trapezoid_last) = amplitudes[1];

        // Backward Euler has bled away more than a third of it.
        assert!(
            euler_last < 0.6 * euler_first,
            "backward Euler still at {euler_last} V from {euler_first} V"
        );
        // Trapezoidal is still where it started, to within a per cent.
        assert!(
            (trapezoid_last - trapezoid_first).abs() < 0.01 * trapezoid_first,
            "trapezoidal at {trapezoid_last} V from {trapezoid_first} V"
        );
    }

    #[test]
    fn trapezoidal_rings_on_a_step_where_backward_euler_does_not() {
        // The other half of the same coin, and the reason backward Euler is
        // the default. The step is four time constants — far longer than the
        // circuit — and trapezoidal answers by overshooting and undershooting
        // alternately about the settled current. Nothing in the circuit
        // oscillates: the ringing belongs to the rule.
        //
        // The companion of `inductor_tests::a_step_into_an_inductor_does_not_ring`,
        // which holds backward Euler to the opposite behaviour.
        let rl = super::inductor_tests::rl_circuit();
        let step = 4.0 * super::inductor_tests::TAU;
        let settled = super::inductor_tests::SUPPLY / super::inductor_tests::OHMS;

        let run = |method| {
            let found = transient(
                &rl.netlist,
                &TransientOptions {
                    step,
                    duration: 10.0 * step,
                    start: InitialCondition::AsPlaced,
                    integration: method,
                },
            )
            .expect("an RL circuit runs");
            super::inductor_tests::current(&rl, &found)
        };

        let euler = run(Integration::BackwardEuler);
        let trapezoid = run(Integration::Trapezoidal);

        // Backward Euler approaches from below and stays there.
        assert!(
            euler.iter().all(|amps| *amps <= settled + 1e-9),
            "backward Euler overshot: {euler:?}"
        );

        // Trapezoidal goes past it, comes back under, and goes past it again.
        let overshoots = trapezoid.iter().filter(|amps| **amps > settled).count();
        assert!(overshoots >= 2, "trapezoidal never rang: {trapezoid:?}");
        let alternating = trapezoid
            .windows(3)
            .filter(|three| (three[1] - three[0]).signum() != (three[2] - three[1]).signum())
            .count();
        assert!(
            alternating >= 2,
            "trapezoidal moved one way throughout: {trapezoid:?}"
        );

        // And it is ringing about the right answer rather than running away:
        // both rules end up at the current the circuit actually settles at.
        for (name, amps) in [("backward Euler", &euler), ("trapezoidal", &trapezoid)] {
            let last = amps.last().copied().unwrap();
            assert!(
                (last - settled).abs() < settled / 100.0,
                "{name} ended at {last} A rather than {settled} A"
            );
        }
    }

    #[test]
    fn trapezoidal_is_the_more_accurate_of_the_two_at_the_same_step() {
        // What the ringing buys: second order. The same steps, over a curve
        // with a closed form to be wrong about.
        let rc = super::tests::rc_circuit();
        let tau = super::tests::TAU;
        let step = tau / 10.0;

        let worst = |method| {
            let run = transient(
                &rc.netlist,
                &TransientOptions {
                    step,
                    duration: 2.0 * tau,
                    start: InitialCondition::AsPlaced,
                    integration: method,
                },
            )
            .expect("an RC circuit charges");
            let volts = between(&run, rc.across, rc.bottom);
            volts
                .iter()
                .zip(run.times())
                .map(|(found, at)| (found - 9.0 * (1.0 - (-at / tau).exp())).abs())
                .fold(0.0f64, f64::max)
        };

        let euler = worst(Integration::BackwardEuler);
        let trapezoid = worst(Integration::Trapezoidal);

        assert!(
            trapezoid < euler / 3.0,
            "trapezoidal off by {trapezoid} V against backward Euler's {euler} V"
        );
    }

    #[test]
    fn the_step_estimator_lets_a_second_order_rule_take_longer_steps() {
        // The same tolerance, asked of both rules, and the estimator knows
        // what each rule's two answers converging means: trapezoidal covers
        // the same run in a fraction of the steps and lands closer.
        let rc = super::tests::rc_circuit();
        let tau = super::tests::TAU;

        let run = |method| {
            adaptive(
                &rc.netlist,
                &AdaptiveOptions {
                    initial_step: tau / 1000.0,
                    min_step: tau / 1e9,
                    max_step: tau,
                    duration: 5.0 * tau,
                    tolerance: 1e-3,
                    start: InitialCondition::AsPlaced,
                    integration: method,
                },
            )
            .expect("an RC circuit charges")
        };

        let euler = run(Integration::BackwardEuler);
        let trapezoid = run(Integration::Trapezoidal);

        assert!(
            trapezoid.len() * 2 < euler.len(),
            "{} steps against {}",
            trapezoid.len(),
            euler.len()
        );

        // Fewer steps and not a worse answer, which is the point: both stay on
        // the analytic curve, well inside a per cent of the supply.
        for (name, found) in [("backward Euler", &euler), ("trapezoidal", &trapezoid)] {
            let volts = between(found, rc.across, rc.bottom);
            let worst = volts
                .iter()
                .zip(found.times())
                .map(|(v, at)| (v - 9.0 * (1.0 - (-at / tau).exp())).abs())
                .fold(0.0f64, f64::max);
            assert!(worst < 0.09, "{name} wandered {worst} V off the curve");
        }
    }

    #[test]
    fn the_first_step_of_a_run_is_the_same_whichever_rule_was_asked_for() {
        // Nothing has solved the circuit yet, so there is no rate for
        // trapezoidal to average against and it takes a backward Euler step
        // instead. Asserted here rather than only on the rule, because the
        // alternative — assuming the circuit starts at rest — is invisible
        // until it has been integrated over a whole run.
        let rc = super::tests::rc_circuit();
        let step = super::tests::TAU / 50.0;

        let first = |method| {
            let run = transient(
                &rc.netlist,
                &TransientOptions {
                    step,
                    duration: step,
                    start: InitialCondition::AsPlaced,
                    integration: method,
                },
            )
            .expect("an RC circuit charges");
            between(&run, rc.across, rc.bottom)[0]
        };

        assert_eq!(
            first(Integration::BackwardEuler),
            first(Integration::Trapezoidal)
        );
    }
}
