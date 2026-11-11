// SPDX-License-Identifier: MIT OR Apache-2.0

// Current along a wire where three pins meet (C-73).
//
// Against the real bundle and a real example, because the answer is the two
// halves together: the core says what each pin puts into a wire, and the app
// says which runs of the drawing that crosses.

import { describe, expect, it } from 'vitest'

import { createCircuit } from './core'
import { EXAMPLES } from './examples'
import { MIN_VISIBLE_AMPS } from './render'
import { pinPosition, toScene, toWires, type SceneItem } from './scene'
import type { Point } from './viewport'

/** "One way only", a frame in. */
async function oneWay() {
	const circuit = await createCircuit()
	circuit.load(EXAMPLES.find((example) => example.id === 'oneway')!.document)
	circuit.advance(1 / 60)
	const scene = toScene(circuit.elements(), circuit.state(), circuit.nets())
	const wires = toWires(circuit.nets(), scene, circuit.state().pinCurrents)
	return { circuit, scene, wires }
}

/** Where a pin is drawn. Elements are named by where they sit in the document. */
function pinOf(scene: readonly SceneItem[], index: number, name: string): Point {
	const item = scene[index]
	return pinPosition(
		item,
		item.symbol.pins.find((pin) => pin.name === name)!,
	)
}

const same = (a: Point, b: Point) => a.x === b.x && a.y === b.y

describe('a wire where three pins meet', () => {
	it('carries current down the branch that takes it, and not the one that refuses it', async () => {
		// The battery's + joins it to both diodes: the forward one (1) passes
		// the current, the reversed one (4) refuses it. However the wire is
		// routed — it may well pass the reversed diode's pin on the way — the
		// current arrives at the forward diode, and whatever reaches the
		// reversed one's pin leaves it again rather than going in.
		const { scene, wires } = await oneWay()
		const battery = pinOf(scene, 0, '+')
		const forwards = pinOf(scene, 1, 'anode')
		const backwards = pinOf(scene, 4, 'cathode')

		const wire = wires.find((candidate) => candidate.pins.some((pin) => same(pin, battery)))!
		expect(wire.pins).toHaveLength(3)
		// Current worth counting only: a reversed diode leaks picoamps, which
		// the canvas does not draw.
		const flows = (wire.flows ?? []).filter((flow) => flow.amps > 1e-9)
		const arriving = (at: Point) =>
			flows
				.filter((flow) => same(flow.segments[flow.segments.length - 1].to, at))
				.reduce((sum, flow) => sum + flow.amps, 0)
		const leaving = (at: Point) =>
			flows
				.filter((flow) => same(flow.segments[0].from, at))
				.reduce((sum, flow) => sum + flow.amps, 0)

		expect(arriving(forwards), 'into the forward diode').toBeGreaterThan(0.01)
		expect(leaving(forwards), 'and none back out').toBe(0)
		expect(arriving(backwards), 'past the reversed one').toBeCloseTo(leaving(backwards), 9)
		expect(leaving(battery), 'out of the battery').toBeCloseTo(arriving(forwards), 9)
	})

	it('draws it at the current the battery supplies', async () => {
		const { circuit, wires } = await oneWay()
		const supplied = Math.abs(
			circuit.state().elementCurrents.find((entry) => entry.elementId === circuit.elements()[0].id)!
				.amps,
		)
		expect(supplied).toBeGreaterThan(0.01)

		// Every wire the current crosses shows it — both three-pin wires
		// included, which before this drew nothing at all — and the two wires
		// of the refused branch show nothing a person could see.
		const visible = (wire: (typeof wires)[number]) =>
			(wire.flows ?? []).filter((flow) => flow.amps > 1e-9)
		const moving = wires.filter((wire) => visible(wire).length > 0)
		expect(moving).toHaveLength(wires.length - 2)
		for (const wire of moving) {
			for (const flow of visible(wire)) {
				expect(flow.amps).toBeCloseTo(supplied, 6)
			}
		}
	})
})

describe('the least current drawn moving', () => {
	it('leaves the wire past a relay at rest still', async () => {
		// "A fan when it's hot" at 25 °C: the relay is at rest, and the wire
		// from its open contact to the motor carries its leak, a few nanoamps
		// — which drew dots along it, as if the motor were being fed (C-87).
		const circuit = await createCircuit()
		circuit.load(EXAMPLES.find((example) => example.id === 'hotfan')!.document)
		for (let frame = 0; frame < 12; frame++) circuit.advance(1 / 60)
		const state = circuit.state()
		const scene = toScene(circuit.elements(), state, circuit.nets())
		const wires = toWires(circuit.nets(), scene, state.pinCurrents)
		const motor = pinOf(scene, 9, 'a')
		const fed = wires.find((candidate) => candidate.pins.some((pin) => same(pin, motor)))!
		const flows = fed.flows ?? []
		expect(flows.length, 'it does carry the leak').toBeGreaterThan(0)
		expect(
			flows.filter((flow) => flow.amps >= MIN_VISIBLE_AMPS),
			'and none of it is drawn',
		).toEqual([])
	})

	it('still draws the current that runs it, once it runs', async () => {
		const circuit = await createCircuit()
		circuit.load(EXAMPLES.find((example) => example.id === 'hotfan')!.document)
		circuit.setWiper(circuit.elements()[1].id, (40 + 40) / 165)
		for (let frame = 0; frame < 20; frame++) circuit.advance(1 / 60)
		const state = circuit.state()
		const scene = toScene(circuit.elements(), state, circuit.nets())
		const wires = toWires(circuit.nets(), scene, state.pinCurrents)
		const motor = pinOf(scene, 9, 'a')
		const fed = wires.find((candidate) => candidate.pins.some((pin) => same(pin, motor)))!
		expect(
			(fed.flows ?? []).some((flow) => flow.amps > 0.1),
			'a third of an amp',
		).toBe(true)
	})
})
