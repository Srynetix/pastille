// SPDX-License-Identifier: MIT OR Apache-2.0

// The switch and the push-button, from the app's side of the boundary (C-19).
//
// Two halves: what a press and a release mean, which is `contacts.ts` and is
// pure; and that the whole path from a gesture to a lit LED really joins up,
// which needs the core and so runs against the real bundle like everything else
// in this directory.

import { describe, expect, it } from 'vitest'

import { afterPress, afterRelease } from './contacts'
import { createCircuit, type Circuit, type ElementView } from './core'
import { hitTest, toScene } from './scene'
import { halfSizeOf } from './symbols'

/** The milestone circuit with a contact in the supply line. */
async function switched(kind: 'Switch' | 'PushButton') {
	const circuit = await createCircuit()
	const battery = circuit.add('DcSource', 0, 0)
	const contact = circuit.add(kind, 80, 0)
	const resistor = circuit.add('Resistor', 160, 0)
	const led = circuit.add('Led', 240, 0)
	circuit.connect(battery, '+', contact, 'a')
	circuit.connect(contact, 'b', resistor, 'a')
	circuit.connect(resistor, 'b', led, 'anode')
	circuit.connect(led, 'cathode', battery, '-')
	return { circuit, contact, led }
}

function viewOf(circuit: Circuit, id: string): ElementView {
	const found = circuit.elements().find((element) => element.id === id)
	if (!found) throw new Error(`no element ${id}`)
	return found
}

function currentThrough(circuit: Circuit, id: string): number {
	const found = circuit.state().elementCurrents.find((current) => current.elementId === id)
	return found?.amps ?? 0
}

/** One click: down on it, and up without having dragged it anywhere. */
function click(circuit: Circuit, id: string) {
	const down = afterPress(viewOf(circuit, id))
	if (down !== undefined) circuit.setClosed(id, down)
	const up = afterRelease(viewOf(circuit, id), false)
	if (up !== undefined) circuit.setClosed(id, up)
}

describe('what a press means', () => {
	it('is nothing at all to anything that is not a contact', async () => {
		const { circuit } = await switched('Switch')
		const resistor = circuit.elements().find((element) => element.kind === 'Resistor')

		expect(afterPress(resistor)).toBeUndefined()
		expect(afterRelease(resistor, false)).toBeUndefined()
		expect(afterRelease(undefined, false)).toBeUndefined()
	})

	it('shuts a button on the way down and opens it on the way up', async () => {
		const { circuit, contact } = await switched('PushButton')
		const button = viewOf(circuit, contact)

		expect(afterPress(button)).toBe(true)
		expect(afterRelease(button, false)).toBe(false)
		// Released whether or not the hand wandered: it is being let go either
		// way, and a button left stuck down would be worse than a stray flip.
		expect(afterRelease(button, true)).toBe(false)
	})

	it('flips a switch on the way up, and not on the way down', async () => {
		const { circuit, contact } = await switched('Switch')

		expect(afterPress(viewOf(circuit, contact))).toBeUndefined()
		expect(afterRelease(viewOf(circuit, contact), false)).toBe(true)

		circuit.setClosed(contact, true)
		expect(afterRelease(viewOf(circuit, contact), false)).toBe(false)
	})

	it('leaves a switch alone when it was dragged rather than clicked', async () => {
		// Otherwise a switch could not be repositioned without being flipped,
		// which is the C-23 frustration arriving by the back door.
		const { circuit, contact } = await switched('Switch')

		expect(afterRelease(viewOf(circuit, contact), true)).toBeUndefined()
	})
})

describe('a contact in a circuit', () => {
	it('turns the LED off and on again', async () => {
		// The acceptance, through every layer the app actually uses: the
		// gesture, the boundary, the solver, and the current the canvas draws.
		const { circuit, contact, led } = await switched('Switch')
		circuit.advance(0.016)
		expect(currentThrough(circuit, led)).toBeCloseTo(0, 12)

		click(circuit, contact)
		circuit.advance(0.016)
		expect(currentThrough(circuit, led)).toBeGreaterThan(1e-3)

		click(circuit, contact)
		circuit.advance(0.016)
		expect(currentThrough(circuit, led)).toBeCloseTo(0, 12)
	})

	it('lights only while a button is held', async () => {
		const { circuit, contact, led } = await switched('PushButton')

		// Held at least as long as a finger holds one; a shorter press is kept
		// that long (C-123).
		circuit.setClosed(contact, afterPress(viewOf(circuit, contact)) ?? false)
		circuit.advance(0.05)
		expect(currentThrough(circuit, led)).toBeGreaterThan(1e-3)

		circuit.setClosed(contact, afterRelease(viewOf(circuit, contact), false) ?? true)
		circuit.advance(0.016)
		expect(currentThrough(circuit, led)).toBeCloseTo(0, 12)
	})

	it('is drawn differently once it is shut', async () => {
		const { circuit, contact } = await switched('Switch')
		const open = toScene(circuit.elements()).find((item) => item.id === contact)

		circuit.setClosed(contact, true)
		const shut = toScene(circuit.elements()).find((item) => item.id === contact)

		expect(open?.closed).toBe(false)
		expect(shut?.closed).toBe(true)
		expect(shut?.symbol).not.toBe(open?.symbol)
		// Same terminals in both, or wiring one and then flipping it would move
		// the wire's anchor.
		expect(shut?.symbol.pins).toEqual(open?.symbol.pins)
	})

	it('is the same thing to click open or shut', async () => {
		// A shut switch draws almost nothing above its own leads, so a hit box
		// derived from the drawing alone would shrink under the pointer and the
		// click that closed it could not reach it to open it again.
		const { circuit, contact } = await switched('Switch')
		const open = toScene(circuit.elements())
		circuit.setClosed(contact, true)
		const shut = toScene(circuit.elements())

		const boxOf = (items: ReturnType<typeof toScene>) => {
			const item = items.find((candidate) => candidate.id === contact)
			if (!item) throw new Error('no contact')
			return halfSizeOf(item.symbol, item.turns)
		}
		expect(boxOf(shut)).toEqual(boxOf(open))

		// And the point of the box: a click a little above the middle lands on
		// the switch either way.
		for (const items of [open, shut]) {
			expect(hitTest(items, { x: 80, y: -8 })).toMatchObject({
				kind: 'element',
				item: { id: contact },
			})
		}
	})
})
