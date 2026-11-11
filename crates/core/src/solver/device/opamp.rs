// SPDX-License-Identifier: MIT OR Apache-2.0

//! The operational amplifier: a difference between two inputs, made big.
//!
//! The second part built on the saturating source of C-53, and the one it was
//! shaped for: five pins that are its five terminals, in the same order, so the
//! wiring is the identity. Modelled from the datasheet (ADR 0010). What it
//! honours: open-loop gain; an output that stops short of each rail by what the
//! part promises; input and output resistance; how much current the output can
//! push and pull. What it ignores: bandwidth and slew rate, so it answers
//! instantly; offset voltage and bias current; the common-mode range, which
//! B-20 checks for rather than the model enforcing.
//!
//! One part, the LM358: single-supply, so it runs off one 9 V battery with its
//! V− on the negative terminal, and its output reaches almost down to that
//! terminal — the op-amp in every beginner's kit because of both.

use super::SaturatingSource;

/// The part it is, for the label and, later, the BOM (ADR 0010, term 3).
pub const PART: &str = "LM358";

/// Most the supply may be across it, in volts.
pub const MAX_SUPPLY_VOLTS: f64 = 32.0;

/// An LM358, one of its two amplifiers.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct OpAmp;

impl OpAmp {
    /// The saturating source that behaves like this part. Its pins are the
    /// source's terminals, in order: in+, in−, out, V+, V−.
    pub fn model(&self) -> SaturatingSource {
        SaturatingSource {
            // 100 V/mV, the datasheet's typical large-signal gain.
            gain: 1e5,
            offset: 0.0,
            // With its inputs equal it sits half way between its rails.
            reference: 0.5,
            // It gets within a volt and a half of V+ and, lightly loaded,
            // within a few tens of millivolts of V−: the asymmetry that makes
            // it a single-supply part.
            headroom_high: 1.5,
            headroom_low: 0.02,
            output_ohms: 50.0,
            source_limit: 30e-3,
            sink_limit: 20e-3,
            // Its datasheet gives about 0.7 mA for the package of two.
            quiescent: 0.35e-3,
            input_ohms: 10e6,
        }
    }
}
