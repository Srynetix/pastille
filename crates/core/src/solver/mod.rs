// SPDX-License-Identifier: MIT OR Apache-2.0

//! The solver.
//!
//! The app never names anything in here directly: it goes through the solver
//! interface of M-12, so that ngspice or another engine can take over later
//! (design doc 7.1.2).

mod dc;
pub mod device;
pub mod heat;
mod integration;
mod lu;
mod mna;
mod newton;
mod transient;

pub(crate) use dc::pin_currents_of;
pub use dc::{DcError, OperatingPoint, operating_point, operating_point_near, step};
pub use integration::{Charge, Integration};
pub use lu::{Lu, SolveError, solve};
pub(crate) use mna::END_RESISTANCE;
pub use mna::{
    CoupledPlacement, MnaSystem, NonlinearPlacement, Timestep, Unstamped, UnstampedReason,
    assemble, assemble_at,
};
pub use newton::{
    Companion, Coupled, CoupledCompanion, CoupledDevice, Device, MAX_TERMINALS, NewtonError,
    NewtonOptions, NewtonSolution, Nonlinear, solve as solve_nonlinear,
    solve_coupled as solve_nonlinear_coupled, solve_from as solve_nonlinear_from,
};
pub use transient::{
    AdaptiveOptions, InitialCondition, Stepper, Transient, TransientError, TransientOptions,
    adaptive, transient,
};
