// SPDX-License-Identifier: MIT OR Apache-2.0

import { describe, expect, it } from 'vitest'

import { createCircuit } from './core'
import { EXAMPLES } from './examples'
import {
	junctions,
	routeBetween,
	routeLeaving,
	routeNet,
	routeThrough,
	withoutRedundant,
	type Segment,
} from './routing'
import {
	bendWire,
	hitWire,
	RouteCache,
	toScene,
	toWires,
	wireJunctions,
	type SceneItem,
	type SceneWire,
} from './scene'
import { SYMBOLS, turned, type SymbolKind } from './symbols'
import type { NetView } from './core'
import type { Point } from './viewport'

/** Two points at the same place. */
function same(a: Point, b: Point): boolean {
	return a.x === b.x && a.y === b.y
}

function isOrthogonal(segment: Segment): boolean {
	return segment.from.x === segment.to.x || segment.from.y === segment.to.y
}

describe('routing a wire', () => {
	it('uses only right angles', () => {
		const segments = routeBetween({ x: 0, y: 0 }, { x: 70, y: 40 })

		expect(segments.every(isOrthogonal)).toBe(true)
	})

	it('joins the two points it was given', () => {
		const from = { x: 10, y: 20 }
		const to = { x: 90, y: -30 }

		const segments = routeBetween(from, to)

		expect(segments[0].from).toEqual(from)
		expect(segments.at(-1)?.to).toEqual(to)
		// And the run is continuous, not two disconnected sticks.
		for (let i = 1; i < segments.length; i++) {
			expect(segments[i].from).toEqual(segments[i - 1].to)
		}
	})

	it('is a single segment when the pins already line up', () => {
		expect(routeBetween({ x: 0, y: 0 }, { x: 50, y: 0 })).toHaveLength(1)
		expect(routeBetween({ x: 0, y: 0 }, { x: 0, y: 50 })).toHaveLength(1)
	})

	it('leaves a pin along the axis the lead already points', () => {
		// A mostly-horizontal wire should set off horizontally, not stab
		// vertically out of the side of the symbol.
		const [first] = routeBetween({ x: 0, y: 0 }, { x: 100, y: 20 })

		expect(first.from.y).toBe(first.to.y)
	})

	it('draws nothing between a point and itself', () => {
		expect(routeBetween({ x: 5, y: 5 }, { x: 5, y: 5 })).toEqual([])
	})
})

describe('junction dots', () => {
	it('marks a pin where a third connection joins', () => {
		// Three pins on one net: two wires leave the trunk pin, so three things
		// meet there and a reader needs the dot to know it.
		const pins = [
			{ x: 0, y: 0 },
			{ x: 40, y: 0 },
			{ x: 0, y: 40 },
		]

		expect(junctions(routeNet(pins), pins)).toContainEqual({ x: 0, y: 0 })
	})

	it('leaves a plain corner undotted', () => {
		// A bend is two segments and no pin, so nothing is joined there.
		const from = { x: 0, y: 0 }
		const to = { x: 40, y: 40 }

		expect(junctions(routeBetween(from, to), [from, to])).toEqual([])
	})

	it('leaves a two-pin wire undotted', () => {
		const pins = [
			{ x: 0, y: 0 },
			{ x: 40, y: 40 },
		]

		expect(junctions(routeNet(pins), pins)).toEqual([])
	})

	it('dots a point a third segment merely passes through', () => {
		// A wire ending on the middle of another is connected, and a reader can
		// only tell from the dot.
		const segments: Segment[] = [
			{ from: { x: 0, y: 0 }, to: { x: 100, y: 0 } },
			{ from: { x: 50, y: 0 }, to: { x: 50, y: 40 } },
		]

		expect(junctions(segments)).toContainEqual({ x: 50, y: 0 })
	})
})

describe('wiring a circuit', () => {
	it('draws a wire between the pins the core says are connected', async () => {
		const circuit = await createCircuit()
		const battery = circuit.add('DcSource', 0, 0)
		const resistor = circuit.add('Resistor', 80, 0)
		circuit.connect(battery, '+', resistor, 'a')

		const items = toScene(circuit.elements())
		const wires = toWires(circuit.nets(), items)

		expect(wires).toHaveLength(1)
		expect(wires[0].segments.every(isOrthogonal)).toBe(true)
	})

	it('puts a dot where a third pin joins a net', async () => {
		const circuit = await createCircuit()
		const battery = circuit.add('DcSource', 0, 0)
		const first = circuit.add('Resistor', 80, 0)
		const second = circuit.add('Resistor', 0, 80)
		circuit.connect(battery, '+', first, 'a')
		circuit.connect(battery, '+', second, 'a')

		const wires = toWires(circuit.nets(), toScene(circuit.elements()))

		expect(wires).toHaveLength(1)
		expect(wireJunctions(wires).length).toBeGreaterThan(0)
	})

	it('skips a net whose element is not on the canvas', async () => {
		// Drawing a wire to the origin because a lookup missed would be worse
		// than drawing nothing.
		const circuit = await createCircuit()
		const battery = circuit.add('DcSource', 0, 0)
		const resistor = circuit.add('Resistor', 80, 0)
		circuit.connect(battery, '+', resistor, 'a')

		const wires = toWires(circuit.nets(), [])

		expect(wires).toEqual([])
	})

	it('wires the milestone circuit into three nets', async () => {
		const circuit = await createCircuit()
		const battery = circuit.add('DcSource', 0, 0)
		const resistor = circuit.add('Resistor', 80, 0)
		const led = circuit.add('Led', 160, 0)

		circuit.connect(battery, '+', resistor, 'a')
		circuit.connect(resistor, 'b', led, 'anode')
		circuit.connect(led, 'cathode', battery, '-')

		expect(circuit.netCount).toBe(3)
		expect(toWires(circuit.nets(), toScene(circuit.elements()))).toHaveLength(3)
	})

	it('draws the same wires from kept routes as from fresh ones, through edits', async () => {
		// The shell keeps routes between frames (RouteCache). Whatever it
		// keeps has to be what routing now would give, or a wire is drawn
		// where the circuit no longer puts it.
		const circuit = await createCircuit()
		const battery = circuit.add('DcSource', 0, 0)
		const resistor = circuit.add('Resistor', 80, 0)
		const led = circuit.add('Led', 160, 0)
		circuit.connect(battery, '+', resistor, 'a')
		circuit.connect(resistor, 'b', led, 'anode')
		circuit.connect(led, 'cathode', battery, '-')
		const routes = new RouteCache()
		const both = () => {
			const items = toScene(circuit.elements())
			const kept = toWires(circuit.nets(), items, [], routes)
			expect(kept).toEqual(toWires(circuit.nets(), items))
			return kept
		}

		const first = both()
		expect(both()).toEqual(first)

		circuit.moveElement(resistor, 80, 120)
		const moved = both()
		expect(moved).not.toEqual(first)

		circuit.setWaypoints(circuit.nets()[0].id, [{ x: 40, y: -80 }])
		both()
		circuit.moveElement(led, 200, 40)
		both()
	})

	it('refuses to wire a pin to itself', async () => {
		const circuit = await createCircuit()
		const resistor = circuit.add('Resistor', 0, 0)

		expect(() => circuit.connect(resistor, 'a', resistor, 'a')).toThrow(/itself/)
		expect(circuit.netCount).toBe(0)
	})
})

describe('aiming at a wire', () => {
	const wire = (segments: Segment[]): SceneWire => ({ id: 'n1', segments, pins: [], waypoints: [] })
	const run = wire([{ from: { x: 0, y: 0 }, to: { x: 100, y: 0 } }])

	it('finds the wire under the pointer', () => {
		expect(hitWire([run], { x: 50, y: 2 })?.wire.id).toBe('n1')
	})

	it('misses when the pointer is merely near the line it runs along', () => {
		// Past the end of a segment, not beside it. Measuring to the infinite
		// line instead of the segment would select a wire from across the
		// canvas.
		expect(hitWire([run], { x: 300, y: 0 })).toBeUndefined()
	})

	it('misses when the pointer is beside it', () => {
		expect(hitWire([run], { x: 50, y: 40 })).toBeUndefined()
	})

	it('finds nothing where there is nothing', () => {
		expect(hitWire([], { x: 0, y: 0 })).toBeUndefined()
	})

	it('takes the tolerance it is given, so zoom does not change the aim', () => {
		expect(hitWire([run], { x: 50, y: 18 }, 20)?.wire.id).toBe('n1')
	})
})

describe('a wire the user has bent', () => {
	const from = { x: 0, y: 0 }
	const to = { x: 100, y: 100 }

	it('runs through every point it was bent at', () => {
		const through = { x: 0, y: 100 }
		const segments = routeThrough([from, through, to])

		expect(segments.every(isOrthogonal)).toBe(true)
		expect(segments[0].from).toEqual(from)
		expect(segments.at(-1)?.to).toEqual(to)
		expect(segments.some((segment) => same(segment.to, through))).toBe(true)
	})

	it('keeps the derived route when it has no bends', () => {
		// Nothing changes for a wire nobody has touched.
		expect(routeNet([from, to], [])).toEqual(routeNet([from, to]))
	})

	it('still branches a third pin from the first, derived', () => {
		// The bends belong to the run between the first two pins (ADR 0007).
		const third = { x: 0, y: -80 }
		const bent = routeNet([from, to, third], [{ x: 0, y: 100 }])

		expect(bent.every(isOrthogonal)).toBe(true)
		expect(bent.some((segment) => same(segment.to, third))).toBe(true)
	})

	it('drops a bend that no longer bends anything', () => {
		// A point in line with its neighbours cannot be seen, so leaving it
		// there would mean a straightened wire keeping invisible bends.
		const straight = withoutRedundant(from, [{ x: 50, y: 0 }], { x: 100, y: 0 })

		expect(straight).toEqual([])
	})

	it('keeps a bend that does bend something', () => {
		const corner = { x: 0, y: 100 }

		expect(withoutRedundant(from, [corner], to)).toEqual([corner])
	})
})

describe('dragging a wire', () => {
	const pins = [
		{ x: 0, y: 0 },
		{ x: 100, y: 0 },
	]
	const straight = (waypoints: Point[] = []): SceneWire => ({
		id: 'n1',
		pins,
		waypoints,
		segments: routeNet(pins, waypoints),
	})

	it('adds a bend where the wire was pulled', () => {
		expect(bendWire(straight(), { x: 50, y: 0 }, { x: 50, y: 60 })).toEqual([{ x: 50, y: 60 }])
	})

	it('moves the bend it was grabbed by, rather than adding another', () => {
		const wire = straight([{ x: 50, y: 60 }])

		expect(bendWire(wire, { x: 50, y: 60 }, { x: 50, y: 20 })).toEqual([{ x: 50, y: 20 }])
	})

	it('puts a new bend in the right place along the run', () => {
		// Grabbed after the existing bend, so it belongs after it — the other
		// way round the wire would fold back over itself.
		const wire = straight([{ x: 50, y: 60 }])
		const grabbed = wire.segments.at(-1)!

		const bends = bendWire(wire, grabbed.from, { x: 80, y: 30 })

		expect(bends).toHaveLength(2)
		expect(bends[0]).toEqual({ x: 50, y: 60 })
	})
})

describe('a net joining three pins', () => {
	// Two LEDs wired in parallel, then joined to a battery sitting between them.
	const led1 = { x: 140, y: 0 }
	const led2 = { x: 140, y: 80 }
	const battery = { x: -20, y: 40 }

	function lengths(segments: readonly Segment[]): number {
		return segments.reduce(
			(total, segment) =>
				total + Math.abs(segment.to.x - segment.from.x) + Math.abs(segment.to.y - segment.from.y),
			0,
		)
	}

	it('joins each pin to its nearest neighbour', () => {
		// The short link between the two LEDs is the one the user drew, and the
		// one a person would draw. A star from whichever pin the netlist listed
		// first drew two runs from the battery instead, and none between them.
		const segments = routeNet([battery, led1, led2])

		expect(segments.some((s) => same(s.from, led1) && same(s.to, led2))).toBe(true)
	})

	it('draws nothing twice', () => {
		// Two branches of a star share their first leg exactly, so the wire is
		// drawn on top of itself — invisible, and enough to confuse the
		// junction dots.
		const segments = routeNet([battery, led1, led2])
		const drawn = segments.map((s) => `${s.from.x},${s.from.y}->${s.to.x},${s.to.y}`)

		expect(new Set(drawn).size).toBe(drawn.length)
	})

	it('uses less wire than a star from the first pin', () => {
		const tree = routeNet([battery, led1, led2])
		const star = [...routeBetween(battery, led1), ...routeBetween(battery, led2)]

		expect(lengths(tree)).toBeLessThan(lengths(star))
	})

	it('draws the same circuit the same way whatever order it was wired in', () => {
		// Insertion order is not geometry. It used to decide the drawing.
		const one = routeNet([battery, led1, led2])
		const other = routeNet([led1, led2, battery])

		expect(lengths(one)).toBe(lengths(other))
	})

	it('still reaches every pin', () => {
		const segments = routeNet([battery, led1, led2])
		const touched = new Set(
			segments.flatMap((s) => [`${s.from.x},${s.from.y}`, `${s.to.x},${s.to.y}`]),
		)

		for (const pin of [battery, led1, led2]) {
			expect(touched.has(`${pin.x},${pin.y}`), `${pin.x},${pin.y}`).toBe(true)
		}
	})

	it('keeps the bends on the run between the first two pins', () => {
		// ADR 0007 puts the waypoints there, so that run is drawn whatever the
		// geometry would otherwise have chosen.
		const corner = { x: 60, y: -60 }
		const segments = routeNet([battery, led1, led2], [corner])

		expect(segments.some((s) => same(s.to, corner) || same(s.from, corner))).toBe(true)
		expect(segments.every(isOrthogonal)).toBe(true)
	})
})

/** Whether a point lies on a segment, its ends included. */
function touches(segment: Segment, point: Point): boolean {
	const { from, to } = segment
	if (from.x === to.x && point.x === from.x) {
		return point.y >= Math.min(from.y, to.y) && point.y <= Math.max(from.y, to.y)
	}
	if (from.y === to.y && point.y === from.y) {
		return point.x >= Math.min(from.x, to.x) && point.x <= Math.max(from.x, to.x)
	}
	return false
}

/**
 * Every place a wire of one net runs over a pin of another — the drawing
 * saying two things are joined that are not (C-61).
 */
function crossings(wires: readonly SceneWire[]): string[] {
	const found: string[] = []
	for (const wire of wires) {
		for (const other of wires) {
			if (other === wire) continue
			for (const pin of other.pins) {
				if (wire.segments.some((segment) => touches(segment, pin))) {
					found.push(`${wire.id} runs over ${other.id}'s pin at ${pin.x},${pin.y}`)
				}
			}
		}
	}
	return found
}

describe('leaving a pin the way its lead points', () => {
	it('runs a cell straight out before it turns', () => {
		// A pin on a part's left edge, its partner well below: the old L went
		// straight down the pin's own column.
		const from = { x: 0, y: 0 }
		const [first] = routeLeaving(from, { x: -1, y: 0 }, { x: -60, y: 80 }, { x: 0, y: -1 })

		expect(first.from).toEqual(from)
		expect(first.to.y).toBe(0)
		expect(first.to.x).toBeLessThanOrEqual(-10)
	})

	it('keeps the straight wire between two pins that face each other', () => {
		const segments = routeLeaving({ x: 0, y: 0 }, { x: 1, y: 0 }, { x: 80, y: 0 }, { x: -1, y: 0 })
		expect(segments).toEqual([{ from: { x: 0, y: 0 }, to: { x: 80, y: 0 } }])
	})

	it('never draws a wire back over itself', () => {
		// A partner behind the pin: carrying on along the lead would head away
		// from it, so the route turns instead of doubling back.
		for (const to of [
			{ x: 40, y: 60 },
			{ x: 40, y: -60 },
			{ x: 10, y: 0 },
			{ x: -40, y: 0 },
		]) {
			const segments = routeLeaving({ x: 0, y: 0 }, { x: -1, y: 0 }, to, { x: 1, y: 0 })
			for (let index = 1; index < segments.length; index++) {
				const [a, b] = [segments[index - 1], segments[index]]
				const dot =
					(a.to.x - a.from.x) * (b.to.x - b.from.x) + (a.to.y - a.from.y) * (b.to.y - b.from.y)
				expect(dot, `to ${to.x},${to.y}`).toBeGreaterThanOrEqual(0)
			}
			expect(segments.every(isOrthogonal)).toBe(true)
			expect(segments.at(-1)?.to).toEqual(to)
		}
	})
})

describe('a part with several pins on one edge', () => {
	/**
	 * The part at the origin, and every one of its pins on a net of its own
	 * with a resistor placed where `partner` says — off the end of the column
	 * the part's pins share, the worst place for it.
	 *
	 * `partner` is given in the part's own frame, before it is turned, and the
	 * answer is turned with the part, so "below" means below the part however
	 * it lies. Each pin's partner is set a further five cells along, so no two
	 * resistors overlap: a fixture nobody could draw proves nothing.
	 */
	function isolated(
		kind: SymbolKind,
		turns: number,
		partner: (pin: Point, index: number) => Point,
	): SceneWire[] {
		const part: SceneItem = {
			id: 'part',
			position: { x: 0, y: 0 },
			symbol: SYMBOLS[kind],
			turns,
		}
		const items: SceneItem[] = [part]
		const nets: NetView[] = part.symbol.pins.map((pin, index) => {
			const anchor: SceneItem = {
				id: `r${index}`,
				position: turned(partner(pin.at, index), turns),
				symbol: SYMBOLS.Resistor,
			}
			items.push(anchor)
			return {
				id: `net${index}`,
				pins: [
					{ elementId: 'part', pin: pin.name },
					{ elementId: anchor.id, pin: 'a' },
				],
				waypoints: [],
			}
		})
		return toWires(nets, items)
	}

	// Up or down the column a pin shares with its neighbours, out to the side
	// it faces, or back across the part: every direction a wire has been seen
	// to go wrong in, and the mirror of each.
	const partners: [string, (pin: Point, index: number) => Point][] = [
		['far below', (pin, index) => ({ x: pin.x - 60 - 50 * index, y: 120 + 20 * index })],
		['far above', (pin, index) => ({ x: pin.x - 60 - 50 * index, y: -120 - 20 * index })],
		['straight below', (pin, index) => ({ x: pin.x + 50 * index, y: 140 + 20 * index })],
		['straight above', (pin, index) => ({ x: pin.x + 50 * index, y: -140 - 20 * index })],
		['off to the right', (pin, index) => ({ x: 160 + 50 * index, y: pin.y + 60 })],
	]

	for (const kind of ['Timer', 'OpAmp', 'Comparator'] as const) {
		for (const turns of [0, 1, 2, 3]) {
			for (const [where, partner] of partners) {
				it(`keeps a ${kind} turned ${turns}'s pins apart, partners ${where}`, () => {
					expect(crossings(isolated(kind, turns, partner))).toEqual([])
				})
			}
		}
	}
})

describe('every example that ships', () => {
	it.each(EXAMPLES.map((example) => [example.id, example] as const))(
		'%s draws no wire over another net’s pin',
		async (_id, example) => {
			const circuit = await createCircuit()
			circuit.load(example.document)
			expect(crossings(toWires(circuit.nets(), toScene(circuit.elements())))).toEqual([])
		},
	)
})
