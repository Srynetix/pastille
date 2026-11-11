// SPDX-License-Identifier: MIT OR Apache-2.0

//! The comparator: which of two voltages is higher, as an output that is
//! either pulled down or let go.
//!
//! The third part built on the saturating source of C-53 (ADR 0010), and the
//! one that tests it hardest: open loop at a gain of 2e5, so the output crosses
//! from one state to the other while the inputs move by fifty microvolts.
//!
//! What it honours: the threshold at the inputs' crossing; an open-collector
//! output, which sinks current and never sources it — so something outside has
//! to pull it up; the saturation voltage it pulls down to. What it ignores:
//! propagation delay, since nothing here has a past; hysteresis, which the part
//! has none of and a circuit adds with a resistor; and a pull-up to a supply
//! higher than its own V+, which the real part tolerates and this one does not.
//! An output with nothing at all to pull it up reads as V+, where a real one
//! would be floating: a number is the least wrong thing to show for a pin that
//! has none, and B-20 is what will say the pull-up is missing.
//!
//! One part, the LM393, the comparator beside the LM358 in the same kits and on
//! the same single supply. Its pins are the op-amp's, in the op-amp's order.

use super::SaturatingSource;

/// The part it is, for the label and, later, the BOM (ADR 0010, term 3).
pub const PART: &str = "LM393";

/// Most the supply may be across it, in volts.
pub const MAX_SUPPLY_VOLTS: f64 = 36.0;

/// Most current its output will sink, in amps: the datasheet's typical figure.
pub const SINK_AMPS: f64 = 16e-3;

/// An LM393, one of its two comparators.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Comparator;

impl Comparator {
    /// The saturating source that behaves like this part. Its pins are the
    /// source's terminals, in order: in+, in−, out, V+, V−.
    pub fn model(&self) -> SaturatingSource {
        SaturatingSource {
            // 200 V/mV, the datasheet's typical voltage gain.
            gain: 2e5,
            offset: 0.0,
            reference: 0.5,
            // Let go, the output is wherever its pull-up puts it; the target
            // sits at V+ so that a pull-up to the same battery sees nothing
            // pushing back.
            headroom_high: 0.0,
            // Pulled down, it sits a tenth of a volt above V− plus what its
            // sink current drops across the output transistor — about a
            // quarter of a volt at 4 mA, which is the datasheet's figure.
            headroom_low: 0.1,
            output_ohms: 40.0,
            // Open collector: it sources nothing. A microamp is the most its
            // datasheet allows to leak, and it keeps the corner at zero that a
            // limit of exactly nothing would be (C-53).
            source_limit: 1e-6,
            sink_limit: SINK_AMPS,
            // About 0.4 mA for the package of two.
            quiescent: 0.2e-3,
            input_ohms: 10e6,
        }
    }
}
