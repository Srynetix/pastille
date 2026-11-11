// SPDX-License-Identifier: MIT OR Apache-2.0

//! A voltage-controlled source that cannot leave its supply rails.
//!
//! The one primitive ADR 0010 builds every integrated circuit on. A regulator,
//! an op-amp and a comparator are each, seen from their pins, the same three
//! promises: an output that tries to sit at a target set by the inputs, that
//! cannot go beyond what its supply pins allow, and that can only push or pull
//! so much current. The parts differ in their numbers, not in their equations
//! (C-53).
//!
//! The output is a Norton source rather than an ideal voltage source: the
//! target behind a small output resistance. That keeps the whole device on the
//! node rows of the matrix — no branch unknown of its own, so it goes through
//! the [`Coupled`] seam the transistor already uses — and an output resistance
//! is one of the figures the ADR says an op-amp honours anyway.
//!
//! The current the output delivers is drawn from the high rail and returned to
//! the low one, so a battery feeding a regulator sees the regulator's load. A
//! model that took it from ground would conserve charge and still lie about
//! where the energy came from.
//!
//! ## What it honours, and what it ignores
//!
//! Honours: gain; a target placed between the rails; headroom at each rail;
//! output resistance; a source and a sink current limit, separately;
//! quiescent current between the rails; input resistance.
//!
//! Ignores: bandwidth, slew rate and every other effect of time — it has no
//! state, so it responds instantly; offset and bias currents; noise;
//! temperature. A circuit that depends on one of those simulates wrongly, and
//! this list is where to find out why (ADR 0010, consequence 5).
//!
//! ## Why it converges
//!
//! Every function in it is smooth. A hard `min` against a rail has a corner,
//! and a corner is where Newton oscillates, so the clamps are hyperbolic —
//! identical to the hard clamp a few [`SMOOTHING`] widths away from the rail,
//! and differentiable everywhere.
//!
//! Smoothness is not enough on its own at a gain of 1e5. An amplifier in
//! feedback, linearised from a point where its output sits on one rail, sees a
//! device with almost no gain; the linear solve then puts the inputs where the
//! output would have to be on the *other* rail, and the next iteration does the
//! same thing back. So the model bounds its own step, as a junction does
//! (M-10): the first time an iteration would throw the target past a rail, it
//! is linearised just inside that rail instead, where the gain is whole and one
//! more solve lands in the linear region if the answer is there. Only a second
//! consecutive push past the same rail is believed, and that is saturation.

use crate::solver::{Coupled, CoupledCompanion, MAX_TERMINALS};

/// Terminal order.
pub const NON_INVERTING: usize = 0;
pub const INVERTING: usize = 1;
pub const OUTPUT: usize = 2;
/// The supply the output current is drawn from when it sources.
pub const HIGH_RAIL: usize = 3;
/// The supply the output current returns to when it sinks.
pub const LOW_RAIL: usize = 4;

/// How many terminals the device uses.
const TERMINALS: usize = 5;

/// Width of the rounding at each rail, in volts.
///
/// A millivolt: small enough that the clamp is a hard clamp to four figures
/// anywhere a reading is taken, wide enough that the derivative near a rail is
/// a number rather than a cliff.
pub const SMOOTHING: f64 = 1e-3;

/// Width of the handover between sourcing from one rail and sinking into the
/// other, in amps.
///
/// Which rail the output current comes from switches when the current changes
/// sign, and a switch is a corner. A microamp of blend makes it a curve, and is
/// far below any current a reading would show.
const HANDOVER: f64 = 1e-6;

/// Supply voltage at which the quiescent current has reached its full value.
///
/// A real part draws almost nothing until its supply is high enough to bias
/// it. Without this knee, a device on an unpowered supply would push its
/// quiescent current into nodes nothing else holds, and drive them wherever
/// that current took them.
const QUIESCENT_KNEE: f64 = 1.0;

/// How far inside a rail a bounded step lands, as a fraction of the room
/// between the rails.
const BOUNDED_STEP_INSET: f64 = 0.01;

/// The state a bounded step leaves behind, in the second point slot.
const PUSHED_HIGH: f64 = 1.0;
const PUSHED_LOW: f64 = -1.0;

/// A saturating voltage-controlled source.
///
/// The target is
///
/// ```text
/// T = V(low) + reference · (V(high) − V(low)) + offset + gain · (V(+) − V(−))
/// ```
///
/// held between `V(low) + headroom_low` and `V(high) − headroom_high`, and
/// driven onto the output through `output_ohms`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SaturatingSource {
    /// Volts at the output per volt between the inputs. Zero for a part whose
    /// target does not depend on its inputs, which is what a regulator is.
    pub gain: f64,
    /// Volts added to the target, measured from the reference point.
    pub offset: f64,
    /// Where between the rails the target is measured from: 0 is the low rail,
    /// 1 the high one, ½ half way. An op-amp's output with its inputs equal
    /// sits half way; a regulator's is measured from its ground pin.
    pub reference: f64,
    /// How close to the high rail the output can get, in volts.
    pub headroom_high: f64,
    /// How close to the low rail the output can get, in volts.
    pub headroom_low: f64,
    /// Resistance the target is driven through, in ohms. Must be positive.
    pub output_ohms: f64,
    /// Most current the output can push into the circuit, in amps.
    pub source_limit: f64,
    /// Most current the output can pull out of it, in amps.
    ///
    /// Small rather than zero for a part that cannot sink, like a regulator: a
    /// limit of exactly nothing is a corner at zero current, and an output
    /// with no load sits precisely there.
    pub sink_limit: f64,
    /// Current from the high rail to the low one, in amps, once powered.
    pub quiescent: f64,
    /// Resistance between the two inputs, in ohms. Infinite for none.
    pub input_ohms: f64,
}

/// A value and how it changes with each terminal's potential.
#[derive(Debug, Clone, Copy)]
struct Dual {
    value: f64,
    slope: [f64; TERMINALS],
}

impl Dual {
    #[inline]
    fn constant(value: f64) -> Self {
        Dual {
            value,
            slope: [0.0; TERMINALS],
        }
    }

    #[inline]
    fn terminal(potentials: &[f64; MAX_TERMINALS], index: usize) -> Self {
        let mut slope = [0.0; TERMINALS];
        slope[index] = 1.0;
        Dual {
            value: potentials[index],
            slope,
        }
    }

    #[inline]
    fn add(self, other: Dual) -> Dual {
        let mut slope = self.slope;
        for (mine, theirs) in slope.iter_mut().zip(other.slope) {
            *mine += theirs;
        }
        Dual {
            value: self.value + other.value,
            slope,
        }
    }

    #[inline]
    fn scale(self, factor: f64) -> Dual {
        Dual {
            value: self.value * factor,
            slope: self.slope.map(|entry| entry * factor),
        }
    }

    #[inline]
    fn sub(self, other: Dual) -> Dual {
        self.add(other.scale(-1.0))
    }

    /// `f(self)`, given `f` and its derivative at this value.
    #[inline]
    fn chain(self, value: f64, derivative: f64) -> Dual {
        Dual {
            value,
            slope: self.slope.map(|entry| entry * derivative),
        }
    }

    /// The product of two duals.
    #[inline]
    fn mul(self, other: Dual) -> Dual {
        let mut slope = [0.0; TERMINALS];
        for (index, entry) in slope.iter_mut().enumerate() {
            *entry = self.slope[index] * other.value + other.slope[index] * self.value;
        }
        Dual {
            value: self.value * other.value,
            slope,
        }
    }
}

/// The smaller of two values, rounded over [`SMOOTHING`].
///
/// `(a + b − √((a − b)² + ε²)) / 2`, written so it never subtracts two large
/// numbers to find a small one. An amplifier thrown into saturation asks for a
/// target of a hundred thousand volts and gets a rail of nine; the textbook
/// form finds the nine by cancelling the hundred thousand, and loses the digits
/// Newton needs to the rounding of it.
#[inline]
fn soft_min(a: Dual, b: Dual) -> Dual {
    let gap = a.value - b.value;
    let root = (gap * gap + SMOOTHING * SMOOTHING).sqrt();
    // √(g² + ε²) − |g|, without the cancellation. The same quantity decides
    // how far the slope has handed over from one argument to the other.
    let rounding = SMOOTHING * SMOOTHING / (root + gap.abs());
    let lesser = rounding / (2.0 * root);
    let (toward_a, toward_b) = if gap > 0.0 {
        (lesser, 1.0 - lesser)
    } else {
        (1.0 - lesser, lesser)
    };
    let mut slope = [0.0; TERMINALS];
    for (index, entry) in slope.iter_mut().enumerate() {
        *entry = toward_a * a.slope[index] + toward_b * b.slope[index];
    }
    Dual {
        value: a.value.min(b.value) - rounding / 2.0,
        slope,
    }
}

/// The larger of two values, rounded over [`SMOOTHING`].
#[inline]
fn soft_max(a: Dual, b: Dual) -> Dual {
    soft_min(a.scale(-1.0), b.scale(-1.0)).scale(-1.0)
}

/// `x`, bent towards `limit` as it grows: `x / √(1 + (x / limit)²)`.
///
/// Slope one at zero, so an output well inside its limits is a plain
/// resistance, and a slope that falls off as a power rather than an
/// exponential, so Newton linearising a limited output still sees which way
/// is downhill. Returns the value and its derivative.
#[inline]
fn bend(x: f64, limit: f64) -> (f64, f64) {
    if limit <= 0.0 {
        return (0.0, 0.0);
    }
    let ratio = x / limit;
    let spread = 1.0 + ratio * ratio;
    (x / spread.sqrt(), spread.powf(-1.5))
}

/// Where to linearise a quantity Newton may throw past one of its limits.
///
/// `proposed` is where this iteration would put it, `was` where it was put
/// last time and `pushed` what the last call returned as its state. Inside
/// `window` nothing is held back. The first time it is thrown past an edge it
/// is placed at that edge's `landing` instead, inside the window, where the
/// model's slope is whole; thrown past the same edge again, or from a point
/// already beyond it, it is believed.
///
/// Returns where to linearise, or `None` to linearise at `proposed`, with the
/// state to remember for next time.
fn bound(
    proposed: f64,
    was: f64,
    pushed: f64,
    (low, high): (f64, f64),
    (land_low, land_high): (f64, f64),
) -> (Option<f64>, f64) {
    if proposed > high && !(pushed == PUSHED_HIGH || was > high) {
        (Some(land_high), PUSHED_HIGH)
    } else if proposed < low && !(pushed == PUSHED_LOW || was < low) {
        (Some(land_low), PUSHED_LOW)
    } else {
        (None, 0.0)
    }
}

impl SaturatingSource {
    /// The input voltage to linearise at: the gain can throw the target past
    /// either rail. Returns it with the state to remember.
    fn bounded_input(
        &self,
        raw: f64,
        base: f64,
        low: f64,
        high: f64,
        previous: &[f64],
    ) -> (f64, f64) {
        // A part with no gain has nothing to throw, and an unpowered one has
        // no rails to throw it past.
        if self.gain == 0.0 || high <= low {
            return (raw, 0.0);
        }
        let target = |input: f64| base + self.gain * input;
        let inset = (BOUNDED_STEP_INSET * (high - low)).clamp(10.0 * SMOOTHING, (high - low) / 2.0);
        let (landed, pushed) = bound(
            target(raw),
            target(previous[0]),
            previous[1],
            (low, high),
            (low + inset, high - inset),
        );
        let input = landed.map_or(raw, |target| (target - base) / self.gain);
        (input, pushed)
    }

    /// The output current to linearise at: the difference between the target
    /// and the output can ask for far more than the part will deliver, and a
    /// limited output linearised deep in its limit is a current source with
    /// no idea where its load wants it. Returns it with the state to remember.
    fn bounded_output(&self, raw: f64, previous: &[f64]) -> (f64, f64) {
        let (landed, pushed) = bound(
            raw,
            previous[2],
            previous[3],
            (-self.sink_limit, self.source_limit),
            // Half the limit: well inside it, where the bend has most of its
            // slope still, so one more solve lands near the load's answer.
            (-self.sink_limit / 2.0, self.source_limit / 2.0),
        );
        (landed.unwrap_or(raw), pushed)
    }
}

impl SaturatingSource {
    /// Current into each terminal at these potentials, taken as they are.
    ///
    /// What the device is actually doing at a solved point, rather than where
    /// Newton should linearise next: linearised from its [`Coupled::settled`]
    /// history, under which neither bound holds anything back. Used to ask
    /// what a part dissipates (C-74), which is the sum of each terminal's
    /// potential times what enters there.
    pub fn currents(&self, potentials: &[f64; MAX_TERMINALS]) -> [f64; MAX_TERMINALS] {
        let companion = self.linearise(potentials, &self.settled(potentials));
        let mut into = [0.0; MAX_TERMINALS];
        for (i, current) in into.iter_mut().enumerate().take(TERMINALS) {
            let conducted: f64 = (0..TERMINALS)
                .map(|j| companion.conductances[i][j] * potentials[j])
                .sum();
            *current = companion.currents[i] + conducted;
        }
        into
    }
}

impl Coupled for SaturatingSource {
    /// The points it hands back where it already is: the inputs as far apart
    /// as they are, and the output pushing what its potential asks of it. A
    /// bound believes a push past a limit when the last point was past it
    /// too, so linearised from these, nothing is held back — a saturated
    /// output stays saturated, and a limited one limited (C-94).
    fn settled(&self, potentials: &[f64; MAX_TERMINALS]) -> [f64; MAX_TERMINALS] {
        // Asked once with a history that believes every push — the input
        // already past whichever rail, the output past its source limit and
        // last pushed past its sink limit — its points are where it is, and
        // the pushes it reports are none.
        let mut believed = [0.0; MAX_TERMINALS];
        believed[0] = potentials[NON_INVERTING] - potentials[INVERTING];
        believed[1] = PUSHED_LOW;
        believed[2] = f64::INFINITY;
        believed[3] = PUSHED_LOW;
        let companion = self.linearise(potentials, &believed);
        debug_assert!(!companion.limited, "a believed history holds nothing back");
        companion.points
    }

    fn linearise(
        &self,
        potentials: &[f64; MAX_TERMINALS],
        previous: &[f64; MAX_TERMINALS],
    ) -> CoupledCompanion {
        let high_rail = Dual::terminal(potentials, HIGH_RAIL);
        let low_rail = Dual::terminal(potentials, LOW_RAIL);
        let output = Dual::terminal(potentials, OUTPUT);
        let span = high_rail.sub(low_rail);

        // Everything in the target but the gain term, which is the only part
        // that can be thrown.
        let base = low_rail
            .add(span.scale(self.reference))
            .add(Dual::constant(self.offset));
        let high = high_rail.sub(Dual::constant(self.headroom_high));
        let low = low_rail.add(Dual::constant(self.headroom_low));

        let raw = potentials[NON_INVERTING] - potentials[INVERTING];
        let (input, pushed) = self.bounded_input(raw, base.value, low.value, high.value, previous);

        // The model is evaluated with the inputs this far apart. Its slope
        // with respect to each input is still ±1: the bound moves the point,
        // not the equations.
        let mut between =
            Dual::terminal(potentials, NON_INVERTING).sub(Dual::terminal(potentials, INVERTING));
        between.value = input;

        let target = base.add(between.scale(self.gain));
        // The low rail is applied last, so a part whose rails have crossed —
        // unpowered, or wired backwards — is held at its low rail rather than
        // driven below it on no energy at all.
        let held = soft_max(soft_min(target, high), low);

        // Bounded the same way, by moving the output potential it is
        // evaluated at: the push is what the output resistance sees.
        let thrown = (held.value - output.value) / self.output_ohms;
        let (drive, driven) = self.bounded_output(thrown, previous);
        let mut output = output;
        if drive != thrown {
            output.value = held.value - drive * self.output_ohms;
        }

        let push = held.sub(output).scale(1.0 / self.output_ohms);
        let limit = if push.value >= 0.0 {
            self.source_limit
        } else {
            self.sink_limit
        };
        let (value, slope) = bend(push.value, limit);
        let delivered = push.chain(value, slope);

        // Which rail pays for it: all from the high rail when sourcing, all
        // into the low one when sinking, blended over a microamp.
        let root = (delivered.value * delivered.value + HANDOVER * HANDOVER).sqrt();
        let from_high = delivered.chain(
            0.5 * (1.0 + delivered.value / root),
            0.5 * HANDOVER * HANDOVER / root.powi(3),
        );
        let drawn_high = delivered.mul(from_high);
        let returned_low = delivered.sub(drawn_high);

        let knee = (span.value * span.value + QUIESCENT_KNEE * QUIESCENT_KNEE).sqrt();
        let quiescent = span.chain(
            self.quiescent * span.value / knee,
            self.quiescent * QUIESCENT_KNEE * QUIESCENT_KNEE / knee.powi(3),
        );

        let leak = if self.input_ohms.is_finite() && self.input_ohms > 0.0 {
            between.scale(1.0 / self.input_ohms)
        } else {
            Dual::constant(0.0)
        };

        // Current into the device at each terminal. They sum to zero by
        // construction: what leaves at the output arrives from the rails.
        let mut into = [Dual::constant(0.0); TERMINALS];
        into[NON_INVERTING] = leak;
        into[INVERTING] = leak.scale(-1.0);
        into[OUTPUT] = delivered.scale(-1.0);
        into[HIGH_RAIL] = drawn_high.add(quiescent);
        into[LOW_RAIL] = returned_low.sub(quiescent);

        // The potentials the model was actually evaluated at: the real ones,
        // with the inverting input moved so the inputs are `input` apart.
        let mut point = [0.0; TERMINALS];
        point.copy_from_slice(&potentials[..TERMINALS]);
        point[NON_INVERTING] = point[INVERTING] + input;
        point[OUTPUT] = output.value;

        let mut conductances = [[0.0; MAX_TERMINALS]; MAX_TERMINALS];
        let mut currents = [0.0; MAX_TERMINALS];
        for (i, current) in into.iter().enumerate() {
            conductances[i][..TERMINALS].copy_from_slice(&current.slope);
            let accounted: f64 = (0..TERMINALS).map(|j| current.slope[j] * point[j]).sum();
            currents[i] = current.value - accounted;
        }

        let mut points = [0.0; MAX_TERMINALS];
        points[0] = input;
        points[1] = pushed;
        points[2] = drive;
        points[3] = driven;

        CoupledCompanion {
            terminals: TERMINALS,
            conductances,
            currents,
            points,
            limited: input != raw || drive != thrown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Something op-amp shaped, for the tests that are about the equations
    /// rather than about a part.
    fn amplifier() -> SaturatingSource {
        SaturatingSource {
            gain: 1e5,
            offset: 0.0,
            reference: 0.5,
            headroom_high: 1.5,
            headroom_low: 0.05,
            output_ohms: 50.0,
            source_limit: 0.03,
            sink_limit: 0.02,
            quiescent: 1e-3,
            input_ohms: 1e7,
        }
    }

    /// A history under which the device is evaluated right where it is asked,
    /// found the way Newton would find it: by asking until nothing is held
    /// back. Every bound lets go on the second push past the same limit.
    fn settled(
        device: &SaturatingSource,
        potentials: &[f64; MAX_TERMINALS],
    ) -> [f64; MAX_TERMINALS] {
        let mut previous = [0.0; MAX_TERMINALS];
        for _ in 0..4 {
            let companion = device.linearise(potentials, &previous);
            if !companion.limited {
                return previous;
            }
            previous = companion.points;
        }
        panic!("still bounded at {potentials:?} after four asks");
    }

    /// Current into each terminal at these potentials, from the companion.
    fn currents_at(device: &SaturatingSource, potentials: &[f64; MAX_TERMINALS]) -> [f64; 5] {
        let companion = device.linearise(potentials, &settled(device, potentials));
        assert!(!companion.limited, "evaluated where it was asked");
        let mut into = [0.0; 5];
        for (i, current) in into.iter_mut().enumerate() {
            *current = companion.currents[i]
                + (0..5)
                    .map(|j| companion.conductances[i][j] * potentials[j])
                    .sum::<f64>();
        }
        into
    }

    fn at(plus: f64, minus: f64, out: f64, high: f64, low: f64) -> [f64; MAX_TERMINALS] {
        let mut potentials = [0.0; MAX_TERMINALS];
        potentials[..5].copy_from_slice(&[plus, minus, out, high, low]);
        potentials
    }

    #[test]
    fn from_where_it_settled_nothing_is_held_back() {
        // What a warm start begins from and a pin's current is read from
        // (C-94, C-101): linearised there, the model is evaluated where it
        // is — saturated against either rail, limited either way, unpowered.
        let device = amplifier();
        for potentials in [
            at(4.5, 4.5, 4.5, 9.0, 0.0),
            at(5.0, 4.0, 7.4, 9.0, 0.0),
            at(4.0, 5.0, 0.05, 9.0, 0.0),
            at(5.0, 4.0, 0.0, 9.0, 0.0),
            at(4.0, 5.0, 9.0, 9.0, 0.0),
            at(0.0, 0.0, 3.0, 0.0, 0.0),
        ] {
            let companion = device.linearise(&potentials, &device.settled(&potentials));
            assert!(!companion.limited, "held back at {potentials:?}");
            assert_eq!(
                device.currents(&potentials)[..5],
                currents_at(&device, &potentials),
                "at {potentials:?}"
            );
        }
    }

    #[test]
    fn what_leaves_at_the_output_arrives_from_the_rails() {
        for potentials in [
            at(4.5, 4.5, 4.5, 9.0, 0.0),
            at(5.0, 4.0, 2.0, 9.0, 0.0),
            at(4.0, 5.0, 7.0, 9.0, 0.0),
            at(0.0, 0.0, 3.0, 0.0, 0.0),
            at(1.0, -1.0, 0.0, 9.0, -9.0),
        ] {
            let into = currents_at(&amplifier(), &potentials);
            let total: f64 = into.iter().sum();
            assert!(total.abs() < 1e-15, "{into:?} sums to {total}");
        }
    }

    #[test]
    fn the_slopes_are_the_derivatives_of_the_currents() {
        // Checked against finite differences, terminal by terminal, in the
        // linear region, near a rail, deep in saturation and while limited.
        // Wrong slopes do not fail loudly: Newton still converges, slowly, or
        // to a point it then refuses. This is the test that catches them.
        let device = amplifier();
        for potentials in [
            at(4.50001, 4.5, 4.4, 9.0, 0.0),
            at(4.5, 4.49996, 7.4, 9.0, 0.0),
            at(5.0, 4.0, 7.0, 9.0, 0.0),
            at(4.0, 5.0, 0.1, 9.0, 0.0),
            at(4.500001, 4.5, 0.0, 9.0, 0.0),
            at(0.3, 0.2, 0.0, 0.4, 0.0),
        ] {
            let companion = device.linearise(&potentials, &settled(&device, &potentials));
            for j in 0..5 {
                let h = 1e-9;
                let mut up = potentials;
                up[j] += h;
                let mut down = potentials;
                down[j] -= h;
                let (above, below) = (currents_at(&device, &up), currents_at(&device, &down));
                for i in 0..5 {
                    let numeric = (above[i] - below[i]) / (2.0 * h);
                    let analytic = companion.conductances[i][j];
                    // Relative where the slope is large; absolute where it is
                    // small enough that a central difference is mostly
                    // rounding. One rounding step of a 30 mA current over a
                    // 2 nV difference is a few nanosiemens, so the floor sits
                    // above that — and 100 nS is below anything Newton feels
                    // beside the output's 20 mS.
                    let allowed = 1e-4 * numeric.abs().max(analytic.abs()) + 1e-7;
                    assert!(
                        (numeric - analytic).abs() <= allowed,
                        "dI{i}/dV{j} at {potentials:?}: {analytic} against {numeric}"
                    );
                }
            }
        }
    }

    #[test]
    fn with_no_gain_the_output_sits_at_its_offset_from_the_reference() {
        // A regulator's shape: 5 V above the low rail whatever the inputs say.
        let regulator = SaturatingSource {
            gain: 0.0,
            offset: 5.0,
            reference: 0.0,
            headroom_high: 2.0,
            headroom_low: 0.0,
            output_ohms: 0.01,
            source_limit: 1.5,
            sink_limit: 1e-3,
            quiescent: 5e-3,
            input_ohms: f64::INFINITY,
        };
        // No current out when the output is already where it is aiming. Not
        // exactly none: the rounding at the rails moves the target by
        // nanovolts, which through ten milliohms is microamps.
        let into = currents_at(&regulator, &at(0.0, 0.0, 5.0, 9.0, 0.0));
        assert!(into[OUTPUT].abs() < 1e-4, "{into:?}");
        // Pulled below it, it pushes back, and the high rail pays.
        let into = currents_at(&regulator, &at(0.0, 0.0, 4.99, 9.0, 0.0));
        assert!(into[OUTPUT] < -0.5, "{into:?}");
        assert!(into[HIGH_RAIL] > 0.5, "{into:?}");
    }

    #[test]
    fn a_short_is_met_with_the_current_limit_and_no_more() {
        let device = amplifier();
        let into = currents_at(&device, &at(9.0, 0.0, 0.0, 9.0, 0.0));
        let delivered = -into[OUTPUT];
        assert!(
            delivered > 0.9 * device.source_limit && delivered <= device.source_limit,
            "{delivered} A"
        );
    }

    #[test]
    fn unpowered_it_drives_nothing_below_its_rail() {
        // Both rails at nothing: the target is held at the low rail, so there
        // is nothing to push a grounded output anywhere.
        let into = currents_at(&amplifier(), &at(1.0, 0.0, 0.0, 0.0, 0.0));
        assert!(into[OUTPUT].abs() < 3e-3, "{into:?}");
        assert!(into[HIGH_RAIL].abs() < 3e-3, "{into:?}");
    }

    #[test]
    fn the_first_push_past_a_rail_is_bounded_and_the_second_believed() {
        let device = amplifier();
        let far = at(5.0, 4.0, 4.5, 9.0, 0.0);

        let first = device.linearise(&far, &[0.0; MAX_TERMINALS]);
        assert!(first.limited, "thrown from mid-rail, it holds back");
        assert_eq!(first.points[1], PUSHED_HIGH);

        let second = device.linearise(&far, &first.points);
        assert!(
            !second.limited,
            "pushed again the same way, it is saturation"
        );
    }
}
