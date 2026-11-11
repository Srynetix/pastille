// SPDX-License-Identifier: MIT OR Apache-2.0

// The potentiometer, from the app's side of the boundary (C-20).

import { describe, expect, it } from 'vitest'

import { createCircuit, type Circuit } from './core'
import { toScene } from './scene'
import { SYMBOLS, symbolFor } from './symbols'

/** A pot dimming an LED: wired as a rheostat, which is how one is first used. */
async function dimmer() {
	const circuit = await createCircuit()
	const battery = circuit.add('DcSource', 0, 0)
	const pot = circuit.add('Potentiometer', 80, 0)
	const led = circuit.add('Led', 160, 0)
	circuit.connect(battery, '+', pot, 'a')
	circuit.connect(pot, 'wiper', led, 'anode')
	circuit.connect(led, 'cathode', battery, '-')
	return { circuit, pot, led }
}

function brightnessOf(circuit: Circuit, led: string): number {
	const found = circuit.state().elementCurrents.find((current) => current.elementId === led)
	return found?.brightness ?? 0
}

describe('turning a wiper', () => {
	it('dims the LED as it is turned up', async () => {
		// The acceptance, from where a person stands: the knob changes the
		// circuit while it moves. `a` to the wiper is the track behind it, so
		// turning it up puts more resistance in the way.
		const { circuit, pot, led } = await dimmer()

		circuit.setWiper(pot, 0.02)
		circuit.advance(0.016)
		const bright = brightnessOf(circuit, led)

		circuit.setWiper(pot, 0.5)
		circuit.advance(0.016)
		const dim = brightnessOf(circuit, led)

		expect(bright).toBeGreaterThan(0)
		expect(dim).toBeLessThan(bright)
	})

	it('is a rheostat with its far end in the air', async () => {
		// Two terminals, not three: the commonest first wiring, and it has to
		// conduct through the half that is wired rather than be dropped for the
		// terminal nobody attached.
		const { circuit } = await dimmer()
		circuit.advance(0.016)

		expect(circuit.state().failure).toBeUndefined()
		expect(circuit.state().netCurrents.some((wire) => wire.amps > 1e-6)).toBe(true)
	})

	it('clamps a knob turned past its stop rather than refusing it', async () => {
		const { circuit, pot } = await dimmer()

		circuit.setWiper(pot, 4)
		expect(circuit.elements().find((element) => element.id === pot)?.wiper).toBe(1)

		circuit.setWiper(pot, -4)
		expect(circuit.elements().find((element) => element.id === pot)?.wiper).toBe(0)
	})

	it('keeps the wiper where it is when the track is retuned', async () => {
		// A fraction still means the same thing about a different total, which
		// is the reason it is a fraction.
		const { circuit, pot } = await dimmer()
		circuit.setWiper(pot, 0.25)

		circuit.setValue(pot, '1k')

		const view = circuit.elements().find((element) => element.id === pot)
		expect(view?.wiper).toBe(0.25)
		expect(view?.value).toBe('1k')
	})

	it('is refused on anything that has not got a wiper', async () => {
		const { circuit, led } = await dimmer()

		expect(() => circuit.setWiper(led, 0.5)).toThrow()
	})

	it('folds a whole turn into one step of undo', async () => {
		// What the shell's gesture is for: a slider reports a hundred values on
		// the way across, and that was one thing the user did.
		const { circuit, pot } = await dimmer()
		circuit.setWiper(pot, 0.5)

		circuit.beginGesture()
		for (let step = 1; step <= 20; step++) circuit.setWiper(pot, step / 20)
		circuit.endGesture()

		circuit.undo()
		expect(circuit.elements().find((element) => element.id === pot)?.wiper).toBe(0.5)
	})
})

describe('drawing a wiper', () => {
	it('moves the arrow along the track', () => {
		// The feedback that the knob did something. Rebuilt per position, which
		// is why the symbol is a function rather than a table entry.
		const low = symbolFor('Potentiometer', { wiper: 0 })
		const high = symbolFor('Potentiometer', { wiper: 1 })

		const tipOf = (symbol: typeof low) =>
			symbol.shapes.filter((shape) => shape.kind === 'polygon').at(-1)?.points[0].x
		expect(tipOf(low)).toBeLessThan(tipOf(high) as number)
	})

	it('keeps its pins wherever the wiper is', () => {
		// Or a wire drawn to a pot would move when the knob was turned.
		const low = symbolFor('Potentiometer', { wiper: 0 })
		const high = symbolFor('Potentiometer', { wiper: 0.8 })

		expect(low.pins).toEqual(high.pins)
		expect(low.pins).toEqual(SYMBOLS.Potentiometer.pins)
		expect(low.pins.map((pin) => pin.name)).toEqual(['a', 'wiper', 'b'])
	})

	it('reaches the canvas through the scene', async () => {
		const { circuit, pot } = await dimmer()
		circuit.setWiper(pot, 0.75)

		const item = toScene(circuit.elements()).find((candidate) => candidate.id === pot)

		expect(item?.wiper).toBe(0.75)
		expect(item?.symbol).not.toBe(SYMBOLS.Potentiometer)
		// Labelled by the track, the way a resistor is.
		expect(item?.label).toBe('10 kΩ')
	})
})
