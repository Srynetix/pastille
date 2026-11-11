// SPDX-License-Identifier: MIT OR Apache-2.0

import { describe, expect, it } from 'vitest'

import { Buzz, TONE_HZ, type ToneContext } from './sound'

/** A real audio context, counting the voices asked of it. */
function counted(): { context: AudioContext; voices: () => number } {
	let voices = 0
	const context = new AudioContext()
	const create = context.createOscillator.bind(context)
	context.createOscillator = () => {
		voices++
		return create()
	}
	return { context, voices: () => voices }
}

describe('a buzzer, heard', () => {
	it('is silent until something sounds, and opens no audio for nothing', () => {
		let opened = 0
		const buzz = new Buzz(() => {
			opened++
			return new AudioContext()
		})

		buzz.set(false)

		expect(buzz.sounding).toBe(false)
		expect(opened).toBe(0)
	})

	it('plays one note at the buzzer’s pitch, however often it is asked', () => {
		const { context, voices } = counted()
		const buzz = new Buzz(() => context)

		buzz.set(true)
		buzz.set(true)
		buzz.set(true)

		expect(buzz.sounding).toBe(true)
		expect(voices(), 'one voice, not one per frame').toBe(1)
		expect(TONE_HZ).toBe(2300)
	})

	it('stops, and starts again with a fresh voice', () => {
		const { context, voices } = counted()
		const buzz = new Buzz(() => context)

		buzz.set(true)
		buzz.set(false)
		expect(buzz.sounding).toBe(false)

		buzz.set(true)
		expect(buzz.sounding).toBe(true)
		// An oscillator plays once; a second start is a second one.
		expect(voices()).toBe(2)
	})

	it('plays a passive buzzer’s note, and follows it on the one voice', () => {
		// The note is the circuit's (C-85): set, then moved, never a second
		// voice for a second note.
		const { context, voices } = counted()
		const buzz = new Buzz(() => context)

		buzz.set(true, 440)
		expect(buzz.hertz).toBe(440)
		buzz.set(true, 523)
		expect(buzz.hertz).toBe(523)
		expect(voices()).toBe(1)

		buzz.set(false)
		expect(buzz.hertz).toBeUndefined()
	})

	it('stays drawable without audio at all', () => {
		// A browser that refuses a context: the buzzer is still drawn sounding
		// by the canvas, and nothing here throws over it.
		const buzz = new Buzz((): ToneContext => {
			throw new Error('no audio here')
		})

		expect(() => buzz.set(true)).not.toThrow()
		expect(buzz.sounding).toBe(false)
	})
})
