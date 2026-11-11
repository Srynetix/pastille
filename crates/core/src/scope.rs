// SPDX-License-Identifier: MIT OR Apache-2.0

//! What each probe has been reading, kept over time.
//!
//! C-22. A scope is a rolling buffer and a way to look at a window of it, and
//! the only interesting decision is how often it is written to. Once per frame
//! is what the app could have managed on its own, and it would make anything
//! faster than about thirty hertz invisible — which is most of what a capacitor
//! and a coil do. So the buffer is written once per *solved step*, inside the
//! run, where the simulation's own resolution is (C-48 bounds that at a
//! thirty-second of the span asked for, so a frame is worth up to five hundred
//! samples rather than one).
//!
//! One trace per probe, because a probe is already the user saying "this is the
//! thing I want to know about" (C-21). Nothing else needs choosing, and there is
//! no trace nobody asked for.

use slotmap::SparseSecondaryMap;

use crate::netlist::{ElementId, ElementKind, Netlist};
use crate::solver::OperatingPoint;

/// Samples kept per trace.
///
/// The run steps between about thirty microseconds and a millisecond (C-48
/// bounds both ends), so this is a second of history at the finest and half a
/// minute at the coarsest. Half a minute is the number that matters: it has to
/// cover a curve somebody is watching, and an RC a person can follow has a time
/// constant of seconds. A quarter of a megabyte per probe buys it.
pub const HISTORY: usize = 32_768;

/// A fixed-length ring, oldest sample first once it has wrapped.
#[derive(Debug, Clone, Default)]
struct Ring {
    samples: Vec<f64>,
    /// Where the next sample goes.
    next: usize,
}

impl Ring {
    fn push(&mut self, value: f64) {
        if self.samples.len() < HISTORY {
            self.samples.push(value);
            self.next = self.samples.len() % HISTORY;
        } else {
            self.samples[self.next] = value;
            self.next = (self.next + 1) % HISTORY;
        }
    }

    /// Every sample, oldest first.
    fn iter(&self) -> impl Iterator<Item = f64> + '_ {
        let (young, old) = if self.samples.len() < HISTORY {
            (&self.samples[..], &[][..])
        } else {
            (&self.samples[self.next..], &self.samples[..self.next])
        };
        young.iter().chain(old).copied()
    }

    fn len(&self) -> usize {
        self.samples.len()
    }

    fn last(&self) -> Option<f64> {
        if self.samples.is_empty() {
            return None;
        }
        // Modulo the samples there are, not the samples there could be: before
        // the ring has wrapped those are different numbers and only one of them
        // is the sample that was written last.
        let index = (self.next + self.samples.len() - 1) % self.samples.len();
        self.samples.get(index).copied()
    }
}

/// One column of a plot: the span of the trace over one pixel's worth of time.
///
/// A column rather than a sample, because a trace is nearly always longer than
/// the place it is drawn and picking one sample per column would alias — a
/// ringing circuit would come out as whatever the sampling happened to land on.
/// The lowest and highest the trace reached is what a scope shows, and it costs
/// nothing extra to draw.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Column {
    pub low: f64,
    pub high: f64,
}

/// What every probe has been reading.
#[derive(Debug, Clone, Default)]
pub struct Traces {
    /// When each sample was taken. Shared: every trace is written on the same
    /// step, so a second copy per probe would say the same thing.
    times: Ring,
    readings: SparseSecondaryMap<ElementId, Ring>,
    /// Rebuilt by every [`Traces::plot`], and handed across the boundary as a
    /// view rather than a copy (ADR 0002).
    plotted: Vec<f64>,
}

impl Traces {
    /// Records what every probe reads at this instant.
    ///
    /// Called from inside the run, once per solved step.
    ///
    /// Every trace takes a sample on every step, including the probes that had
    /// nothing to say: a `NaN` for one pin still dangling, and `NaN`s back to
    /// the start of the buffer for one placed a moment ago. They share one list
    /// of times, and a trace that skipped a step would be read against the
    /// wrong ones from then on — an off-by-one that grows.
    pub fn record(&mut self, netlist: &Netlist, point: &OperatingPoint, at: f64) {
        let probes: Vec<(ElementId, f64)> = netlist
            .elements()
            .filter(|(_, element)| {
                matches!(element.kind, ElementKind::Voltmeter | ElementKind::Ammeter)
            })
            .map(|(id, _)| (id, reading(netlist, point, id).unwrap_or(f64::NAN)))
            .collect();

        // No probes, no history: nothing has been asked about.
        if probes.is_empty() {
            return;
        }

        self.times.push(at);
        let depth = self.times.len();
        for (id, value) in probes {
            let ring = self.readings.entry(id).unwrap().or_default();
            while ring.len() + 1 < depth {
                ring.push(f64::NAN);
            }
            ring.push(value);
        }
    }

    /// Drops the history of every probe that is no longer on the canvas.
    pub fn prune(&mut self, netlist: &Netlist) {
        self.readings
            .retain(|element, _| netlist.element(element).is_some());
    }

    /// Drops one element's trace: a probe replaced by another kind reads
    /// something else from now on (C-137).
    pub fn forget(&mut self, element: ElementId) {
        self.readings.remove(element);
    }

    /// The most recent sample's time, or `None` before anything was recorded.
    pub fn latest(&self) -> Option<f64> {
        self.times.last()
    }

    /// How many samples this element's trace holds.
    pub fn depth(&self, element: ElementId) -> usize {
        self.readings.get(element).map_or(0, Ring::len)
    }

    /// Reduces the last `span` seconds of one trace to `columns` columns, and
    /// reports how many were written.
    ///
    /// Written into a buffer this owns, so the caller reads it as a view rather
    /// than being handed a copy every frame. A column the trace does not reach
    /// carries the last value it did reach — the sample-and-hold a scope does —
    /// and columns before the trace starts are left as `NaN`, which is the only
    /// honest answer to "what was it doing before it existed".
    pub fn plot(&mut self, element: ElementId, span: f64, columns: usize) -> usize {
        self.plotted.clear();
        if columns == 0 || !span.is_finite() || span <= 0.0 {
            return 0;
        }

        let Some(ring) = self.readings.get(element) else {
            return 0;
        };
        let Some(newest) = self.times.last() else {
            return 0;
        };
        let oldest = newest - span;

        let mut columns_of: Vec<Option<Column>> = vec![None; columns];
        for (at, value) in self.times.iter().zip(ring.iter()) {
            // Outside the window, or a step this probe had no answer for.
            if at < oldest || !value.is_finite() {
                continue;
            }
            // The newest sample belongs in the last column, not one past it.
            let along = ((at - oldest) / span * columns as f64) as usize;
            let column = along.min(columns - 1);
            match &mut columns_of[column] {
                Some(held) => {
                    held.low = held.low.min(value);
                    held.high = held.high.max(value);
                }
                empty => {
                    *empty = Some(Column {
                        low: value,
                        high: value,
                    })
                }
            }
        }

        let mut carried: Option<Column> = None;
        for column in columns_of {
            let shown = match column {
                Some(column) => {
                    carried = Some(column);
                    column
                }
                None => match carried {
                    // Nothing yet: the trace had not started by this column.
                    None => Column {
                        low: f64::NAN,
                        high: f64::NAN,
                    },
                    // Held, the way a scope holds between samples.
                    Some(held) => Column {
                        low: held.high,
                        high: held.high,
                    },
                },
            };
            self.plotted.push(shown.low);
            self.plotted.push(shown.high);
        }

        self.plotted.len()
    }

    /// Start of the plotted block inside wasm memory.
    ///
    /// Read after every [`Traces::plot`] and never kept: filling the buffer can
    /// grow wasm memory, which detaches every view over it (ADR 0002).
    pub fn plotted(&self) -> &[f64] {
        &self.plotted
    }
}

/// What one probe reads, given a solved circuit.
///
/// The one place the arithmetic lives. A voltmeter is the difference between
/// the potentials its two pins sit on; an ammeter is stamped as a zero-volt
/// source, so its branch unknown *is* the reading, negated because
/// `source_current` reports what leaves a first pin and a meter is marked for
/// what enters one (C-21).
pub fn reading(netlist: &Netlist, point: &OperatingPoint, id: ElementId) -> Option<f64> {
    let element = netlist.element(id)?;
    match element.kind {
        ElementKind::Voltmeter => {
            let plus = point.voltage(element.pins.first()?.net?)?;
            let minus = point.voltage(element.pins.get(1)?.net?)?;
            Some(plus - minus)
        }
        ElementKind::Ammeter => Some(-point.source_current(id)?),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ring_keeps_the_newest_and_drops_the_oldest() {
        let mut ring = Ring::default();
        for value in 0..(HISTORY + 10) {
            ring.push(value as f64);
        }

        let kept: Vec<f64> = ring.iter().collect();
        assert_eq!(kept.len(), HISTORY);
        assert_eq!(kept[0], 10.0);
        assert_eq!(*kept.last().unwrap(), (HISTORY + 9) as f64);
        assert_eq!(ring.last(), Some((HISTORY + 9) as f64));
    }

    #[test]
    fn a_ring_reads_back_in_order_before_it_wraps() {
        let mut ring = Ring::default();
        for value in 0..5 {
            ring.push(value as f64);
        }

        assert_eq!(
            ring.iter().collect::<Vec<_>>(),
            vec![0.0, 1.0, 2.0, 3.0, 4.0]
        );
        assert_eq!(ring.last(), Some(4.0));
    }

    #[test]
    fn an_empty_ring_has_no_last_sample() {
        assert_eq!(Ring::default().last(), None);
    }
}
