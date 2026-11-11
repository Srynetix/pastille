// SPDX-License-Identifier: MIT OR Apache-2.0

import { describe, expect, it } from 'vitest'

import { createCircuit } from './core'
import { clickPosition, dropPosition, moveDrag, startDrag } from './placement'
import { toScene } from './scene'
import { halfSizeOf, SYMBOLS } from './symbols'
import { GRID_SPACING, INITIAL_VIEWPORT } from './viewport'

/** A canvas occupying the middle of the window. */
const CANVAS = new DOMRect(200, 100, 600, 400)

describe('dragging a component out of the palette', () => {
	it('knows when the pointer is over the canvas', () => {
		let drag = startDrag('Resistor', { x: 10, y: 10 })
		expect(drag.overCanvas).toBe(false)

		drag = moveDrag(drag, { x: 400, y: 300 }, CANVAS)
		expect(drag.overCanvas).toBe(true)

		drag = moveDrag(drag, { x: 50, y: 300 }, CANVAS)
		expect(drag.overCanvas).toBe(false)
	})

	it('drops onto a grid intersection', () => {
		const drag = moveDrag(startDrag('Led', { x: 0, y: 0 }), { x: 413, y: 247 }, CANVAS)

		const at = dropPosition(drag, CANVAS, INITIAL_VIEWPORT)

		expect(at).toBeDefined()
		expect(Math.abs(at!.x % GRID_SPACING)).toBe(0)
		expect(Math.abs(at!.y % GRID_SPACING)).toBe(0)
	})

	it('drops where the pointer is, whatever the camera is doing', () => {
		const pointer = { x: 500, y: 300 }
		const drag = moveDrag(startDrag('Resistor', { x: 0, y: 0 }), pointer, CANVAS)

		for (const viewport of [
			INITIAL_VIEWPORT,
			{ pan: { x: 120, y: -40 }, zoom: 1 },
			{ pan: { x: -30, y: 75 }, zoom: 3 },
		]) {
			const at = dropPosition(drag, CANVAS, viewport)!
			// The dropped point, converted back, is within half a grid cell of the
			// pointer — the difference being the snap.
			const back = {
				x: (at.x - viewport.pan.x) * viewport.zoom + CANVAS.left,
				y: (at.y - viewport.pan.y) * viewport.zoom + CANVAS.top,
			}
			const slack = (GRID_SPACING * viewport.zoom) / 2
			expect(Math.abs(back.x - pointer.x)).toBeLessThanOrEqual(slack)
			expect(Math.abs(back.y - pointer.y)).toBeLessThanOrEqual(slack)
		}
	})

	it('has nowhere to drop outside the canvas', () => {
		const drag = moveDrag(startDrag('Resistor', { x: 0, y: 0 }), { x: 40, y: 40 }, CANVAS)

		expect(dropPosition(drag, CANVAS, INITIAL_VIEWPORT)).toBeUndefined()
	})

	it('has nowhere to drop when there is no canvas at all', () => {
		const drag = startDrag('Resistor', { x: 400, y: 300 })

		expect(dropPosition(drag, undefined, INITIAL_VIEWPORT)).toBeUndefined()
	})
})

describe('placing into the circuit', () => {
	it('creates the element the drag was carrying', async () => {
		const circuit = await createCircuit()
		const drag = moveDrag(startDrag('Led', { x: 0, y: 0 }), { x: 400, y: 300 }, CANVAS)
		const at = dropPosition(drag, CANVAS, INITIAL_VIEWPORT)!

		circuit.add(drag.kind, at.x, at.y)

		const elements = circuit.elements()
		expect(elements).toHaveLength(1)
		expect(elements[0]).toMatchObject({ kind: 'Led', x: at.x, y: at.y })
	})

	it('leaves the circuit untouched when the drop is cancelled', async () => {
		// The acceptance. Nothing is created on drag start, so a cancelled drop
		// has nothing to undo.
		const circuit = await createCircuit()
		const drag = moveDrag(startDrag('Resistor', { x: 0, y: 0 }), { x: 20, y: 20 }, CANVAS)

		const at = dropPosition(drag, CANVAS, INITIAL_VIEWPORT)
		if (at) circuit.add(drag.kind, at.x, at.y)

		expect(circuit.size).toBe(0)
		expect(circuit.elements()).toEqual([])
	})

	it('renders what the core says, not what the app remembers', async () => {
		const circuit = await createCircuit()
		circuit.add('DcSource', 20, 30)
		circuit.add('Resistor', 60, 30)

		const scene = toScene(circuit.elements())

		expect(scene).toHaveLength(2)
		expect(scene[0].symbol).toBe(SYMBOLS.DcSource)
		expect(scene[1].symbol).toBe(SYMBOLS.Resistor)
		expect(scene[0].position).toEqual({ x: 20, y: 30 })
	})

	it('gives each placed component a readable label', async () => {
		const circuit = await createCircuit()
		circuit.add('DcSource', 0, 0)
		circuit.add('Resistor', 40, 0)

		expect(toScene(circuit.elements()).map((item) => item.label)).toEqual(['9 V', '470 Ω'])
	})

	it('refuses a kind the core does not know', async () => {
		const circuit = await createCircuit()

		expect(() => circuit.add('Flux capacitor', 0, 0)).toThrow(/unknown element kind/)
		expect(circuit.size).toBe(0)
	})

	it('hands out handles that survive other elements being removed', async () => {
		const circuit = await createCircuit()
		const first = circuit.add('Resistor', 0, 0)
		const second = circuit.add('Led', 40, 0)

		circuit.remove(first)

		expect(circuit.size).toBe(1)
		expect(circuit.elements()[0].id).toBe(second)
		expect(() => circuit.remove(first)).toThrow()
	})
})

describe('placing a component by clicking it in the palette (C-97)', () => {
	const size = { width: 600, height: 400 }

	it('lands in the middle of the view, on the grid', () => {
		const at = clickPosition('Resistor', size, INITIAL_VIEWPORT, [])
		expect(at).toEqual({ x: 300, y: 200 })
	})

	it('follows the view when it has been panned and zoomed', () => {
		const at = clickPosition('Resistor', size, { pan: { x: 1000, y: -500 }, zoom: 2 }, [])
		expect(at).toEqual({ x: 1150, y: -400 })
	})

	it('never lands on something already there', () => {
		const half = halfSizeOf(SYMBOLS.Resistor)
		const occupants = [{ position: { x: 300, y: 200 }, half }]
		const at = clickPosition('Resistor', size, INITIAL_VIEWPORT, occupants)
		expect(at).not.toEqual({ x: 300, y: 200 })
		const apart =
			Math.abs(at.x - 300) >= 2 * half.x + 2 * GRID_SPACING ||
			Math.abs(at.y - 200) >= 2 * half.y + 2 * GRID_SPACING
		expect(apart, `${at.x}, ${at.y}`).toBe(true)
		expect(at.x % GRID_SPACING === 0 && at.y % GRID_SPACING === 0).toBe(true)
	})

	it('puts a second and a third click beside the first, not on it', () => {
		const placed: { position: { x: number; y: number }; half: { x: number; y: number } }[] = []
		for (let click = 0; click < 3; click++) {
			const position = clickPosition('Led', size, INITIAL_VIEWPORT, placed)
			placed.push({ position, half: halfSizeOf(SYMBOLS.Led) })
		}
		const spots = new Set(placed.map(({ position }) => `${position.x},${position.y}`))
		expect(spots.size).toBe(3)
	})
})
