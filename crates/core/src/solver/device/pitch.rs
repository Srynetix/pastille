// SPDX-License-Identifier: MIT OR Apache-2.0

//! A passive buzzer, and the note it is being fed (C-85).
//!
//! A passive buzzer has no oscillator of its own: it is a small speaker, and
//! sounds at whatever frequency the circuit drives it with — which is the whole
//! difference from the active buzzer (C-66), and the reason a kit carries both.
//! The part is the magnetic kind kits sell under that name, a 12 mm transducer
//! with a coil of about 16 Ω. Electrically it is that coil's resistance, which
//! is a lot of current for a chip's output and why a kit drives it through a
//! transistor.
//!
//! **The note is read from the run**, not from any source's setting, so
//! anything that oscillates plays. [`Pitch`] watches the voltage across it at
//! every step the run takes and counts the times it rises through the middle
//! of its own swing: the frequency is how many it did over the last tenth of a
//! second. The middle rather than zero, because a buzzer under a transistor is
//! switched between nothing and the supply and never crosses zero at all; and
//! with hysteresis of a quarter of the swing, so a step that lands on the edge
//! is counted once. Beside the run, as the LCD's RMS is (C-76): what it hears
//! changes nothing electrical.
//!
//! What it honours: the coil's resistance; silence on DC, and on a swing too
//! small or too slow to be a note; any waveform, either way round. What it
//! ignores: the coil's inductance and the resonance that makes a real one
//! loudest near 2–3 kHz; loudness altogether.

use crate::netlist::Element;
use crate::solver::OperatingPoint;

/// The coil's resistance, in ohms.
pub const OHMS: f64 = 16.0;

/// The part, for the Inspector.
pub const PART: &str = "12 mm magnetic buzzer, 16 Ω";

/// The least swing, peak to peak, that is a note rather than noise, in volts:
/// about 6 mA through the coil. Small, because the coil is: behind the 100 Ω
/// a kit puts in front of it, a buzzer loud on the bench sees well under a
/// volt.
pub const SOUNDS_FROM: f64 = 0.1;

/// How far back the frequency is counted, in seconds. Two rising crossings
/// inside it are the least a frequency can be read from, so it is also what
/// sets [`LOWEST_HZ`]'s floor.
pub const WINDOW: f64 = 0.1;

/// Lower than this is a click at a time, not a note: the bottom of hearing.
pub const LOWEST_HZ: f64 = 20.0;

/// How fast the swing it measures against forgets a peak, in seconds. Long
/// beside a note's period, so the middle holds still through a cycle; short
/// beside a person's patience, so a circuit turned down is heard to stop.
const ENVELOPE: f64 = 0.05;

/// How many crossings it keeps. Past this, the newest are enough: at 320 Hz
/// they already fill the window.
const KEPT: usize = 32;

/// What a passive buzzer has been hearing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pitch {
    /// The swing's top and bottom, each following the voltage at once one way
    /// and forgetting it over [`ENVELOPE`] the other.
    high: f64,
    low: f64,
    /// Which side of the middle it was last on.
    above: bool,
    /// The last sample, `None` before the first.
    last: Option<(f64, f64)>,
    /// When it rose through the middle, newest last.
    crossings: [f64; KEPT],
    count: usize,
}

impl Default for Pitch {
    fn default() -> Self {
        Pitch {
            high: 0.0,
            low: 0.0,
            above: false,
            last: None,
            crossings: [0.0; KEPT],
            count: 0,
        }
    }
}

impl Pitch {
    /// Takes in `volts` across it at `time`, in seconds of the run.
    pub fn advance(&mut self, volts: f64, time: f64) {
        let Some((was, then)) = self.last else {
            *self = Pitch {
                high: volts,
                low: volts,
                last: Some((volts, time)),
                ..Pitch::default()
            };
            return;
        };
        let forget = 1.0 - (-(time - then).max(0.0) / ENVELOPE).exp();
        self.high = volts.max(self.high - (self.high - volts) * forget);
        self.low = volts.min(self.low + (volts - self.low) * forget);

        let swing = self.high - self.low;
        let middle = (self.high + self.low) / 2.0;
        if swing >= SOUNDS_FROM {
            let rising = middle + swing / 4.0;
            if !self.above && volts > rising {
                self.above = true;
                // Where between the two samples it passed the level, so a
                // run's uneven steps do not jitter the note.
                let fraction = if volts > was {
                    ((rising - was) / (volts - was)).clamp(0.0, 1.0)
                } else {
                    1.0
                };
                self.cross(then + fraction * (time - then));
            } else if self.above && volts < middle - swing / 4.0 {
                self.above = false;
            }
        }
        self.last = Some((volts, time));
    }

    fn cross(&mut self, at: f64) {
        if self.count == KEPT {
            self.crossings.copy_within(1.., 0);
            self.count -= 1;
        }
        self.crossings[self.count] = at;
        self.count += 1;
    }

    /// The note, in hertz, or nothing while it is silent.
    pub fn hertz(&self) -> Option<f64> {
        let (_, now) = self.last?;
        if self.high - self.low < SOUNDS_FROM {
            return None;
        }
        let recent: Vec<f64> = self.crossings[..self.count]
            .iter()
            .copied()
            .filter(|&at| now - at <= WINDOW)
            .collect();
        let (&first, &last) = (recent.first()?, recent.last()?);
        if recent.len() < 2 || last <= first {
            return None;
        }
        let hertz = (recent.len() - 1) as f64 / (last - first);
        (hertz >= LOWEST_HZ).then_some(hertz)
    }
}

/// The voltage across a passive buzzer, + over −, or nothing for any other
/// part or one not wired at both ends.
pub fn across(element: &Element, point: &OperatingPoint) -> Option<f64> {
    if !matches!(element.kind, crate::netlist::ElementKind::PassiveBuzzer) {
        return None;
    }
    let plus = point.voltage(element.pins.first()?.net?)?;
    let minus = point.voltage(element.pins.get(1)?.net?)?;
    Some(plus - minus)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A square between `low` and `high` at `hertz`, sampled every `step` for
    /// `seconds`.
    fn square(pitch: &mut Pitch, hertz: f64, low: f64, high: f64, step: f64, seconds: f64) {
        let steps = (seconds / step).round() as usize;
        for index in 0..=steps {
            let time = index as f64 * step;
            let phase = (time * hertz).fract();
            pitch.advance(if phase < 0.5 { high } else { low }, time);
        }
    }

    #[test]
    fn a_square_is_heard_at_its_frequency() {
        for hertz in [55.0, 440.0, 2300.0] {
            let mut pitch = Pitch::default();
            square(&mut pitch, hertz, -5.0, 5.0, 1.0 / (hertz * 40.0), 0.3);
            let heard = pitch.hertz().expect("a note");
            assert!(
                (heard / hertz - 1.0).abs() < 0.01,
                "{hertz} Hz heard as {heard}"
            );
        }
    }

    #[test]
    fn a_switched_buzzer_is_heard_though_it_never_crosses_zero() {
        // Under a transistor: nothing, then the supply, never below zero.
        let mut pitch = Pitch::default();
        square(&mut pitch, 440.0, 0.0, 5.0, 1e-5, 0.2);
        let heard = pitch.hertz().unwrap();
        assert!((heard - 440.0).abs() < 2.0, "{heard}");
    }

    #[test]
    fn uneven_steps_do_not_move_the_note() {
        // The run's steps are its own choice, short at an edge and long
        // between: here alternately 7 µs and 61 µs.
        let mut pitch = Pitch::default();
        let mut time = 0.0;
        let mut short = true;
        while time < 0.3 {
            let phase = (time * 440.0f64).fract();
            pitch.advance(if phase < 0.5 { 5.0 } else { 0.0 }, time);
            time += if short { 7e-6 } else { 61e-6 };
            short = !short;
        }
        let heard = pitch.hertz().unwrap();
        assert!((heard - 440.0).abs() < 1.0, "{heard}");
    }

    #[test]
    fn a_sine_is_heard_as_well_as_a_square() {
        let mut pitch = Pitch::default();
        for index in 0..30_000 {
            let time = index as f64 * 1e-5;
            pitch.advance(
                3.0 * (2.0 * std::f64::consts::PI * 262.0 * time).sin(),
                time,
            );
        }
        let heard = pitch.hertz().unwrap();
        assert!((heard - 262.0).abs() < 1.0, "{heard}");
    }

    #[test]
    fn dc_is_silent() {
        let mut pitch = Pitch::default();
        for index in 0..100 {
            pitch.advance(5.0, index as f64 / 60.0);
        }
        assert_eq!(pitch.hertz(), None);
    }

    #[test]
    fn a_swing_too_small_or_too_slow_is_not_a_note() {
        let mut small = Pitch::default();
        square(&mut small, 440.0, 0.0, 0.05, 1e-5, 0.2);
        assert_eq!(small.hertz(), None, "50 mV, 3 mA");

        let mut slow = Pitch::default();
        square(&mut slow, 2.0, 0.0, 5.0, 1e-3, 3.0);
        assert_eq!(slow.hertz(), None, "two clicks a second");
    }

    #[test]
    fn it_falls_silent_when_the_note_stops() {
        let mut pitch = Pitch::default();
        square(&mut pitch, 440.0, 0.0, 5.0, 1e-5, 0.2);
        assert!(pitch.hertz().is_some());
        // Held high: within a window it has no crossings left to count.
        for index in 0..20 {
            pitch.advance(5.0, 0.2 + index as f64 / 60.0);
        }
        assert_eq!(pitch.hertz(), None);
    }

    #[test]
    fn it_follows_a_note_that_changes() {
        let mut pitch = Pitch::default();
        square(&mut pitch, 440.0, 0.0, 5.0, 1e-5, 0.2);
        let mut time = 0.2;
        while time < 0.5 {
            let phase = (time * 880.0f64).fract();
            pitch.advance(if phase < 0.5 { 5.0 } else { 0.0 }, time);
            time += 1e-5;
        }
        let heard = pitch.hertz().unwrap();
        assert!((heard - 880.0).abs() < 4.0, "{heard}");
    }
}
