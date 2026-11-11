// SPDX-License-Identifier: MIT OR Apache-2.0

// How much simulated time a frame can afford (C-83).
//
// The frame loop tells the core how much wall time passed, and the core
// simulates that much (C-13). That is right for as long as simulating a
// second costs less than a second. Past it, it is a spiral: a frame that took
// 30 ms to simulate makes the next one owe 30 ms of simulation, which takes
// 60 ms, and so on until the core's own catch-up cap stops it at a quarter of a
// second a frame — half a second of the page not answering between paints.
// That is what froze "Switches to a number" (C-82), and what two counters on
// the dev server's unoptimised core would do again.
//
// So the loop keeps a budget. It measures what a simulated second has been
// costing, and gives each frame no more simulated time than the budget
// affords. A circuit too big for real time then runs slower than real time,
// and the page keeps answering, instead of the other way round. The core is
// not the place for this: it has no clock, on purpose, so that it runs
// headless (C-13), and wall time is the frame loop's to know.

/**
 * Milliseconds of each frame the simulation may take, of the 16.7 a
 * sixty-hertz frame has: the rest is for drawing it and for the browser.
 */
export const BUDGET_MS = 10

/** A sixtieth of a second: the frame the budget is written for. */
const FRAME = 1 / 60

/**
 * The most frames' worth of budget one call may spend.
 *
 * A frame that comes late — a slow machine, a throttled pane, a tab just
 * back — had idle time before it, so it may spend the same share of it: a
 * budget of 10 ms a sixtieth is 60 % of the wall time, whatever the frame
 * rate. But never more than three frames' worth at once, 30 ms, because
 * however long the wait was, the page cannot answer while it is spent.
 */
const MOST_FRAMES = 3

/**
 * How much of the newest measurement goes into the estimate, per frame.
 *
 * Enough to follow an edit within a few frames, little enough that one frame
 * the garbage collector ran in does not halve the simulation's speed.
 */
const SMOOTHING = 0.3

/**
 * The same for how fast the simulation is running, more slowly: a status line
 * saying it has slowed down should mean it has, over a second or so, not
 * that one frame was.
 */
const SPEED_SMOOTHING = 0.1

/**
 * Below this share of real time, the simulation says it is slowed down: a
 * frame or two dropped to a hiccup is not worth a word, a steady shortfall is.
 * It stops saying so only back above [`KEEPING_UP`], so a circuit that runs
 * at the edge of the budget is not a status line switching back and forth.
 */
const SLOW_BELOW = 0.85
const KEEPING_UP = 0.95

/**
 * The most wall time one frame is counted as owing, in seconds, for how fast
 * the simulation runs: the core's own catch-up cap. A tab coming back from the
 * background reports the whole time it was away, and the core would drop all
 * but a quarter of a second of it anyway.
 */
const LONGEST_FRAME = 0.25

/**
 * How much of real time a slowed simulation is keeping up with, in words for
 * the status line: "40 %", "4 %", "under 1 %".
 *
 * To the nearest ten while it is a good part of real time, where a closer
 * figure would only flicker; to the nearest one below that, where rounding to
 * ten said "0 %" of a circuit that was running, slowly — a 555 at audio
 * frequency on the unoptimised core does (C-85). Never 0: if it said so, the
 * simulation would have stopped.
 */
export function slowedWords(share: number): string {
	if (share >= 0.1) return `${Math.round(share * 10) * 10} %`
	if (share >= 0.01) return `${Math.round(share * 100)} %`
	return 'under 1 %'
}

export class Pace {
	/** Milliseconds a simulated second has been costing; unknown to begin with. */
	#msPerSecond: number | undefined
	/** How much of real time the simulation has lately been keeping up with. */
	#speed = 1
	/** Whether it is saying it is slowed down. */
	#slowed = false

	/**
	 * The simulated time to ask of the core for `elapsed` seconds of wall time:
	 * all of it when it fits the budget, and what fits when it does not.
	 */
	grant(elapsed: number): number {
		if (!(elapsed > 0)) return 0
		const cost = this.#msPerSecond
		if (cost === undefined || cost <= 0) return elapsed
		const frames = Math.min(Math.max(elapsed / FRAME, 1), MOST_FRAMES)
		return Math.min(elapsed, (BUDGET_MS * frames) / cost)
	}

	/**
	 * What the frame just cost: `granted` seconds simulated, of `elapsed`
	 * owed, in `ms` milliseconds.
	 */
	record(elapsed: number, granted: number, ms: number): void {
		if (granted > 0 && ms >= 0) {
			const cost = ms / granted
			this.#msPerSecond =
				this.#msPerSecond === undefined
					? cost
					: this.#msPerSecond + SMOOTHING * (cost - this.#msPerSecond)
		}
		if (elapsed > 0) {
			const kept = Math.min(1, granted / Math.min(elapsed, LONGEST_FRAME))
			this.#speed += SPEED_SMOOTHING * (kept - this.#speed)
		}
		if (this.#speed < SLOW_BELOW) this.#slowed = true
		else if (this.#speed > KEEPING_UP) this.#slowed = false
	}

	/**
	 * The share of real time the simulation is running at, when it is slowed
	 * down enough to say so; `undefined` when it is keeping up.
	 */
	slowedTo(): number | undefined {
		return this.#slowed ? this.#speed : undefined
	}
}
