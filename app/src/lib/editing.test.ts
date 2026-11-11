// SPDX-License-Identifier: MIT OR Apache-2.0

// Moving, deleting and undoing, from the app's side of the boundary.

import { describe, expect, it } from 'vitest'

import { createCircuit, type Circuit } from './core'
import { toScene, toWires } from './scene'

async function milestone() {
	// Spaced 80 apart, not 40: pins reach 20 either side, so components exactly
	// 40 apart have their pins in the same place and the wire between them has
	// no length to draw.
	const circuit = await createCircuit()
	const battery = circuit.add('DcSource', 0, 0)
	const resistor = circuit.add('Resistor', 80, 0)
	const led = circuit.add('Led', 160, 0)
	circuit.connect(battery, '+', resistor, 'a')
	circuit.connect(resistor, 'b', led, 'anode')
	circuit.connect(led, 'cathode', battery, '-')
	return { circuit, battery, resistor, led }
}

function positionOf(circuit: Circuit, id: string) {
	const found = circuit.elements().find((element) => element.id === id)
	if (!found) throw new Error(`no element ${id}`)
	return { x: found.x, y: found.y }
}

describe('moving components', () => {
	it('moves the component and its wires together', async () => {
		const { circuit, resistor } = await milestone()
		const before = toWires(circuit.nets(), toScene(circuit.elements()))

		circuit.moveElement(resistor, 200, 160)

		const after = toWires(circuit.nets(), toScene(circuit.elements()))
		expect(positionOf(circuit, resistor)).toEqual({ x: 200, y: 160 })
		expect(after).toHaveLength(before.length)
		// The wires are drawn somewhere else now, because they are drawn from
		// where the pins are rather than remembered.
		expect(JSON.stringify(after)).not.toBe(JSON.stringify(before))
	})

	it('keeps the circuit solving while a component is dragged', async () => {
		const { circuit, resistor } = await milestone()

		for (let step = 0; step < 60; step++) {
			circuit.moveElement(resistor, step * 3, step)
			circuit.advance(0.016)
			expect(circuit.state().failure).toBeUndefined()
		}
	})

	it('folds a whole drag into one undo step', async () => {
		const { circuit, resistor } = await milestone()
		const start = positionOf(circuit, resistor)

		circuit.beginGesture()
		for (let step = 1; step <= 40; step++) circuit.moveElement(resistor, step * 5, 0)

		circuit.undo()

		expect(positionOf(circuit, resistor)).toEqual(start)
	})

	it('spends no undo on a press that moved nothing', async () => {
		// Clicking a component to select it opens a gesture and closes it
		// having changed nothing. Each of those used to be a step of history
		// identical to the one before, so undo appeared to do nothing until it
		// had been pressed past every click since the last real edit.
		const { circuit, resistor } = await milestone()
		const start = positionOf(circuit, resistor)

		circuit.beginGesture()
		for (let step = 1; step <= 40; step++) circuit.moveElement(resistor, step * 5, 0)
		circuit.endGesture()
		for (let click = 0; click < 3; click++) {
			circuit.beginGesture()
			circuit.endGesture()
		}

		circuit.undo()

		expect(positionOf(circuit, resistor)).toEqual(start)
	})
})

describe('deleting', () => {
	it('removes the component and the wires left dangling', async () => {
		const { circuit, resistor } = await milestone()
		expect(circuit.netCount).toBe(3)

		circuit.remove(resistor)

		expect(circuit.size).toBe(2)
		expect(circuit.netCount).toBe(1)
	})

	it('can be undone, wires and all', async () => {
		const { circuit, resistor } = await milestone()

		circuit.remove(resistor)
		circuit.undo()

		expect(circuit.size).toBe(3)
		expect(circuit.netCount).toBe(3)
		circuit.advance(0.016)
		expect(circuit.state().failure).toBeUndefined()
	})
})

describe('undo and redo', () => {
	it('knows whether there is anywhere to go', async () => {
		const circuit = await createCircuit()
		expect(circuit.canUndo).toBe(false)
		expect(circuit.canRedo).toBe(false)

		circuit.add('Resistor', 0, 0)
		expect(circuit.canUndo).toBe(true)

		circuit.undo()
		expect(circuit.canUndo).toBe(false)
		expect(circuit.canRedo).toBe(true)
	})

	it('walks back and forward through a session', async () => {
		const circuit = await createCircuit()
		circuit.add('DcSource', 0, 0)
		circuit.add('Resistor', 40, 0)
		circuit.add('Led', 80, 0)

		circuit.undo()
		circuit.undo()
		expect(circuit.size).toBe(1)

		circuit.redo()
		circuit.redo()
		expect(circuit.size).toBe(3)
	})

	it('restores a circuit that still solves', async () => {
		// A half-restored netlist would load without complaint and then fail to
		// solve, which is the failure this is really checking for.
		const { circuit, led } = await milestone()
		circuit.advance(0.016)
		const before = circuit.state().elementCurrents.length

		circuit.remove(led)
		circuit.undo()
		circuit.advance(0.016)

		expect(circuit.state().failure).toBeUndefined()
		expect(circuit.state().elementCurrents).toHaveLength(before)
	})
})

describe('deleting a wire', () => {
	it('removes the wire and leaves the components alone', async () => {
		// The finding behind C-41: until now the only way to be rid of a wire
		// was to delete a component that was perfectly fine.
		const { circuit } = await milestone()
		const [wire] = circuit.nets()

		circuit.removeNet(wire.id)

		expect(circuit.nets()).toHaveLength(2)
		expect(circuit.elements()).toHaveLength(3)
	})

	it('stops the circuit without breaking it', async () => {
		// An open circuit is a circuit, not an error. Cutting one wire of the
		// milestone loop should put the LED out, not produce a failure.
		const { circuit } = await milestone()
		const [wire] = circuit.nets()

		circuit.removeNet(wire.id)
		circuit.advance(0.1)

		const solution = circuit.state()
		// `failure` first: an empty list of currents would satisfy the second
		// assertion on its own, and an unsolved circuit reports exactly that.
		expect(solution.failure).toBeUndefined()
		expect(solution.elementCurrents).not.toHaveLength(0)
		expect(solution.elementCurrents.every((current) => Math.abs(current.amps) < 1e-9)).toBe(true)
	})

	it('can be undone', async () => {
		const { circuit } = await milestone()
		const [wire] = circuit.nets()
		circuit.removeNet(wire.id)

		circuit.undo()

		expect(circuit.nets()).toHaveLength(3)
	})

	it('frees the pins it was on', async () => {
		const { circuit, battery, resistor } = await milestone()
		const wire = circuit
			.nets()
			.find((net) => net.pins.some((pin) => pin.elementId === battery && pin.pin === '+'))!

		circuit.removeNet(wire.id)
		circuit.connect(battery, '+', resistor, 'a')

		expect(circuit.nets()).toHaveLength(3)
	})
})

describe('turning a component', () => {
	it('takes its wires with it', async () => {
		// The wire is drawn from where the pins are, so turning the component
		// re-routes it without anything having to tell the wire.
		const { circuit, resistor } = await milestone()
		const before = toWires(circuit.nets(), toScene(circuit.elements()))

		circuit.rotate(resistor)

		const after = toWires(circuit.nets(), toScene(circuit.elements()))
		expect(after).toHaveLength(before.length)
		expect(after).not.toEqual(before)
	})

	it('keeps the circuit running', async () => {
		// Which way a component faces changes the drawing, not the solve.
		const { circuit, resistor } = await milestone()
		circuit.advance(0.1)
		const before = circuit.state()

		circuit.rotate(resistor)
		circuit.advance(0.1)

		const after = circuit.state()
		expect(after.failure).toBeUndefined()
		expect(after.time).toBeGreaterThan(before.time)
	})

	it('reports which way each component now faces', async () => {
		const { circuit, resistor } = await milestone()

		circuit.rotate(resistor)

		const turned = circuit.elements().find((element) => element.id === resistor)
		expect(turned?.turns).toBe(1)
		expect(circuit.elements().filter((element) => element.turns !== 0)).toHaveLength(1)
	})

	it('can be undone', async () => {
		const { circuit, resistor } = await milestone()
		circuit.rotate(resistor)

		circuit.undo()

		expect(circuit.elements().find((element) => element.id === resistor)?.turns).toBe(0)
	})

	it('leaves a turned component still wireable', async () => {
		// The pins moved. If the netlist and the drawing disagreed about where
		// they are, this is where it would show.
		const circuit = await createCircuit()
		const battery = circuit.add('DcSource', 0, 0)
		const resistor = circuit.add('Resistor', 80, 0)
		circuit.rotate(resistor)

		circuit.connect(battery, '+', resistor, 'a')

		const [wire] = toWires(circuit.nets(), toScene(circuit.elements()))
		expect(wire.segments.length).toBeGreaterThan(0)
	})
})

describe('bending a wire', () => {
	it('routes through the bend, orthogonally', async () => {
		const { circuit } = await milestone()
		const [wire] = circuit.nets()

		circuit.setWaypoints(wire.id, [{ x: 40, y: -80 }])

		const [drawn] = toWires(circuit.nets(), toScene(circuit.elements()))
		expect(drawn.waypoints).toEqual([{ x: 40, y: -80 }])
		expect(
			drawn.segments.every(
				(segment) => segment.from.x === segment.to.x || segment.from.y === segment.to.y,
			),
		).toBe(true)
	})

	it('keeps the bend where it was put when a component moves', async () => {
		// The asymmetry of ADR 0007: the interior is the user's, the runs to the
		// pins are re-derived.
		const { circuit, battery } = await milestone()
		const [wire] = circuit.nets()
		circuit.setWaypoints(wire.id, [{ x: 40, y: -80 }])

		circuit.moveElement(battery, 0, 200)

		const [drawn] = toWires(circuit.nets(), toScene(circuit.elements()))
		expect(drawn.waypoints).toEqual([{ x: 40, y: -80 }])
		expect(drawn.segments[0].from).toEqual({ x: -20, y: 200 })
	})

	it('leaves the simulation alone', async () => {
		const { circuit } = await milestone()
		const [wire] = circuit.nets()
		circuit.advance(0.1)
		const before = circuit.state()

		circuit.setWaypoints(wire.id, [{ x: 40, y: -80 }])
		circuit.advance(0.1)

		const after = circuit.state()
		expect(after.failure).toBeUndefined()
		expect(after.time).toBeGreaterThan(before.time)
	})

	it('can be undone', async () => {
		const { circuit } = await milestone()
		const [wire] = circuit.nets()

		circuit.setWaypoints(wire.id, [{ x: 40, y: -80 }])
		circuit.undo()

		expect(circuit.nets().find((net) => net.id === wire.id)?.waypoints).toEqual([])
	})
})
