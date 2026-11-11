// SPDX-License-Identifier: MIT OR Apache-2.0

// C-24's acceptance, at the seam: the circuit is edited, and what the app draws
// follows without anyone having said so.
//
// Every edit here goes straight to the circuit handle. Nothing tells the
// projection anything, which is the whole point — the old arrangement was
// correct only for as long as every call site remembered a `refresh()`.

import { describe, expect, it } from 'vitest'

import { createCircuit, type Circuit } from './core'
import { Projection } from './projection.svelte'

/** A frame of the shell's loop, which is the only thing that re-reads. */
const FRAME = 1 / 60

async function milestone(): Promise<{ circuit: Circuit; ids: Record<string, string> }> {
	const circuit = await createCircuit()
	const battery = circuit.add('DcSource', 0, 0)
	const resistor = circuit.add('Resistor', 40, 0)
	const led = circuit.add('Led', 80, 0)

	circuit.connect(battery, '+', resistor, 'a')
	circuit.connect(resistor, 'b', led, 'anode')
	circuit.connect(led, 'cathode', battery, '-')

	return { circuit, ids: { battery, resistor, led } }
}

describe('a projection of a circuit', () => {
	it('has the circuit before the first frame has run', async () => {
		// Otherwise the first paint is of an empty canvas, and a circuit that
		// was restored from last time flickers in a frame later.
		const { circuit } = await milestone()

		const projection = new Projection(circuit)

		expect(projection.elements).toHaveLength(3)
		expect(projection.nets).toHaveLength(3)
	})

	it('follows a component added behind its back', async () => {
		const { circuit } = await milestone()
		const projection = new Projection(circuit)

		circuit.add('Capacitor', 120, 0)
		projection.advance(FRAME)

		expect(projection.elements.map((element) => element.kind)).toContain('Capacitor')
	})

	it('follows a component removed behind its back', async () => {
		const { circuit, ids } = await milestone()
		const projection = new Projection(circuit)

		circuit.remove(ids.led)
		projection.advance(FRAME)

		expect(projection.elements.map((element) => element.id)).not.toContain(ids.led)
	})

	it('follows a value edited behind its back', async () => {
		const { circuit, ids } = await milestone()
		const projection = new Projection(circuit)

		circuit.setValue(ids.resistor, '1k')
		projection.advance(FRAME)

		const resistor = projection.elements.find((element) => element.id === ids.resistor)
		expect(resistor?.label).toBe('1 kΩ')
	})

	it('follows a move, which the solver never hears about', async () => {
		// The case a counter on the netlist alone would miss: dragging a
		// component changes the drawing and nothing else, and the canvas is the
		// only thing that cares.
		const { circuit, ids } = await milestone()
		const projection = new Projection(circuit)

		circuit.moveElement(ids.led, 200, 120)
		projection.advance(FRAME)

		const led = projection.elements.find((element) => element.id === ids.led)
		expect([led?.x, led?.y]).toEqual([200, 120])
	})

	it('follows a description, which the solver never hears about either', async () => {
		// Counted as an edit, so the autosave keeps it, though the run goes on.
		const { circuit } = await milestone()
		const projection = new Projection(circuit)
		const before = projection.edits

		circuit.setDescription('The first circuit.')
		projection.advance(FRAME)

		expect(projection.description).toBe('The first circuit.')
		expect(projection.edits).toBe(before + 1)
	})

	it('follows an undo', async () => {
		const { circuit } = await milestone()
		const projection = new Projection(circuit)
		circuit.add('Capacitor', 120, 0)
		projection.advance(FRAME)

		circuit.undo()
		projection.advance(FRAME)

		expect(projection.elements).toHaveLength(3)
	})

	it('follows a circuit opened over the top of it', async () => {
		const { circuit } = await milestone()
		const saved = circuit.save()

		const other = await createCircuit()
		const otherProjection = new Projection(other)
		other.load(saved)
		otherProjection.advance(FRAME)

		expect(otherProjection.elements).toHaveLength(3)
		expect(otherProjection.nets).toHaveLength(3)
	})
})

describe('a frame in which nothing was edited', () => {
	it('hands on the same arrays it handed on before', async () => {
		// Identity, not equality. Everything downstream — the inspector, the
		// scene, the scope's lanes — is derived from these, and handing over a
		// new array sixty times a second would wake all of it for nothing.
		const { circuit } = await milestone()
		const projection = new Projection(circuit)
		const elements = projection.elements
		const nets = projection.nets

		for (let frame = 0; frame < 10; frame++) projection.advance(FRAME)

		expect(projection.elements).toBe(elements)
		expect(projection.nets).toBe(nets)
	})

	it('still moves the solution on', async () => {
		// The structure holding still is not the circuit holding still.
		const { circuit } = await milestone()
		const projection = new Projection(circuit)

		projection.advance(FRAME)
		const first = projection.solution?.time ?? 0
		projection.advance(FRAME)

		expect(projection.solution?.time).toBeGreaterThan(first)
	})

	it('counts no edit', async () => {
		const { circuit } = await milestone()
		const projection = new Projection(circuit)
		const edits = projection.edits

		for (let frame = 0; frame < 10; frame++) projection.advance(FRAME)

		expect(projection.edits).toBe(edits)
	})
})

describe('the edit count', () => {
	it('moves once per change, whatever else happened that frame', async () => {
		const { circuit, ids } = await milestone()
		const projection = new Projection(circuit)
		const before = projection.edits

		circuit.setValue(ids.resistor, '470')
		circuit.moveElement(ids.led, 10, 10)
		projection.advance(FRAME)

		expect(projection.edits).toBe(before + 1)
	})

	it('is what a save can be hung off', async () => {
		// C-29's autosave watches this rather than the frame, which is the
		// difference between writing the file on an edit and writing it sixty
		// times a second.
		const { circuit } = await milestone()
		const projection = new Projection(circuit)

		const quiet = projection.edits
		projection.advance(FRAME)
		circuit.add('Resistor', 160, 0)
		projection.advance(FRAME)

		expect(projection.edits).toBe(quiet + 1)
	})
})
