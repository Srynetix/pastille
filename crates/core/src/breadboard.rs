// SPDX-License-Identifier: MIT OR Apache-2.0

//! The breadboard, as data (B-12, design doc §6.3).
//!
//! What the breadboard view draws and what the build steps read out — "the
//! resistor from 12c to 18c" — are both questions about one object: which
//! holes there are, where each sits, and which are joined underneath. That is
//! this module, and nothing about drawing is in it.
//!
//! **The board** is the solderless kind every kit holds, in its two sizes: a
//! half (30 rows, 400 holes) and a full (63 rows, 830). Each numbered row is
//! two strips of five joined holes, `a`–`e` and `f`–`j`, either side of a
//! channel 0.3″ across — the width of a chip, which is why a chip straddles
//! it. Along each long edge run two power rails, `+` outermost, their holes
//! in groups of five; each rail is joined end to end, as on most boards sold
//! now (some older full boards split theirs in the middle, and this one does
//! not). Positions are in pitches of 0.1″ (2.54 mm), the grid every lead
//! fits.
//!
//! **What is in it** is a [`Wiring`]: which hole each component's pin is in,
//! and the jumpers between holes. From that, [`Wiring::check`] says what a
//! person building it would find: two nets on one strip (a short the
//! schematic does not have), one net on strips nothing joins (a jumper
//! missing), two leads pushed into one hole. Placing a circuit so none of
//! those happen is B-13's; drawing it, B-14's.

use serde::{Deserialize, Serialize};

use crate::netlist::{ElementId, ElementKind, NetId, Netlist, PinRef};

/// The board's distance between neighbouring holes, in millimetres.
pub const PITCH_MM: f64 = 2.54;

/// Holes a strip of a row holds, and a group of a rail.
const STRIP: u16 = 5;

/// One of the two sizes kits come in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Size {
    /// 30 rows: 300 holes in rows, 100 on the rails.
    Half,
    /// 63 rows: 630 in rows, 200 on the rails.
    Full,
}

/// A row's columns, `a` to `j`, as an index from 0.
pub const COLUMNS: [char; 10] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j'];

/// Which side of the channel a strip is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Side {
    /// `a`–`e`.
    Upper,
    /// `f`–`j`.
    Lower,
}

/// One of the four power rails.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rail {
    TopPlus,
    TopMinus,
    BottomMinus,
    BottomPlus,
}

impl Rail {
    pub const ALL: [Rail; 4] = [
        Rail::TopPlus,
        Rail::TopMinus,
        Rail::BottomMinus,
        Rail::BottomPlus,
    ];

    /// What it is called where a person reads it.
    pub fn name(self) -> &'static str {
        match self {
            Rail::TopPlus => "top + rail",
            Rail::TopMinus => "top − rail",
            Rail::BottomMinus => "bottom − rail",
            Rail::BottomPlus => "bottom + rail",
        }
    }
}

/// A hole.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Hole {
    /// In a numbered row, from 1, and a column, `a` = 0 to `j` = 9.
    Row { row: u16, column: u8 },
    /// On a rail, its holes counted from 1 along it.
    Rail { rail: Rail, index: u16 },
}

/// A set of holes joined underneath by one metal clip: what a lead in any of
/// them touches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Strip {
    Row { row: u16, side: Side },
    Rail(Rail),
}

impl Hole {
    /// Where a person reads it: "12c", "top + rail".
    pub fn name(self) -> String {
        match self {
            Hole::Row { row, column } => format!("{row}{}", COLUMNS[usize::from(column)]),
            Hole::Rail { rail, .. } => rail.name().to_string(),
        }
    }

    /// The strip it is part of.
    pub fn strip(self) -> Strip {
        match self {
            Hole::Row { row, column } => Strip::Row {
                row,
                side: if column < 5 { Side::Upper } else { Side::Lower },
            },
            Hole::Rail { rail, .. } => Strip::Rail(rail),
        }
    }
}

/// A breadboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Board {
    pub size: Size,
}

impl Board {
    pub const fn new(size: Size) -> Board {
        Board { size }
    }

    /// How many numbered rows it has.
    pub fn rows(self) -> u16 {
        match self.size {
            Size::Half => 30,
            Size::Full => 63,
        }
    }

    /// How many holes each rail has: as many groups of five, with a hole's
    /// gap between groups, as fit along the rows.
    pub fn rail_holes(self) -> u16 {
        // A group and its gap take six pitches; the last group needs no gap.
        (self.rows() + 1) / (STRIP + 1) * STRIP
    }

    /// Whether a hole is on this board.
    pub fn has(self, hole: Hole) -> bool {
        match hole {
            Hole::Row { row, column } => (1..=self.rows()).contains(&row) && column < 10,
            Hole::Rail { index, .. } => (1..=self.rail_holes()).contains(&index),
        }
    }

    /// Every hole, rows first, then the rails.
    pub fn holes(self) -> impl Iterator<Item = Hole> {
        let rows =
            (1..=self.rows()).flat_map(|row| (0..10).map(move |column| Hole::Row { row, column }));
        let rails = Rail::ALL.into_iter().flat_map(move |rail| {
            (1..=self.rail_holes()).map(move |index| Hole::Rail { rail, index })
        });
        rows.chain(rails)
    }

    /// Where a hole is, in pitches from the board's top-left hole: `x` along
    /// the rows, `y` across them. `None` for a hole not on this board.
    ///
    /// Across, from the top: the top rails' `+` and `−`, then `a`–`e`, the
    /// channel, `f`–`j`, and the bottom rails' `−` and `+`. The channel is
    /// three pitches from `e` to `f`, so a chip's 0.3″ rows of legs land in
    /// both; a rail is two pitches from its nearest row.
    pub fn position(self, hole: Hole) -> Option<(f64, f64)> {
        if !self.has(hole) {
            return None;
        }
        Some(match hole {
            Hole::Row { row, column } => {
                let y = match column {
                    0..=4 => 3 + u16::from(column),
                    _ => 10 + u16::from(column - 5),
                };
                (f64::from(row - 1), f64::from(y))
            }
            Hole::Rail { rail, index } => {
                let y = match rail {
                    Rail::TopPlus => 0,
                    Rail::TopMinus => 1,
                    Rail::BottomMinus => 16,
                    Rail::BottomPlus => 17,
                };
                // Groups of five with a gap between, centred on the rows.
                let span = self.rail_holes() / STRIP * (STRIP + 1) - 1;
                let start = (self.rows() - span) / 2;
                let before = index - 1;
                let x = start + before / STRIP * (STRIP + 1) + before % STRIP;
                (f64::from(x), f64::from(y))
            }
        })
    }

    /// The hole at a position, in pitches, if there is one there.
    pub fn hole_at(self, x: f64, y: f64) -> Option<Hole> {
        let (x, y) = (x.round(), y.round());
        if x < 0.0 || y < 0.0 {
            return None;
        }
        let (x, y) = (x as u16, y as u16);
        let column = match y {
            3..=7 => Some(y - 3),
            10..=14 => Some(y - 5),
            _ => None,
        };
        if let Some(column) = column {
            let hole = Hole::Row {
                row: x + 1,
                column: column as u8,
            };
            return self.has(hole).then_some(hole);
        }
        let rail = match y {
            0 => Rail::TopPlus,
            1 => Rail::TopMinus,
            16 => Rail::BottomMinus,
            17 => Rail::BottomPlus,
            _ => return None,
        };
        (1..=self.rail_holes())
            .map(|index| Hole::Rail { rail, index })
            .find(|&hole| self.position(hole) == Some((f64::from(x), f64::from(y))))
    }

    /// The holes of a strip.
    pub fn holes_of(self, strip: Strip) -> Vec<Hole> {
        match strip {
            Strip::Row { row, side } => {
                let first = match side {
                    Side::Upper => 0,
                    Side::Lower => 5,
                };
                (first..first + 5)
                    .map(|column| Hole::Row { row, column })
                    .filter(|&hole| self.has(hole))
                    .collect()
            }
            Strip::Rail(rail) => (1..=self.rail_holes())
                .map(|index| Hole::Rail { rail, index })
                .collect(),
        }
    }
}

/// What is pushed into a board: each pin's hole, and the jumpers.
///
/// Not saved yet: a circuit file holds the schematic, and the board's layout
/// joins it when there is a view to make one in (B-14).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Wiring {
    pub pins: Vec<(PinRef, Hole)>,
    pub jumpers: Vec<(Hole, Hole)>,
}

/// What a person building it would find (B-12): each an empty list when the
/// board does what the schematic says.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Findings {
    /// Strips joined together, and the nets they hold: two nets one place,
    /// a short the schematic does not have.
    pub shorts: Vec<(Vec<Strip>, Vec<NetId>)>,
    /// A net whose pins are on strips nothing joins, each group of joined
    /// strips it is on: a jumper missing.
    pub splits: Vec<(NetId, Vec<Vec<Strip>>)>,
    /// A hole holding more than one lead or jumper end.
    pub crowded: Vec<Hole>,
    /// A pin or a jumper end in a hole this board does not have.
    pub off_board: Vec<Hole>,
}

impl Findings {
    pub fn is_clean(&self) -> bool {
        self.shorts.is_empty()
            && self.splits.is_empty()
            && self.crowded.is_empty()
            && self.off_board.is_empty()
    }
}

impl Wiring {
    /// Which strips are one node, joined by a jumper or by being the same
    /// strip: each group, in the order its first strip appears.
    pub fn nodes(&self) -> Vec<Vec<Strip>> {
        let mut groups: Vec<Vec<Strip>> = Vec::new();
        let find = |groups: &Vec<Vec<Strip>>, strip: Strip| {
            groups.iter().position(|group| group.contains(&strip))
        };
        let strips = self.pins.iter().map(|&(_, hole)| hole.strip()).chain(
            self.jumpers
                .iter()
                .flat_map(|&(a, b)| [a.strip(), b.strip()]),
        );
        for strip in strips {
            if find(&groups, strip).is_none() {
                groups.push(vec![strip]);
            }
        }
        for &(a, b) in &self.jumpers {
            let (Some(first), Some(second)) = (find(&groups, a.strip()), find(&groups, b.strip()))
            else {
                continue;
            };
            if first != second {
                let (keep, merge) = (first.min(second), first.max(second));
                let moved = groups.remove(merge);
                groups[keep].extend(moved);
            }
        }
        groups
    }

    /// Holds this wiring to the netlist it is meant to build.
    ///
    /// Only pins that are placed are judged: a part not yet on the board is
    /// the board being built, not a mistake on it. A pin wired to nothing in
    /// the schematic is in its hole for nothing, and is not a net.
    pub fn check(&self, board: Board, netlist: &Netlist) -> Findings {
        let mut findings = Findings::default();
        let ends = self
            .pins
            .iter()
            .map(|&(_, hole)| hole)
            .chain(self.jumpers.iter().flat_map(|&(a, b)| [a, b]));
        let mut seen: Vec<Hole> = Vec::new();
        for hole in ends {
            if !board.has(hole) {
                if !findings.off_board.contains(&hole) {
                    findings.off_board.push(hole);
                }
            } else if seen.contains(&hole) {
                if !findings.crowded.contains(&hole) {
                    findings.crowded.push(hole);
                }
            } else {
                seen.push(hole);
            }
        }

        let nodes = self.nodes();
        let node_of = |hole: Hole| nodes.iter().position(|node| node.contains(&hole.strip()));
        let net_of = |pin: PinRef| node_net(netlist, pin);
        // Each node's nets, and each net's nodes.
        let mut nets_on: Vec<Vec<NetId>> = vec![Vec::new(); nodes.len()];
        let mut nodes_of: Vec<(NetId, Vec<usize>)> = Vec::new();
        for &(pin, hole) in &self.pins {
            let (Some(net), Some(node)) = (net_of(pin), node_of(hole)) else {
                continue;
            };
            if !nets_on[node].contains(&net) {
                nets_on[node].push(net);
            }
            match nodes_of.iter_mut().find(|(other, _)| *other == net) {
                Some((_, list)) if !list.contains(&node) => list.push(node),
                Some(_) => {}
                None => nodes_of.push((net, vec![node])),
            }
        }
        for (node, nets) in nets_on.into_iter().enumerate() {
            if nets.len() > 1 {
                findings.shorts.push((nodes[node].clone(), nets));
            }
        }
        for (net, list) in nodes_of {
            if list.len() > 1 {
                findings.splits.push((
                    net,
                    list.into_iter().map(|node| nodes[node].clone()).collect(),
                ));
            }
        }
        findings
    }
}

/// The net a pin is on, as the board sees it: every net with a ground symbol
/// is one node (ADR 0003), so they are all the first of them.
fn node_net(netlist: &Netlist, pin: PinRef) -> Option<NetId> {
    let net = netlist
        .element(pin.element)
        .and_then(|element| element.pins.get(pin.index))
        .and_then(|pin| pin.net)?;
    if grounded(netlist, net) {
        return netlist
            .nets()
            .map(|(id, _)| id)
            .find(|&id| grounded(netlist, id));
    }
    Some(net)
}

/// Whether a net has a ground symbol on it.
fn grounded(netlist: &Netlist, net: NetId) -> bool {
    netlist.net(net).is_some_and(|net| {
        net.pins().iter().any(|pin| {
            netlist
                .element(pin.element)
                .is_some_and(|element| matches!(element.kind, ElementKind::Ground))
        })
    })
}

/// Whether a part sits off the board with its leads in the rails: a battery,
/// or a signal generator's clips.
pub fn off_board(kind: &ElementKind) -> bool {
    matches!(
        kind,
        ElementKind::DcSource { .. }
            | ElementKind::Cell { .. }
            | ElementKind::SignalGenerator { .. }
    )
}

/// Whether a part is not on the board at all, its own or its leads' — an
/// Uno, which sits beside the board (A-02). Jumpering its headers to the
/// rows is A-11; until then it takes no hole.
pub fn beside_board(kind: &ElementKind) -> bool {
    matches!(kind, ElementKind::Uno)
}

/// Package pin numbers 1 to 18, for a chip whose model already lists its
/// pins in the order the package does.
const IN_ORDER: [u8; 18] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
];

/// A chip's DIP: how many pins the package has, and for each of the model's
/// pins, in the model's order, its pin number on the package. `None` for a
/// part that is not a chip in a DIP.
///
/// Most models list their pins round the package already. The rest, and the
/// pins they leave out — a 555's RESET and CTRL, an L293D's other three GND,
/// a CD4026's DEO and ungated c — are from each maker's sheet; a package pin
/// the model has none for gets a row and nothing in it.
///
/// The seven not in order were read off TI's and Vishay's sheets on
/// 2026-10-02 (NE555 SLFS022K, LM358 SLOS068AB, LM393 SLCS005AH, 4N35
/// 81181, L293D SLRS008D, CD4511B SCHS072B, CD4026B SCHS031B).
pub fn dip(kind: &ElementKind) -> Option<(u8, &'static [u8])> {
    Some(match kind {
        // VCC, GND, TRIG, THR, DIS, OUT.
        ElementKind::Timer => (8, &[8, 1, 2, 6, 7, 3]),
        // in+, in−, out, V+, V−: the first of the package's two.
        ElementKind::OpAmp | ElementKind::Comparator => (8, &[3, 2, 1, 8, 4]),
        // A, K, C, E.
        ElementKind::Optocoupler => (6, &[1, 2, 5, 4]),
        // 1,2EN 1A 1Y 2Y 2A VCC2 3,4EN 3A 3Y 4Y 4A VCC1 GND.
        ElementKind::MotorDriver => (16, &[1, 2, 3, 6, 7, 8, 9, 10, 11, 14, 15, 16, 4]),
        // A B C D LT BI LE, a–g, VDD VSS.
        ElementKind::Decoder => (16, &[7, 1, 2, 6, 3, 4, 5, 13, 12, 11, 10, 9, 15, 14, 16, 8]),
        // CLK INH RST DEI CO, a–g, VDD VSS. Its c is pin 13; 14 is the
        // ungated c, which the model has none of.
        ElementKind::Counter => (16, &[1, 2, 15, 3, 5, 10, 12, 13, 9, 11, 6, 7, 16, 8]),
        ElementKind::ShiftRegister | ElementKind::Decade => (16, &IN_ORDER[..16]),
        ElementKind::Gate { .. }
        | ElementKind::Inverter
        | ElementKind::Schmitt
        | ElementKind::FlipFlop => (14, &IN_ORDER[..14]),
        ElementKind::Lm3914 => (18, &IN_ORDER[..18]),
        _ => return None,
    })
}

/// The pins of a chip the model leaves out and a real one must have tied,
/// each with its name and the model pin whose net it is tied to (B-15): a
/// 555's RESET,
/// pin 4, to VCC — left free it can hold the chip reset, and the circuit
/// that works on the screen does nothing on the bench.
pub fn ties(kind: &ElementKind) -> &'static [(u8, &'static str, usize)] {
    match kind {
        ElementKind::Timer => &[(4, "RESET", 0)],
        _ => &[],
    }
}

/// A circuit placed on a board (B-13).
#[derive(Debug, Clone, PartialEq)]
pub struct Placement {
    pub board: Board,
    pub wiring: Wiring,
}

/// Why a circuit could not be placed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaceError {
    /// More than a full board holds.
    TooBig,
}

/// Places a circuit on a board (B-13): the smallest of the two it fits,
/// with no row holding two nets, every chip across the channel, and jumpers
/// for what the rows do not join.
///
/// How, in order — each step a choice a person building it would make:
///
/// 1. The first battery's `+` and `−` go on the top rails, and so does any
///    net with a ground symbol, on the `−`: every ground is one node.
/// 2. Each chip straddles the channel, pin 1 at the top left: the first half
///    of its pins down column `e`, the rest back up column `f`, as on the
///    package. Each pin's strip is its net's.
/// 3. Each other part with three legs or more — a transistor, a regulator,
///    a pot — puts each leg in a row of its own, in rows side by side.
/// 4. A net steps 1 to 3 put on more than one strip is joined, each strip
///    jumpered to the nearest of the others.
/// 5. Each two-lead part — a resistor, an LED, a meter's probe — goes in
///    whole, after the parts it shares a net with: both leads where they are
///    nearest each other, in strips their nets already have (keeping a hole
///    free to jumper on from) or fresh ones jumpered to the nearest of their
///    net's, at the least cost in jumpers, in leads stretched further than
///    leads reach, in chips crossed and in parts drawn on top of each other.
///
/// A pin wired to nothing is left off the board. Ground symbols are not
/// parts and take no hole.
pub fn place(netlist: &Netlist) -> Result<Placement, PlaceError> {
    place_with(netlist, &[])
}

/// A part someone has put where they want it on the board (B-14): each of
/// its pins' holes, by pin index, `None` for a pin left out.
pub type Pinned = (ElementId, Vec<Option<Hole>>);

/// [`place`], keeping the parts someone has moved where they put them (B-14).
///
/// A moved part's pins go in first, and its strips are its nets'; everything
/// else is placed around it, in rows it does not use, and the jumpers are
/// worked out again. A move into a row another net already has is the
/// person's to make: the strip stays the first net's, and [`Wiring::check`]
/// says the board now joins what the schematic does not.
pub fn place_with(netlist: &Netlist, pinned: &[Pinned]) -> Result<Placement, PlaceError> {
    [Size::Half, Size::Full]
        .into_iter()
        .find_map(|size| {
            let board = Board::new(size);
            let fits = pinned
                .iter()
                .flat_map(|(_, holes)| holes.iter().flatten())
                .all(|&hole| board.has(hole));
            if !fits {
                return None;
            }
            let wiring = Placer::new(board, netlist, pinned).run().ok()?;
            Some(Placement { board, wiring })
        })
        .ok_or(PlaceError::TooBig)
}

struct Placer<'a> {
    board: Board,
    netlist: &'a Netlist,
    pinned: &'a [Pinned],
    wiring: Wiring,
    used: Vec<Hole>,
    /// Which strips are taken, and by which net (`None`: a chip's pin with
    /// nothing on it).
    owner: Vec<(Strip, Option<NetId>)>,
    /// Each net's strips, in the order it was given them.
    strips: Vec<(NetId, Vec<Strip>)>,
    /// The first row nothing has been put in.
    next_row: u16,
    /// The rows each chip spans, first and last.
    chip_rows: Vec<(u16, u16)>,
    /// The leads placed so far, end to end, in pitches.
    leads_drawn: Vec<((f64, f64), (f64, f64))>,
}

/// What a jumper costs over its length, in pitches, weighing where a lead
/// goes: a part's leads further apart are better than a jumper more.
const JUMPER: f64 = 4.0;
/// What crossing a chip costs, a lead or a jumper drawn over it.
const OVER_CHIP: f64 = 8.0;
/// How far a part's leads reach before they cost more than their length,
/// and how much more, per pitch.
const REACH: f64 = 6.0;
const STRETCHED: f64 = 2.0;
/// What a part costs for each drawn about where it would be.
const CROWDED: f64 = 3.0;
/// What two leads one row apart cost, with no room to see the part.
const CRAMPED: f64 = 2.0;

/// Where a strip is, roughly, in pitches: a row's middle on its side.
fn middle(strip: Strip) -> (f64, f64) {
    match strip {
        Strip::Row { row, side } => (
            f64::from(row - 1),
            match side {
                Side::Upper => 5.0,
                Side::Lower => 12.0,
            },
        ),
        Strip::Rail(_) => (0.0, 0.0),
    }
}

/// Where a lead goes, and whether its strip is a fresh one.
type Choice = (Strip, bool);

/// Which holes of a strip a lead or a jumper leans to.
#[derive(Clone, Copy)]
enum Lean {
    Channel,
    Edge,
}

impl<'a> Placer<'a> {
    fn new(board: Board, netlist: &'a Netlist, pinned: &'a [Pinned]) -> Self {
        Placer {
            board,
            netlist,
            pinned,
            wiring: Wiring::default(),
            used: Vec::new(),
            owner: Vec::new(),
            strips: Vec::new(),
            next_row: 1,
            chip_rows: Vec::new(),
            leads_drawn: Vec::new(),
        }
    }

    fn net(&self, pin: PinRef) -> Option<NetId> {
        node_net(self.netlist, pin)
    }

    fn strips_of(&mut self, net: NetId) -> &mut Vec<Strip> {
        if let Some(index) = self.strips.iter().position(|(other, _)| *other == net) {
            return &mut self.strips[index].1;
        }
        self.strips.push((net, Vec::new()));
        &mut self.strips.last_mut().expect("just pushed").1
    }

    fn give(&mut self, strip: Strip, net: Option<NetId>) {
        self.owner.push((strip, net));
        if let Some(net) = net {
            self.strips_of(net).push(strip);
        }
    }

    fn free_holes(&self, strip: Strip) -> Vec<Hole> {
        self.board
            .holes_of(strip)
            .into_iter()
            .filter(|hole| !self.used.contains(hole))
            .collect()
    }

    fn take(&mut self, hole: Hole) {
        self.used.push(hole);
    }

    /// Whether a row has anything on either side.
    fn row_taken(&self, row: u16) -> bool {
        self.owner
            .iter()
            .any(|(strip, _)| matches!(strip, Strip::Row { row: other, .. } if *other == row))
            || self
                .used
                .iter()
                .any(|hole| matches!(hole, Hole::Row { row: other, .. } if *other == row))
    }

    /// Rows from `next_row` on, `count` of them side by side, nothing in
    /// either side of any; the row after is left as a gap, so two parts never
    /// share an edge.
    fn rows(&mut self, count: u16) -> Result<u16, PlaceError> {
        let last_first = (self.board.rows() + 1)
            .checked_sub(count)
            .ok_or(PlaceError::TooBig)?;
        let first = (self.next_row..=last_first)
            .find(|&first| (first..first + count).all(|row| !self.row_taken(row)))
            .ok_or(PlaceError::TooBig)?;
        self.next_row = first + count + 1;
        Ok(first)
    }

    fn is_pinned(&self, id: ElementId) -> bool {
        self.pinned.iter().any(|(other, _)| *other == id)
    }

    /// A strip nobody has, after every part already placed: the upper side
    /// of every other row, so the parts between two of them have room to be
    /// seen, then the lower sides the same way, then whatever is left.
    /// Chips and legs came first and took their rows, so these are after them.
    fn fresh(&mut self) -> Result<Strip, PlaceError> {
        let free = |strip: &Strip| !self.owner.iter().any(|(other, _)| other == strip);
        let rows = self.next_row..=self.board.rows();
        let spaced = |side: Side| {
            rows.clone()
                .step_by(2)
                .map(move |row| Strip::Row { row, side })
        };
        spaced(Side::Upper)
            .chain(spaced(Side::Lower))
            .chain(
                rows.clone().flat_map(|row| {
                    [Side::Upper, Side::Lower].map(|side| Strip::Row { row, side })
                }),
            )
            .find(free)
            .ok_or(PlaceError::TooBig)
    }

    /// A jumper between two strips, from the free holes of each nearest the
    /// other.
    fn join(&mut self, a: Strip, b: Strip) -> Result<(), PlaceError> {
        let (from, to) = self.nearest(a, b, Lean::Edge).ok_or(PlaceError::TooBig)?;
        self.take(from);
        self.take(to);
        self.wiring.jumpers.push((from, to));
        Ok(())
    }

    fn put(&mut self, pin: PinRef, hole: Hole) {
        self.take(hole);
        self.wiring.pins.push((pin, hole));
    }

    fn run(mut self) -> Result<Wiring, PlaceError> {
        self.rails();
        self.moved();
        self.chips()?;
        self.inline()?;
        self.join_all()?;
        self.leads()?;
        Ok(self.wiring)
    }

    /// Step 1.
    fn rails(&mut self) {
        let source = self.netlist.elements().find(|(_, element)| {
            matches!(
                element.kind,
                ElementKind::DcSource { .. } | ElementKind::Cell { .. }
            )
        });
        if let Some((id, _)) = source {
            if let Some(net) = self.net(PinRef::new(id, 0)) {
                self.give(Strip::Rail(Rail::TopPlus), Some(net));
            }
            if let Some(net) = self.net(PinRef::new(id, 1)) {
                self.give(Strip::Rail(Rail::TopMinus), Some(net));
            }
        }
        let ground = self
            .netlist
            .nets()
            .map(|(id, _)| id)
            .find(|&id| grounded(self.netlist, id));
        if let Some(ground) = ground
            && !self.strips.iter().any(|(net, _)| *net == ground)
        {
            let rail = if self
                .owner
                .iter()
                .any(|(strip, _)| *strip == Strip::Rail(Rail::TopMinus))
            {
                Rail::BottomMinus
            } else {
                Rail::TopMinus
            };
            self.give(Strip::Rail(rail), Some(ground));
        }
    }

    /// What someone has moved, where they put it (B-14).
    fn moved(&mut self) {
        for (id, holes) in self.pinned {
            let Some(element) = self.netlist.element(*id) else {
                continue;
            };
            for (index, hole) in holes.iter().enumerate().take(element.pins.len()) {
                let Some(hole) = *hole else { continue };
                let pin = PinRef::new(*id, index);
                let net = self.net(pin);
                let strip = hole.strip();
                if !self.owner.iter().any(|(other, _)| *other == strip) {
                    self.give(strip, net);
                }
                self.put(pin, hole);
            }
        }
    }

    /// Step 2.
    fn chips(&mut self) -> Result<(), PlaceError> {
        let chips: Vec<_> = self
            .netlist
            .elements()
            .filter(|(id, _)| !self.is_pinned(*id))
            .filter_map(|(id, element)| Some((id, dip(&element.kind)?, element.pins.len())))
            .collect();
        for (id, (count, numbers), pins) in chips {
            let half = u16::from(count / 2);
            let first = self.rows(half)?;
            self.chip_rows.push((first, first + half - 1));
            // Where package pin `n` sits.
            let hole = |n: u16| {
                if n <= half {
                    Hole::Row {
                        row: first + n - 1,
                        column: 4,
                    }
                } else {
                    Hole::Row {
                        row: first + 2 * half - n,
                        column: 5,
                    }
                }
            };
            for n in 1..=u16::from(count) {
                let model = numbers
                    .iter()
                    .take(pins)
                    .position(|&number| u16::from(number) == n);
                // A pin the model leaves out but the chip needs tied joins
                // the net it is tied to: its row is that net's, and is
                // jumpered to it as any of the net's rows are.
                let tied = self
                    .netlist
                    .element(id)
                    .map_or(&[][..], |element| ties(&element.kind))
                    .iter()
                    .find(|&&(pin, _, _)| u16::from(pin) == n)
                    .map(|&(_, _, to)| to);
                let net = model
                    .or(tied)
                    .and_then(|index| self.net(PinRef::new(id, index)));
                let at = hole(n);
                self.give(at.strip(), net);
                if let Some(index) = model {
                    self.put(PinRef::new(id, index), at);
                }
            }
        }
        Ok(())
    }

    /// Step 3.
    fn inline(&mut self) -> Result<(), PlaceError> {
        let parts: Vec<_> = self
            .netlist
            .elements()
            .filter(|(id, element)| {
                element.pins.len() > 2
                    && !self.is_pinned(*id)
                    && dip(&element.kind).is_none()
                    && !matches!(element.kind, ElementKind::Ground)
                    && !beside_board(&element.kind)
            })
            .map(|(id, element)| (id, element.pins.len()))
            .collect();
        for (id, legs) in parts {
            let first = self.rows(legs as u16)?;
            for index in 0..legs {
                let pin = PinRef::new(id, index);
                let at = Hole::Row {
                    row: first + index as u16,
                    column: 4,
                };
                let net = self.net(pin);
                self.give(at.strip(), net);
                if net.is_some() {
                    self.put(pin, at);
                }
            }
        }
        Ok(())
    }

    /// Step 4, for what steps 1 to 3 gave a net more than one of: each strip
    /// jumpered to the nearest of those already joined, so the jumpers are
    /// short and as few as the strips less one.
    fn join_all(&mut self) -> Result<(), PlaceError> {
        let nets: Vec<(NetId, Vec<Strip>)> = self.strips.clone();
        for (_, strips) in nets {
            let Some((&first, rest)) = strips.split_first() else {
                continue;
            };
            let mut joined = vec![first];
            let mut rest = rest.to_vec();
            while !rest.is_empty() {
                let mut best: Option<(f64, usize, Strip)> = None;
                for (index, &strip) in rest.iter().enumerate() {
                    for &other in &joined {
                        let gap = self.gap(strip, other);
                        if best.as_ref().is_none_or(|(least, _, _)| gap < *least) {
                            best = Some((gap, index, other));
                        }
                    }
                }
                let (_, index, other) = best.expect("rest is not empty");
                let strip = rest.remove(index);
                if strip != other {
                    self.join(strip, other)?;
                }
                joined.push(strip);
            }
        }
        Ok(())
    }

    /// Step 5. A battery's leads go in the rails; every other
    /// part is placed whole, after the parts it shares a net with: both its
    /// leads at once, where they are nearest each other and need the fewest
    /// and shortest jumpers, a person's way of building it. A lead goes into
    /// a strip its net already has, keeping a hole free there, or into a
    /// fresh one jumpered to the nearest of its net's.
    fn leads(&mut self) -> Result<(), PlaceError> {
        let mut parts: Vec<Vec<(PinRef, NetId)>> = Vec::new();
        for (id, element) in self.netlist.elements() {
            if element.pins.len() > 2
                || self.is_pinned(id)
                || matches!(element.kind, ElementKind::Ground)
            {
                continue;
            }
            let pins: Vec<(PinRef, NetId)> = (0..element.pins.len())
                .filter_map(|index| {
                    let pin = PinRef::new(id, index);
                    Some((pin, self.net(pin)?))
                })
                .collect();
            if off_board(&element.kind) {
                for (pin, net) in pins {
                    self.battery_lead(pin, net)?;
                }
            } else if !pins.is_empty() {
                parts.push(pins);
            }
        }
        // The part with most of its nets already on the board goes next, so
        // a circuit grows outwards from its chips and rails; on a tie, the
        // first listed.
        while !parts.is_empty() {
            let known = |pins: &Vec<(PinRef, NetId)>| {
                pins.iter()
                    .filter(|(_, net)| {
                        self.strips
                            .iter()
                            .any(|(other, s)| other == net && !s.is_empty())
                    })
                    .count()
            };
            let mut next = 0;
            for (index, pins) in parts.iter().enumerate() {
                if known(pins) > known(&parts[next]) {
                    next = index;
                }
            }
            let pins = parts.remove(next);
            self.part(&pins)?;
        }
        Ok(())
    }

    /// A battery's lead: into its net's rail, or a row jumpered to it.
    fn battery_lead(&mut self, pin: PinRef, net: NetId) -> Result<(), PlaceError> {
        let strips = self.strips_of(net).clone();
        let rail = strips
            .iter()
            .copied()
            .find(|&strip| matches!(strip, Strip::Rail(_)) && !self.free_holes(strip).is_empty());
        let strip = match rail {
            Some(strip) => strip,
            None => {
                let strip = self.fresh()?;
                self.give(strip, Some(net));
                if let Some(&other) = strips.last() {
                    self.join(strip, other)?;
                }
                strip
            }
        };
        let hole = *self.free_holes(strip).first().ok_or(PlaceError::TooBig)?;
        self.put(pin, hole);
        Ok(())
    }

    /// Where one lead of a part may go: a strip of its net with a hole to
    /// spare, at no cost, or a fresh strip, at the cost of the jumper to the
    /// nearest strip of its net, when it has one.
    fn options(&self, net: NetId) -> Vec<(Strip, f64, bool)> {
        let mine: Vec<Strip> = self
            .strips
            .iter()
            .find(|(other, _)| *other == net)
            .map(|(_, strips)| strips.clone())
            .unwrap_or_default();
        let mut options: Vec<(Strip, f64, bool)> = mine
            .iter()
            .filter(|strip| {
                matches!(strip, Strip::Row { .. }) && self.free_holes(**strip).len() >= 2
            })
            .map(|&strip| (strip, 0.0, false))
            .collect();
        let reachable: Vec<Strip> = mine
            .into_iter()
            .filter(|&strip| !self.free_holes(strip).is_empty())
            .collect();
        let had_none = !self
            .strips
            .iter()
            .any(|(other, s)| *other == net && !s.is_empty());
        for row in 1..=self.board.rows() {
            for side in [Side::Upper, Side::Lower] {
                let strip = Strip::Row { row, side };
                if self.owner.iter().any(|(other, _)| *other == strip) {
                    continue;
                }
                let cost = if had_none {
                    0.0
                } else {
                    match reachable
                        .iter()
                        .map(|&other| self.gap(strip, other))
                        .min_by(f64::total_cmp)
                    {
                        Some(gap) => JUMPER + gap,
                        None => continue,
                    }
                };
                options.push((strip, cost, true));
            }
        }
        options
    }

    /// How far apart two strips are where they are nearest, in pitches, and
    /// more when the way between them crosses a chip.
    fn gap(&self, a: Strip, b: Strip) -> f64 {
        let rail_y = |rail: Rail| -> f64 {
            match rail {
                Rail::TopPlus => 0.0,
                Rail::TopMinus => 1.0,
                Rail::BottomMinus => 16.0,
                Rail::BottomPlus => 17.0,
            }
        };
        let (along, across, crossing) = match (a, b) {
            (Strip::Row { row: r1, side: s1 }, Strip::Row { row: r2, side: s2 }) => {
                let along = f64::from(r1.abs_diff(r2));
                if s1 == s2 {
                    (along, 0.0, None)
                } else {
                    (along, 3.0, Some((r1.min(r2), r1.max(r2))))
                }
            }
            (Strip::Row { row, side }, Strip::Rail(rail))
            | (Strip::Rail(rail), Strip::Row { row, side }) => {
                let (top, bottom) = match side {
                    Side::Upper => (3.0, 7.0),
                    Side::Lower => (10.0, 14.0),
                };
                let y = rail_y(rail);
                let across = if y < top { top - y } else { y - bottom };
                let crosses = (y > 8.0) != (side == Side::Lower);
                (0.0, across, crosses.then_some((row, row)))
            }
            (Strip::Rail(r1), Strip::Rail(r2)) => (0.0, (rail_y(r1) - rail_y(r2)).abs(), None),
        };
        let over_chip = crossing.is_some_and(|(from, to)| {
            self.chip_rows
                .iter()
                .any(|&(first, last)| first <= to && from <= last)
        });
        along.hypot(across) + if over_chip { OVER_CHIP } else { 0.0 }
    }

    /// One part, both its leads at once (or its one).
    fn part(&mut self, pins: &[(PinRef, NetId)]) -> Result<(), PlaceError> {
        let choice: Vec<Choice> = match pins {
            [(_, net)] => {
                let best = self
                    .options(*net)
                    .into_iter()
                    .min_by(|a, b| a.1.total_cmp(&b.1))
                    .ok_or(PlaceError::TooBig)?;
                vec![(best.0, best.2)]
            }
            [(_, a), (_, b)] => {
                let (from, to) = (self.options(*a), self.options(*b));
                let mut best: Option<(f64, Choice, Choice)> = None;
                for &(sa, ca, fa) in &from {
                    for &(sb, cb, fb) in &to {
                        if sa == sb {
                            continue;
                        }
                        // A part's leads reach so far: past that, a jumper
                        // covers the distance better than they do.
                        let span = self.gap(sa, sb);
                        let mut cost = ca + cb + span + STRETCHED * (span - REACH).max(0.0);
                        // A part drawn where another already is is hard to
                        // tell apart from it, on the screen and the board.
                        let (p, q) = (middle(sa), middle(sb));
                        let at = ((p.0 + q.0) / 2.0, (p.1 + q.1) / 2.0);
                        cost += CROWDED
                            * self
                                .leads_drawn
                                .iter()
                                .filter(|&&(u, v)| {
                                    let mid = ((u.0 + v.0) / 2.0, (u.1 + v.1) / 2.0);
                                    (mid.0 - at.0).abs() < 2.0 && (mid.1 - at.1).abs() < 2.5
                                })
                                .count() as f64;
                        // Two leads a row apart on one side leave no room
                        // to see the part between them.
                        if let (Strip::Row { row: r1, side: s1 }, Strip::Row { row: r2, side: s2 }) =
                            (sa, sb)
                            && s1 == s2
                            && r1.abs_diff(r2) < 2
                        {
                            cost += CRAMPED;
                        }
                        if best.as_ref().is_none_or(|(least, _, _)| cost < *least) {
                            best = Some((cost, (sa, fa), (sb, fb)));
                        }
                    }
                }
                let (_, a, b) = best.ok_or(PlaceError::TooBig)?;
                vec![a, b]
            }
            _ => return Ok(()),
        };
        for (&(_, net), &(strip, fresh)) in pins.iter().zip(&choice) {
            if fresh {
                self.give(strip, Some(net));
            }
        }
        // The leads, in the holes of their strips nearest each other.
        let holes = match choice.as_slice() {
            [(a, _), (b, _)] => {
                let (from, to) = self
                    .nearest(*a, *b, Lean::Channel)
                    .ok_or(PlaceError::TooBig)?;
                vec![from, to]
            }
            [(a, _)] => vec![*self.free_holes(*a).first().ok_or(PlaceError::TooBig)?],
            _ => Vec::new(),
        };
        if let [from, to] = holes.as_slice() {
            let (from, to) = (self.position(*from), self.position(*to));
            self.leads_drawn.push((from, to));
        }
        for (&(pin, _), &hole) in pins.iter().zip(&holes) {
            self.put(pin, hole);
        }
        // Then the jumper of each fresh strip, to the nearest of its net's.
        for (&(_, net), &(strip, fresh)) in pins.iter().zip(&choice) {
            if !fresh {
                continue;
            }
            let other = self
                .strips_of(net)
                .clone()
                .into_iter()
                .filter(|&other| other != strip && !self.free_holes(other).is_empty())
                .min_by(|&x, &y| self.gap(strip, x).total_cmp(&self.gap(strip, y)));
            if let Some(other) = other {
                self.join(strip, other)?;
            }
        }
        Ok(())
    }

    fn position(&self, hole: Hole) -> (f64, f64) {
        self.board.position(hole).unwrap_or_default()
    }

    /// The free holes of two strips nearest each other, leaning for a part's
    /// leads towards the channel, and for a jumper towards the edges, so the
    /// two leave each other room; and away from a lead already drawn along
    /// the same line.
    fn nearest(&self, a: Strip, b: Strip, lean: Lean) -> Option<(Hole, Hole)> {
        let edge = |hole: Hole| match hole {
            Hole::Row { column, .. } if column < 5 => f64::from(4 - column),
            Hole::Row { column, .. } => f64::from(column - 5),
            Hole::Rail { .. } => 0.0,
        };
        let mut best: Option<(f64, Hole, Hole)> = None;
        for from in self.free_holes(a) {
            for to in self.free_holes(b) {
                let (p, q) = (self.position(from), self.position(to));
                let mut cost = (p.0 - q.0).hypot(p.1 - q.1);
                cost += match lean {
                    Lean::Channel => 0.01 * (edge(from) + edge(to)),
                    Lean::Edge => -0.01 * (edge(from) + edge(to)),
                };
                // Across the channel where a chip is.
                if (p.1 < 8.0) != (q.1 < 8.0)
                    && self.chip_rows.iter().any(|&(first, last)| {
                        let (from, to) = (f64::from(first - 1), f64::from(last - 1));
                        p.0.min(q.0) <= to + 0.5 && from - 0.5 <= p.0.max(q.0)
                    })
                {
                    cost += OVER_CHIP;
                }
                // Along a line another lead already runs on.
                if p.1 == q.1
                    && self.leads_drawn.iter().any(|&(u, v)| {
                        u.1 == v.1
                            && u.1 == p.1
                            && p.0.min(q.0) < u.0.max(v.0)
                            && u.0.min(v.0) < p.0.max(q.0)
                    })
                {
                    cost += 1.0;
                }
                if best.as_ref().is_none_or(|(least, _, _)| cost < *least) {
                    best = Some((cost, from, to));
                }
            }
        }
        best.map(|(_, from, to)| (from, to))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_sizes_have_the_holes_their_boxes_say() {
        // A half is sold as 400 tie points, a full as 830.
        assert_eq!(Board::new(Size::Half).holes().count(), 400);
        assert_eq!(Board::new(Size::Full).holes().count(), 830);
        assert_eq!(Board::new(Size::Half).rail_holes(), 25);
        assert_eq!(Board::new(Size::Full).rail_holes(), 50);
    }

    #[test]
    fn a_row_is_two_strips_of_five_either_side_of_the_channel() {
        let board = Board::new(Size::Half);
        let upper = board.holes_of(Strip::Row {
            row: 12,
            side: Side::Upper,
        });
        assert_eq!(
            upper.iter().map(|hole| hole.name()).collect::<Vec<_>>(),
            ["12a", "12b", "12c", "12d", "12e"]
        );
        assert_ne!(
            Hole::Row { row: 12, column: 4 }.strip(),
            Hole::Row { row: 12, column: 5 }.strip(),
            "e and f are not joined"
        );
        assert_eq!(
            Hole::Row { row: 12, column: 0 }.strip(),
            Hole::Row { row: 12, column: 4 }.strip()
        );
    }

    #[test]
    fn a_chip_straddles_the_channel() {
        // A DIP's two rows of legs are 0.3″ apart: e to f is three pitches.
        let board = Board::new(Size::Full);
        let (_, e) = board.position(Hole::Row { row: 5, column: 4 }).unwrap();
        let (_, f) = board.position(Hole::Row { row: 5, column: 5 }).unwrap();
        assert_eq!(f - e, 3.0);
        // And rows are a pitch apart, as a chip's legs are along it.
        let (x1, _) = board.position(Hole::Row { row: 5, column: 0 }).unwrap();
        let (x2, _) = board.position(Hole::Row { row: 6, column: 0 }).unwrap();
        assert_eq!(x2 - x1, 1.0);
    }

    #[test]
    fn rail_holes_come_in_fives_and_stay_on_the_board() {
        for size in [Size::Half, Size::Full] {
            let board = Board::new(size);
            let xs: Vec<f64> = (1..=board.rail_holes())
                .map(|index| {
                    board
                        .position(Hole::Rail {
                            rail: Rail::TopPlus,
                            index,
                        })
                        .unwrap()
                        .0
                })
                .collect();
            // A pitch apart within a group, two between groups.
            for (index, pair) in xs.windows(2).enumerate() {
                let step = if (index + 1) % 5 == 0 { 2.0 } else { 1.0 };
                assert_eq!(pair[1] - pair[0], step, "{size:?} after hole {}", index + 1);
            }
            assert!(xs[0] >= 0.0 && *xs.last().unwrap() <= f64::from(board.rows() - 1));
        }
    }

    #[test]
    fn every_hole_has_a_place_of_its_own() {
        let board = Board::new(Size::Full);
        let mut places: Vec<(i64, i64)> = board
            .holes()
            .map(|hole| {
                let (x, y) = board.position(hole).unwrap();
                (x as i64, y as i64)
            })
            .collect();
        let count = places.len();
        places.sort_unstable();
        places.dedup();
        assert_eq!(places.len(), count);
        assert_eq!(board.position(Hole::Row { row: 64, column: 0 }), None);
    }

    /// A battery, a resistor and an LED in a loop: three nets.
    fn milestone() -> (Netlist, [PinRef; 6]) {
        let mut netlist = Netlist::new();
        let battery = netlist.add_element(ElementKind::DcSource {
            volts: 9.0,
            series_ohms: 0.0,
        });
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: 470.0 });
        let led = netlist.add_element(ElementKind::Led {
            colour: crate::netlist::LedColour::Red,
        });
        let pins = [
            PinRef::new(battery, 0),
            PinRef::new(resistor, 0),
            PinRef::new(resistor, 1),
            PinRef::new(led, 0),
            PinRef::new(led, 1),
            PinRef::new(battery, 1),
        ];
        for pair in pins.chunks(2) {
            let net = netlist.add_net();
            for &pin in pair {
                netlist.connect(pin, net).unwrap();
            }
        }
        (netlist, pins)
    }

    fn at(row: u16, column: char) -> Hole {
        Hole::Row {
            row,
            column: COLUMNS.iter().position(|&c| c == column).unwrap() as u8,
        }
    }

    #[test]
    fn a_circuit_built_right_is_clean() {
        // + rail → resistor 10–14 → LED 14–16 → − rail through a jumper.
        let (netlist, [plus, ra, rb, anode, cathode, minus]) = milestone();
        let wiring = Wiring {
            pins: vec![
                (
                    plus,
                    Hole::Rail {
                        rail: Rail::TopPlus,
                        index: 1,
                    },
                ),
                (ra, at(10, 'a')),
                (rb, at(14, 'a')),
                (anode, at(14, 'b')),
                (cathode, at(16, 'b')),
                (
                    minus,
                    Hole::Rail {
                        rail: Rail::TopMinus,
                        index: 1,
                    },
                ),
            ],
            jumpers: vec![
                (
                    Hole::Rail {
                        rail: Rail::TopPlus,
                        index: 2,
                    },
                    at(10, 'e'),
                ),
                (
                    at(16, 'e'),
                    Hole::Rail {
                        rail: Rail::TopMinus,
                        index: 2,
                    },
                ),
            ],
        };
        let findings = wiring.check(Board::new(Size::Half), &netlist);
        assert!(findings.is_clean(), "{findings:?}");
    }

    #[test]
    fn two_nets_in_one_strip_is_a_short() {
        // Both resistor legs in row 10: the resistor is shorted out.
        let (netlist, [_, ra, rb, ..]) = milestone();
        let wiring = Wiring {
            pins: vec![(ra, at(10, 'a')), (rb, at(10, 'c'))],
            jumpers: vec![],
        };
        let findings = wiring.check(Board::new(Size::Half), &netlist);
        assert_eq!(findings.shorts.len(), 1, "{findings:?}");
        assert_eq!(findings.shorts[0].0, vec![at(10, 'a').strip()]);
        assert_eq!(findings.shorts[0].1.len(), 2);
        // Across the channel they are not joined, and nothing is wrong.
        let wiring = Wiring {
            pins: vec![(ra, at(10, 'a')), (rb, at(10, 'f'))],
            jumpers: vec![],
        };
        assert!(
            wiring
                .check(Board::new(Size::Half), &netlist)
                .shorts
                .is_empty()
        );
    }

    #[test]
    fn one_net_on_strips_nothing_joins_needs_a_jumper() {
        let (netlist, [_, _, rb, anode, ..]) = milestone();
        let apart = Wiring {
            pins: vec![(rb, at(14, 'a')), (anode, at(20, 'a'))],
            jumpers: vec![],
        };
        let findings = apart.check(Board::new(Size::Half), &netlist);
        assert_eq!(findings.splits.len(), 1, "{findings:?}");
        assert_eq!(findings.splits[0].1.len(), 2);
        // A jumper from 14 to 20, and it is one node.
        let joined = Wiring {
            jumpers: vec![(at(14, 'e'), at(20, 'e'))],
            ..apart
        };
        assert!(joined.check(Board::new(Size::Half), &netlist).is_clean());
    }

    #[test]
    fn a_hole_holds_one_lead_and_the_board_has_edges() {
        let (netlist, [_, ra, rb, anode, ..]) = milestone();
        let wiring = Wiring {
            pins: vec![(rb, at(14, 'a')), (anode, at(14, 'a')), (ra, at(31, 'a'))],
            jumpers: vec![],
        };
        let findings = wiring.check(Board::new(Size::Half), &netlist);
        assert_eq!(findings.crowded, vec![at(14, 'a')]);
        assert_eq!(findings.off_board, vec![at(31, 'a')]);
        // Row 31 is on a full board.
        assert!(
            wiring
                .check(Board::new(Size::Full), &netlist)
                .off_board
                .is_empty()
        );
    }

    #[test]
    fn a_rail_is_one_strip_end_to_end() {
        let first = Hole::Rail {
            rail: Rail::BottomMinus,
            index: 1,
        };
        let last = Hole::Rail {
            rail: Rail::BottomMinus,
            index: 50,
        };
        assert_eq!(first.strip(), last.strip());
        assert_ne!(
            first.strip(),
            Hole::Rail {
                rail: Rail::BottomPlus,
                index: 1
            }
            .strip()
        );
        assert_eq!(first.name(), "bottom − rail");
    }

    /// Every shipped example, as the app opens it.
    fn examples() -> Vec<(String, crate::circuit::Circuit)> {
        let mut out = Vec::new();
        for entry in std::fs::read_dir("../../app/src/examples").unwrap() {
            let path = entry.unwrap().path();
            let mut circuit = crate::circuit::Circuit::new();
            circuit
                .try_open(&std::fs::read_to_string(&path).unwrap())
                .unwrap();
            out.push((
                path.file_name().unwrap().to_string_lossy().into_owned(),
                circuit,
            ));
        }
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    #[test]
    fn every_example_places_with_nothing_shorted_and_nothing_left_unjoined() {
        let all = examples();
        assert!(all.len() > 90, "{} examples", all.len());
        for (name, circuit) in all {
            let netlist = circuit.netlist();
            let placement = place(netlist).unwrap_or_else(|error| panic!("{name}: {error:?}"));
            let findings = placement.wiring.check(placement.board, netlist);
            assert!(findings.is_clean(), "{name}: {findings:?}");
            // Every wired pin of every part is on the board.
            for (id, element) in netlist.elements() {
                // An Uno takes no hole until A-11 jumpers its headers to the
                // board; A-11 takes this exemption away.
                if matches!(element.kind, ElementKind::Ground) || beside_board(&element.kind) {
                    continue;
                }
                for (index, pin) in element.pins.iter().enumerate() {
                    if pin.net.is_some() {
                        let placed = placement
                            .wiring
                            .pins
                            .iter()
                            .any(|(other, _)| *other == PinRef::new(id, index));
                        assert!(placed, "{name}: {:?} pin {index} not placed", element.kind);
                    }
                }
            }
        }
    }

    #[test]
    fn a_chip_straddles_the_channel_pin_1_at_the_top_left() {
        for (name, circuit) in examples() {
            let netlist = circuit.netlist();
            let placement = place(netlist).unwrap();
            for (id, element) in netlist.elements() {
                let Some((count, numbers)) = dip(&element.kind) else {
                    continue;
                };
                let half = u16::from(count / 2);
                let hole_of = |index: usize| {
                    placement
                        .wiring
                        .pins
                        .iter()
                        .find(|(pin, _)| *pin == PinRef::new(id, index))
                        .map(|&(_, hole)| hole)
                };
                let placed: Vec<(u16, Hole)> = (0..element.pins.len())
                    .filter_map(|index| Some((u16::from(numbers[index]), hole_of(index)?)))
                    .collect();
                let first = placed
                    .iter()
                    .filter_map(|&(n, hole)| match hole {
                        Hole::Row { row, .. } if n <= half => Some(row + 1 - n),
                        Hole::Row { row, .. } => Some(row + n - 2 * half),
                        Hole::Rail { .. } => None,
                    })
                    .collect::<Vec<_>>();
                // One chip, one place: every pin agrees where pin 1 is.
                assert!(
                    first.windows(2).all(|pair| pair[0] == pair[1]),
                    "{name}: {first:?}"
                );
                for (n, hole) in placed {
                    let Hole::Row { column, .. } = hole else {
                        panic!("{name}: a chip's pin on a rail");
                    };
                    assert_eq!(column, if n <= half { 4 } else { 5 }, "{name}: pin {n}");
                }
            }
        }
    }

    #[test]
    fn a_part_sits_where_its_leads_reach() {
        // Placed a lead at a time, the chaser's resistors ran 22 pitches,
        // from one end of the board to the other, across its chips.
        for (name, circuit) in examples() {
            let netlist = circuit.netlist();
            let placement = place(netlist).unwrap();
            let at = |hole| placement.board.position(hole).unwrap();
            for (id, element) in netlist.elements() {
                if element.pins.len() != 2 || off_board(&element.kind) {
                    continue;
                }
                let holes: Vec<(f64, f64)> = placement
                    .wiring
                    .pins
                    .iter()
                    .filter(|(pin, _)| pin.element == id)
                    .map(|&(_, hole)| at(hole))
                    .collect();
                if let [p, q] = holes[..] {
                    let span = (p.0 - q.0).hypot(p.1 - q.1);
                    assert!(span <= 10.0, "{name}: {:?} spans {span:.1}", element.kind);
                }
            }
        }
    }

    #[test]
    fn a_jumper_takes_the_holes_nearest_its_ends() {
        // A resistor across the board from its battery: its row joins the
        // rail straight above it, not at the rail's first hole.
        let placement = place(&milestone().0).unwrap();
        for &(from, to) in &placement.wiring.jumpers {
            let (p, q) = (
                placement.board.position(from).unwrap(),
                placement.board.position(to).unwrap(),
            );
            if matches!(from, Hole::Rail { .. }) != matches!(to, Hole::Rail { .. }) {
                assert!((p.0 - q.0).abs() <= 1.0, "{from:?} to {to:?}");
            }
        }
    }

    #[test]
    fn the_milestone_runs_from_the_rails() {
        let (netlist, [plus, _, _, _, _, minus]) = milestone();
        let placement = place(&netlist).unwrap();
        assert_eq!(placement.board.size, Size::Half);
        let hole_of = |pin: PinRef| {
            placement
                .wiring
                .pins
                .iter()
                .find(|(other, _)| *other == pin)
                .unwrap()
                .1
        };
        assert_eq!(hole_of(plus).strip(), Strip::Rail(Rail::TopPlus));
        assert_eq!(hole_of(minus).strip(), Strip::Rail(Rail::TopMinus));
        assert!(placement.wiring.check(placement.board, &netlist).is_clean());
    }

    #[test]
    fn too_much_for_a_half_board_goes_on_a_full_one() {
        // A battery and a chain of 70 resistors: 69 nets of their own, more
        // strips than a half board's 60.
        let mut netlist = Netlist::new();
        let battery = netlist.add_element(ElementKind::DcSource {
            volts: 9.0,
            series_ohms: 0.0,
        });
        let mut from = PinRef::new(battery, 0);
        for _ in 0..70 {
            let resistor = netlist.add_element(ElementKind::Resistor { ohms: 1000.0 });
            let net = netlist.add_net();
            netlist.connect(from, net).unwrap();
            netlist.connect(PinRef::new(resistor, 0), net).unwrap();
            from = PinRef::new(resistor, 1);
        }
        let net = netlist.add_net();
        netlist.connect(from, net).unwrap();
        netlist.connect(PinRef::new(battery, 1), net).unwrap();
        let placement = place(&netlist).unwrap();
        assert_eq!(placement.board.size, Size::Full);
        assert!(placement.wiring.check(placement.board, &netlist).is_clean());
    }

    #[test]
    fn package_numbers_are_each_used_once() {
        for component in crate::registry::REGISTRY {
            let kind = (component.blank)();
            let Some((count, numbers)) = dip(&kind) else {
                continue;
            };
            let numbers = &numbers[..component.pins.len()];
            assert_eq!(numbers.len(), component.pins.len(), "{}", component.kind);
            let mut sorted = numbers.to_vec();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted.len(), numbers.len(), "{}", component.kind);
            assert!(
                numbers.iter().all(|&n| (1..=count).contains(&n)),
                "{}",
                component.kind
            );
        }
    }

    #[test]
    fn the_pinouts_are_the_sheets() {
        // Read off the makers' sheets (2026-10-02), pin by pin: the model's
        // pin name, and its number on the package.
        let cases: [(&str, &[(&str, u8)]); 7] = [
            (
                "Timer",
                &[
                    ("VCC", 8),
                    ("GND", 1),
                    ("TRIG", 2),
                    ("THR", 6),
                    ("DIS", 7),
                    ("OUT", 3),
                ],
            ),
            (
                "OpAmp",
                &[("in+", 3), ("in-", 2), ("out", 1), ("V+", 8), ("V-", 4)],
            ),
            (
                "Comparator",
                &[("in+", 3), ("in-", 2), ("out", 1), ("V+", 8), ("V-", 4)],
            ),
            ("Optocoupler", &[("A", 1), ("K", 2), ("C", 5), ("E", 4)]),
            (
                "MotorDriver",
                &[
                    ("1,2EN", 1),
                    ("1Y", 3),
                    ("2Y", 6),
                    ("VCC2", 8),
                    ("4A", 15),
                    ("VCC1", 16),
                    ("GND", 4),
                ],
            ),
            (
                "Decoder",
                &[
                    ("A", 7),
                    ("B", 1),
                    ("C", 2),
                    ("D", 6),
                    ("LT", 3),
                    ("BI", 4),
                    ("LE", 5),
                    ("a", 13),
                    ("e", 9),
                    ("f", 15),
                    ("g", 14),
                    ("VSS", 8),
                ],
            ),
            (
                "Counter",
                &[
                    ("CLK", 1),
                    ("INH", 2),
                    ("DEI", 3),
                    ("CO", 5),
                    ("f", 6),
                    ("g", 7),
                    ("d", 9),
                    ("a", 10),
                    ("e", 11),
                    ("b", 12),
                    ("c", 13),
                    ("RST", 15),
                ],
            ),
        ];
        for (kind, pins) in cases {
            let component = crate::registry::definition(kind).unwrap();
            let (_, numbers) = dip(&(component.blank)()).unwrap();
            for &(name, number) in pins {
                let index = component.pins.iter().position(|&pin| pin == name).unwrap();
                assert_eq!(numbers[index], number, "{kind} {name}");
            }
        }
    }
}
