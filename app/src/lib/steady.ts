// SPDX-License-Identifier: MIT OR Apache-2.0

// Numbers a person can read while the circuit moves.
//
// A reading is solved sixty times a second, and on anything that changes fast
// — a 555's output, a signal generator, a capacitor charging — writing each
// one out as it comes makes the digits a blur: right every frame, legible in
// none. A multimeter has the same problem and the same answer: it refreshes its
// display a few times a second and shows the average of what it saw in
// between. So does this, on the wall clock, because readability is the
// person's pace and not the simulation's — a circuit slowed to a tenth of real
// time still changes its digits three times a second.
//
// The average, not the last value: on a fast square the last value is
// whichever half of the cycle the frame happened to land in, and holding it
// would only make the flicker slower. What a fast signal does inside a frame is
// the scope's to show, not a number's.

/** How long a number stays on screen before it may change, in milliseconds. */
export const REFRESH_MS = 333

/** How long a number nobody asks for is remembered, in milliseconds. */
const FORGET_MS = 1000

interface Held {
	/** What is on screen. */
	shown: number | undefined
	/** What has come in since it was put there. */
	sum: number
	count: number
	/** When it was put there. */
	since: number
	/** When it was last asked for. */
	seen: number
}

/** A display that holds each number it shows for a moment, by key. */
export class Steady {
	private readonly held = new Map<string, Held>()
	private swept = 0

	/**
	 * The number to show for `key` at `now`, given what was measured this
	 * frame. The first time a key is asked for it is shown at once, so a
	 * readout does not open blank; after that it moves only every
	 * [`REFRESH_MS`], to the mean of what came in. `undefined` measured for a
	 * whole refresh is shown as `undefined`: a meter whose leads came off.
	 */
	value(key: string, now: number, measured: number | undefined): number | undefined {
		this.sweep(now)
		const held = this.held.get(key)
		if (!held) {
			this.held.set(key, { shown: measured, sum: 0, count: 0, since: now, seen: now })
			return measured
		}
		held.seen = now
		if (measured !== undefined && Number.isFinite(measured)) {
			held.sum += measured
			held.count++
		}
		if (now - held.since >= REFRESH_MS) {
			held.shown = held.count > 0 ? held.sum / held.count : undefined
			held.sum = 0
			held.count = 0
			held.since = now
		}
		return held.shown
	}

	/**
	 * Forgets what has not been asked for lately, so a readout that comes
	 * back later starts from what is true then rather than from an old mean.
	 */
	private sweep(now: number): void {
		if (now - this.swept < FORGET_MS) return
		this.swept = now
		for (const [key, held] of this.held) {
			if (now - held.seen > FORGET_MS) this.held.delete(key)
		}
	}
}

/** What a caller that holds numbers steady hands to the code that writes them. */
export type Settle = (key: string, measured: number | undefined) => number | undefined

/** A settle that holds nothing, for a caller with no clock: a test, a one-off. */
export const AS_MEASURED: Settle = (_key, measured) => measured
