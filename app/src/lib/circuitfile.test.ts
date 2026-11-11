// SPDX-License-Identifier: MIT OR Apache-2.0

import { afterEach, describe, expect, it } from 'vitest'

import { createCircuit } from './core'
import { forgetCircuit, recallCircuit, rememberCircuit } from './circuitfile'

afterEach(() => forgetCircuit())

/** The milestone circuit, built the way the app builds one. */
async function milestone() {
	const circuit = await createCircuit()
	const battery = circuit.add('DcSource', 0, 0)
	const resistor = circuit.add('Resistor', 80, 0)
	const led = circuit.add('Led', 160, 0)
	circuit.connect(battery, '+', resistor, 'a')
	circuit.connect(resistor, 'b', led, 'anode')
	circuit.connect(led, 'cathode', battery, '-')
	return circuit
}

describe('a circuit that leaves the app and comes back', () => {
	it('is the same circuit', async () => {
		const saved = (await milestone()).save()

		const reopened = await createCircuit()
		reopened.load(saved)

		expect(reopened.elements()).toHaveLength(3)
		expect(reopened.nets()).toHaveLength(3)
		expect(reopened.save()).toBe(saved)
	})

	it('still runs', async () => {
		const saved = (await milestone()).save()
		const reopened = await createCircuit()

		reopened.load(saved)
		reopened.advance(0.05)

		expect(reopened.state().failure).toBeUndefined()
	})

	it('replaces whatever was on the canvas', async () => {
		// Opening is not merging: two circuits on one canvas would be a
		// surprise nobody asked for.
		const saved = (await milestone()).save()
		const busy = await createCircuit()
		busy.add('Resistor', 300, 300)

		busy.load(saved)

		expect(busy.elements()).toHaveLength(3)
	})

	it('refuses a file it cannot read, and leaves the canvas alone', async () => {
		const circuit = await milestone()

		expect(() => circuit.load('not a circuit')).toThrow()
		expect(circuit.elements(), 'the canvas should be untouched').toHaveLength(3)
	})

	it('says so when the file is from a newer version', async () => {
		const circuit = await createCircuit()
		const saved = (await milestone()).save().replace('"schema":1', '"schema":99')

		expect(() => circuit.load(saved)).toThrow(/format 99/)
	})
})

describe('remembering the circuit between visits', () => {
	it('gives back what it was given', async () => {
		const saved = (await milestone()).save()

		rememberCircuit(saved)

		expect(recallCircuit()).toBe(saved)
	})

	it('has nothing to give on a first visit', () => {
		expect(recallCircuit()).toBeUndefined()
	})

	it('can be forgotten', async () => {
		rememberCircuit((await milestone()).save())

		forgetCircuit()

		expect(recallCircuit()).toBeUndefined()
	})

	it('survives storage it cannot reach', () => {
		// Safari in private browsing throws outright. Losing an autosave is a
		// smaller failure than a canvas that will not draw.
		const original = Object.getOwnPropertyDescriptor(window, 'localStorage')
		Object.defineProperty(window, 'localStorage', {
			configurable: true,
			get() {
				throw new Error('denied')
			},
		})

		expect(() => rememberCircuit('{}')).not.toThrow()
		expect(recallCircuit()).toBeUndefined()

		if (original) Object.defineProperty(window, 'localStorage', original)
	})
})
