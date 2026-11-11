// SPDX-License-Identifier: MIT OR Apache-2.0

//! The MOSFET: a transistor steered by a voltage, whose gate takes no current.
//!
//! A discrete part, so a physical model (ADR 0010, term 1): the square law of
//! Shichman and Hodges, SPICE's level 1, which is what every textbook first
//! draws. Off below its threshold; above it, a resistor that shrinks as the
//! gate rises while the drain is low, and a current source once the drain is
//! high enough to pinch the channel off.
//!
//! What it honours: the threshold; the square law and its two regions; that
//! the channel is symmetric, so drain and source trade places when it is
//! reversed; that the gate draws nothing. What it ignores: the body diode of
//! the small parts (the power part has its own, C-112), channel-length
//! modulation, the gate's capacitance — so it switches
//! instantly — and the subthreshold current beyond the few tens of millivolts
//! the rounding below gives it.
//!
//! The corner at the threshold is rounded, over [`ROUNDING`], with a softplus:
//! the square law is continuous there but its slope into cutoff is zero, and a
//! part that is exactly off is a node Newton cannot see. Reimplemented from the
//! published level-1 equations, not from any GPL source (ADR 0001).

use crate::solver::{Coupled, CoupledCompanion, MAX_TERMINALS};

use super::Diode;

/// The power part's body diode, source to drain: 0.8 V at an amp, which is
/// 0.88 V at 25 A against the 1.3 V its sheet gives as a most (C-112).
const BODY_DIODE: Diode = Diode {
    saturation_current: 3.7e-14,
    emission_coefficient: 1.0,
};

/// Terminal order, and the order the pins are named in.
pub const GATE: usize = 0;
pub const DRAIN: usize = 1;
pub const SOURCE: usize = 2;

/// Width of the rounding at the threshold, in volts.
const ROUNDING: f64 = 0.05;

/// The gate's leak to its source: a teraohm, far below anything a gate
/// resistor would notice, and enough that a gate wired to nothing else is
/// still a node the matrix can solve for.
const GATE_LEAK: f64 = 1e-12;

/// The channel's leak when off, drain to source: a gigaohm.
///
/// A real 2N7000 leaks a microamp at 48 V, and this is less. Without it a drain
/// whose only other connection is an LED hangs on almost nothing while the part
/// is off, its voltage is barely decided, and the step-doubling error estimate
/// sees that indecision as movement: the stepper shrank to its shortest step for
/// every frame the part was off, and a blinking LED cost thirty times what the
/// same LED costs on a switch (C-63).
const CHANNEL_LEAK: f64 = 1e-9;

/// Which way round.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Channel {
    /// Gate above source turns it on: a 2N7000, the one in every kit.
    N,
    /// The mirror image: gate below source. A BS250.
    P,
    /// An N-channel power part driven straight from 5 V logic: an IRLZ44N,
    /// for a motor or a lamp (C-112).
    Power,
}

impl Channel {
    pub const ALL: &'static [Channel] = &[Channel::N, Channel::P, Channel::Power];

    /// The name it crosses the boundary under.
    pub fn name(&self) -> &'static str {
        match self {
            Channel::N => "N-channel",
            Channel::P => "P-channel",
            Channel::Power => "Power N-channel",
        }
    }

    pub fn named(name: &str) -> Option<Channel> {
        Channel::ALL
            .iter()
            .copied()
            .find(|channel| channel.name() == name)
    }

    /// What is printed on it.
    pub fn part_number(&self) -> &'static str {
        match self {
            Channel::N => "2N7000",
            Channel::P => "BS250",
            Channel::Power => "IRLZ44N",
        }
    }
}

/// A MOSFET.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mosfet {
    pub channel: Channel,
    /// Gate-to-source voltage at which it starts to conduct, as a magnitude.
    pub threshold: f64,
    /// The square law's constant, in amps per volt squared.
    pub transconductance: f64,
    /// Whether its body diode, source to drain, is modelled: a power part's
    /// is drawn on its symbol and carries a motor's kick, so it is (C-112).
    pub body_diode: bool,
}

impl Mosfet {
    /// Typical figures from the part's datasheet: a 2N7000's 2.1 V threshold
    /// and the 1.8 Ω it has fully on at 10 V; a BS250's 2.4 V and 14 Ω.
    ///
    /// An IRLZ44N's sheet (Infineon) gives its threshold only as 1 to 2 V,
    /// and its on-resistance as 25 mΩ at 5 V and 35 mΩ at 4 V — a logic-level
    /// part is driven at those, not at 10 V. The square law through those two
    /// points puts the threshold at 1.5 V, the middle of the sheet's range,
    /// and the constant at 11.4 A/V²; at 10 V it then reads 10 mΩ where the
    /// sheet says 22, which is level 1 running out, at a gate this part is
    /// not bought for (C-112).
    pub fn new(channel: Channel) -> Self {
        // Fully on, in the linear region with a small drain voltage, the
        // channel is 1 / (K · (Vgs − Vth)) ohms.
        let (threshold, transconductance) = match channel {
            Channel::N => (2.1, 1.0 / (1.8 * (10.0 - 2.1))),
            Channel::P => (2.4, 1.0 / (14.0 * (10.0 - 2.4))),
            Channel::Power => (1.5, 1.0 / (0.025 * (5.0 - 1.5))),
        };
        Mosfet {
            channel,
            threshold,
            transconductance,
            body_diode: channel == Channel::Power,
        }
    }

    fn sign(&self) -> f64 {
        match self.channel {
            Channel::N | Channel::Power => 1.0,
            Channel::P => -1.0,
        }
    }
}

/// `ln(1 + eˣ)`, without overflowing, and its slope.
fn softplus(x: f64) -> (f64, f64) {
    let slope = 1.0 / (1.0 + (-x).exp());
    let value = if x > 30.0 { x } else { x.exp().ln_1p() };
    (value, slope)
}

impl Coupled for Mosfet {
    fn settled(&self, potentials: &[f64; MAX_TERMINALS]) -> [f64; MAX_TERMINALS] {
        let mut points = [0.0; MAX_TERMINALS];
        if self.body_diode {
            points[0] = self.sign() * (potentials[SOURCE] - potentials[DRAIN]);
        }
        points
    }

    fn linearise(
        &self,
        potentials: &[f64; MAX_TERMINALS],
        previous: &[f64; MAX_TERMINALS],
    ) -> CoupledCompanion {
        // Worked in N-channel terms, mirrored on the way in and out, as the
        // transistor's PNP is (C-12).
        let sign = self.sign();
        let v = [
            sign * potentials[GATE],
            sign * potentials[DRAIN],
            sign * potentials[SOURCE],
        ];

        // The channel is symmetric: whichever end is lower acts as the source.
        let (high, low) = if v[DRAIN] >= v[SOURCE] {
            (DRAIN, SOURCE)
        } else {
            (SOURCE, DRAIN)
        };
        let vgs = v[GATE] - v[low];
        let vds = v[high] - v[low];

        let (overdrive, on) = softplus((vgs - self.threshold) / ROUNDING);
        let overdrive = overdrive * ROUNDING;
        let k = self.transconductance;

        // Current from `high` to `low` through the channel, and its slopes
        // with respect to the gate and the high end's voltage above the low.
        let (current, gm, gds) = if vds >= overdrive {
            (0.5 * k * overdrive * overdrive, k * overdrive * on, 0.0)
        } else {
            (
                k * (overdrive * vds - 0.5 * vds * vds),
                k * vds * on,
                k * (overdrive - vds),
            )
        };

        let mut conductances = [[0.0; MAX_TERMINALS]; MAX_TERMINALS];
        conductances[high][GATE] = gm;
        conductances[high][high] = gds;
        conductances[high][low] = -gm - gds;
        conductances[low][GATE] = -gm;
        conductances[low][high] = -gds;
        conductances[low][low] = gm + gds;
        // The channel's leak, high end to low.
        conductances[high][high] += CHANNEL_LEAK;
        conductances[high][low] -= CHANNEL_LEAK;
        conductances[low][high] -= CHANNEL_LEAK;
        conductances[low][low] += CHANNEL_LEAK;
        // The gate's leak, gate to source.
        conductances[GATE][GATE] += GATE_LEAK;
        conductances[GATE][low] -= GATE_LEAK;
        conductances[low][GATE] -= GATE_LEAK;
        conductances[low][low] += GATE_LEAK;

        let mut into = [0.0; 3];
        into[high] = current;
        into[low] = -current;
        into[high] += CHANNEL_LEAK * vds;
        into[low] -= CHANNEL_LEAK * vds;
        into[GATE] += GATE_LEAK * vgs;
        into[low] -= GATE_LEAK * vgs;

        let mut currents = [0.0; MAX_TERMINALS];
        for i in 0..3 {
            let accounted: f64 = (0..3).map(|j| conductances[i][j] * v[j]).sum();
            // Mirrored back: a P-channel's currents run the other way, and its
            // slopes need no mirroring (C-12).
            currents[i] = sign * (into[i] - accounted);
        }

        // The body diode, source to drain, bounded as a diode's junction is
        // and taken about its own point (C-112).
        let mut points = [0.0; MAX_TERMINALS];
        let mut limited = false;
        if self.body_diode {
            let raw = v[SOURCE] - v[DRAIN];
            let vsd = BODY_DIODE.limit(raw, previous[0]);
            let (i, g) = (BODY_DIODE.current(vsd), BODY_DIODE.conductance(vsd));
            conductances[SOURCE][SOURCE] += g;
            conductances[SOURCE][DRAIN] -= g;
            conductances[DRAIN][SOURCE] -= g;
            conductances[DRAIN][DRAIN] += g;
            let companion = sign * (i - g * vsd);
            currents[SOURCE] += companion;
            currents[DRAIN] -= companion;
            points[0] = vsd;
            limited = vsd != raw;
        }

        CoupledCompanion {
            terminals: 3,
            conductances,
            currents,
            points,
            limited,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Current into each terminal at these potentials.
    fn into(mosfet: &Mosfet, gate: f64, drain: f64, source: f64) -> [f64; 3] {
        let mut potentials = [0.0; MAX_TERMINALS];
        potentials[..3].copy_from_slice(&[gate, drain, source]);
        let companion = mosfet.linearise(&potentials, &mosfet.settled(&potentials));
        let mut out = [0.0; 3];
        for (i, value) in out.iter_mut().enumerate() {
            *value = companion.currents[i]
                + (0..3)
                    .map(|j| companion.conductances[i][j] * potentials[j])
                    .sum::<f64>();
        }
        out
    }

    #[test]
    fn below_its_threshold_it_is_off() {
        // Off, it passes only its leak: nine nanoamps at nine volts, the
        // gigaohm that keeps its drain a node the stepper can see.
        let n = Mosfet::new(Channel::N);
        let leak = into(&n, 1.0, 9.0, 0.0)[DRAIN];
        assert!((leak - 9.0 * CHANNEL_LEAK).abs() < 1e-10, "{leak} A");
    }

    #[test]
    fn fully_on_it_is_the_resistance_its_datasheet_gives() {
        // 10 V on the gate, a tenth of a volt across the channel: very nearly
        // the 1.8 Ω the 2N7000's datasheet quotes.
        let n = Mosfet::new(Channel::N);
        let amps = into(&n, 10.0, 0.1, 0.0)[DRAIN];
        let ohms = 0.1 / amps;
        assert!((ohms - 1.8).abs() < 0.05, "{ohms} Ω");
    }

    #[test]
    fn a_power_parts_body_diode_carries_current_backwards_and_no_one_elses_does() {
        // Drain 0.8 V below source, gate off: an amp through the IRLZ44N's
        // body diode, and nothing but the leak through a 2N7000 (C-112).
        let power = into(&Mosfet::new(Channel::Power), 0.0, -0.8, 0.0);
        assert!((power[SOURCE] - 1.0).abs() < 0.05, "{} A", power[SOURCE]);
        assert!(power.iter().sum::<f64>().abs() < 1e-9);
        let small = into(&Mosfet::new(Channel::N), 0.0, -0.8, 0.0);
        assert!(small[SOURCE].abs() < 1e-6, "{} A", small[SOURCE]);
        // Forward, it is reverse biased and changes nothing that matters.
        let forward = into(&Mosfet::new(Channel::Power), 0.0, 9.0, 0.0);
        assert!(forward[DRAIN].abs() < 1e-6, "{} A", forward[DRAIN]);
    }

    #[test]
    fn a_power_part_on_logic_level_is_the_resistance_its_datasheet_gives() {
        // 5 V and 4 V on the gate: the IRLZ44N's 25 mΩ and 35 mΩ, the two
        // points its square law was fitted through (C-112).
        let power = Mosfet::new(Channel::Power);
        for (gate, sheet) in [(5.0, 0.025), (4.0, 0.035)] {
            let ohms = 0.01 / into(&power, gate, 0.01, 0.0)[DRAIN];
            assert!((ohms - sheet).abs() < 0.002, "{ohms} Ω at {gate} V");
        }
    }

    #[test]
    fn the_gate_takes_nothing_and_the_rest_balances() {
        for channel in Channel::ALL {
            let fet = Mosfet::new(*channel);
            let sign = if *channel == Channel::P { -1.0 } else { 1.0 };
            let currents = into(&fet, sign * 5.0, sign * 3.0, 0.0);
            assert!(currents[GATE].abs() < 1e-9, "{channel:?} gate");
            assert!(currents.iter().sum::<f64>().abs() < 1e-12);
            assert!(currents[DRAIN] * sign > 1e-3, "{channel:?} is on");
        }
    }

    #[test]
    fn the_slopes_are_the_derivatives_of_the_currents() {
        let n = Mosfet::new(Channel::N);
        for (gate, drain, source) in [
            (3.0, 5.0, 0.0),
            (5.0, 0.5, 0.0),
            (2.1, 3.0, 0.0),
            (4.0, -1.0, 0.0),
        ] {
            let mut potentials = [0.0; MAX_TERMINALS];
            potentials[..3].copy_from_slice(&[gate, drain, source]);
            let companion = n.linearise(&potentials, &[0.0; MAX_TERMINALS]);
            for j in 0..3 {
                let h = 1e-6;
                let mut up = [gate, drain, source];
                up[j] += h;
                let mut down = [gate, drain, source];
                down[j] -= h;
                let (above, below) = (
                    into(&n, up[0], up[1], up[2]),
                    into(&n, down[0], down[1], down[2]),
                );
                for i in 0..3 {
                    let numeric = (above[i] - below[i]) / (2.0 * h);
                    let analytic = companion.conductances[i][j];
                    assert!(
                        (numeric - analytic).abs()
                            <= 1e-4 * numeric.abs().max(analytic.abs()) + 1e-7,
                        "dI{i}/dV{j} at {gate},{drain},{source}: {analytic} against {numeric}"
                    );
                }
            }
        }
    }
}
