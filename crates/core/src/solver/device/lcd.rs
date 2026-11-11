// SPDX-License-Identifier: MIT OR Apache-2.0

//! A seven-segment liquid-crystal digit: eight pieces of glass sharing one
//! backplane (C-76).
//!
//! What the LED digit (C-75) cannot teach. A segment is not a junction: it is
//! two electrodes with a film of liquid crystal between them, and it darkens
//! when the voltage across it — its own pin against the backplane, COM — is
//! large enough, whichever way round. So it draws almost nothing, which is why
//! a watch runs a year on a coin cell; and it answers the *RMS* voltage, not
//! the instantaneous one, so it has to be driven with AC. A segment swinging in
//! antiphase with COM sees twice the swing and shows; one swinging in phase
//! sees nothing and stays clear. DC shows too, and damages the glass: the
//! current it drives through the film plates the electrodes a little more
//! every second, and 50 mV of it is already too much.
//!
//! Modelled on a Lumex LCD-S101D22TR — a single static TN digit, grey on a
//! reflective background, 5 V AC — which gives its pinout and its drive and
//! nothing else. The optical figures are typical ones for TN glass, said so
//! beside each constant, which is why the registry calls the model
//! approximate.
//!
//! What it honours: the RMS response, its threshold and saturation, the time
//! the crystal takes to turn, and the DC limit. What it ignores: the
//! segment's capacitance, a few hundred picofarads. Driven by anything on the
//! palette it charges in well under a microsecond, which nothing on the canvas
//! can show, and carrying a charge per segment would ask the run for eight
//! where it holds one an element (C-48). The current that charging draws —
//! tenths of a microamp — is left out with it, so the digit reads the leak
//! alone. And temperature, which moves every figure here.
//!
//! Like the heat (C-74), what a segment shows changes nothing electrical, so
//! it is carried beside the solve rather than in it: [`Glass`], advanced over
//! every step the run takes.

use crate::netlist::{Element, ElementKind};

use super::digit::{COMMON, SEGMENTS};
use crate::solver::OperatingPoint;

/// The part it is.
pub const PART: &str = "LCD-S101D22TR";

/// What each segment leaks, pin to COM, in ohms.
///
/// TN glass is rated at no less than 20 MΩ of DC resistance for a whole
/// display (Excel Display's TN specification, a 3½-digit baseline), and that
/// is shared by every segment on it — some two dozen. A segment of its own is
/// then a few hundred megohms, and 500 MΩ is where that lands.
pub const LEAK_OHMS: f64 = 500e6;

/// The RMS voltage at which a segment starts to darken.
///
/// TN glass starts at 1.5–2.5 V; a part rated for 5 V AC sits near the top
/// of that range.
pub const THRESHOLD_VOLTS: f64 = 2.0;

/// The RMS voltage at which it is as dark as it goes.
///
/// TN glass saturates at 4–5 V, which is why its drive is 5 V.
pub const SATURATION_VOLTS: f64 = 4.0;

/// How long the crystal takes to turn, in seconds: the window its RMS is taken
/// over.
///
/// TN glass reaches 90 % of its contrast in at most 50 ms and falls to 10 %
/// in at most 100 ms, at 25 °C. One time constant for both, long enough to
/// average a 30 Hz drive — the slowest a static one runs at before it
/// flickers — into a steady reading.
pub const RESPONSE_SECONDS: f64 = 0.03;

/// The most DC the glass tolerates across a segment, in volts.
///
/// 50 mV, the TN figure for an allowable DC offset: past it, the film is
/// being plated a little more every second.
pub const DC_LIMIT_VOLTS: f64 = 0.05;

/// How long the DC across a segment is averaged over, in seconds — each of
/// the two averages it is taken through, one after the other.
///
/// Twice rather than once, because a single average of a square wave ripples
/// by the wave's own swing over its half cycle: over 2 s, at 10 Hz and ±5 V,
/// that was 60 mV either side of nothing, past the limit and back every half
/// cycle, and a warning that blinked with it (found on "A watch's digit"
/// turned down to 10 Hz). The second average takes the ripple down by the
/// same factor again, to a tenth of a millivolt at 10 Hz.
///
/// Four seconds is a trade. What is left to mistake for DC is the half cycle a
/// square begins on, which at a slow drive is a long one: from 3 Hz up a
/// symmetric drive is never warned about; at 1–2 Hz it is for the first
/// quarter of a minute, and under a hertz — a second of DC one way, then the
/// other, far below the 30 Hz a static LCD wants — steadily. Longer would
/// spare 1 Hz too, and take longer to notice a battery, which it does now in
/// 0.6 s at 5 V; and longer to let go, which is why an edit starts the
/// reading over ([`Glass::restart_dc`]).
pub const DC_WINDOW_SECONDS: f64 = 4.0;

/// Below what DC, in volts, a warning once raised is taken back: half the
/// limit, so a DC hovering at the limit is not a warning switching on and off.
pub const DC_CLEAR_VOLTS: f64 = DC_LIMIT_VOLTS / 2.0;

/// What each segment of one digit has been seeing: the running mean of the
/// square of the voltage across it, which is its RMS, and the running mean
/// of the voltage itself, twice over, which is its DC.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Glass {
    mean_square: [f64; SEGMENTS.len()],
    mean: [f64; SEGMENTS.len()],
    /// The first average's output, which the second one averages.
    drifting: [f64; SEGMENTS.len()],
    /// The largest DC on any segment lately, fading over the window, so that a
    /// drive slow enough to swing its DC through zero — under a hertz — holds
    /// the warning steady instead of dropping it at every crossing.
    peak: f64,
    /// Whether the glass is being told it is on DC.
    plated: bool,
}

impl Glass {
    /// Takes the glass `seconds` further, with `across` held over them — each
    /// segment's pin against COM, or nothing for a segment with no field
    /// across it.
    ///
    /// Each running mean is taken exactly over the step, as the heat is
    /// (C-74): with the voltage constant across it, a first-order average —
    /// and two of them in a row with the same time constant — has a closed
    /// form, so a microsecond step and a frame land on the same curve.
    pub fn advance(&mut self, across: &[Option<f64>; SEGMENTS.len()], seconds: f64) {
        let seconds = seconds.max(0.0);
        let turn = (-seconds / RESPONSE_SECONDS).exp();
        let drift = (-seconds / DC_WINDOW_SECONDS).exp();
        let mut largest: f64 = 0.0;
        for (segment, volts) in across.iter().enumerate() {
            let volts = volts.unwrap_or(0.0);
            let square = volts * volts;
            self.mean_square[segment] = square + (self.mean_square[segment] - square) * turn;
            // The second stage first, because it is fed by where the first
            // one was at the start of the step.
            let first = self.drifting[segment] - volts;
            self.mean[segment] = volts
                + (self.mean[segment] - volts) * drift
                + first * (seconds / DC_WINDOW_SECONDS) * drift;
            self.drifting[segment] = volts + first * drift;
            largest = largest.max(self.mean[segment].abs());
        }
        self.peak = largest.max(self.peak * drift);
        if largest > DC_LIMIT_VOLTS {
            self.plated = true;
        } else if self.peak < DC_CLEAR_VOLTS {
            self.plated = false;
        }
    }

    /// A segment's RMS voltage, over the crystal's response.
    pub fn rms(&self, segment: usize) -> f64 {
        self.mean_square[segment].max(0.0).sqrt()
    }

    /// How dark a segment is, from 0 (clear) to 1 (as dark as it goes).
    ///
    /// A smooth step between the threshold and saturation: TN glass has a
    /// curve rather than a switch, and a drive of 3 V on glass rated for 5 V
    /// shows half-dark, which is what it does.
    pub fn contrast(&self, segment: usize) -> f64 {
        let through = ((self.rms(segment) - THRESHOLD_VOLTS)
            / (SATURATION_VOLTS - THRESHOLD_VOLTS))
            .clamp(0.0, 1.0);
        through * through * (3.0 - 2.0 * through)
    }

    /// The DC across a segment, averaged over [`DC_WINDOW_SECONDS`].
    pub fn offset(&self, segment: usize) -> f64 {
        self.mean[segment]
    }

    /// Forgets the DC it has seen, keeping what it shows: for a circuit that
    /// has just been edited, whose drive is not the one the average was taken
    /// of. Without it, a digit taken off a battery and put on a signal would
    /// be told about the battery for half a minute more.
    pub fn restart_dc(&mut self) {
        *self = Glass {
            mean_square: self.mean_square,
            ..Glass::default()
        };
    }

    /// Whether the glass is carrying more DC than it tolerates: raised as
    /// soon as any segment's passes [`DC_LIMIT_VOLTS`], and taken back only
    /// once the largest lately has faded under [`DC_CLEAR_VOLTS`].
    pub fn driven_with_dc(&self) -> bool {
        self.plated
    }
}

/// Each segment's pin against COM at `point`, for an LCD digit: nothing for a
/// segment either of whose pins is free, which has no field across it.
pub fn across(element: &Element, point: &OperatingPoint) -> Option<[Option<f64>; SEGMENTS.len()]> {
    if !matches!(element.kind, ElementKind::LcdDigit) {
        return None;
    }
    let volts = |pin: usize| point.voltage(element.pins.get(pin)?.net?);
    let common = volts(COMMON);
    let mut across = [None; SEGMENTS.len()];
    for (segment, slot) in across.iter_mut().enumerate() {
        *slot = volts(segment).zip(common).map(|(own, shared)| own - shared);
    }
    Some(across)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A glass held at `volts` on segment a for `seconds`, in steps of `step`.
    fn held(volts: impl Fn(f64) -> f64, seconds: f64, step: f64) -> Glass {
        let mut glass = Glass::default();
        let mut at = 0.0;
        while at < seconds {
            let mut across = [None; SEGMENTS.len()];
            across[0] = Some(volts(at + step / 2.0));
            glass.advance(&across, step);
            at += step;
        }
        glass
    }

    fn square(volts: f64, hertz: f64) -> impl Fn(f64) -> f64 {
        move |at: f64| {
            if (at * hertz).fract() < 0.5 {
                volts
            } else {
                -volts
            }
        }
    }

    #[test]
    fn a_square_wave_reads_its_own_amplitude_as_rms() {
        let glass = held(square(5.0, 32.0), 0.5, 1e-4);
        assert!((glass.rms(0) - 5.0).abs() < 1e-6, "{}", glass.rms(0));
        assert_eq!(glass.contrast(0), 1.0);
    }

    #[test]
    fn under_the_threshold_it_stays_clear_and_past_saturation_it_is_dark() {
        assert_eq!(held(|_| 1.5, 0.5, 1e-3).contrast(0), 0.0);
        assert_eq!(held(|_| 4.5, 0.5, 1e-3).contrast(0), 1.0);
        let half = held(|_| 3.0, 0.5, 1e-3).contrast(0);
        assert!((half - 0.5).abs() < 1e-6, "{half}");
    }

    #[test]
    fn it_turns_within_the_time_the_sheet_allows() {
        // 90 % of its contrast within 50 ms of 5 V arriving.
        let on = held(square(5.0, 64.0), 0.05, 1e-4);
        assert!(on.contrast(0) >= 0.9, "{}", on.contrast(0));
        // Not before it has had a chance to.
        let early = held(square(5.0, 64.0), 0.005, 1e-4);
        assert!(early.contrast(0) < 0.1, "{}", early.contrast(0));
    }

    #[test]
    fn a_step_size_does_not_change_where_it_lands() {
        let fine = held(|_| 3.0, 0.02, 1e-5);
        let coarse = held(|_| 3.0, 0.02, 0.02);
        assert!((fine.rms(0) - coarse.rms(0)).abs() < 1e-3);
    }

    /// How many times the DC warning changes over `seconds` of `volts` on
    /// segment a, looked at every step, and what it ends on.
    fn flips(volts: impl Fn(f64) -> f64, seconds: f64, step: f64) -> (usize, bool) {
        let mut glass = Glass::default();
        let (mut at, mut was, mut changes) = (0.0, false, 0);
        while at < seconds {
            let mut across = [None; SEGMENTS.len()];
            across[0] = Some(volts(at + step / 2.0));
            glass.advance(&across, step);
            if glass.driven_with_dc() != was {
                changes += 1;
                was = !was;
            }
            at += step;
        }
        (changes, was)
    }

    #[test]
    fn a_symmetric_drive_is_never_warned_about_at_any_moment() {
        // Down to 3 Hz, a tenth of what a static LCD wants. Looked at every
        // step, not only at the end: a warning that comes and goes twenty
        // times a second is what a single average used to give at 10 Hz.
        for hertz in [3.0, 5.0, 10.0, 32.0, 100.0] {
            assert_eq!(
                flips(square(5.0, hertz), 8.0, 1e-3),
                (0, false),
                "{hertz} Hz"
            );
        }
    }

    #[test]
    fn a_drive_too_slow_to_average_is_warned_about_steadily() {
        // Half a hertz is a second of DC one way, then the other: warned, and
        // held, rather than dropped at every crossing.
        let (changes, warned) = flips(square(5.0, 0.5), 10.0, 1e-3);
        assert!(warned);
        assert_eq!(changes, 1);
    }

    #[test]
    fn a_battery_is_warned_about_inside_a_second() {
        assert!(!held(|_| 5.0, 0.3, 1e-4).driven_with_dc());
        assert!(held(|_| 5.0, 0.7, 1e-4).driven_with_dc());
        // And the warning goes once the battery does, a while after, and
        // only once.
        let (changes, warned) = flips(|at| if at < 1.0 { 5.0 } else { 0.0 }, 60.0, 1e-3);
        assert_eq!((changes, warned), (2, false));
    }

    #[test]
    fn restarting_the_dc_keeps_what_the_glass_shows() {
        let mut glass = held(|_| 5.0, 1.0, 1e-3);
        assert!(glass.driven_with_dc());
        glass.restart_dc();
        assert!(!glass.driven_with_dc());
        assert_eq!(glass.contrast(0), 1.0);
    }

    #[test]
    fn a_free_segment_sees_nothing() {
        let mut glass = Glass::default();
        glass.advance(&[None; SEGMENTS.len()], 1.0);
        assert_eq!(glass.rms(0), 0.0);
        assert!(!glass.driven_with_dc());
    }
}
