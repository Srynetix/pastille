// SPDX-License-Identifier: MIT OR Apache-2.0

//! A battery that runs down (C-121).
//!
//! The ideal source (C-17) holds its voltage for ever; a real battery is a
//! voltage that falls as its charge is used, behind a resistance that makes
//! it sag under load, until it is flat. It is bought by its type, not typed
//! in — a "9 V with 2 000 mAh" is a part nobody can buy (ADR 0010, term 3) —
//! so each type here is a real one, from its maker's sheet:
//!
//! ```text
//! V = E(depth) − I·R          depth′ = I / C(I)
//! ```
//!
//! **E(depth)** is the sheet's discharge curve under its lightest load, read
//! off the graph at a handful of points and joined straight, with what that
//! light load dropped across R put back. **C(I)** is its capacity at the
//! current it is giving: the sheet's capacity bars, joined straight on log
//! scales and carried on past the last, so a heavy load empties it sooner
//! than a light one — which is what an alkaline does, and the larger part of
//! why a 9 V is a poor choice for a motor. **R** is the sheet's own internal
//! resistance where it gives one, and where it does not, the one its curves
//! imply, said for each.
//!
//! **Its charge is a dial**, 0 to 100 % left, as an LDR's light is: real
//! time hides discharge — a coin cell under an LED lasts days — so a
//! beginner sets how full it is to see what a nearly flat one does, and reads
//! how long it has left at the current it is giving rather than waiting. The
//! run draws it down from where the dial was set, at the circuit's own pace.
//!
//! What it ignores: temperature; recovery when rested, which an alkaline
//! does; the voltage at no load being higher than any curve shows; leakage
//! on the shelf; charging, which these cannot take.

use crate::solver::Charge;

/// The batteries there are.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum CellPart {
    #[default]
    NineVolt,
    FourAa,
    Aa,
    Coin,
}

/// One battery's sheet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sheet {
    pub part: &'static str,
    pub datasheet: &'static str,
    /// Its voltage at each depth of discharge, 0 to 1, under its lightest
    /// load, with that load's drop across R put back.
    curve: &'static [(f64, f64)],
    /// Its capacity in mAh at each current in mA, from its bars.
    capacity: &'static [(f64, f64)],
    /// Its internal resistance, in ohms.
    pub ohms: f64,
    /// How many cells in series, each one of the sheet's.
    cells: f64,
}

/// Energizer 522 (form 522GL1019): its 620 Ω curve (about 13 mA) from 9.0 V
/// to 5.0 V over 50 hours, and 600, 565 and 440 mAh to 4.8 V at 10, 25 and
/// 50 mA. No internal resistance on the sheet; its older revision's
/// constant-current curve reaches 4.8 V after about an hour at 300 mA, half
/// way down the 620 Ω curve, where that curve is at 7.55 V — so
/// 4.8 = 7.55 + 0.013·R − 0.3·R, R = 9.6 Ω: an effective figure, for a load
/// of a motor's sort.
const NINE_VOLT: Sheet = Sheet {
    part: "Energizer 522",
    datasheet: "https://data.energizer.com/pdfs/522.pdf",
    curve: &[
        (0.0, 9.12),
        (0.1, 8.52),
        (0.2, 8.22),
        (0.4, 7.82),
        (0.6, 7.47),
        (0.74, 7.12),
        (0.88, 6.52),
        (1.0, 5.12),
    ],
    capacity: &[(10.0, 600.0), (25.0, 565.0), (50.0, 440.0)],
    ohms: 9.6,
    cells: 1.0,
};

/// Energizer E91 (form E91NA1018): its 50 mA curve from 1.5 V to 0.8 V over
/// 57 hours, 3 000, 2 500, 2 000 and 1 500 mAh to 0.8 V at 25, 100, 250 and
/// 500 mA, and its own nominal internal resistance, 150 to 300 mΩ fresh: the
/// middle.
const AA: Sheet = Sheet {
    part: "Energizer E91",
    datasheet: "https://data.energizer.com/pdfs/e91.pdf",
    curve: &[
        (0.0, 1.511),
        (0.175, 1.411),
        (0.35, 1.331),
        (0.53, 1.271),
        (0.7, 1.211),
        (0.79, 1.111),
        (0.88, 1.011),
        (1.0, 0.811),
    ],
    capacity: &[
        (25.0, 3000.0),
        (100.0, 2500.0),
        (250.0, 2000.0),
        (500.0, 1500.0),
    ],
    ohms: 0.225,
    cells: 1.0,
};

/// Energizer CR2032: 235 mAh to 2.0 V under 15 kΩ (0.19 mA), flat near
/// 2.9 V for most of it and falling away at the end; 2.9 V at its 0.19 mA
/// drain and 2.7 V on its 6.8 mA pulse, so R = 0.2 V / 6.6 mA, 30 Ω. One
/// capacity only: the sheet gives none at a heavier drain, where a real coin
/// cell gives less.
const COIN: Sheet = Sheet {
    part: "Energizer CR2032",
    datasheet: "https://data.energizer.com/pdfs/cr2032.pdf",
    curve: &[
        (0.0, 2.906),
        (0.5, 2.906),
        (0.7, 2.856),
        (0.8, 2.806),
        (0.9, 2.656),
        (0.95, 2.456),
        (1.0, 2.006),
    ],
    capacity: &[(0.19, 235.0)],
    ohms: 30.0,
    cells: 1.0,
};

impl CellPart {
    pub const ALL: &'static [CellPart] = &[
        CellPart::NineVolt,
        CellPart::FourAa,
        CellPart::Aa,
        CellPart::Coin,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            CellPart::NineVolt => "9 V",
            CellPart::FourAa => "4 × AA",
            CellPart::Aa => "AA",
            CellPart::Coin => "Coin cell",
        }
    }

    pub fn from_name(name: &str) -> Option<CellPart> {
        CellPart::ALL
            .iter()
            .copied()
            .find(|part| part.name() == name)
    }

    pub fn sheet(&self) -> Sheet {
        match self {
            CellPart::NineVolt => NINE_VOLT,
            CellPart::FourAa => Sheet { cells: 4.0, ..AA },
            CellPart::Aa => AA,
            CellPart::Coin => COIN,
        }
    }
}

/// Straight between the points either side of `x`, held at the ends.
fn between(points: &[(f64, f64)], x: f64) -> f64 {
    let first = points[0];
    let last = points[points.len() - 1];
    if x <= first.0 {
        return first.1;
    }
    if x >= last.0 {
        return last.1;
    }
    let after = points
        .iter()
        .position(|&(at, _)| at >= x)
        .unwrap_or(points.len() - 1);
    let (x0, y0) = points[after - 1];
    let (x1, y1) = points[after];
    y0 + (y1 - y0) * (x - x0) / (x1 - x0)
}

/// How far past empty it falls to nothing: a flat battery under load is a
/// volt or so, then nothing.
const PAST_FLAT: f64 = 0.05;

impl Sheet {
    /// Its voltage with nothing drawn at `depth`, 0 full to 1 flat.
    pub fn open_volts(&self, depth: f64) -> f64 {
        let end = self.curve[self.curve.len() - 1].1;
        let volts = if depth <= 1.0 {
            between(self.curve, depth.max(0.0))
        } else {
            end * (1.0 - (depth - 1.0) / PAST_FLAT).max(0.0)
        };
        volts * self.cells
    }

    /// Its whole resistance, cells in series.
    pub fn series_ohms(&self) -> f64 {
        self.ohms * self.cells
    }

    /// Its capacity in mAh when giving `amps`.
    pub fn capacity(&self, amps: f64) -> f64 {
        let ma = (amps.abs() * 1e3).max(1e-6);
        let points = self.capacity;
        if points.len() == 1 {
            return points[0].1;
        }
        let logs: Vec<(f64, f64)> = points.iter().map(|&(i, c)| (i.ln(), c.ln())).collect();
        let x = ma.ln();
        // Carried on past either end along the line from the first bar to
        // the last — a Peukert exponent fitted to the whole sheet, which puts
        // the 9 V's 300 mA where its older sheet has it — but never more than
        // its lightest-load capacity.
        let (a, b) = if x <= logs[0].0 || x >= logs[logs.len() - 1].0 {
            (logs[0], logs[logs.len() - 1])
        } else {
            let after = logs.iter().position(|&(at, _)| at >= x).unwrap();
            (logs[after - 1], logs[after])
        };
        let c = a.1 + (b.1 - a.1) * (x - a.0) / (b.0 - a.0);
        c.exp().min(points[0].1)
    }

    /// Hours left at `amps`, from `depth`; `None` with nothing drawn.
    pub fn hours_left(&self, depth: f64, amps: f64) -> Option<f64> {
        if amps <= 1e-9 {
            return None;
        }
        Some((1.0 - depth).max(0.0) * self.capacity(amps) * 1e-3 / amps)
    }
}

/// What the run carries for a battery: how much of it has been drawn since
/// its dial was set, as a share of the whole.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct State {
    pub drawn: f64,
}

impl State {
    pub fn encode(self) -> Charge {
        Charge::settled(self.drawn)
    }

    pub fn decode(carried: Charge) -> State {
        State {
            drawn: carried.value,
        }
    }

    /// Where it is, 0 full to 1 flat, set to `left` on its dial.
    pub fn depth(self, left: f64) -> f64 {
        1.0 - left.clamp(0.0, 1.0) + self.drawn
    }

    /// After `seconds` giving `amps`.
    pub fn advance(self, sheet: &Sheet, amps: f64, seconds: f64) -> State {
        let amps = amps.max(0.0);
        let share = amps * seconds.max(0.0) / 3600.0 / (sheet.capacity(amps) * 1e-3);
        State {
            drawn: self.drawn + share,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_full_battery_gives_its_sheet_s_voltage_and_sags_by_its_resistance() {
        let nine = CellPart::NineVolt.sheet();
        // At the 620 Ω test's 13 mA, the sheet's 9.0 V; at 300 mA, 2.8 V less.
        let at = |amps: f64| nine.open_volts(0.0) - amps * nine.series_ohms();
        assert!((at(0.013) - 9.0).abs() < 0.01, "{}", at(0.013));
        assert!((at(0.013) - at(0.3) - 2.76).abs() < 0.01);
        let four = CellPart::FourAa.sheet();
        assert!((four.open_volts(0.0) - 6.04).abs() < 0.01);
        assert!((four.series_ohms() - 0.9).abs() < 1e-9);
    }

    #[test]
    fn its_voltage_follows_the_curve_down_to_its_knee_and_then_nothing() {
        let aa = CellPart::Aa.sheet();
        let mut last = f64::INFINITY;
        for step in 0..=100 {
            let volts = aa.open_volts(step as f64 / 100.0);
            assert!(volts <= last, "it never rises as it empties");
            last = volts;
        }
        // The knee: the last fifth falls at least twice as steeply as the
        // first half.
        let first_half = (aa.open_volts(0.0) - aa.open_volts(0.5)) / 0.5;
        let last_fifth = (aa.open_volts(0.8) - aa.open_volts(1.0)) / 0.2;
        assert!(
            last_fifth > 2.0 * first_half,
            "{first_half} then {last_fifth} V a unit"
        );
        assert_eq!(aa.open_volts(1.0 + PAST_FLAT), 0.0);
    }

    #[test]
    fn its_capacity_is_the_sheet_s_at_each_current_and_less_under_more() {
        let aa = CellPart::Aa.sheet();
        for (ma, mah) in [
            (25.0, 3000.0),
            (100.0, 2500.0),
            (250.0, 2000.0),
            (500.0, 1500.0),
        ] {
            assert!((aa.capacity(ma / 1e3) - mah).abs() < 1e-6, "{ma} mA");
        }
        assert!(aa.capacity(1.0) < 1500.0);
        assert!(aa.capacity(0.001) <= 3000.0);
        // Its older sheet's 300 mAh at 300 mA, from the bars carried on.
        let nine = CellPart::NineVolt.sheet();
        assert!(
            (nine.capacity(0.3) - 300.0).abs() < 30.0,
            "{}",
            nine.capacity(0.3)
        );
    }

    #[test]
    fn the_charge_drawn_is_the_current_over_the_time() {
        // 25 mA for an hour out of an AA's 2 772 mAh at that rate is 0.9 %.
        let aa = CellPart::Aa.sheet();
        let mut state = State::default();
        for _ in 0..3600 {
            state = state.advance(&aa, 0.025, 1.0);
        }
        let expected = 25.0 / aa.capacity(0.025);
        assert!((state.drawn - expected).abs() < 1e-9, "{}", state.drawn);
        // And how long it has left at that rate is the rest over the rate.
        let hours = aa.hours_left(state.depth(1.0), 0.025).unwrap();
        assert!((hours - (1.0 - expected) * aa.capacity(0.025) / 25.0).abs() < 1e-9);
        assert_eq!(aa.hours_left(0.0, 0.0), None);
    }

    #[test]
    fn the_coin_cell_s_pulse_is_its_sheet_s() {
        let coin = CellPart::Coin.sheet();
        let at = |amps: f64| coin.open_volts(0.1) - amps * coin.series_ohms();
        assert!((at(0.19e-3) - 2.9).abs() < 0.01);
        assert!((at(6.8e-3) - 2.7).abs() < 0.01);
    }
}
