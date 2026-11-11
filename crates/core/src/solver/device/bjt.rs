// SPDX-License-Identifier: MIT OR Apache-2.0

//! The bipolar transistor, Ebers-Moll in its transport form.
//!
//! Two junctions and the coupling between them. What makes a transistor a
//! transistor is that the collector current is set by the base-emitter voltage
//! rather than by the collector's own, so it is the first element here that
//! cannot be linearised one pair of terminals at a time — see [`Coupled`].
//!
//! Reimplemented from the published formulation (Ebers & Moll's original, and
//! the standard treatments in Vlach & Singhal and Najm), not from any GPL
//! source — see `docs/adrs/0001-permissive-dual-licence.md`. Gummel-Poon adds
//! high-level injection, Early voltage and a dozen parameters nobody in §4's
//! audience would know how to set; Ebers-Moll gets cutoff, active and
//! saturation right, which is what §5 asks for.

use crate::solver::{Coupled, CoupledCompanion, MAX_TERMINALS};

use super::Diode;
use super::diode::THERMAL_VOLTAGE;

/// Which way round the junctions face.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Polarity {
    /// Base positive with respect to the emitter turns it on.
    Npn,
    /// The mirror image: every voltage and current reversed.
    Pnp,
    /// Two NPNs in one package, the first's emitter driving the second's
    /// base: a TIP120, for loads a small transistor cannot carry (C-112).
    Darlington,
}

/// Terminal order, and the order the pins are named in.
pub const BASE: usize = 0;
pub const COLLECTOR: usize = 1;
pub const EMITTER: usize = 2;

/// A bipolar transistor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bjt {
    pub polarity: Polarity,
    /// Transport saturation current, in amps.
    pub saturation_current: f64,
    /// Forward current gain, the β on the datasheet.
    pub forward_beta: f64,
    /// Reverse gain, which is poor by construction — a transistor run
    /// backwards is a bad transistor, and that is the point of the number.
    pub reverse_beta: f64,
    /// The base-emitter junction's emission coefficient: 1 for one junction,
    /// 2 for a Darlington's two in series (C-112).
    pub forward_emission: f64,
}

impl Polarity {
    /// Both polarities, in the order a chooser should offer them.
    pub const ALL: &'static [Polarity] = &[Polarity::Npn, Polarity::Pnp, Polarity::Darlington];

    /// The name this polarity crosses the boundary under.
    pub fn name(&self) -> &'static str {
        match self {
            Polarity::Npn => "Npn",
            Polarity::Pnp => "Pnp",
            Polarity::Darlington => "Darlington",
        }
    }

    /// The part it is closest to: the 2N3904 its figures are rounded from,
    /// and the 2N3906 that is its mirror image.
    pub fn part_number(&self) -> &'static str {
        match self {
            Polarity::Npn => "2N3904",
            Polarity::Pnp => "2N3906",
            Polarity::Darlington => "TIP120",
        }
    }

    /// That part's datasheet (onsemi).
    pub fn datasheet(&self) -> &'static str {
        match self {
            Polarity::Npn => "https://www.onsemi.com/download/data-sheet/pdf/2n3903-d.pdf",
            Polarity::Pnp => "https://www.onsemi.com/download/data-sheet/pdf/2n3906-d.pdf",
            Polarity::Darlington => "https://www.onsemi.com/download/data-sheet/pdf/tip120-d.pdf",
        }
    }

    /// The polarity of that name, or nothing if it is not one.
    pub fn named(name: &str) -> Option<Polarity> {
        Polarity::ALL
            .iter()
            .copied()
            .find(|polarity| polarity.name() == name)
    }
}

impl Bjt {
    /// A small-signal switching transistor, roughly a 2N3904; or a TIP120.
    ///
    /// **The Darlington** is one Ebers-Moll transistor with its base-emitter
    /// junction's emission coefficient doubled, rather than two transistors
    /// and the node between them (C-112). Doubled, V_BE is two junctions' worth
    /// — 1.4 V at an amp — and in saturation the collector junction, still a
    /// single one, comes on at half that, so V_CE never falls below about
    /// 0.7 V: the floor a Darlington has because its output transistor's base
    /// is fed through the first one's collector. From onsemi's TIP120/D: β of
    /// 1000 at least at 0.5 A and 3 A, taken as β (its typical, 2500, is at
    /// 4 A); V_BE(on) under 2.5 V and V_CE(sat) under 2 V at 3 A. The
    /// saturation current puts V_BE at 1.4 V at 1 A, a typical figure for the
    /// part, which gives 1.46 V at 3 A. Its base-emitter shunt resistors are
    /// left out: they only matter below a milliamp.
    pub fn new(polarity: Polarity) -> Self {
        match polarity {
            Polarity::Darlington => Bjt {
                polarity,
                saturation_current: (-1.4 / (2.0 * THERMAL_VOLTAGE)).exp(),
                forward_beta: 1000.0,
                reverse_beta: 2.0,
                forward_emission: 2.0,
            },
            _ => Bjt {
                polarity,
                saturation_current: 1e-14,
                forward_beta: 100.0,
                reverse_beta: 2.0,
                forward_emission: 1.0,
            },
        }
    }

    /// The sign that turns a PNP into an NPN seen in a mirror.
    fn sign(&self) -> f64 {
        match self.polarity {
            Polarity::Npn | Polarity::Darlington => 1.0,
            Polarity::Pnp => -1.0,
        }
    }

    /// The base-emitter junction, borrowed from the diode for its limiting.
    fn emitter_junction(&self) -> Diode {
        Diode {
            saturation_current: self.saturation_current,
            emission_coefficient: self.forward_emission,
        }
    }

    /// The base-collector junction: always a single one.
    fn collector_junction(&self) -> Diode {
        Diode {
            saturation_current: self.saturation_current,
            emission_coefficient: 1.0,
        }
    }
}

impl Bjt {
    /// The current into each terminal — base, collector, emitter — at these
    /// potentials, for what it dissipates (C-112).
    pub fn terminal_currents(&self, potentials: &[f64; MAX_TERMINALS]) -> [f64; 3] {
        let sign = self.sign();
        let vbe = sign * (potentials[BASE] - potentials[EMITTER]);
        let vbc = sign * (potentials[BASE] - potentials[COLLECTOR]);
        let forward = self.emitter_junction().current(vbe);
        let reverse = self.collector_junction().current(vbc);
        let base = forward / self.forward_beta + reverse / self.reverse_beta;
        let collector = forward - reverse * (1.0 + 1.0 / self.reverse_beta);
        [sign * base, sign * collector, -sign * (base + collector)]
    }
}

impl Coupled for Bjt {
    fn settled(&self, potentials: &[f64; MAX_TERMINALS]) -> [f64; MAX_TERMINALS] {
        let sign = self.sign();
        let mut points = [0.0; MAX_TERMINALS];
        points[0] = sign * (potentials[BASE] - potentials[EMITTER]);
        points[1] = sign * (potentials[BASE] - potentials[COLLECTOR]);
        points
    }

    fn linearise(
        &self,
        potentials: &[f64; MAX_TERMINALS],
        previous: &[f64; MAX_TERMINALS],
    ) -> CoupledCompanion {
        let sign = self.sign();
        let (emitter, collector_side) = (self.emitter_junction(), self.collector_junction());

        // Worked in NPN terms throughout and mirrored on the way in and out,
        // so there is one set of equations to get right rather than two.
        let raw_be = sign * (potentials[BASE] - potentials[EMITTER]);
        let raw_bc = sign * (potentials[BASE] - potentials[COLLECTOR]);
        // Both junctions are exponentials and both need the same bounding a
        // lone diode does (M-10), or one unlucky Newton step overflows.
        let vbe = emitter.limit(raw_be, previous[0]);
        let vbc = collector_side.limit(raw_bc, previous[1]);

        // Transport currents: what each junction injects, and how much of it
        // arrives at the far side.
        let forward = emitter.current(vbe);
        let reverse = collector_side.current(vbc);
        let g_forward = emitter.conductance(vbe);
        let g_reverse = collector_side.conductance(vbc);

        let reverse_share = 1.0 + 1.0 / self.reverse_beta;

        // Currents into the device at each terminal, in NPN terms.
        let base = forward / self.forward_beta + reverse / self.reverse_beta;
        let collector = forward - reverse * reverse_share;
        let emitter = -(base + collector);

        // dI/dV, with vbe = vb − ve and vbc = vb − vc.
        let mut conductances = [[0.0; MAX_TERMINALS]; MAX_TERMINALS];
        let gb_b = g_forward / self.forward_beta + g_reverse / self.reverse_beta;
        conductances[BASE][BASE] = gb_b;
        conductances[BASE][COLLECTOR] = -g_reverse / self.reverse_beta;
        conductances[BASE][EMITTER] = -g_forward / self.forward_beta;

        let gc_b = g_forward - g_reverse * reverse_share;
        conductances[COLLECTOR][BASE] = gc_b;
        conductances[COLLECTOR][COLLECTOR] = g_reverse * reverse_share;
        conductances[COLLECTOR][EMITTER] = -g_forward;

        // The emitter row is whatever the other two leave over: current in has
        // to equal current out, and deriving it rather than writing it is what
        // guarantees that.
        let (base_row, collector_row) = (conductances[BASE], conductances[COLLECTOR]);
        for (column, entry) in conductances[EMITTER].iter_mut().enumerate() {
            *entry = -(base_row[column] + collector_row[column]);
        }

        // The companion source carries the part of each current the
        // conductances do not, taken about the point they were built at.
        //
        // Only differences between terminals mean anything to the model, so
        // the emitter is taken as the origin. That choice is free: every row
        // of the conductance matrix sums to zero, so shifting all three
        // potentials by the same amount changes nothing.
        let mut linearised = [0.0; MAX_TERMINALS];
        linearised[BASE] = vbe;
        linearised[COLLECTOR] = vbe - vbc;

        let mut currents = [0.0; MAX_TERMINALS];
        for (i, current) in [base, collector, emitter].iter().enumerate() {
            let accounted: f64 = (0..MAX_TERMINALS)
                .map(|j| conductances[i][j] * linearised[j])
                .sum();
            // Mirrored back on the way out: a PNP's currents run the other
            // way. The conductances need no mirroring — reversing both the
            // voltage and the current leaves dI/dV alone.
            currents[i] = sign * (current - accounted);
        }

        CoupledCompanion {
            terminals: 3,
            conductances,
            currents,
            points: {
                let mut points = [0.0; MAX_TERMINALS];
                points[0] = vbe;
                points[1] = vbc;
                points
            },
            limited: vbe != raw_be || vbc != raw_bc,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::netlist::{ElementKind, NetId, Netlist};
    use crate::solver::operating_point;

    use super::*;

    /// A common-emitter stage: supply through a collector resistor to the
    /// collector, and a base resistor from the same supply to the base.
    fn common_emitter(
        volts: f64,
        base_ohms: f64,
        collector_ohms: f64,
        polarity: Polarity,
    ) -> (Netlist, NetId, NetId, NetId) {
        let mut netlist = Netlist::new();
        let source = netlist.add_element(ElementKind::DcSource {
            volts,
            series_ohms: 0.0,
        });
        let base_resistor = netlist.add_element(ElementKind::Resistor { ohms: base_ohms });
        let collector_resistor = netlist.add_element(ElementKind::Resistor {
            ohms: collector_ohms,
        });
        let transistor = netlist.add_element(ElementKind::Bjt { polarity });

        let rail = netlist.add_net();
        let base = netlist.add_net();
        let collector = netlist.add_net();
        let ground = netlist.add_net();

        netlist.connect_named(source, "+", rail).unwrap();
        netlist.connect_named(source, "-", ground).unwrap();
        netlist.connect_named(base_resistor, "a", rail).unwrap();
        netlist.connect_named(base_resistor, "b", base).unwrap();
        netlist
            .connect_named(collector_resistor, "a", rail)
            .unwrap();
        netlist
            .connect_named(collector_resistor, "b", collector)
            .unwrap();
        netlist.connect_named(transistor, "b", base).unwrap();
        netlist.connect_named(transistor, "c", collector).unwrap();
        netlist.connect_named(transistor, "e", ground).unwrap();

        (netlist, base, collector, ground)
    }

    #[test]
    fn a_common_emitter_stage_biases() {
        // 9 V, a 470 kΩ base resistor and a 1 kΩ collector load. The base sits
        // a diode drop above the emitter, and β times the base current runs in
        // the collector.
        let (netlist, base, collector, ground) = common_emitter(9.0, 470e3, 1e3, Polarity::Npn);

        let point = operating_point(&netlist).expect("a biased stage solves");

        let vbe = point.voltage(base).unwrap() - point.voltage(ground).unwrap();
        assert!((0.55..0.75).contains(&vbe), "Vbe {vbe} V");

        // Ib ≈ (9 − 0.65) / 470k ≈ 17.8 µA, so Ic ≈ 1.78 mA and the 1 kΩ load
        // drops about 1.8 V, leaving the collector near 7.2 V.
        let vc = point.voltage(collector).unwrap() - point.voltage(ground).unwrap();
        assert!((6.5..8.0).contains(&vc), "Vc {vc} V");
        assert!(vc > vbe, "active region: the collector is above the base");
    }

    #[test]
    fn a_pnp_is_the_same_stage_upside_down() {
        // Every voltage mirrored: the supply is negative, the base sits a
        // diode drop below the emitter, and the collector rises towards it.
        let (netlist, base, collector, ground) = common_emitter(-9.0, 470e3, 1e3, Polarity::Pnp);

        let point = operating_point(&netlist).expect("a PNP stage solves");

        let vbe = point.voltage(base).unwrap() - point.voltage(ground).unwrap();
        assert!((-0.75..-0.55).contains(&vbe), "Vbe {vbe} V");

        let vc = point.voltage(collector).unwrap() - point.voltage(ground).unwrap();
        assert!((-8.0..-6.5).contains(&vc), "Vc {vc} V");
    }

    #[test]
    fn a_base_left_open_holds_the_transistor_off() {
        // Cutoff: no base current, so no collector current, so the collector
        // sits at the supply.
        let (netlist, _, collector, ground) = common_emitter(9.0, 1e12, 1e3, Polarity::Npn);

        let point = operating_point(&netlist).expect("a cut-off stage solves");

        let vc = point.voltage(collector).unwrap() - point.voltage(ground).unwrap();
        assert!(vc > 8.99, "Vc {vc} V — it should be sitting at the rail");
    }

    #[test]
    fn a_switching_transistor_saturates() {
        // Plenty of base drive: the collector is pulled down to a couple of
        // hundred millivolts and stays there, which is what makes it a switch.
        let (netlist, base, collector, ground) = common_emitter(9.0, 4.7e3, 1e3, Polarity::Npn);

        let point = operating_point(&netlist).expect("a saturated stage solves");

        let vc = point.voltage(collector).unwrap() - point.voltage(ground).unwrap();
        let vb = point.voltage(base).unwrap() - point.voltage(ground).unwrap();
        assert!(vc < 0.4, "Vce(sat) {vc} V");
        assert!(vc < vb, "saturated: the base is above the collector");
    }

    #[test]
    fn a_darlington_needs_two_junctions_of_base_and_never_saturates_flat() {
        // 12 V, 1 kΩ to the base and 12 Ω in the collector: about 10 mA of
        // base for an amp of load, far more than a β of 1000 needs, so it is
        // saturated — and its V_CE stays near 0.7 V, where a single
        // transistor's goes to a couple of hundred millivolts (C-112).
        let (netlist, base, collector, ground) =
            common_emitter(12.0, 1e3, 12.0, Polarity::Darlington);
        let point = operating_point(&netlist).expect("a Darlington stage solves");
        let at = |net| point.voltage(net).unwrap() - point.voltage(ground).unwrap();
        assert!((1.3..1.6).contains(&at(base)), "Vbe {} V", at(base));
        assert!(
            (0.6..1.0).contains(&at(collector)),
            "Vce(sat) {} V",
            at(collector)
        );
    }

    #[test]
    fn a_darlington_gains_a_thousand() {
        // A megohm to the base: about 10.6 µA, and a thousand times that in
        // the collector, its 12 Ω dropping a tenth of a volt — well active.
        let (netlist, base, collector, ground) =
            common_emitter(12.0, 1e6, 12.0, Polarity::Darlington);
        let point = operating_point(&netlist).unwrap();
        let at = |net| point.voltage(net).unwrap() - point.voltage(ground).unwrap();
        let base_amps = (12.0 - at(base)) / 1e6;
        let collector_amps = (12.0 - at(collector)) / 12.0;
        let gain = collector_amps / base_amps;
        assert!((900.0..1100.0).contains(&gain), "β {gain}");
    }

    #[test]
    fn its_terminal_currents_balance() {
        let bjt = Bjt::new(Polarity::Darlington);
        let mut potentials = [0.0; MAX_TERMINALS];
        potentials[BASE] = 1.4;
        potentials[COLLECTOR] = 5.0;
        let [b, c, e] = bjt.terminal_currents(&potentials);
        assert!((b + c + e).abs() < 1e-12);
        assert!(
            c > 0.5 && b > 0.0,
            "{b} A in at the base, {c} A at the collector"
        );
    }

    #[test]
    fn beta_sets_the_collector_current() {
        // The defining relationship, read off two stages that differ only in
        // base drive: twice the base current is twice the collector current.
        let (one, _, collector_one, ground_one) = common_emitter(9.0, 940e3, 1e3, Polarity::Npn);
        let (two, _, collector_two, ground_two) = common_emitter(9.0, 470e3, 1e3, Polarity::Npn);

        let point_one = operating_point(&one).unwrap();
        let point_two = operating_point(&two).unwrap();

        let drop = |point: &crate::solver::OperatingPoint, collector, ground| {
            9.0 - (point.voltage(collector).unwrap() - point.voltage(ground).unwrap())
        };
        let half = drop(&point_one, collector_one, ground_one);
        let full = drop(&point_two, collector_two, ground_two);

        assert!((full / half - 2.0).abs() < 0.1, "{half} V then {full} V");
    }
}
