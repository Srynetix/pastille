// SPDX-License-Identifier: MIT OR Apache-2.0

import { describe, expect, it } from 'vitest'

import { componentRegistry, pinNames } from './core'
import { drawScene, type Theme } from './render'
import {
	boundsOf,
	hitTest,
	itemsIn,
	PIN_PICK_RADIUS,
	pinPosition,
	pinRadiusFor,
	type SceneItem,
	type SceneWire,
} from './scene'
import { describe as describeComponent, PALETTE } from './palette'
import {
	DRAWN_BY_CHOICE,
	halfSizeOf,
	labelSide,
	symbolFor,
	SYMBOLS,
	turned,
	type SymbolDefinition,
	type SymbolKind,
} from './symbols'
import {
	GRID_SPACING,
	fitTo,
	INITIAL_VIEWPORT,
	MAX_ZOOM,
	MIN_ZOOM,
	panBy,
	snapToGrid,
	toGrid,
	toScreen,
	toWorld,
	zoomAt,
	type Point,
} from './viewport'

const THEME: Theme = {
	grid: '#333',
	wire: '#9cf',
	current: '#4d4',
	selection: '#7af',
	body: '#222',
	outline: '#888',
	pin: '#eee',
	text: '#eee',
	font: 'system-ui, sans-serif',
}

function item(id: string, position: Point): SceneItem {
	return { id, label: id, position, symbol: SYMBOLS.Resistor }
}

describe('viewport', () => {
	it('round-trips a point through screen and back', () => {
		const viewport = { pan: { x: 37, y: -12 }, zoom: 2.5 }
		const world = { x: 123.5, y: -47.25 }

		const back = toWorld(viewport, toScreen(viewport, world))

		expect(back.x).toBeCloseTo(world.x, 10)
		expect(back.y).toBeCloseTo(world.y, 10)
	})

	it('maps the pointer to grid coordinates at any zoom', () => {
		// The acceptance: the same screen pixel means different grid cells at
		// different zooms, and both answers have to be right.
		for (const zoom of [MIN_ZOOM, 0.5, 1, 2.5, MAX_ZOOM]) {
			const viewport = { pan: { x: 0, y: 0 }, zoom }
			const screen = { x: 3 * GRID_SPACING * zoom, y: 2 * GRID_SPACING * zoom }

			expect(toGrid(toWorld(viewport, screen)), `zoom ${zoom}`).toEqual({ x: 3, y: 2 })
		}
	})

	it('keeps the point under the cursor fixed while zooming', () => {
		// The difference between a magnifier and the drawing running away.
		const cursor = { x: 240, y: 130 }
		let viewport = { pan: { x: 11, y: -5 }, zoom: 1 }
		const anchored = toWorld(viewport, cursor)

		for (const factor of [1.2, 1.2, 0.5, 3]) {
			viewport = zoomAt(viewport, cursor, factor)
			const now = toWorld(viewport, cursor)
			expect(now.x).toBeCloseTo(anchored.x, 8)
			expect(now.y).toBeCloseTo(anchored.y, 8)
		}
	})

	it('refuses to zoom past its bounds', () => {
		let viewport = INITIAL_VIEWPORT
		for (let i = 0; i < 40; i++) viewport = zoomAt(viewport, { x: 0, y: 0 }, 2)
		expect(viewport.zoom).toBe(MAX_ZOOM)

		for (let i = 0; i < 80; i++) viewport = zoomAt(viewport, { x: 0, y: 0 }, 0.5)
		expect(viewport.zoom).toBe(MIN_ZOOM)
	})

	it('pans by screen pixels regardless of zoom', () => {
		// Dragging 100px should move the drawing 100px, whatever the zoom.
		for (const zoom of [0.5, 1, 4]) {
			const before = { pan: { x: 0, y: 0 }, zoom }
			const world = { x: 20, y: 20 }
			const at = toScreen(before, world)

			const after = panBy(before, 100, -40)
			const moved = toScreen(after, world)

			expect(moved.x - at.x).toBeCloseTo(100, 8)
			expect(moved.y - at.y).toBeCloseTo(-40, 8)
		}
	})

	it('snaps to the nearest grid intersection', () => {
		expect(snapToGrid({ x: 14, y: -4 })).toEqual({ x: 10, y: -0 })
		expect(snapToGrid({ x: 16, y: -6 })).toEqual({ x: 20, y: -10 })
	})
})

describe('fitting the view to a circuit', () => {
	// C-47. A circuit that arrives from a file rather than from the pointer has
	// to be brought into view, or it opens off screen onto what looks like an
	// empty canvas — which is what C-29's Open… did, and what a bundled example
	// would have done every time.
	const screen = { width: 800, height: 600 }

	it('brings a circuit drawn far from the origin into view', () => {
		const far = { min: { x: 5000, y: 3000 }, max: { x: 5400, y: 3200 } }

		const viewport = fitTo(far, screen)

		for (const corner of [far.min, far.max]) {
			const at = toScreen(viewport, corner)
			expect(at.x).toBeGreaterThanOrEqual(0)
			expect(at.x).toBeLessThanOrEqual(screen.width)
			expect(at.y).toBeGreaterThanOrEqual(0)
			expect(at.y).toBeLessThanOrEqual(screen.height)
		}
	})

	it('centres what it was given', () => {
		const bounds = { min: { x: -100, y: -50 }, max: { x: 100, y: 50 } }

		const middle = toScreen(fitTo(bounds, screen), { x: 0, y: 0 })

		expect(middle.x).toBeCloseTo(screen.width / 2, 6)
		expect(middle.y).toBeCloseTo(screen.height / 2, 6)
	})

	it('does not magnify a small circuit', () => {
		// "Let me see it" is not "make it as big as it will go": three
		// components blown up to fill an eight-hundred-pixel canvas arrive
		// unrecognisable, and every symbol is drawn for 1:1.
		const small = { min: { x: 0, y: 0 }, max: { x: 40, y: 0 } }

		expect(fitTo(small, screen).zoom).toBe(1)
	})

	it('does not shrink past what can be read', () => {
		const enormous = { min: { x: 0, y: 0 }, max: { x: 100_000, y: 80_000 } }

		expect(fitTo(enormous, screen).zoom).toBe(MIN_ZOOM)
	})

	it('leaves room to wire to the outermost pins', () => {
		// Fitted flush, the pins on the edge would have nothing to drag a wire
		// to. The margin is a pin's own reach, which is the smallest gesture
		// that has to fit.
		const bounds = { min: { x: 0, y: 0 }, max: { x: 400, y: 300 } }

		const viewport = fitTo(bounds, screen)
		const corner = toScreen(viewport, bounds.min)

		expect(corner.x).toBeGreaterThan(0)
		expect(corner.y).toBeGreaterThan(0)
	})

	it('says nothing rather than guessing when there is no canvas yet', () => {
		// The size arrives from a resize observer a frame or two after mount,
		// and a fit computed against zero would divide by it.
		const bounds = { min: { x: 0, y: 0 }, max: { x: 100, y: 100 } }

		expect(fitTo(bounds, { width: 0, height: 0 })).toEqual(INITIAL_VIEWPORT)
	})
})

describe('where a label goes', () => {
	it('never on a side a wire leaves from', () => {
		// A label below a pin that points down is written on the wire that
		// leaves it. Every symbol, every quarter turn: whatever side is chosen,
		// no pin points that way.
		for (const [kind, symbol] of Object.entries(SYMBOLS)) {
			// A chip's name is written inside it, where no wire runs (C-77).
			if (symbol.labelInside) continue
			for (const turns of [0, 1, 2, 3]) {
				const side = labelSide(symbol, turns)
				for (const pin of symbol.pins) {
					const at = turned(pin.at, turns)
					const pointing =
						Math.abs(at.y) > Math.abs(at.x)
							? at.y > 0
								? 'below'
								: 'above'
							: at.x > 0
								? 'right'
								: 'left'
					expect(pointing, `${kind} turned ${turns}: ${pin.name}`).not.toBe(side)
					// A corner is taken by a pin that is off both axes in it.
					if (at.x !== 0 && at.y !== 0) {
						const corner = `${at.y > 0 ? 'below' : 'above'}-${at.x > 0 ? 'right' : 'left'}`
						expect(corner, `${kind} turned ${turns}: ${pin.name}`).not.toBe(side)
					}
				}
			}
		}
	})

	it('stays below wherever it can, as it always has', () => {
		// Only a symbol with a pin underneath moves its label, so the
		// two-terminal parts read exactly as before.
		expect(labelSide(SYMBOLS.Resistor)).toBe('below')
		expect(labelSide(SYMBOLS.DcSource)).toBe('below')
		// The regulator's ground lead points down, so its part number goes on
		// top (C-54); the transistor has pins up and down, so beside it.
		expect(labelSide(SYMBOLS.Regulator)).toBe('above')
		expect(labelSide(SYMBOLS.Bjt)).toBe('right')
		// An op-amp has a pin on every side, so its part number goes in the
		// corner the inputs are not in (C-55).
		expect(labelSide(SYMBOLS.OpAmp)).toBe('below-right')
	})
})

describe('the extent of what is drawn', () => {
	it('covers every symbol, not just the points they sit on', () => {
		// The bounds have to include what each symbol draws around its origin,
		// or a circuit fitted to them has its outermost components cut in half.
		const scene = [
			{ id: 'a', position: { x: 0, y: 0 }, symbol: SYMBOLS.Resistor },
			{ id: 'b', position: { x: 100, y: 0 }, symbol: SYMBOLS.Resistor },
		]

		const bounds = boundsOf(scene)!

		expect(bounds.min.x).toBeLessThan(0)
		expect(bounds.max.x).toBeGreaterThan(100)
	})

	it('has nothing to say about an empty canvas', () => {
		expect(boundsOf([])).toBeUndefined()
	})
})

describe('selecting an area', () => {
	const scene = [
		{ id: 'left', position: { x: 0, y: 0 }, symbol: SYMBOLS.Resistor },
		{ id: 'right', position: { x: 200, y: 0 }, symbol: SYMBOLS.Resistor },
		{ id: 'below', position: { x: 0, y: 200 }, symbol: SYMBOLS.Resistor },
	]

	it('takes what the rectangle touches, not only what it encloses', () => {
		// A rectangle from the middle of the first resistor to just short of
		// the second takes the first alone: half of it is enough.
		expect(itemsIn(scene, { min: { x: 0, y: -10 }, max: { x: 120, y: 10 } })).toEqual(['left'])
	})

	it('takes everything it spans', () => {
		expect(itemsIn(scene, { min: { x: -50, y: -50 }, max: { x: 250, y: 250 } })).toEqual([
			'left',
			'right',
			'below',
		])
	})

	it('is the same rectangle whichever way it was dragged', () => {
		const forwards = itemsIn(scene, { min: { x: -50, y: -50 }, max: { x: 50, y: 250 } })
		const backwards = itemsIn(scene, { min: { x: 50, y: 250 }, max: { x: -50, y: -50 } })

		expect(backwards).toEqual(forwards)
		expect(forwards).toEqual(['left', 'below'])
	})

	it('takes nothing from empty grid', () => {
		expect(itemsIn(scene, { min: { x: 90, y: 90 }, max: { x: 110, y: 110 } })).toEqual([])
	})
})

describe('hit testing', () => {
	it('finds the element under the point', () => {
		const items = [item('r1', { x: 100, y: 100 })]

		expect(hitTest(items, { x: 100, y: 100 })).toMatchObject({ kind: 'element' })
		expect(hitTest(items, { x: 100, y: 200 })).toBeUndefined()
	})

	it('prefers a pin to the body it sits on', () => {
		// Pins sit on the body's edge, so every pin is also inside the body. The
		// other order would make wiring impossible.
		const subject = item('r1', { x: 100, y: 100 })
		const onPin = pinPosition(subject, subject.symbol.pins[0])

		const hit = hitTest([subject], onPin)

		expect(hit?.kind).toBe('pin')
		expect(hit).toMatchObject({ pin: { name: 'a' } })
	})

	it('has a pin tolerance a pointer can actually hit', () => {
		const subject = item('r1', { x: 100, y: 100 })
		const onPin = pinPosition(subject, subject.symbol.pins[1])

		const nearby = { x: onPin.x + PIN_PICK_RADIUS * 0.9, y: onPin.y }
		expect(hitTest([subject], nearby)?.kind).toBe('pin')
	})

	it('prefers a pin on a lower item to the body of one above it', () => {
		// Overlapping components are normal while dragging, and the pin still has
		// to be reachable.
		const lower = item('r1', { x: 100, y: 100 })
		const upper = item('r2', { x: 118, y: 100 })
		const pin = pinPosition(lower, lower.symbol.pins[1])

		expect(hitTest([lower, upper], pin)).toMatchObject({
			kind: 'pin',
			item: { id: 'r1' },
		})
	})

	it('keeps the pin tolerance the same size under the hand at any zoom', () => {
		// In world units a pin shrinks with the drawing and becomes impossible to
		// click when zoomed out. The tolerance is a screen distance, converted.
		const subject = item('r1', { x: 100, y: 100 })
		const onPin = pinPosition(subject, subject.symbol.pins[0])

		for (const zoom of [0.25, 1, 4]) {
			const radius = pinRadiusFor(zoom)
			// Just inside the tolerance, measured on screen, at every zoom.
			const nearby = { x: onPin.x + (PIN_PICK_RADIUS * 0.9) / zoom, y: onPin.y }
			expect(hitTest([subject], nearby, radius)?.kind, `zoom ${zoom}`).toBe('pin')

			// And just outside it is not a pin at any zoom either. Measured
			// perpendicular to the leads: along them, a generous tolerance at low
			// zoom legitimately reaches the pin at the other end.
			const far = { x: onPin.x, y: onPin.y + (PIN_PICK_RADIUS * 1.5) / zoom }
			expect(hitTest([subject], far, radius)?.kind, `zoom ${zoom}`).not.toBe('pin')
		}
	})

	it('returns the topmost element when bodies overlap', () => {
		const items = [item('r1', { x: 100, y: 100 }), item('r2', { x: 105, y: 100 })]

		expect(hitTest(items, { x: 103, y: 100 })).toMatchObject({ item: { id: 'r2' } })
	})
})

describe('rendering', () => {
	function context(width = 900, height = 600) {
		const canvas = document.createElement('canvas')
		canvas.width = width
		canvas.height = height
		const ctx = canvas.getContext('2d')
		if (!ctx) throw new Error('no 2d context')
		return { ctx, size: { width, height } }
	}

	// Timed against the wall clock, so as much about the machine as the code:
	// behind the same flag as the C-15 bench (C-107), which `just ci` sets.
	// A plain test run on a laptop on battery does not fail on them; the gate
	// still holds them.
	const timed = import.meta.env.VITE_BENCH === '1' ? it : it.skip

	timed('draws a hundred elements inside a frame budget', () => {
		// The acceptance: 60fps with 100 elements. Measured over enough frames
		// that one unlucky garbage collection cannot decide the result.
		const { ctx, size } = context()
		const items = Array.from({ length: 100 }, (_, index) =>
			item(`r${index}`, { x: (index % 10) * 60 + 40, y: Math.floor(index / 10) * 50 + 40 }),
		)
		const viewport = { pan: { x: 0, y: 0 }, zoom: 1 }

		// Warm up, so the first frame's compilation is not in the measurement.
		for (let i = 0; i < 10; i++) drawScene(ctx, items, viewport, THEME, size)

		const frames = 60
		const started = performance.now()
		for (let i = 0; i < frames; i++) drawScene(ctx, items, viewport, THEME, size)
		const perFrame = (performance.now() - started) / frames

		// 16.7ms is the whole budget; the scene may not eat all of it, because
		// the browser still has to composite and Svelte still has to react.
		expect(perFrame, `${perFrame.toFixed(2)}ms per frame`).toBeLessThan(8)
	})

	timed('stays cheap when zoomed far out', () => {
		// Zoomed out, a naive grid draws the whole plane. This asserts the grid
		// is clipped to the viewport rather than to the canvas.
		const { ctx, size } = context()
		const items = [item('r1', { x: 0, y: 0 })]

		const started = performance.now()
		for (let i = 0; i < 60; i++) {
			drawScene(ctx, items, { pan: { x: -5000, y: -5000 }, zoom: MIN_ZOOM }, THEME, size)
		}
		const perFrame = (performance.now() - started) / 60

		expect(perFrame, `${perFrame.toFixed(2)}ms per frame`).toBeLessThan(8)
	})

	it('draws something', () => {
		// A frame budget met by drawing nothing is not a frame budget.
		const { ctx, size } = context(200, 120)
		drawScene(ctx, [item('r1', { x: 100, y: 60 })], INITIAL_VIEWPORT, THEME, size)

		const pixels = ctx.getImageData(0, 0, size.width, size.height).data
		expect(pixels.some((channel) => channel !== 0)).toBe(true)
	})
})

describe('symbols', () => {
	it('names its pins exactly as the core does', async () => {
		// The cross-check that matters: the app draws the pin and the core wires
		// it, and "Anode" against "anode" would connect nothing while looking
		// perfectly fine on screen.
		for (const kind of Object.keys(SYMBOLS) as (keyof typeof SYMBOLS)[]) {
			const fromCore = await pinNames(kind)
			const fromSymbol = SYMBOLS[kind].pins.map((pin) => pin.name)

			expect(fromSymbol, kind).toEqual(fromCore)
		}
	})

	it('names the pins of every drawing a kind has, not only its first (C-105)', async () => {
		// A choice's drawing and a state's are built beside the base one, and
		// agreed with the core only because they happened to be built alike.
		const drawings: [string, SymbolKind, SymbolDefinition][] = [
			...Object.entries(DRAWN_BY_CHOICE).flatMap(([kind, byChoice]) =>
				Object.entries(byChoice).map(
					([choice, drawing]) =>
						[`${kind} ${choice}`, kind as SymbolKind, drawing] as [
							string,
							SymbolKind,
							SymbolDefinition,
						],
				),
			),
			['Potentiometer at 0.3', 'Potentiometer', symbolFor('Potentiometer', { wiper: 0.3 })],
			['Servo at 45°', 'Servo', symbolFor('Servo', { angle: 45 })],
			['Motor turning', 'Motor', symbolFor('Motor', { closed: true })],
			['Motor backwards', 'Motor', symbolFor('Motor', { closed: true, reversed: true })],
			['Switch shut', 'Switch', symbolFor('Switch', { closed: true })],
			['Relay energised', 'Relay', symbolFor('Relay', { closed: true })],
			['TiltSwitch upright', 'TiltSwitch', symbolFor('TiltSwitch', { closed: true })],
		]
		for (const [name, kind, drawing] of drawings) {
			expect(
				drawing.pins.map((pin) => pin.name),
				name,
			).toEqual(await pinNames(kind))
		}
	})

	it('puts every pin on a grid intersection', () => {
		// Orthogonal wire routing in M-17 assumes this, and it is much harder to
		// retrofit than to decide now.
		for (const [kind, symbol] of Object.entries(SYMBOLS)) {
			for (const pin of symbol.pins) {
				// Math.abs because -20 % 10 is -0, which Object.is separates from 0.
				expect(Math.abs(pin.at.x % GRID_SPACING), `${kind}.${pin.name}`).toBe(0)
				expect(Math.abs(pin.at.y % GRID_SPACING), `${kind}.${pin.name}`).toBe(0)
			}
		}
	})

	it('derives a hit box that contains everything it draws', () => {
		for (const [kind, symbol] of Object.entries(SYMBOLS)) {
			const halfSize = halfSizeOf(symbol)

			for (const shape of symbol.shapes) {
				if (shape.kind === 'circle') continue
				for (const point of shape.points) {
					expect(Math.abs(point.x), `${kind} x`).toBeLessThanOrEqual(halfSize.x)
					expect(Math.abs(point.y), `${kind} y`).toBeLessThanOrEqual(halfSize.y)
				}
			}
		}
	})

	// C-28's acceptance: every component of §5 has a symbol, and the set of them
	// is consistent rather than eleven separate drawings that happen to coexist.
	// Every state counts, not just the one in the table — a switch shut, a
	// button held, a wiper anywhere along its track, a transistor either
	// polarity.
	const everyDrawing: [string, SymbolDefinition][] = [
		...Object.entries(SYMBOLS),
		['Switch shut', symbolFor('Switch', { closed: true })],
		['PushButton held', symbolFor('PushButton', { closed: true })],
		['Relay energised', symbolFor('Relay', { closed: true })],
		['Buzzer sounding', symbolFor('Buzzer', { closed: true })],
		['PassiveBuzzer sounding', symbolFor('PassiveBuzzer', { closed: true })],
		['Motor turning', symbolFor('Motor', { closed: true })],
		['Motor turning backwards', symbolFor('Motor', { closed: true, reversed: true })],
		...[0, 45, 180].map(
			(angle) =>
				[`Servo at ${angle}°`, symbolFor('Servo', { angle })] as [string, SymbolDefinition],
		),
		...[0, 0.25, 1].map(
			(wiper) =>
				[`Potentiometer at ${wiper}`, symbolFor('Potentiometer', { wiper })] as [
					string,
					SymbolDefinition,
				],
		),
		['Bjt Npn', symbolFor('Bjt', { choice: 'Npn' })],
		['Bjt Pnp', symbolFor('Bjt', { choice: 'Pnp' })],
		['Mosfet N', symbolFor('Mosfet', { choice: 'N-channel' })],
		['Mosfet P', symbolFor('Mosfet', { choice: 'P-channel' })],
		...['Square', 'Sine', 'Triangle'].map(
			(choice) =>
				[`SignalGenerator ${choice}`, symbolFor('SignalGenerator', { choice })] as [
					string,
					SymbolDefinition,
				],
		),
	]

	it('spells a choice the way the core spells it', async () => {
		// The bug this exists for was live for ten minutes and looked like
		// nothing: the table was keyed by `NPN`, the label written *beside* the
		// symbol, while the core hands the app `Npn`. Nothing threw — every
		// transistor simply came out an NPN with the right word underneath it.
		//
		// So it is checked both ways, as the LED's colours are: a kind that
		// draws by its choice covers every choice the core offers, and offers
		// no drawing for a choice the core has never heard of.
		const registry = await componentRegistry()

		for (const [kind, drawings] of Object.entries(DRAWN_BY_CHOICE)) {
			const component = registry.find((entry) => entry.kind === kind)
			expect(component, `${kind} is a kind the core knows`).toBeDefined()

			expect(Object.keys(drawings).sort(), kind).toEqual([...component!.choices].sort())
		}
	})

	it('draws a different picture for each choice it distinguishes', () => {
		// And they are genuinely different drawings. A table whose entries were
		// the same object would pass the spelling check above while saying
		// nothing about the part.
		for (const [kind, drawings] of Object.entries(DRAWN_BY_CHOICE)) {
			const shapes = Object.values(drawings).map((drawing) => JSON.stringify(drawing.shapes))

			expect(new Set(shapes).size, kind).toBe(shapes.length)
		}
	})

	it('puts every pin two cells out, on an intersection', () => {
		// Consistent pin spacing, which is what makes every component the same
		// size to wire and keeps a pin on an intersection through any quarter
		// turn. The pin is on the edge of the two-cell square, and both its
		// coordinates are whole cells, which a quarter turn only swaps and
		// negates — so it stays on an intersection however it is turned.
		//
		// This said "on an axis" until the op-amp (C-55), and for two to four
		// pins that is the same rule. Five pins cannot all be on an axis, and
		// the op-amp's inputs one cell either side of its centre keep the
		// property the rule was for.
		//
		// Two cells for everything but a chip with too many pins for that
		// square, which declares its own (C-77) and keeps the rule on it.
		for (const [name, symbol] of everyDrawing) {
			for (const pin of symbol.pins) {
				const reach = Math.max(Math.abs(pin.at.x), Math.abs(pin.at.y))
				expect(reach, `${name}.${pin.name}`).toBe(symbol.reach ?? 2 * GRID_SPACING)
				expect(Math.abs(pin.at.x) % GRID_SPACING, `${name}.${pin.name} x`).toBe(0)
				expect(Math.abs(pin.at.y) % GRID_SPACING, `${name}.${pin.name} y`).toBe(0)
			}
			// And no two in the same place, which a wire could not tell apart.
			const places = symbol.pins.map((pin) => `${pin.at.x},${pin.at.y}`)
			expect(new Set(places).size, name).toBe(places.length)
		}
	})

	it('draws nothing outside the square its pins reach to', () => {
		// So a component takes the footprint it claims. One that drew past its
		// pins would collide with whatever is placed beside it while the grid
		// said there was room.
		for (const [name, symbol] of everyDrawing) {
			const half = halfSizeOf(symbol)
			const reach = symbol.reach ?? 2 * GRID_SPACING

			expect(half.x, `${name} width`).toBeLessThanOrEqual(reach)
			expect(half.y, `${name} height`).toBeLessThanOrEqual(reach)
		}
	})

	it('keeps the pins still when the drawing changes', () => {
		// A switch flipped, a knob turned, a polarity chosen: the picture
		// changes and the wiring must not notice (C-19, C-20, C-46). A pin that
		// moved would drag a wire with it, or worse, not drag it.
		const states = [
			['Switch', { closed: false }, { closed: true }],
			['PushButton', { closed: false }, { closed: true }],
			['Relay', { closed: false }, { closed: true }],
			['Buzzer', { closed: false }, { closed: true }],
			['PassiveBuzzer', { closed: false }, { closed: true }],
			['Motor', { closed: false }, { closed: true }],
			['Motor', { closed: true }, { closed: true, reversed: true }],
			['Servo', { angle: 0 }, { angle: 180 }],
			['Potentiometer', { wiper: 0 }, { wiper: 1 }],
			['Bjt', { choice: 'Npn' }, { choice: 'Pnp' }],
			['Mosfet', { choice: 'N-channel' }, { choice: 'P-channel' }],
			['RgbLed', { choice: 'Common cathode' }, { choice: 'Common anode' }],
			['SignalGenerator', { choice: 'Square' }, { choice: 'Sine' }],
			['SignalGenerator', { choice: 'Sine' }, { choice: 'Triangle' }],
		] as const

		for (const [kind, one, other] of states) {
			expect(symbolFor(kind as SymbolKind, other).pins, kind).toEqual(
				symbolFor(kind as SymbolKind, one).pins,
			)
		}
	})

	it('draws every one of them with the same pen', () => {
		// Consistent stroke weight. The format carries no width — a symbol is
		// shapes and pins — so this holds by construction, and the test is here
		// to catch the day somebody decides one component should stand out.
		const widths = (symbol: SymbolDefinition): number[] => {
			const canvas = document.createElement('canvas')
			canvas.width = 120
			canvas.height = 120
			const context = canvas.getContext('2d')!
			const seen: number[] = []
			const spy = new Proxy(context, {
				set(target, property, value) {
					if (property === 'lineWidth') seen.push(value as number)
					return Reflect.set(target, property, value)
				},
				get(target, property) {
					const found: unknown = Reflect.get(target, property)
					return typeof found === 'function'
						? (found as (...args: unknown[]) => unknown).bind(target)
						: found
				},
			})

			drawScene(spy, [{ id: 'one', position: { x: 60, y: 60 }, symbol }], INITIAL_VIEWPORT, THEME, {
				width: 120,
				height: 120,
			})
			return [...new Set(seen)].sort((a, b) => a - b)
		}

		const reference = widths(SYMBOLS.Resistor)
		expect(reference.length, 'a symbol is drawn with at least one width').toBeGreaterThan(0)
		for (const [name, symbol] of everyDrawing) {
			expect(widths(symbol), name).toEqual(reference)
		}
	})

	it('draws from the description rather than from a branch per component', () => {
		// A symbol nobody has ever heard of renders too, which is what makes this
		// a format rather than three special cases.
		const invented: SymbolDefinition = {
			shapes: [
				{
					kind: 'polyline',
					points: [
						{ x: -20, y: 0 },
						{ x: 20, y: 0 },
					],
				},
				{ kind: 'circle', centre: { x: 0, y: 0 }, radius: 8 },
			],
			pins: [{ name: 'only', at: { x: -20, y: 0 } }],
		}

		const canvas = document.createElement('canvas')
		canvas.width = 200
		canvas.height = 120
		const context = canvas.getContext('2d')!

		drawScene(
			context,
			[{ id: 'x', position: { x: 100, y: 60 }, symbol: invented }],
			INITIAL_VIEWPORT,
			THEME,
			{ width: 200, height: 120 },
		)

		const pixels = context.getImageData(0, 0, 200, 120).data
		expect(pixels.some((channel) => channel !== 0)).toBe(true)
	})

	it('gives each component a distinguishable drawing', () => {
		// Three symbols that render identically would pass every other test here.
		const render = (symbol: SymbolDefinition) => {
			const canvas = document.createElement('canvas')
			canvas.width = 120
			canvas.height = 80
			const context = canvas.getContext('2d')!
			drawScene(
				context,
				[{ id: 'x', position: { x: 60, y: 40 }, symbol }],
				INITIAL_VIEWPORT,
				THEME,
				{ width: 120, height: 80 },
			)
			return context.getImageData(0, 0, 120, 80).data.join(',')
		}

		const drawings = Object.values(SYMBOLS).map(render)

		expect(new Set(drawings).size).toBe(drawings.length)
	})
})

describe('current and light', () => {
	function frame(items: SceneItem[], wires: SceneWire[], time: number, still = false) {
		const canvas = document.createElement('canvas')
		canvas.width = 300
		canvas.height = 200
		const context = canvas.getContext('2d')!
		const size = { width: 300, height: 200 }
		drawScene(
			context,
			items,
			INITIAL_VIEWPORT,
			THEME,
			size,
			wires,
			time,
			undefined,
			undefined,
			undefined,
			still,
		)
		return context.getImageData(0, 0, 300, 200).data.join(',')
	}

	const wire = (amps: number | undefined, forwards = true): SceneWire => {
		const run = { from: { x: 40, y: 100 }, to: { x: 260, y: 100 } }
		return {
			id: 'n1',
			waypoints: [],
			segments: [run],
			pins: [
				{ x: 40, y: 100 },
				{ x: 260, y: 100 },
			],
			// The current along the run, in the direction it goes — here the
			// whole wire, one way round or the other.
			flows:
				amps === undefined
					? undefined
					: [{ segments: [forwards ? run : { from: run.to, to: run.from }], amps }],
		}
	}

	it('animates a wire that carries current', () => {
		// The dots have to move, or the circuit reads as stopped.
		const early = frame([], [wire(0.02)], 0)
		const later = frame([], [wire(0.02)], 0.05)

		expect(early).not.toBe(later)
	})

	it('leaves a wire with no current still', () => {
		const early = frame([], [wire(undefined)], 0)
		const later = frame([], [wire(undefined)], 0.5)

		expect(early).toBe(later)
	})

	it('moves the dots the other way when the current reverses', () => {
		// Direction is the whole point of the overlay: a beginner learning which
		// way round a battery goes has nothing else to look at.
		const forwards = frame([], [wire(0.02, true)], 0.02)
		const backwards = frame([], [wire(0.02, false)], 0.02)

		expect(forwards).not.toBe(backwards)
	})

	it('holds current still for someone who asked for less motion, and still shows its way', () => {
		// prefers-reduced-motion: nothing moves, but direction is what the
		// overlay teaches, so it is drawn rather than dropped.
		const early = frame([], [wire(0.02)], 0, true)
		const later = frame([], [wire(0.02)], 0.05, true)
		expect(early).toBe(later)

		const backwards = frame([], [wire(0.02, false)], 0, true)
		expect(backwards).not.toBe(early)

		const none = frame([], [wire(undefined)], 0, true)
		expect(none).not.toBe(early)
	})

	it('moves faster with more current', () => {
		const trickle = frame([], [wire(1e-4)], 0.05)
		const torrent = frame([], [wire(0.5)], 0.05)

		expect(trickle).not.toBe(torrent)
	})

	it('draws a lit LED differently from a dark one', () => {
		const led = (brightness: number): SceneItem => ({
			id: 'd1',
			position: { x: 150, y: 100 },
			symbol: SYMBOLS.Led,
			brightness,
			colour: 'Red',
		})

		const dark = frame([led(0)], [], 0)
		const dim = frame([led(0.3)], [], 0)
		const bright = frame([led(1)], [], 0)

		expect(dark).not.toBe(dim)
		expect(dim).not.toBe(bright)
	})

	it('gives each LED colour its own light', () => {
		const led = (colour: string): SceneItem => ({
			id: 'd1',
			position: { x: 150, y: 100 },
			symbol: SYMBOLS.Led,
			brightness: 1,
			colour,
		})

		const drawings = ['Red', 'Green', 'Blue', 'White'].map((colour) => frame([led(colour)], [], 0))

		expect(new Set(drawings).size).toBe(drawings.length)
	})

	it('does not light a component that emits nothing', () => {
		const resistor: SceneItem = {
			id: 'r1',
			position: { x: 150, y: 100 },
			symbol: SYMBOLS.Resistor,
		}

		// No brightness, so nothing to draw under it — and the drawing is the
		// same as it was before any of this existed.
		expect(frame([resistor], [], 0)).toBe(frame([resistor], [], 5))
	})
})

describe('turning a component', () => {
	const resistor = (turns: number): SceneItem => ({
		id: 'r1',
		position: { x: 0, y: 0 },
		symbol: SYMBOLS.Resistor,
		turns,
	})

	it('turns clockwise on screen, where y runs down', () => {
		// A point out to the right goes to the bottom. Getting this backwards
		// draws a mirror image that still passes every symmetric test.
		expect(turned({ x: 10, y: 0 }, 1)).toEqual({ x: 0, y: 10 })
		expect(turned({ x: 10, y: 0 }, 2)).toEqual({ x: -10, y: 0 })
		expect(turned({ x: 10, y: 0 }, 3)).toEqual({ x: 0, y: -10 })
	})

	it('comes back to where it started after four', () => {
		expect(turned({ x: 7, y: -3 }, 4)).toEqual({ x: 7, y: -3 })
	})

	it('keeps every pin on a grid intersection', () => {
		// Quarter turns exist for this reason: wiring by pointer and orthogonal
		// routing both assume pins sit on intersections.
		for (const symbol of Object.values(SYMBOLS)) {
			for (const turns of [0, 1, 2, 3]) {
				for (const pin of symbol.pins) {
					const at = turned(pin.at, turns)
					// `===` rather than `toBe`, because -20 % 10 is -0 and
					// `Object.is(-0, 0)` is false. On the grid is on the grid.
					expect(at.x % GRID_SPACING === 0, `x after ${turns}`).toBe(true)
					expect(at.y % GRID_SPACING === 0, `y after ${turns}`).toBe(true)
				}
			}
		}
	})

	it('moves the pins with the symbol', () => {
		const upright = pinPosition(resistor(0), SYMBOLS.Resistor.pins[0])
		const turnedOver = pinPosition(resistor(1), SYMBOLS.Resistor.pins[0])

		expect(upright).toEqual({ x: -2 * GRID_SPACING, y: 0 })
		expect(turnedOver).toEqual({ x: 0, y: -2 * GRID_SPACING })
	})

	it('turns the hit box with the symbol', () => {
		// A hit box that stayed wide while the drawing went tall would make a
		// turned component unclickable where it is and clickable where it is
		// not.
		const wide = halfSizeOf(SYMBOLS.Resistor, 0)
		const tall = halfSizeOf(SYMBOLS.Resistor, 1)

		expect(tall).toEqual({ x: wide.y, y: wide.x })
	})

	it('can be picked where it is now drawn', () => {
		const item = resistor(1)
		// On the body of a component now standing on end: below the origin, and
		// far enough from the pin at (0, -20) that the pin does not claim it.
		expect(hitTest([item], { x: 0, y: 8 })?.kind).toBe('element')
		// Where it used to be, and is not any more.
		expect(hitTest([item], { x: 25, y: 0 })).toBeUndefined()
	})

	it('offers its pins where they now are', () => {
		const item = resistor(1)

		const hit = hitTest([item], { x: 0, y: -2 * GRID_SPACING })
		expect(hit?.kind).toBe('pin')
	})
})

describe('the app and the core agree about what a component is', () => {
	// ADR 0008 splits a component's definition three ways: the core owns what
	// is true about it, `symbols.ts` owns how it is drawn, `palette.ts` owns
	// how it is described. These are the tests that keep the three from
	// drifting — the only thing standing between two languages, so weakening
	// one to make a change land is reintroducing the bug the registry exists
	// to prevent.

	it('draws every kind the core says is drawable', async () => {
		const registry = await componentRegistry()

		for (const component of registry.filter((entry) => entry.drawable)) {
			expect(SYMBOLS[component.kind], component.kind).toBeDefined()
		}
	})

	it('draws nothing the core has never heard of', async () => {
		const registry = await componentRegistry()
		const known = new Set<string>(registry.map((component) => component.kind))

		for (const kind of Object.keys(SYMBOLS)) {
			expect(known.has(kind), kind).toBe(true)
		}
	})

	it('offers exactly what the core offers, in the same order', async () => {
		// The palette's list used to be its own opinion, guarded by a test that
		// restated §5. Now it is the core's, and this is where they meet.
		const registry = await componentRegistry()
		const offered = registry.filter((entry) => entry.offered).map((entry) => entry.kind)

		expect(PALETTE.map((entry) => entry.kind)).toEqual(offered)
	})

	it('describes every kind it can draw', async () => {
		// A component with a symbol and no words is one the inspector would
		// show by its core spelling, which is what C-37 was about.
		const registry = await componentRegistry()

		for (const component of registry.filter((entry) => entry.drawable)) {
			expect(describeComponent(component.kind), component.kind).toBeDefined()
		}
	})

	it('takes the unit of a value from the core', async () => {
		// The inspector shows it beside the field, and the core is what decides
		// a resistor is in ohms.
		const registry = await componentRegistry()
		const resistor = registry.find((entry) => entry.kind === 'Resistor')

		expect(resistor?.unit).toBe('Ω')
	})
})
