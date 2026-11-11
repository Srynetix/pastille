// SPDX-License-Identifier: MIT OR Apache-2.0

// Choosing an LED's colour, from the app's side of the boundary (C-46).
//
// The point of the ticket was that the *list* of colours belongs to the core,
// so most of what is worth testing here is that the app does not keep one.

import { describe, expect, it } from 'vitest'

import { componentRegistry, createCircuit } from './core'
import { LED_LIGHT, mixedLight } from './render'
import { toScene } from './scene'
import { SYMBOLS } from './symbols'

describe('the colours an LED can be', () => {
	it('come from the core and nowhere else', async () => {
		const registry = await componentRegistry()
		const led = registry.find((component) => component.kind === 'Led')

		expect(led?.choices).toEqual(['Red', 'Green', 'Blue', 'White'])
	})

	it('all have something to glow with', async () => {
		// The second place the closed set has to be agreed on: the core owns
		// which colours exist, the app owns what they look like. Without this a
		// fifth colour would arrive and quietly glow white.
		const registry = await componentRegistry()
		const led = registry.find((component) => component.kind === 'Led')

		for (const colour of led?.choices ?? []) {
			expect(LED_LIGHT[colour], colour).toBeDefined()
		}
	})

	it('are the only colours the app draws', async () => {
		// And the other direction, so the table cannot keep a colour the core
		// has dropped.
		const registry = await componentRegistry()
		const known = new Set(registry.find((c) => c.kind === 'Led')?.choices ?? [])

		for (const colour of Object.keys(LED_LIGHT)) {
			expect(known.has(colour), colour).toBe(true)
		}
	})
})

describe('recolouring an LED', () => {
	async function milestone() {
		const circuit = await createCircuit()
		const battery = circuit.add('DcSource', 0, 0)
		const resistor = circuit.add('Resistor', 80, 0)
		const led = circuit.add('Led', 160, 0)
		circuit.setValue(resistor, '1k')
		circuit.connect(battery, '+', resistor, 'a')
		circuit.connect(resistor, 'b', led, 'anode')
		circuit.connect(led, 'cathode', battery, '-')
		return { circuit, led }
	}

	it('reaches the canvas as a colour and a label', async () => {
		const { circuit, led } = await milestone()

		circuit.setChoice(led, 'Blue')
		circuit.advance(0.016)

		const item = toScene(circuit.elements(), circuit.state()).find((i) => i.id === led)
		expect(item?.colour).toBe('Blue')
		// The word stays: an unlit LED draws no glow, so it is the only thing
		// that says which colour it is.
		expect(item?.label).toBe('Blue')
	})

	it('changes what it needs to light', async () => {
		// Blue wants 3.2 V where red wants 1.8, so the same resistor lets less
		// through. Picking a colour and watching it dim is the shortest route
		// to what a forward voltage is.
		const { circuit, led } = await milestone()
		circuit.advance(0.016)
		const brightnessOf = () =>
			circuit.state().elementCurrents.find((c) => c.elementId === led)?.brightness ?? 0
		const red = brightnessOf()

		circuit.setChoice(led, 'Blue')
		circuit.advance(0.016)

		expect(brightnessOf()).toBeLessThan(red)
	})

	it('refuses a colour that is not one', async () => {
		const { circuit, led } = await milestone()

		expect(() => circuit.setChoice(led, 'Purple')).toThrow()
	})
})

describe('an RGB LED’s light', () => {
	const DICE = ['Red', 'Green', 'Blue']
	const rgb = (hex: string) => [1, 3, 5].map((at) => Number.parseInt(hex.slice(at, at + 2), 16))

	it('lights a channel of its own with every die', () => {
		expect(SYMBOLS.RgbLed.dice).toEqual(DICE)
	})

	it('is white with all three lit, however brightly', () => {
		expect(mixedLight(DICE, [1, 1, 1])).toBe('#ffffff')
		expect(mixedLight(DICE, [0.4, 0.4, 0.4])).toBe('#ffffff')
	})

	it('is one die’s own colour with only that die lit', () => {
		for (const [index, colour] of DICE.entries()) {
			const lit = [0, 0, 0]
			lit[index] = 1
			const channels = rgb(mixedLight(DICE, lit))
			expect(channels[index], colour).toBe(255)
			for (const [at, other] of channels.entries()) {
				if (at !== index) expect(other, colour).toBe(0)
			}
		}
	})

	it('glows yellow from red and green, and purple from red and blue', () => {
		expect(mixedLight(DICE, [1, 1, 0])).toBe('#ffff00')
		expect(mixedLight(DICE, [1, 0, 1])).toBe('#ff00ff')
		// Half as much green as red is orange.
		expect(mixedLight(DICE, [1, 0.5, 0])).toBe('#ff8000')
	})

	it('reaches the canvas as its brightest die', async () => {
		const circuit = await createCircuit()
		const battery = circuit.add('DcSource', 0, 0)
		const resistor = circuit.add('Resistor', 80, 0)
		const led = circuit.add('RgbLed', 160, 0)
		circuit.setValue(resistor, '1k')
		circuit.connect(battery, '+', resistor, 'a')
		circuit.connect(resistor, 'b', led, 'G')
		circuit.connect(led, 'com', battery, '-')
		circuit.advance(0.016)

		const item = toScene(circuit.elements(), circuit.state()).find((i) => i.id === led)
		expect(item?.segments?.[1]).toBeGreaterThan(0.5)
		expect(item?.brightness).toBe(item?.segments?.[1])
		expect(item?.label).toBe('Common cathode')
	})
})
