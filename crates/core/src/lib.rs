// SPDX-License-Identifier: MIT OR Apache-2.0

//! Core of the simulator: netlist, solver and linter.
//!
//! Nothing in here may reach for the canvas or the DOM. The wasm boundary is a
//! thin layer over an API that also works natively (design doc 7.1).

pub mod avr;
mod boundary;
pub mod breadboard;
pub mod build;
pub mod catalogue;
pub mod check;
pub mod circuit;
pub use circuit::{
    Circuit, CircuitError, ElementCurrent, ElementView, ElementViews, Failure, NetView, NetViews,
    NetVoltage, PinView, SolutionView,
};

pub mod document;
pub mod engine;
mod json;
pub mod kicad;
pub mod lint;
pub mod netlist;
pub mod registry;
pub mod replace;
pub mod scheduler;
pub mod scope;
pub mod serial;
pub mod series;
pub mod solver;
pub mod value;

pub use boundary::{
    BoundaryProbe, ComponentView, ComponentViews, SolverFrame, components, echo_probe, pin_names,
    probe_with_validation,
};

use wasm_bindgen::prelude::wasm_bindgen;

/// Runs as the wasm module is instantiated, before anything else.
///
/// A Rust panic across the wasm boundary arrives in the console as
/// `unreachable executed`: no message, no location, and nothing to tell a
/// dropped invariant from a bug in the browser. The hook replaces that with
/// the panic's own message and the file and line it came from (C-31).
///
/// Every build, not just debug ones. C-31 proposed debug only, on the
/// reasoning that the hook drags in the formatting machinery panics need and
/// the bundle is on the cold-load path M-21 budgets three seconds for. Measured,
/// it costs 1,709 bytes of 284 kB — 0.6%, against a cold load of 120 ms — and
/// what it buys is that the build people actually run is the one that can say
/// what went wrong. A hook in debug builds alone also means the app tests, which
/// run against the release bundle, could never cover it.
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// Panics on purpose, so the hook above can be shown to work.
///
/// Testing a panic hook needs a panic; the alternative is a test that asserts
/// the hook was installed without ever asking what it does. It ships, because
/// a test against a build nobody runs is worth less than the few bytes this
/// costs — and nothing in the app calls it.
#[wasm_bindgen]
pub fn provoke_panic() {
    panic!("a deliberate panic, to prove the hook reports one");
}

/// Version of the core, as reported to the shell. Exists so the scaffold can
/// prove the bundle is really loaded rather than mocked away.
#[wasm_bindgen]
pub fn core_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// What this simulation is, in the words the app says it in (C-26).
///
/// A claim about the *solver* rather than about a screen, which is why it is
/// written here and not in the shell: the v1 core is educational by decision
/// (design doc §3), and the sentence is only true because of what is in this
/// crate. Everything that repeats it — the status bar, the file a circuit is
/// saved to — quotes this, and a test holds the app's own copy to it, the way
/// ADR 0008 holds every other fact the two share.
///
/// The app keeps a copy rather than asking at run time on purpose. The one
/// sentence that must always be on screen cannot be the one that waits for a
/// wasm bundle to load, or that goes missing precisely when the core failed.
pub const ACCURACY: &str = "Educational simulation — not accurate enough to design from";

/// [`ACCURACY`], for the shell's test to check its copy against.
#[wasm_bindgen]
pub fn accuracy_note() -> String {
    ACCURACY.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_its_crate_version() {
        assert_eq!(core_version(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn says_what_the_simulation_is() {
        // Said once, here, so that the screen and the file cannot come to
        // disagree about it (C-26).
        assert_eq!(accuracy_note(), ACCURACY);
        assert!(ACCURACY.contains("not accurate enough to design from"));
    }
}
