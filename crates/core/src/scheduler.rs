// SPDX-License-Identifier: MIT OR Apache-2.0

//! Who advances time.
//!
//! Seam #4 of the design doc (7.1.4). The renderer says how much wall time has
//! passed; the scheduler decides how many simulation steps that is worth and of
//! what size. Frame rate and simulation rate are then independent, and a
//! headless caller — a test, the runner of C-30 — drives the same code with no
//! renderer at all.
//!
//! CircuitJS1 advances time inside its 16 ms render timer, which is why it
//! cannot run headless and why its simulation rate is whatever the browser
//! feels like giving it (see the prior-art study, §1). That is the mistake this
//! module exists to avoid.

use crate::engine::{Engine, EngineError};

/// Simulation steps per second by default.
///
/// Well above a display's refresh rate: the point of the seam is that they are
/// unrelated, and a transient (C-09) will want far more than this.
pub const DEFAULT_STEPS_PER_SECOND: f64 = 1_000.0;

/// Most wall time one call will try to make up, in seconds.
///
/// A tab left in the background reports an enormous elapsed time when it wakes.
/// Simulating all of it would take longer than it took to accrue and the app
/// would never catch up, so the excess is dropped and the simulation admits to
/// being behind rather than freezing.
///
/// Expressed as time rather than as a step count on purpose: a step cap would
/// silently become a frame-rate cap the moment the simulation rate rose.
const MAX_CATCH_UP: f64 = 0.25;

/// Drives an engine at a fixed simulation rate.
pub struct Scheduler {
    engine: Box<dyn Engine>,
    step: f64,
    /// Wall time owed to the simulation but not yet stepped.
    accumulator: f64,
}

/// What one call to [`Scheduler::advance`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Advanced {
    pub steps: usize,
    /// Steps that were owed and deliberately skipped, because catching up would
    /// have cost more than the time it was catching up on.
    pub dropped: usize,
}

impl Scheduler {
    pub fn new(engine: Box<dyn Engine>) -> Self {
        Scheduler::with_rate(engine, DEFAULT_STEPS_PER_SECOND)
    }

    pub fn with_rate(engine: Box<dyn Engine>, steps_per_second: f64) -> Self {
        Scheduler {
            engine,
            step: 1.0 / steps_per_second,
            accumulator: 0.0,
        }
    }

    /// Advances by `elapsed` seconds of wall time.
    ///
    /// Time left over is carried, so many small calls and one large call cover
    /// the same simulated span. The step size never changes with the caller's
    /// timing, which is what keeps a result reproducible.
    pub fn advance(&mut self, elapsed: f64) -> Result<Advanced, EngineError> {
        let mut dropped = 0;

        if elapsed.is_finite() && elapsed > 0.0 {
            let honoured = elapsed.min(MAX_CATCH_UP);
            dropped = ((elapsed - honoured) / self.step) as usize;
            self.accumulator += honoured;
        }

        let steps = (self.accumulator / self.step) as usize;
        for _ in 0..steps {
            if let Err(error) = self.engine.step(self.step) {
                // Nothing is owed after a step that failed: before, the time
                // already stepped this batch was owed again after the next
                // edit, and the run ran ahead of the clock by up to a quarter
                // of a second (C-107).
                self.accumulator = 0.0;
                return Err(error);
            }
        }
        self.accumulator -= steps as f64 * self.step;

        Ok(Advanced { steps, dropped })
    }

    /// Runs a fixed number of steps, for a caller that has no clock — a test, or
    /// a headless run.
    pub fn run(&mut self, steps: usize) -> Result<(), EngineError> {
        for _ in 0..steps {
            self.engine.step(self.step)?;
        }
        Ok(())
    }

    pub fn engine(&self) -> &dyn Engine {
        self.engine.as_ref()
    }

    pub fn engine_mut(&mut self) -> &mut dyn Engine {
        self.engine.as_mut()
    }

    /// Seconds per simulation step.
    pub fn step(&self) -> f64 {
        self.step
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::engine_for;
    use crate::netlist::{ElementKind, Netlist};

    use super::*;

    /// An engine that steps as the real one does until its `fail_at`th step,
    /// which fails, and counts the steps it was asked for.
    struct Failing {
        inner: Box<dyn Engine>,
        asked: usize,
        fail_at: usize,
    }

    impl Engine for Failing {
        fn step(&mut self, dt: f64) -> Result<(), EngineError> {
            self.asked += 1;
            if self.asked == self.fail_at {
                return Err(EngineError::NotASource(crate::netlist::ElementId::default()));
            }
            self.inner.step(dt)
        }
        fn adopt(&mut self, netlist: Netlist) {
            self.inner.adopt(netlist)
        }
        fn traces(&self) -> &crate::scope::Traces {
            self.inner.traces()
        }
        fn traces_mut(&mut self) -> &mut crate::scope::Traces {
            self.inner.traces_mut()
        }
        fn set_source_voltage(
            &mut self,
            source: crate::netlist::ElementId,
            volts: f64,
        ) -> Result<(), EngineError> {
            self.inner.set_source_voltage(source, volts)
        }
        fn node_voltage(&self, net: crate::netlist::NetId) -> Option<f64> {
            self.inner.node_voltage(net)
        }
        fn voltage_across(&self, element: crate::netlist::ElementId) -> Option<f64> {
            self.inner.voltage_across(element)
        }
        fn source_current(&self, source: crate::netlist::ElementId) -> Option<f64> {
            self.inner.source_current(source)
        }
        fn time(&self) -> f64 {
            self.inner.time()
        }
    }

    #[test]
    fn a_failed_step_leaves_no_time_owed() {
        // A tenth of a second at the default rate, failing part way through:
        // the next frame steps its own time and no more (C-107).
        let engine = engine_for(circuit()).unwrap();
        let mut scheduler = Scheduler::new(Box::new(Failing {
            inner: engine,
            asked: 0,
            fail_at: 3,
        }));
        assert!(scheduler.advance(0.1).is_err());
        let frame = 1.0 / 60.0;
        let advanced = scheduler.advance(frame).unwrap();
        let expected = (frame / scheduler.step()) as usize;
        assert!(
            advanced.steps <= expected + 1,
            "{} steps for one frame, against {expected}",
            advanced.steps
        );
    }

    /// A battery across a resistor — enough to step, nothing more.
    fn circuit() -> Netlist {
        let mut netlist = Netlist::new();
        let battery = netlist.add_element(ElementKind::DcSource {
            volts: 9.0,
            series_ohms: 0.0,
        });
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: 220.0 });
        let top = netlist.add_net();
        let bottom = netlist.add_net();
        netlist.connect_named(battery, "+", top).unwrap();
        netlist.connect_named(resistor, "a", top).unwrap();
        netlist.connect_named(resistor, "b", bottom).unwrap();
        netlist.connect_named(battery, "-", bottom).unwrap();
        netlist
    }

    fn scheduler(steps_per_second: f64) -> Scheduler {
        Scheduler::with_rate(engine_for(circuit()).unwrap(), steps_per_second)
    }

    #[test]
    fn runs_with_no_renderer_at_all() {
        // The property C-13 is really about: the simulation is drivable by
        // something that has no frames, no canvas and no clock.
        let mut scheduler = scheduler(1_000.0);

        scheduler.run(500).unwrap();

        assert!((scheduler.engine().time() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn simulated_time_does_not_depend_on_how_it_was_asked_for() {
        // One big call and many small ones cover the same span. Without this,
        // a stuttering frame rate would change the answer.
        let mut coarse = scheduler(1_000.0);
        let mut fine = scheduler(1_000.0);

        coarse.advance(0.05).unwrap();
        for _ in 0..50 {
            fine.advance(0.001).unwrap();
        }

        assert!((coarse.engine().time() - fine.engine().time()).abs() < 1e-9);
    }

    #[test]
    fn keeps_the_remainder_rather_than_losing_it() {
        // A frame worth less than one step still counts; three of them add up.
        let mut scheduler = scheduler(100.0); // 10 ms per step
        let mut stepped = 0;

        for _ in 0..3 {
            stepped += scheduler.advance(0.004).unwrap().steps;
        }

        // 12 ms of wall time is one whole step, with 2 ms carried.
        assert_eq!(stepped, 1);
        assert!((scheduler.engine().time() - 0.01).abs() < 1e-9);
    }

    #[test]
    fn the_step_size_is_the_schedulers_choice_not_the_callers() {
        let mut scheduler = scheduler(500.0);

        let advanced = scheduler.advance(0.1).unwrap();

        assert_eq!(advanced.steps, 50);
        assert!((scheduler.step() - 0.002).abs() < 1e-12);
    }

    #[test]
    fn frame_rate_and_simulation_rate_are_unrelated() {
        // Same wall time, different simulation rates: more steps, same span.
        let mut slow = scheduler(100.0);
        let mut fast = scheduler(10_000.0);

        let slow_steps = slow.advance(0.02).unwrap().steps;
        let fast_steps = fast.advance(0.02).unwrap().steps;

        assert_eq!(slow_steps, 2);
        assert!(fast_steps > slow_steps);
        assert!((slow.engine().time() - fast.engine().time()).abs() < 1e-9);
    }

    #[test]
    fn a_backgrounded_tab_does_not_try_to_catch_up_forever() {
        // Ten minutes of wall time at 1 kHz is 600,000 steps, which would take
        // far longer to simulate than it took to accrue. It is dropped, and the
        // drop is reported rather than hidden.
        let mut scheduler = scheduler(1_000.0);

        let advanced = scheduler.advance(600.0).unwrap();

        // A quarter second's worth is made up; the rest is let go.
        assert_eq!(advanced.steps, 250);
        assert!(advanced.dropped > 500_000);
        // And the debt is let go, so the next frame is normal rather than also
        // maxed out.
        assert_eq!(scheduler.advance(0.001).unwrap().steps, 1);
    }

    #[test]
    fn ignores_a_nonsensical_elapsed_time() {
        let mut scheduler = scheduler(1_000.0);

        assert_eq!(scheduler.advance(-1.0).unwrap().steps, 0);
        assert_eq!(scheduler.advance(f64::NAN).unwrap().steps, 0);
        assert_eq!(scheduler.engine().time(), 0.0);
    }
}
