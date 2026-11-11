// SPDX-License-Identifier: MIT OR Apache-2.0

// The arithmetic behind the scope, kept out of the thing that draws it.
//
// C-22. The core owns the buffer and reduces a window of it to columns (one low
// and one high per column, so a trace longer than the place it is drawn cannot
// alias). What is left is the two questions a scope asks of a person — how much
// time to show, and how much of the vertical to use — and turning the answers
// into pixels. All of it is arithmetic on numbers, so all of it is testable
// without a canvas.

import type { ElementView } from './core'
import { describe } from './palette'
import { formatQuantity } from './units'

/** The kinds that read something worth plotting (C-21). */
const PROBES = ['Voltmeter', 'Ammeter']

/** One trace: one probe, and what it is called. */
export interface Lane {
	readonly id: string
	/** "Voltmeter", "Ammeter" — what the catalogue calls it. */
	readonly label: string
	/** The unit it reads in, which the core owns. */
	readonly unit: string
}

/** A trace per probe on the canvas, in the order they were placed. */
export function lanesFor(elements: readonly ElementView[]): Lane[] {
	return elements
		.filter((element) => PROBES.includes(element.kind))
		.map((element) => ({
			id: element.id,
			label: describe(element.kind)?.label ?? element.kind,
			unit: element.unit,
		}))
}

/**
 * How much time the scope shows, shortest first.
 *
 * Five, spaced by tens, because a decade apart is the smallest step where the
 * picture changes into a different picture rather than the same one stretched.
 * Ten seconds is the default: an RC a person can watch has a time constant of
 * seconds, and half a minute is all the history the buffer keeps anyway.
 */
export const TIME_BASES: readonly number[] = [0.01, 0.1, 1, 10, 30]

export const DEFAULT_TIME_BASE = 10

/** How much of a lane's own unit is shown, from bottom to top. */
export interface Extent {
	readonly low: number
	readonly high: number
}

/** Fraction of the range left as air above and below the trace. */
const MARGIN = 0.08

/**
 * The lowest and highest a trace reached in a plotted block, or nothing when
 * it has not reached anything yet.
 *
 * `NaN` columns are the core saying the trace had not started yet, and are
 * skipped rather than read as zero.
 */
export function reached(block: ArrayLike<number>): Extent | undefined {
	let low = Number.POSITIVE_INFINITY
	let high = Number.NEGATIVE_INFINITY
	for (let index = 0; index < block.length; index++) {
		const value = block[index]
		if (!Number.isFinite(value)) continue
		if (value < low) low = value
		if (value > high) high = value
	}
	return low > high ? undefined : { low, high }
}

/**
 * The extent to draw a plotted block in: what it reached, with some air.
 *
 * For drawing, never for reading (C-69): the air is there so a trace does not
 * run along the lane's edge, and a label read from it would report values the
 * trace never had — a negative voltage on a circuit without one. A block with
 * nothing in it at all gets one unit of height, because a lane still has to be
 * drawn and dividing by a zero-height extent would put the whole trace on one
 * row of pixels.
 */
export function fit(block: ArrayLike<number>): Extent {
	const trace = reached(block)
	if (!trace) return { low: 0, high: 1 }
	const { low, high } = trace

	// A flat trace has no range of its own, so it is given one around itself —
	// otherwise a steady nine volts would be drawn on the same row as a steady
	// nothing, and the two would look alike.
	if (high - low < Number.EPSILON) {
		const reach = Math.abs(high) > 0 ? Math.abs(high) * 0.5 : 1
		return { low: low - reach, high: high + reach }
	}

	const air = (high - low) * MARGIN
	return { low: low - air, high: high + air }
}

/**
 * Full-scale values a lane can be pinned to, from what it is currently showing.
 *
 * A 1-2-5 ladder from the trace's own magnitude upwards, so the choices are
 * always the right order of magnitude for what is being measured — a
 * milliamp trace is never offered ten amps, which is what a fixed list of
 * volts-per-division would have done.
 */
export function scaleChoices(extent: Extent, count = 3): number[] {
	const reach = Math.max(Math.abs(extent.low), Math.abs(extent.high))
	if (!Number.isFinite(reach) || reach <= 0) return []

	const steps = [1, 2, 5]
	let decade = Math.floor(Math.log10(reach))
	const choices: number[] = []
	while (choices.length < count) {
		for (const step of steps) {
			const candidate = step * 10 ** decade
			// Strictly above what is already shown: an option identical to the
			// fit is an option that does nothing.
			if (candidate > reach * 1.001) choices.push(candidate)
			if (choices.length >= count) break
		}
		decade += 1
	}

	return choices
}

/**
 * The full scales to offer a lane, including the one it is already pinned to.
 *
 * Built from what the lane *fits*, never from what it is being drawn at: pinning
 * a lane to ten volts makes its extent ten volts, and a ladder built from that
 * starts at twenty — so the option in force would drop out of its own list and
 * the control would go blank. The pinned value is put back even when the trace
 * has since grown past it, because a control that forgets its own setting is
 * worse than one offering a scale that no longer fits.
 */
export function choicesFor(fitted: Extent, pinned: number | undefined): number[] {
	const choices = scaleChoices(fitted)
	if (pinned === undefined || choices.includes(pinned)) return choices

	return [...choices, pinned].sort((left, right) => left - right)
}

/**
 * The extent to draw a lane at: what it fits, or what it was pinned to.
 *
 * A pinned lane keeps the side of zero its trace is on. A charging curve runs
 * from nothing upwards, and giving it the negative half of the lane as well
 * would throw away half the height to draw a region the trace never enters.
 */
export function extentFor(fitted: Extent, fullScale: number | undefined): Extent {
	if (fullScale === undefined) return fitted
	if (fitted.low >= 0) return { low: 0, high: fullScale }
	if (fitted.high <= 0) return { low: -fullScale, high: 0 }
	return { low: -fullScale, high: fullScale }
}

/** Where a value sits in a lane `height` pixels tall, measured from the top. */
export function toY(value: number, extent: Extent, height: number): number {
	const range = extent.high - extent.low
	if (range <= 0) return height / 2
	return height - ((value - extent.low) / range) * height
}

/**
 * What a trace reached, written beside its name: a reading, so the lowest and
 * the highest it actually was, never the edges of the lane it is drawn in
 * (C-69). One number for a trace that has not moved — as far as the label can
 * show — and a dash for one with nothing in it yet. A pinned lane reads the
 * same: its scale is what the control beside it says.
 */
export function labelReached(trace: Extent | undefined, unit: string): string {
	if (!trace) return '—'
	const low = formatQuantity(trace.low, unit)
	const high = formatQuantity(trace.high, unit)
	return low === high ? low : `${low} → ${high}`
}

/** A time base, written for the control that offers it. */
export function labelTimeBase(seconds: number): string {
	return formatQuantity(seconds, 's')
}
