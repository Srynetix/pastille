// SPDX-License-Identifier: MIT OR Apache-2.0

//! Newton–Raphson around a linear MNA system.
//!
//! A nonlinear device cannot be stamped once, because its conductance depends
//! on the voltage across it, which is what we are solving for. The loop guesses,
//! linearises every nonlinear device about that guess, solves the resulting
//! linear system, and repeats until the guess stops moving *and* the circuit
//! actually obeys KCL.
//!
//! Both of those checks matter. CircuitJS1 tests only the first — it declares
//! convergence when no device moved by more than 0.01 V — which can accept a
//! point that does not satisfy KCL (see the prior-art study, §1). Testing the residual
//! as well costs one extra stamping pass per iteration, and the pass is reused
//! as the next iteration's matrix, so it is very nearly free.
//!
//! Convergence *aids* — gmin stepping, source stepping — are C-11 and are not
//! here. This is the loop they will hang off.

use super::{Lu, MnaSystem, SolveError};

/// The companion model of a nonlinear device at one operating point: a
/// conductance in parallel with a current source.
///
/// This is the whole interface a nonlinear element has to implement. Keeping it
/// this small is what stops the solver knowing anything about diodes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Companion {
    /// Slope dI/dV at the operating point, in siemens.
    pub conductance: f64,
    /// Current the companion source pushes from the `+` node to the `-` node,
    /// in amps. It carries the part of the device current the conductance does
    /// not account for.
    pub current: f64,
    /// The voltage this companion was actually built at.
    ///
    /// Normally the voltage the device was asked about. A device that limits
    /// its own step — a pn junction damping an exponential, M-10 — reports the
    /// point it moved to instead, and the loop then knows this iteration cannot
    /// be the converged one however small the step looked.
    pub point: f64,
}

/// A two-terminal nonlinear element, seen from the solver.
pub trait Nonlinear {
    /// Linearises the device about `voltage_across`, the potential of its `+`
    /// terminal minus that of its `-` terminal.
    ///
    /// `previous_point` is where this device was last linearised, which is what
    /// a device needs to bound its own step. The loop owns that history so the
    /// device can stay a pure function of its arguments.
    fn linearise(&self, voltage_across: f64, previous_point: f64) -> Companion;
}

/// How many terminals a coupled device may have.
///
/// Six, because the 555 shows six of its pins (ADR 0012); an op-amp has five
/// (ADR 0010), and it was three while the transistor was the only coupled
/// device. A fixed size keeps the stamping
/// loop free of allocation, which matters because it runs once per device per
/// Newton iteration; a device uses the first `terminals` of each array and the
/// loop never looks past them.
pub const MAX_TERMINALS: usize = 6;

/// A device whose terminals cannot be linearised one pair at a time.
///
/// A diode, an LED and a capacitor all reduce to a conductance and a current
/// source across two nodes, which is why [`Companion`] is the interface they
/// share and why it is worth keeping that small. A transistor does not reduce
/// that way: its collector current is set by its base-emitter voltage, and a
/// model that linearised each pair of terminals independently would throw away
/// exactly the coupling that makes it a transistor.
///
/// So this is a second, wider seam rather than a generalisation of the first.
/// Making every diode pay for a multi-terminal stamp to accommodate the few
/// elements that need one is the wrong trade. The second coupled device did
/// appear — the saturating source of C-53 — and it fitted this seam as it
/// stood, which is the argument for leaving the two-terminal one alone.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CoupledCompanion {
    /// How many of the terminals are in use.
    pub terminals: usize,
    /// dI/dV over the terminals: `conductances[i][j]` is the change in the
    /// current into terminal `i` per volt at terminal `j`.
    pub conductances: [[f64; MAX_TERMINALS]; MAX_TERMINALS],
    /// Current into each terminal that the conductances do not account for.
    pub currents: [f64; MAX_TERMINALS],
    /// Where each of the device's controlling voltages was linearised, handed
    /// straight back on the next call. What they mean is the model's business.
    pub points: [f64; MAX_TERMINALS],
    /// Whether the model damped its own step.
    ///
    /// Said outright rather than inferred. A two-terminal device is asked
    /// about a voltage the loop already knows, so the loop can see for itself
    /// whether the answer came back moved; a coupled device is asked about
    /// terminal potentials and limits the junction voltages it derives from
    /// them, which the loop cannot see. Inferring it from `points` changing
    /// would call every iteration limited and convergence would never be
    /// declared.
    pub limited: bool,
}

/// A device seen through the wider seam.
pub trait Coupled {
    /// Linearises about the potentials at the device's terminals.
    ///
    /// A terminal on the reference node is passed as zero, which is what it is.
    fn linearise(
        &self,
        potentials: &[f64; MAX_TERMINALS],
        previous: &[f64; MAX_TERMINALS],
    ) -> CoupledCompanion;

    /// The points it would hand back linearised at `potentials` with nothing
    /// to bound — where a solve that starts from a known answer starts its
    /// limiting from (C-94).
    ///
    /// Zero for a model that does not say, which is where a cold start begins
    /// and where every warm start used to: a transistor already on was then
    /// limited as if its junction were leaping up from nothing, and walked
    /// back up a few tens of millivolts an iteration, every step.
    fn settled(&self, _potentials: &[f64; MAX_TERMINALS]) -> [f64; MAX_TERMINALS] {
        [0.0; MAX_TERMINALS]
    }
}

/// A coupled device placed in the matrix, one row per terminal.
pub struct CoupledDevice<'a> {
    pub rows: [Option<usize>; MAX_TERMINALS],
    pub model: &'a dyn Coupled,
}

/// A nonlinear device placed in the matrix.
pub struct Device<'a> {
    /// Matrix row of the `+` terminal, or `None` when it is the reference node.
    pub plus: Option<usize>,
    /// Matrix row of the `-` terminal, or `None` when it is the reference node.
    pub minus: Option<usize>,
    pub model: &'a dyn Nonlinear,
}

/// When to stop iterating.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NewtonOptions {
    /// Largest movement between iterations that still counts as settled, in volts.
    pub voltage_abs_tolerance: f64,
    /// Same, as a fraction of the value itself, so that a 100 V node is not held
    /// to a 1 µV standard.
    pub voltage_rel_tolerance: f64,
    /// Largest acceptable KCL error at a node, in amps.
    pub residual_tolerance: f64,
    /// A floor under that, as a fraction of the largest current in the
    /// equation.
    ///
    /// Not a relaxation — it is far below the absolute tolerance for any
    /// ordinary circuit, and changes nothing there. It exists for the other
    /// end: a circuit carrying thousands of amps cannot have its KCL checked
    /// to a picoamp, because a picoamp is nineteen digits below its own
    /// currents and a double holds sixteen. Without this the loop spends its
    /// hundred iterations refusing an answer it found on the third, and
    /// reports a convergence failure for arithmetic that had already run out
    /// of precision.
    pub residual_rel_tolerance: f64,
    /// Hard stop. A circuit that has not converged by here is not going to.
    pub max_iterations: usize,
}

impl Default for NewtonOptions {
    fn default() -> Self {
        // Roughly SPICE's defaults: 1 µV and 1 pA absolute, 0.1% relative.
        NewtonOptions {
            voltage_abs_tolerance: 1e-6,
            voltage_rel_tolerance: 1e-3,
            residual_tolerance: 1e-12,
            // About ten thousand times the machine epsilon of a double: high
            // enough above the noise to be reachable, low enough that it only
            // ever binds when the absolute tolerance has become unreachable.
            residual_rel_tolerance: 1e-12,
            max_iterations: 100,
        }
    }
}

/// A converged solve.
#[derive(Debug, Clone, PartialEq)]
pub struct NewtonSolution {
    pub x: Vec<f64>,
    pub iterations: usize,
    /// KCL error at the returned point — how wrong the answer still is.
    pub residual: f64,
}

/// Why a nonlinear solve failed.
#[derive(Debug, Clone, PartialEq)]
pub enum NewtonError {
    Solve(SolveError),
    /// Ran out of iterations. Carries the last numbers seen, because that is
    /// what tells C-11 whether the circuit was oscillating or creeping.
    NotConverged {
        iterations: usize,
        last_delta: f64,
        last_residual: f64,
    },
}

impl From<SolveError> for NewtonError {
    fn from(error: SolveError) -> Self {
        NewtonError::Solve(error)
    }
}

impl core::fmt::Display for NewtonError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            NewtonError::Solve(error) => write!(f, "{error}"),
            NewtonError::NotConverged {
                iterations,
                last_delta,
                last_residual,
            } => write!(
                f,
                "no operating point after {iterations} iterations \
                 (last step {last_delta:.3e} V, residual {last_residual:.3e} A)"
            ),
        }
    }
}

impl core::error::Error for NewtonError {}

/// Solves a system with nonlinear devices, starting from the zero state.
pub fn solve(
    base: &MnaSystem,
    devices: &[Device<'_>],
    options: &NewtonOptions,
) -> Result<NewtonSolution, NewtonError> {
    solve_from(base, devices, options, &[])
}

/// The loop with coupled devices as well as two-terminal ones.
pub fn solve_coupled(
    base: &MnaSystem,
    devices: &[Device<'_>],
    coupled: &[CoupledDevice<'_>],
    options: &NewtonOptions,
    start: &[f64],
) -> Result<NewtonSolution, NewtonError> {
    run(base, devices, coupled, options, start)
}

/// The same loop, started from a point already known to be nearby.
///
/// A cold start guesses every unknown is zero, which for a diode means every
/// junction begins fully off. C-11's stepping methods solve an easier circuit
/// first and then walk it towards the real one, and handing the previous
/// answer forward is the entire point: the easier circuit's answer is a good
/// guess for the next, and a good guess is the difference between converging
/// and not.
///
/// A `start` of the wrong length, or an empty one, means a cold start.
pub fn solve_from(
    base: &MnaSystem,
    devices: &[Device<'_>],
    options: &NewtonOptions,
    start: &[f64],
) -> Result<NewtonSolution, NewtonError> {
    run(base, devices, &[], options, start)
}

fn run(
    base: &MnaSystem,
    devices: &[Device<'_>],
    coupled: &[CoupledDevice<'_>],
    options: &NewtonOptions,
    start: &[f64],
) -> Result<NewtonSolution, NewtonError> {
    let size = base.size();
    let mut x = if start.len() == size {
        start.to_vec()
    } else {
        vec![0.0; size]
    };
    let mut a = vec![0.0; size * size];
    let mut z = vec![0.0; size];
    // Where each device was last linearised, so a device can bound its own step.
    //
    // From the start, when there is one, as SPICE bounds a step from the last
    // timepoint's answer (C-94). From zero, a warm start's first iteration
    // was limited as a leap from nothing to where the device already was: a
    // conducting transistor took fourteen iterations a step where two do.
    let warm = start.len() == size;
    let at = |row: Option<usize>| row.map_or(0.0, |row| x[row]);
    let mut points: Vec<f64> = devices
        .iter()
        .map(|device| {
            if warm {
                at(device.plus) - at(device.minus)
            } else {
                0.0
            }
        })
        .collect();
    let mut coupled_points: Vec<[f64; MAX_TERMINALS]> = coupled
        .iter()
        .map(|device| {
            if !warm {
                return [0.0; MAX_TERMINALS];
            }
            let mut potentials = [0.0; MAX_TERMINALS];
            for (potential, row) in potentials.iter_mut().zip(&device.rows) {
                *potential = at(*row);
            }
            device.model.settled(&potentials)
        })
        .collect();
    // No previous step on the first pass, so nothing can be called settled yet.
    let mut delta = f64::INFINITY;
    // Room for each iteration's factorisation and answer, reused rather than
    // allocated every time (C-94).
    let mut lu = Lu::factor(&[], 0)?;
    let mut next = Vec::with_capacity(size);

    for iteration in 0..options.max_iterations {
        let limited = stamp(
            base,
            devices,
            coupled,
            &x,
            &mut points,
            &mut coupled_points,
            &mut a,
            &mut z,
        );
        let residual = residual_norm(&a, &x, &z, size);

        if !limited
            && delta_is_settled(&x, delta, options)
            && residual
                <= options.residual_tolerance
                    + options.residual_rel_tolerance * current_scale(&a, &x, &z, size)
        {
            return Ok(NewtonSolution {
                x,
                iterations: iteration,
                residual,
            });
        }

        lu.refactor(&a, size)?;
        lu.solve_into(&z, &mut next)?;
        delta = next
            .iter()
            .zip(&x)
            .map(|(new, old)| (new - old).abs())
            .fold(0.0, f64::max);
        std::mem::swap(&mut x, &mut next);
    }

    // One last stamp so the reported residual describes the point we stopped at.
    stamp(
        base,
        devices,
        coupled,
        &x,
        &mut points,
        &mut coupled_points,
        &mut a,
        &mut z,
    );
    Err(NewtonError::NotConverged {
        iterations: options.max_iterations,
        last_delta: delta,
        last_residual: residual_norm(&a, &x, &z, size),
    })
}

/// Rebuilds the system at `x`: the linear part, plus every device linearised
/// about its present voltage.
///
/// Returns whether any device limited its own step, in which case the system was
/// built somewhere other than `x` and this iteration cannot be the answer.
#[allow(clippy::too_many_arguments)]
fn stamp(
    base: &MnaSystem,
    devices: &[Device<'_>],
    coupled: &[CoupledDevice<'_>],
    x: &[f64],
    points: &mut [f64],
    coupled_points: &mut [[f64; MAX_TERMINALS]],
    a: &mut [f64],
    z: &mut [f64],
) -> bool {
    let size = base.size();
    a.copy_from_slice(base.a());
    z.copy_from_slice(base.z());
    let mut limited = false;

    for (device, point) in devices.iter().zip(points.iter_mut()) {
        let at = |row: Option<usize>| row.map_or(0.0, |row| x[row]);
        let across = at(device.plus) - at(device.minus);
        let companion = device.model.linearise(across, *point);
        limited |= companion.point != across;
        *point = companion.point;

        if let Some(plus) = device.plus {
            a[plus * size + plus] += companion.conductance;
            z[plus] -= companion.current;
        }
        if let Some(minus) = device.minus {
            a[minus * size + minus] += companion.conductance;
            z[minus] += companion.current;
        }
        if let (Some(plus), Some(minus)) = (device.plus, device.minus) {
            a[plus * size + minus] -= companion.conductance;
            a[minus * size + plus] -= companion.conductance;
        }
    }

    for (device, previous) in coupled.iter().zip(coupled_points.iter_mut()) {
        let mut potentials = [0.0; MAX_TERMINALS];
        for (index, row) in device.rows.iter().enumerate() {
            potentials[index] = row.map_or(0.0, |row| x[row]);
        }

        let companion = device.model.linearise(&potentials, previous);
        limited |= companion.limited;
        *previous = companion.points;

        for (i, row) in device.rows.iter().take(companion.terminals).enumerate() {
            let Some(row) = *row else { continue };
            z[row] -= companion.currents[i];

            for (j, other) in device.rows.iter().take(companion.terminals).enumerate() {
                let Some(other) = *other else { continue };
                a[row * size + other] += companion.conductances[i][j];
            }
        }
    }

    limited
}

/// ‖A·x − z‖∞ — how badly the circuit fails KCL at this point.
fn residual_norm(a: &[f64], x: &[f64], z: &[f64], size: usize) -> f64 {
    (0..size)
        .map(|row| {
            let ax: f64 = (0..size).map(|col| a[row * size + col] * x[col]).sum();
            (ax - z[row]).abs()
        })
        .fold(0.0, f64::max)
}

/// The largest current the equations mention, which is what a KCL error has to
/// be judged against.
fn current_scale(a: &[f64], x: &[f64], z: &[f64], size: usize) -> f64 {
    let largest_z = z.iter().fold(0.0f64, |most, value| most.max(value.abs()));
    (0..size)
        .map(|row| {
            (0..size)
                .map(|col| (a[row * size + col] * x[col]).abs())
                .fold(0.0f64, f64::max)
        })
        .fold(largest_z, f64::max)
}

/// Whether the last step was small enough, per unknown, to call the guess settled.
fn delta_is_settled(x: &[f64], delta: f64, options: &NewtonOptions) -> bool {
    if !delta.is_finite() {
        return false;
    }
    let largest = x.iter().map(|value| value.abs()).fold(0.0, f64::max);
    delta <= options.voltage_abs_tolerance + options.voltage_rel_tolerance * largest
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::netlist::{ElementKind, Netlist};

    /// A device obeying i = k·v³. Not a real component — it is the simplest
    /// nonlinearity with an exactly known answer, which is what a framework
    /// test wants.
    struct CubicDevice {
        k: f64,
    }

    impl Nonlinear for CubicDevice {
        fn linearise(&self, v: f64, _previous: f64) -> Companion {
            let i = self.k * v.powi(3);
            let g = 3.0 * self.k * v * v;
            Companion {
                conductance: g,
                // The companion source carries what the linear part over-counts.
                current: i - g * v,
                point: v,
            }
        }
    }

    /// A device obeying i = v, but reporting a slope a hundred times too steep.
    ///
    /// Overstating the slope does not move the answer — the companion source
    /// absorbs the difference, so the fixed point is still the true one — but it
    /// turns Newton into a chord method that creeps towards it by about 2% per
    /// iteration. That is the regime where steps look settled long before the
    /// circuit obeys KCL.
    struct CreepingDevice;

    impl Nonlinear for CreepingDevice {
        fn linearise(&self, v: f64, _previous: f64) -> Companion {
            const OVERSTATED_SLOPE: f64 = 100.0;
            Companion {
                conductance: OVERSTATED_SLOPE,
                current: v - OVERSTATED_SLOPE * v,
                point: v,
            }
        }
    }

    /// A device that flips sign every time it is asked, so the guess never settles.
    struct OscillatingDevice;

    impl Nonlinear for OscillatingDevice {
        fn linearise(&self, v: f64, _previous: f64) -> Companion {
            Companion {
                conductance: 1e-3,
                current: if v > 0.0 { 1.0 } else { -1.0 },
                point: v,
            }
        }
    }

    /// 1 V source through a 1 Ω resistor into the device under test.
    fn one_volt_through_one_ohm() -> (Netlist, crate::netlist::NetId) {
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts: 1.0,
            series_ohms: 0.0,
        });
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: 1.0 });

        let top = netlist.add_net();
        let middle = netlist.add_net();
        let bottom = netlist.add_net();

        netlist.connect_named(source, "+", top).unwrap();
        netlist.connect_named(resistor, "a", top).unwrap();
        netlist.connect_named(resistor, "b", middle).unwrap();
        netlist.connect_named(source, "-", bottom).unwrap();

        (netlist, middle)
    }

    /// Builds the base system and the row of the node the device hangs from.
    fn base_system(netlist: &Netlist, node: crate::netlist::NetId) -> (MnaSystem, usize) {
        let map = netlist.compile().unwrap();
        let system = crate::solver::assemble(netlist, &map);
        let row = system
            .row_of_node(map.index_of(node).unwrap())
            .expect("the device node is not the reference");
        (system, row)
    }

    #[test]
    fn finds_the_root_of_a_nonlinear_circuit() {
        // KCL at the middle node: v³ = (1 − v)/1, i.e. v³ + v − 1 = 0.
        // Root computed independently to 16 digits.
        const ROOT: f64 = 0.6823278038280193;

        let (netlist, middle) = one_volt_through_one_ohm();
        let (system, row) = base_system(&netlist, middle);
        let device = CubicDevice { k: 1.0 };

        let solution = solve(
            &system,
            &[Device {
                plus: Some(row),
                minus: None,
                model: &device,
            }],
            &NewtonOptions::default(),
        )
        .unwrap();

        assert!((solution.x[row] - ROOT).abs() < 1e-9);
        assert!(solution.residual <= NewtonOptions::default().residual_tolerance);
        assert!(solution.iterations > 1, "a nonlinear solve takes steps");
    }

    #[test]
    fn a_circuit_with_no_nonlinear_devices_converges_immediately() {
        let (netlist, middle) = one_volt_through_one_ohm();
        let (system, _) = base_system(&netlist, middle);

        let solution = solve(&system, &[], &NewtonOptions::default()).unwrap();

        // Nothing to linearise, so the first solve is already the answer: one
        // step to find it, one pass to confirm it.
        assert_eq!(solution.iterations, 2);
        assert!(solution.residual < 1e-12);
    }

    #[test]
    fn a_small_step_is_not_enough_on_its_own() {
        // The creeping device settles to within a millivolt per step while
        // still a tenth of a volt away from the answer. Testing Δv alone, as
        // CircuitJS1 does, would call this converged and report a wrong circuit
        // with no indication anything went wrong.
        let (netlist, middle) = one_volt_through_one_ohm();
        let (system, row) = base_system(&netlist, middle);

        let options = NewtonOptions {
            // Lax on movement, strict on physics.
            voltage_abs_tolerance: 1e-3,
            voltage_rel_tolerance: 1e-3,
            max_iterations: 150,
            ..NewtonOptions::default()
        };
        let error = solve(
            &system,
            &[Device {
                plus: Some(row),
                minus: None,
                model: &CreepingDevice,
            }],
            &options,
        )
        .unwrap_err();

        match error {
            NewtonError::NotConverged {
                last_delta,
                last_residual,
                ..
            } => {
                assert!(
                    last_delta <= options.voltage_abs_tolerance,
                    "steps did settle: {last_delta:e}"
                );
                assert!(
                    last_residual > options.residual_tolerance,
                    "KCL is still violated: {last_residual:e}"
                );
            }
            other => panic!("expected NotConverged, got {other:?}"),
        }
    }

    #[test]
    fn a_circuit_that_oscillates_stops_at_the_cap() {
        let (netlist, middle) = one_volt_through_one_ohm();
        let (system, row) = base_system(&netlist, middle);

        let options = NewtonOptions {
            max_iterations: 25,
            ..NewtonOptions::default()
        };
        let error = solve(
            &system,
            &[Device {
                plus: Some(row),
                minus: None,
                model: &OscillatingDevice,
            }],
            &options,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            NewtonError::NotConverged { iterations: 25, .. }
        ));
    }

    #[test]
    fn the_failure_carries_something_to_act_on() {
        let (netlist, middle) = one_volt_through_one_ohm();
        let (system, row) = base_system(&netlist, middle);

        let error = solve(
            &system,
            &[Device {
                plus: Some(row),
                minus: None,
                model: &OscillatingDevice,
            }],
            &NewtonOptions {
                max_iterations: 10,
                ..NewtonOptions::default()
            },
        )
        .unwrap_err();

        // C-11 needs to tell creeping apart from oscillating, and C-25 needs
        // something better to say than "Convergence failed!".
        let message = error.to_string();
        assert!(message.contains("10 iterations"), "{message}");
        assert!(message.contains("residual"), "{message}");
    }

    #[test]
    fn a_stiffer_nonlinearity_still_lands_on_its_root() {
        // k = 1000: v satisfies 1000·v³ + v − 1 = 0, a much sharper curve.
        let (netlist, middle) = one_volt_through_one_ohm();
        let (system, row) = base_system(&netlist, middle);
        let device = CubicDevice { k: 1_000.0 };

        let solution = solve(
            &system,
            &[Device {
                plus: Some(row),
                minus: None,
                model: &device,
            }],
            &NewtonOptions::default(),
        )
        .unwrap();

        let v = solution.x[row];
        assert!((1_000.0 * v.powi(3) + v - 1.0).abs() < 1e-9, "v = {v}");
    }
}
