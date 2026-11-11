// SPDX-License-Identifier: MIT OR Apache-2.0

import { describe, expect, it } from 'vitest'

import { REFRESH_MS, Steady } from './steady'

/** Asks for `key` once a frame from `from` to `to` milliseconds, as `measure` says. */
function frames(
	steady: Steady,
	from: number,
	to: number,
	measure: (now: number) => number | undefined,
	key = 'v',
): (number | undefined)[] {
	const shown: (number | undefined)[] = []
	for (let now = from; now < to; now += 1000 / 60) shown.push(steady.value(key, now, measure(now)))
	return shown
}

describe('a number held still long enough to read', () => {
	it('shows the first reading at once', () => {
		expect(new Steady().value('v', 0, 4.2)).toBe(4.2)
	})

	it('changes at most three times a second, however fast the reading does', () => {
		// A 5 V square at 7 Hz against 60 frames: every frame different.
		const shown = frames(new Steady(), 0, 3000, (now) => (Math.floor(now / 71) % 2 ? 5 : 0))
		const changes = shown.filter((value, index) => index > 0 && value !== shown[index - 1]).length
		expect(changes).toBeLessThanOrEqual(Math.ceil(3000 / REFRESH_MS))
	})

	it('shows the mean of what it saw, not whichever moment it landed on', () => {
		// Half the frames at 5, half at 0: a meter on DC reads 2.5.
		let frame = 0
		const shown = frames(new Steady(), 0, 1000, () => (frame++ % 2 ? 5 : 0))
		expect(shown.at(-1)).toBeCloseTo(2.5, 1)
	})

	it('follows a reading that has settled', () => {
		const steady = new Steady()
		frames(steady, 0, 500, () => 1)
		const shown = frames(steady, 500, 1500, () => 9)
		expect(shown.at(-1)).toBe(9)
	})

	it('says nothing once a whole refresh has measured nothing', () => {
		const steady = new Steady()
		frames(steady, 0, 500, () => 1)
		expect(frames(steady, 500, 1500, () => undefined).at(-1)).toBeUndefined()
	})

	it('holds each key on its own', () => {
		const steady = new Steady()
		steady.value('a', 0, 1)
		expect(steady.value('b', 10, 7)).toBe(7)
		expect(steady.value('a', 20, 3)).toBe(1)
	})

	it('starts afresh for a reading nobody asked for in a while', () => {
		// The pointer leaves a part and comes back: what it shows is what is
		// true now, not the mean from before.
		const steady = new Steady()
		frames(steady, 0, 500, () => 1)
		steady.value('other', 3000, 0)
		expect(steady.value('v', 3010, 8)).toBe(8)
	})
})
