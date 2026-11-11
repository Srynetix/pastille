// SPDX-License-Identifier: MIT OR Apache-2.0

import { describe, expect, it } from 'vitest'

import { BUDGET_MS, Pace, slowedWords } from './pace'

/** Sixty frames of a circuit whose simulated second costs `msPerSecond`. */
function run(pace: Pace, msPerSecond: number, frames = 60): number[] {
	const granted: number[] = []
	for (let frame = 0; frame < frames; frame++) {
		const elapsed = 1 / 60
		const given = pace.grant(elapsed)
		pace.record(elapsed, given, given * msPerSecond)
		granted.push(given)
	}
	return granted
}

describe('the pace of the simulation (C-83)', () => {
	it('gives a circuit that keeps up all the time that passed', () => {
		const pace = new Pace()
		// 2 ms a simulated second: "Counting to 99" on the optimised core.
		expect(run(pace, 120).every((given) => given === 1 / 60)).toBe(true)
		expect(pace.slowedTo()).toBeUndefined()
	})

	it('gives one that cannot no more than the budget affords, and says so', () => {
		const pace = new Pace()
		// 30 ms a frame's worth: the same circuit on the unoptimised core.
		const granted = run(pace, 1800)
		const last = granted[granted.length - 1]
		expect(last * 1800).toBeCloseTo(BUDGET_MS, 6)
		expect(pace.slowedTo()).toBeCloseTo(BUDGET_MS / 1800 / (1 / 60), 1)
	})

	it('never spirals: what a frame costs stays inside the budget', () => {
		// The freeze of C-82: each frame owed more than the last cost.
		const pace = new Pace()
		const granted = run(pace, 5000, 120)
		for (const given of granted.slice(1)) expect(given * 5000).toBeLessThanOrEqual(BUDGET_MS + 1e-9)
	})

	it('follows an edit that makes the circuit cheaper again', () => {
		const pace = new Pace()
		run(pace, 1800)
		run(pace, 100, 30)
		expect(pace.grant(1 / 60)).toBe(1 / 60)
		expect(pace.slowedTo()).toBeUndefined()
	})

	it('spends the same share of a late frame, but never more than 30 ms at once', () => {
		// The browser pane, throttled: a second between two frames. The frame
		// may spend three frames' budget of it, not one, and not the second.
		const pace = new Pace()
		run(pace, 1800)
		const given = pace.grant(1)
		expect(given * 1800).toBeCloseTo(3 * BUDGET_MS, 6)
	})

	it('counts late frames when it says how fast it runs', () => {
		// Frames a second apart on a circuit that costs 1.8 s a simulated
		// second: it keeps up with a few per cent of real time, not the 80 %
		// the frames in between would say on their own.
		const pace = new Pace()
		for (let second = 0; second < 20; second++) {
			for (const elapsed of [1, 1 / 60, 1 / 60, 1 / 60]) {
				const given = pace.grant(elapsed)
				pace.record(elapsed, given, given * 1800)
			}
		}
		expect(pace.slowedTo()).toBeLessThan(0.5)
	})

	it('does not take a tab coming back from the background for a slow circuit', () => {
		const pace = new Pace()
		run(pace, 100)
		pace.record(30, pace.grant(30), 3)
		expect(pace.slowedTo()).toBeUndefined()
	})

	it('does not flicker at the edge of the budget', () => {
		// A circuit that costs exactly one budget a frame, jittering by a
		// tenth either way from frame to frame.
		const pace = new Pace()
		const said: boolean[] = []
		for (let frame = 0; frame < 600; frame++) {
			const elapsed = 1 / 60
			const given = pace.grant(elapsed)
			const cost = (BUDGET_MS / (1 / 60) / 0.88) * (frame % 2 ? 1.1 : 0.9)
			pace.record(elapsed, given, given * cost)
			said.push(pace.slowedTo() !== undefined)
		}
		const changes = said.filter((value, index) => index > 0 && value !== said[index - 1]).length
		expect(changes).toBeLessThanOrEqual(1)
	})
})

describe('how slow it says it is', () => {
	it('rounds to tens while it is a good part of real time', () => {
		expect(slowedWords(0.46)).toBe('50 %')
		expect(slowedWords(0.1)).toBe('10 %')
	})

	it('says a few per cent rather than none', () => {
		// Rounded to tens, this was "0 %" of a circuit that was running.
		expect(slowedWords(0.04)).toBe('4 %')
		expect(slowedWords(0.012)).toBe('1 %')
		expect(slowedWords(0.003)).toBe('under 1 %')
	})
})
