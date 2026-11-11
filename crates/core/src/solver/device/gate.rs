// SPDX-License-Identifier: MIT OR Apache-2.0

//! Logic gates: a 74HC00, 08, 32 or 86 — four two-input gates — and a 74HC04,
//! six inverters (C-108); and a 74HC14, the 04 with a Schmitt trigger on each
//! input (C-109).
//!
//! The parts every truth table is taught on, and the ones a 555, a counter and
//! a shift register were all built from. The four quads share one package —
//! the same fourteen pins in the same places — and differ only in what each
//! gate does, so they are one component with a choice; the hex inverter has
//! its pins elsewhere, so it is a component of its own.
//!
//! The 74HC595's shape (C-93), with no register: every gate's inputs are read
//! from the solved potentials after each step, at half the supply, and what
//! each output should be is carried; within a solve the chip is linear, each
//! output driven high or low behind its own resistance, each input a leak. So
//! a gate answers one step after its inputs change — its propagation delay is
//! the step's, not the sheet's 8 ns — and gates wired in a loop work as a
//! latch does, each holding the other.
//!
//! **From TI's SN74HC00 sheet** (SCLS181H), at 4.5 V and 25 °C, typical: an
//! output 4.3 V at −4 mA and 0.17 V at 4 mA; inputs switching between 1.35 V
//! and 3.15 V, ±0.1 nA; 2 to 6 V; 25 mA an output at most. The 08, 32, 86 and
//! 04 are the same family's standard output and have the same table. The
//! outputs are taken as the resistances those points give, as the 595's are.
//!
//! **The 74HC14** switches up and down at two different voltages, so an input
//! creeping slowly past either changes its output once and not many times —
//! the one-gate oscillator and the debounced button both depend on it. From
//! TI's SN74HC14 sheet (SCLS085L), at 4.5 V typical: up at 2.5 V, down at
//! 1.6 V; taken here as the same fractions of whatever the supply is. Its
//! outputs are the 04's. Which way each input last went is its output, so the
//! outputs are still all that is carried.
//!
//! What it ignores: propagation delay and transition times, the supply's own
//! current, and an input left free — which on a real chip reads whatever is
//! near it, and here reads low through its leak. The review says so rather
//! than let the guess pass (B-05).

/// What each gate of a quad does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Logic {
    /// High unless both inputs are: the 74HC00, the one every other gate can
    /// be built from.
    Nand,
    And,
    Or,
    /// High when the two inputs differ.
    Xor,
}

impl Logic {
    /// In the order a chooser offers them: the NAND first, as the sheet
    /// numbers run.
    pub const ALL: &'static [Logic] = &[Logic::Nand, Logic::And, Logic::Or, Logic::Xor];

    /// The name it crosses the boundary under.
    pub fn name(&self) -> &'static str {
        match self {
            Logic::Nand => "NAND",
            Logic::And => "AND",
            Logic::Or => "OR",
            Logic::Xor => "XOR",
        }
    }

    pub fn named(name: &str) -> Option<Logic> {
        Logic::ALL
            .iter()
            .copied()
            .find(|logic| logic.name() == name)
    }

    /// What is printed on the part.
    pub fn part_number(&self) -> &'static str {
        match self {
            Logic::Nand => "74HC00",
            Logic::And => "74HC08",
            Logic::Or => "74HC32",
            Logic::Xor => "74HC86",
        }
    }

    pub fn datasheet(&self) -> &'static str {
        match self {
            Logic::Nand => "https://www.ti.com/lit/ds/symlink/sn74hc00.pdf",
            Logic::And => "https://www.ti.com/lit/ds/symlink/sn74hc08.pdf",
            Logic::Or => "https://www.ti.com/lit/ds/symlink/sn74hc32.pdf",
            Logic::Xor => "https://www.ti.com/lit/ds/symlink/sn74hc86.pdf",
        }
    }

    /// Its truth table.
    pub fn of(&self, a: bool, b: bool) -> bool {
        match self {
            Logic::Nand => !(a && b),
            Logic::And => a && b,
            Logic::Or => a || b,
            Logic::Xor => a != b,
        }
    }
}

/// A quad's pins: the sheet's order round the package, which all four share.
pub const QUAD_PINS: [&str; 14] = [
    "1A", "1B", "1Y", "2A", "2B", "2Y", "GND", "3Y", "3A", "3B", "4Y", "4A", "4B", "VCC",
];

/// Each of a quad's gates: its two inputs and its output.
pub const QUAD_GATES: [([usize; 2], usize); 4] =
    [([0, 1], 2), ([3, 4], 5), ([8, 9], 7), ([11, 12], 10)];

/// A hex inverter's pins: the sheet's order round the package.
pub const HEX_PINS: [&str; 14] = [
    "1A", "1Y", "2A", "2Y", "3A", "3Y", "GND", "4Y", "4A", "5Y", "5A", "6Y", "6A", "VCC",
];

/// Each of its inverters: input and output.
pub const HEX_GATES: [(usize, usize); 6] = [(0, 1), (2, 3), (4, 5), (8, 7), (10, 9), (12, 11)];

/// The supply, at the same corners of both packages.
pub const GND: usize = 6;
pub const VCC: usize = 13;

/// What is on the inverter.
pub const INVERTER_PART: &str = "74HC04";
pub const INVERTER_DATASHEET: &str = "https://www.ti.com/lit/ds/symlink/sn74hc04.pdf";

/// What is on the Schmitt-trigger inverter.
pub const SCHMITT_PART: &str = "74HC14";
pub const SCHMITT_DATASHEET: &str = "https://www.ti.com/lit/ds/symlink/sn74hc14.pdf";

/// Where a 74HC14's input switches, as a fraction of its supply: 2.5 V going
/// up and 1.6 V going down, at 4.5 V.
pub const RISING: f64 = 2.5 / 4.5;
pub const FALLING: f64 = 1.6 / 4.5;

/// Behind an output driven high, in ohms: 0.2 V under 4.5 V at 4 mA.
pub const HIGH_OHMS: f64 = 50.0;
/// Behind one driven low: 0.17 V at 4 mA.
pub const LOW_OHMS: f64 = 42.5;

/// An input's leak, in siemens: a node with nothing on it is one the solver
/// cannot place (C-63).
pub const LEAK: f64 = 1e-9;

/// The family runs on 2 V to 6 V.
pub const MOST_SUPPLY: f64 = 6.0;

/// Which of the three chips: what decides how its pins are laid out and how
/// its inputs are read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chip {
    /// Four two-input gates.
    Quad(Logic),
    /// Six inverters, reading their inputs at half the supply.
    Inverter,
    /// Six inverters, reading theirs with hysteresis.
    Schmitt,
}

impl Chip {
    pub fn of(kind: &crate::netlist::ElementKind) -> Option<Chip> {
        use crate::netlist::ElementKind;
        match *kind {
            ElementKind::Gate { logic } => Some(Chip::Quad(logic)),
            ElementKind::Inverter => Some(Chip::Inverter),
            ElementKind::Schmitt => Some(Chip::Schmitt),
            _ => None,
        }
    }

    /// What is printed on it.
    pub fn part(&self) -> &'static str {
        match self {
            Chip::Quad(logic) => logic.part_number(),
            Chip::Inverter => INVERTER_PART,
            Chip::Schmitt => SCHMITT_PART,
        }
    }
}

/// One output's level and the pins that decide it, for either package.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wiring {
    pub inputs: &'static [usize],
    pub output: usize,
}

/// Every gate of a package, in the order their outputs are carried.
pub fn wiring(chip: Chip) -> Vec<Wiring> {
    match chip {
        Chip::Quad(_) => QUAD_GATES
            .iter()
            .map(|(inputs, output)| Wiring {
                inputs,
                output: *output,
            })
            .collect(),
        Chip::Inverter | Chip::Schmitt => HEX_GATES
            .iter()
            .map(|(input, output)| Wiring {
                inputs: std::slice::from_ref(input),
                output: *output,
            })
            .collect(),
    }
}

/// What each output should be, one bit a gate in [`wiring`]'s order, from the
/// levels on the pins and, for a 74HC14, what each output `was`.
///
/// `joined(a, b)` says whether two of its pins share a wire. A gate whose
/// input is wired to the output of a gate already worked out this step reads
/// that output's new level, not the one solved before it: within a chip the
/// gates answer one after another, in the sheet's order, rather than all at
/// once. All at once, two NANDs wired into a latch and both fed high flip
/// together every step forever; a real pair settles, because one is always a
/// little quicker. This settles them the same way every time — the first
/// gate wins — which is a choice a real chip makes by chance (C-108).
pub fn outputs(
    chip: Chip,
    potentials: &[f64; 14],
    joined: &dyn Fn(usize, usize) -> bool,
    was: u8,
) -> u8 {
    let (ground, supply) = (potentials[GND], potentials[VCC] - potentials[GND]);
    let gates = wiring(chip);
    let mut bits = 0u8;
    for (gate, wiring) in gates.iter().enumerate() {
        // A 74HC14's input that was low — its output high — has to climb to
        // the upper threshold to read high, and one that was high has to fall
        // to the lower one to read low.
        let threshold = match chip {
            Chip::Schmitt if was & (1 << gate) != 0 => ground + RISING * supply,
            Chip::Schmitt => ground + FALLING * supply,
            _ => ground + supply / 2.0,
        };
        let high = |pin: usize| {
            gates[..gate]
                .iter()
                .enumerate()
                .find(|(_, earlier)| joined(pin, earlier.output))
                .map_or(potentials[pin] > threshold, |(earlier, _)| {
                    bits & (1 << earlier) != 0
                })
        };
        let level = match (chip, wiring.inputs) {
            (Chip::Quad(logic), &[a, b]) => logic.of(high(a), high(b)),
            (_, inputs) => !high(inputs[0]),
        };
        bits |= u8::from(level) << gate;
    }
    bits
}

/// What the outputs are before anything has been read: every input low, as
/// its leak would read it, so a NAND starts high and an AND low.
pub fn resting(chip: Chip) -> u8 {
    outputs(chip, &[0.0; 14], &|_, _| false, 0)
}

/// The outputs as the run carries them (C-48): six bits, which a double holds
/// exactly.
pub fn encode(bits: u8) -> f64 {
    f64::from(bits)
}

pub fn decode(value: f64) -> u8 {
    value.round().clamp(0.0, 63.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A quad at 5 V with its first gate's inputs as given, the rest low.
    fn quad(a: bool, b: bool) -> [f64; 14] {
        let mut volts = [0.0; 14];
        volts[VCC] = 5.0;
        volts[0] = if a { 5.0 } else { 0.0 };
        volts[1] = if b { 5.0 } else { 0.0 };
        volts
    }

    #[test]
    fn each_quad_follows_its_truth_table() {
        let tables = [
            (Logic::Nand, [true, true, true, false]),
            (Logic::And, [false, false, false, true]),
            (Logic::Or, [false, true, true, true]),
            (Logic::Xor, [false, true, true, false]),
        ];
        for (logic, table) in tables {
            for (row, inputs) in [(false, false), (false, true), (true, false), (true, true)]
                .into_iter()
                .enumerate()
            {
                let first =
                    outputs(Chip::Quad(logic), &quad(inputs.0, inputs.1), &apart, 0) & 1 != 0;
                assert_eq!(first, table[row], "{} on {inputs:?}", logic.name());
            }
        }
    }

    #[test]
    fn every_gate_of_a_quad_is_read_on_its_own_pins() {
        // One gate at a time high on both inputs: only its own NAND goes low.
        for (gate, (inputs, _)) in QUAD_GATES.iter().enumerate() {
            let mut volts = [0.0; 14];
            volts[VCC] = 5.0;
            for &pin in inputs {
                volts[pin] = 5.0;
            }
            assert_eq!(
                outputs(Chip::Quad(Logic::Nand), &volts, &apart, 0),
                0b1111 & !(1 << gate)
            );
        }
    }

    #[test]
    fn every_inverter_is_read_on_its_own_pin() {
        for (gate, (input, _)) in HEX_GATES.iter().enumerate() {
            let mut volts = [0.0; 14];
            volts[VCC] = 5.0;
            volts[*input] = 5.0;
            assert_eq!(
                outputs(Chip::Inverter, &volts, &apart, 0),
                0b11_1111 & !(1 << gate)
            );
        }
    }

    #[test]
    fn an_input_reads_against_half_its_own_supply() {
        // 2 V is high on a 3 V supply and low on a 5 V one.
        let mut volts = quad(true, true);
        volts[0] = 2.0;
        volts[1] = 2.0;
        volts[VCC] = 3.0;
        assert_eq!(outputs(Chip::Quad(Logic::And), &volts, &apart, 0) & 1, 1);
        volts[VCC] = 5.0;
        assert_eq!(outputs(Chip::Quad(Logic::And), &volts, &apart, 0) & 1, 0);
    }

    /// No two pins on one wire.
    fn apart(_: usize, _: usize) -> bool {
        false
    }

    #[test]
    fn two_nands_in_a_latch_settle_rather_than_flip_together() {
        // 1Y on 2A and 2Y on 1B, the two free inputs high, both outputs high
        // to begin with: all at once, each would go low, then both high
        // again. One after another, the first wins and it stays.
        let joined = |a: usize, b: usize| {
            let pair = (a.min(b), a.max(b));
            pair == (2, 3) || pair == (1, 5) || a == b
        };
        let mut volts = [0.0; 14];
        volts[VCC] = 5.0;
        for pin in [0, 1, 3, 4] {
            volts[pin] = 5.0;
        }
        let first = outputs(Chip::Quad(Logic::Nand), &volts, &joined, 0) & 0b11;
        assert_eq!(first, 0b10, "1Y low, 2Y high");
        volts[1] = 5.0;
        volts[3] = 0.0;
        assert_eq!(
            outputs(Chip::Quad(Logic::Nand), &volts, &joined, 0) & 0b11,
            first,
            "and held"
        );
    }

    #[test]
    fn a_schmitt_input_switches_up_and_down_at_two_voltages() {
        // At 4.5 V: an input climbing from low reads high only past 2.5 V,
        // and one falling from high reads low only under 1.6 V.
        let at = |volts: f64, was: u8| {
            let mut pins = [0.0; 14];
            pins[VCC] = 4.5;
            pins[0] = volts;
            outputs(Chip::Schmitt, &pins, &apart, was) & 1
        };
        // Output high: the input was low.
        assert_eq!(at(2.4, 1), 1, "climbing, not yet");
        assert_eq!(at(2.6, 1), 0, "climbing, past 2.5 V");
        // Output low: the input was high.
        assert_eq!(at(1.7, 0), 0, "falling, not yet");
        assert_eq!(at(1.5, 0), 1, "falling, under 1.6 V");
        // A plain 04 switches at half its supply either way.
        let mut pins = [0.0; 14];
        pins[VCC] = 4.5;
        pins[0] = 2.4;
        assert_eq!(outputs(Chip::Inverter, &pins, &apart, 1) & 1, 0);
    }

    #[test]
    fn a_slow_ramp_flips_a_schmitt_output_once_and_a_plain_one_on_every_wobble() {
        // An input crawling up past half the supply with 0.2 V of noise on
        // it: the 04 flips on each wobble across 2.25 V, the 14 once.
        let flips = |chip: Chip| {
            let mut was = resting(chip);
            let mut flips = 0;
            for step in 0..200 {
                let ramp = 4.5 * f64::from(step) / 200.0;
                let wobble = if step % 2 == 0 { 0.1 } else { -0.1 };
                let mut pins = [0.0; 14];
                pins[VCC] = 4.5;
                pins[0] = ramp + wobble;
                let now = outputs(chip, &pins, &apart, was);
                flips += u32::from((now ^ was) & 1);
                was = now;
            }
            flips
        };
        assert_eq!(flips(Chip::Schmitt), 1);
        assert!(flips(Chip::Inverter) > 1, "{}", flips(Chip::Inverter));
    }

    #[test]
    fn at_rest_a_nand_is_high_and_an_and_low() {
        assert_eq!(resting(Chip::Quad(Logic::Nand)), 0b1111);
        assert_eq!(resting(Chip::Quad(Logic::And)), 0);
        assert_eq!(resting(Chip::Inverter), 0b11_1111);
        assert_eq!(resting(Chip::Schmitt), 0b11_1111);
    }

    #[test]
    fn the_pin_lists_put_every_gate_where_its_name_says() {
        for (inputs, output) in QUAD_GATES {
            let gate = &QUAD_PINS[output][..1];
            assert_eq!(QUAD_PINS[inputs[0]], format!("{gate}A"));
            assert_eq!(QUAD_PINS[inputs[1]], format!("{gate}B"));
            assert_eq!(QUAD_PINS[output], format!("{gate}Y"));
        }
        for (input, output) in HEX_GATES {
            let gate = &HEX_PINS[output][..1];
            assert_eq!(HEX_PINS[input], format!("{gate}A"));
        }
        for pins in [QUAD_PINS, HEX_PINS] {
            assert_eq!((pins[GND], pins[VCC]), ("GND", "VCC"));
        }
    }

    #[test]
    fn its_outputs_survive_being_carried() {
        for bits in 0..64u8 {
            assert_eq!(decode(encode(bits)), bits);
        }
    }
}
