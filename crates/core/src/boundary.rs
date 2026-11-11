// SPDX-License-Identifier: MIT OR Apache-2.0

//! The wasm boundary, and the two ways a value is allowed to cross it.
//!
//! Traffic splits by rate, not by type (see `docs/adrs/0002-wasm-boundary-serialization.md`):
//!
//! - **Control plane** — netlist edits, component values, save/load. Rare,
//!   structured, ergonomics matter. Crosses as a JS object via serde.
//! - **Data plane** — node voltages and branch currents, read every frame.
//!   Hot, uniform, allocation-free. Crosses as a view over wasm memory.
//!
//! What lives here now is the thinnest example of each, enough to pin the
//! mechanism down before the netlist (M-03) and the solver (M-06) arrive.

use serde::{Deserialize, Serialize};
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::{JsError, wasm_bindgen};

/// A structured value crossing the control plane. Stands in for the netlist
/// payloads of M-03 onwards; the shape is deliberately boring.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct BoundaryProbe {
    pub label: String,
    pub node_voltages: Vec<f64>,
}

/// Takes a structured value from JS and hands back one built in Rust.
///
/// The returned label is rewritten and the voltages are scaled, so a caller
/// cannot pass the test by receiving the object it sent: the value really was
/// deserialized, reconstructed and serialized again.
///
/// Values cross wrapped in [`Ts`], which carries the JS handle across the ABI
/// and converts explicitly. The older `into_wasm_abi`/`from_wasm_abi` attributes
/// were simpler to write and leaked memory on every call — not something a live
/// editing loop (M-19) can afford.
#[wasm_bindgen]
pub fn echo_probe(probe: Ts<BoundaryProbe>, gain: f64) -> Result<Ts<BoundaryProbe>, JsError> {
    Ok(echo(&probe.to_rust()?, gain).into_ts()?)
}

/// Fails the way every fallible core call will fail: a typed error on the Rust
/// side becomes a thrown `Error` on the JS side, never a panic and never a
/// sentinel value.
#[wasm_bindgen]
pub fn probe_with_validation(probe: Ts<BoundaryProbe>) -> Result<Ts<BoundaryProbe>, JsError> {
    let probe = probe.to_rust()?;
    if probe.node_voltages.is_empty() {
        return Err(JsError::new("a probe needs at least one node voltage"));
    }
    Ok(echo(&probe, 1.0).into_ts()?)
}

/// The rebuild itself, kept out of the boundary functions so the native tests
/// can exercise it without a JS engine.
fn echo(probe: &BoundaryProbe, gain: f64) -> BoundaryProbe {
    BoundaryProbe {
        label: format!("{} (via rust)", probe.label),
        node_voltages: probe.node_voltages.iter().map(|v| v * gain).collect(),
    }
}

/// One frame of solver output, living in wasm memory so JS can read it without
/// a copy. Placeholder contents until the real solver lands at M-08.
#[wasm_bindgen]
pub struct SolverFrame {
    voltages: Vec<f64>,
}

#[wasm_bindgen]
impl SolverFrame {
    /// Allocates a frame of `node_count` voltages, each node `i` sitting at
    /// `i * step` volts.
    #[wasm_bindgen(constructor)]
    pub fn new(node_count: usize, step: f64) -> SolverFrame {
        SolverFrame {
            voltages: (0..node_count).map(|i| i as f64 * step).collect(),
        }
    }

    /// Start of the voltage block inside wasm linear memory.
    ///
    /// Any allocation in wasm can grow the memory and detach every existing
    /// `ArrayBuffer` view over it, so this is read once per use and never
    /// cached across calls.
    #[wasm_bindgen(getter)]
    pub fn ptr(&self) -> *const f64 {
        self.voltages.as_ptr()
    }

    #[wasm_bindgen(getter)]
    pub fn len(&self) -> usize {
        self.voltages.len()
    }

    #[wasm_bindgen(getter)]
    pub fn is_empty(&self) -> bool {
        self.voltages.is_empty()
    }

    /// Mutates the block in place — the whole point of the data plane is that
    /// a new frame costs no allocation and no serialization.
    pub fn shift(&mut self, delta: f64) {
        for v in &mut self.voltages {
            *v += delta;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn echo_rebuilds_the_value_rather_than_returning_it() {
        let sent = BoundaryProbe {
            label: "divider".into(),
            node_voltages: vec![9.0, 4.5],
        };
        let back = echo(&sent, 2.0);

        assert_eq!(back.label, "divider (via rust)");
        assert_eq!(back.node_voltages, vec![18.0, 9.0]);
        assert_ne!(back, sent);
    }

    #[test]
    fn a_frame_is_mutated_in_place() {
        let mut frame = SolverFrame::new(3, 1.5);
        assert_eq!(frame.voltages, vec![0.0, 1.5, 3.0]);

        frame.shift(0.5);
        assert_eq!(frame.voltages, vec![0.5, 2.0, 3.5]);
        assert_eq!(frame.len(), 3);
    }
}

/// Pin names of an element kind, in pin order.
///
/// The app draws symbols and the core wires netlists, and both have to agree on
/// what a pin is called. Rather than keeping a second copy of the table in
/// TypeScript and hoping, the app asks — so "Anode" instead of "anode" is a
/// failing test rather than a wire that silently never connects.
#[wasm_bindgen]
pub fn pin_names(kind: &str) -> Result<Vec<String>, JsError> {
    crate::registry::definition(kind)
        .map(|component| {
            component
                .pins
                .iter()
                .map(|&name| name.to_string())
                .collect()
        })
        .ok_or_else(|| JsError::new(&format!("unknown element kind: {kind}")))
}

/// One kind of component, as the app needs to know it.
///
/// Everything electrical and nothing else: what it is drawn like is
/// `symbols.ts` and what it is called is `palette.ts`, and ADR 0008 says why.
/// The app holds its own copies of both and proves they agree with this.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ComponentView {
    #[tsify(type = "ComponentKind")]
    pub kind: String,
    pub pins: Vec<String>,
    /// Unit of the editable value, empty for a component that has none.
    pub unit: String,
    /// Whether v1's palette offers it.
    pub offered: bool,
    /// Whether the app is expected to be able to draw one.
    pub drawable: bool,
    /// The alternatives one can be set to, empty for a kind with none (C-46).
    pub choices: Vec<String>,
    /// Largest current it is meant to carry, in amps, if that is a limit it
    /// has. B-03 is the rule that will read it.
    pub rated_current: Option<f64>,
    /// Largest power it is meant to dissipate, in watts, likewise.
    pub rated_power: Option<f64>,
}

/// Every kind the core knows, in the order the palette should offer them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(transparent)]
pub struct ComponentViews(pub Vec<ComponentView>);

/// The component registry, for the app to hold itself to (ADR 0008).
#[wasm_bindgen]
pub fn components() -> Ts<ComponentViews> {
    let views: Vec<ComponentView> = crate::registry::REGISTRY
        .iter()
        .map(|component| ComponentView {
            kind: component.kind.to_string(),
            pins: component
                .pins
                .iter()
                .map(|&name| name.to_string())
                .collect(),
            unit: component.unit.to_string(),
            offered: component.offered,
            drawable: component.drawable,
            choices: (component.choices)()
                .into_iter()
                .map(str::to_string)
                .collect(),
            // A fresh one's, since this describes the kind: a regulator's
            // limits are its default part's until somebody chooses another.
            rated_current: (component.rating)(&(component.blank)()).current,
            rated_power: (component.rating)(&(component.blank)()).power,
        })
        .collect();

    Ts::from_rust(&ComponentViews(views)).expect("component views serialize")
}
