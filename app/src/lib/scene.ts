// SPDX-License-Identifier: MIT OR Apache-2.0

// What is drawn, as data.
//
// The canvas renders a scene and hit-tests against it; it does not know what a
// netlist is. M-16 is what builds a scene from one, and until then the shape is
// enough to draw symbols and test the interaction.

import type { ElementView, NetView, PinCurrent, PinView, SolutionView } from './core'
import { warmth } from './heat'
import { angleLabel, batteryLabel, probeLabel, toneLabel } from './readings'
import { AS_MEASURED, type Settle } from './steady'
import { junctions, routeBetween, routeRuns, type Run, type Segment } from './routing'
import {
	leadOf,
	halfSizeOf,
	symbolFor,
	turned,
	type SymbolDefinition,
	type SymbolKind,
	type SymbolPin,
} from './symbols'

import type { Bounds, Point } from './viewport'

/** Something drawn on the canvas. */
export interface SceneItem {
	readonly id: string
	/**
	 * What it is, for whatever has to name it in words — the list of parts
	 * a screen reader and a keyboard reach the canvas through. Absent for
	 * something drawn that is not a part, as in a test.
	 */
	readonly kind?: SymbolKind
	/** Origin in world coordinates, normally on a grid intersection. */
	readonly position: Point
	readonly symbol: SymbolDefinition
	/** Quarter turns clockwise from the way the symbol is drawn (C-40). */
	readonly turns?: number
	/** Shown beside the symbol — "220 Ω", "9 V". */
	readonly label?: string
	/**
	 * 0 to 1 for a lit LED, absent for anything that does not emit light. For
	 * an RGB LED, its brightest die's (C-84).
	 */
	readonly brightness?: number
	/** The light's colour, when there is one. */
	readonly colour?: string
	/**
	 * Whether a contact is made, absent for anything that is not one (C-19).
	 *
	 * The symbol already differs by it — that is how a switch looks shut — and
	 * this is here so the shell can tell a press what kind of press it was
	 * without asking the core again.
	 */
	readonly closed?: boolean
	/** Where a wiper sits, absent for anything without one (C-20). */
	readonly wiper?: number
	/**
	 * 0 to 1 for each of a digit's segments, in the order its symbol lists
	 * them, or for each of an RGB LED's dice (C-84); absent for anything else
	 * (C-75).
	 */
	readonly segments?: readonly number[]
	/**
	 * How far a part that warms has come from the room towards the hottest
	 * it may run: 0 to 1, more past its limit (C-74). Absent for a part with
	 * no temperature.
	 */
	readonly warmth?: number
}

/**
 * Projects what the core says the circuit contains into something drawable.
 *
 * One direction only. The canvas is a projection of the circuit and never the
 * other way round, which is the difference between this and Fritzing's three
 * views each holding their own copy (see the prior-art study).
 */
/**
 * Kinds whose drawing follows what they are doing rather than anything the
 * user set: a relay pulled in, a buzzer sounding, a motor turning (C-64, C-66).
 */
const DRAWN_BY_RUN = new Set(['Relay', 'Buzzer', 'PassiveBuzzer', 'Motor'])

export function toScene(
	elements: readonly ElementView[],
	solution?: SolutionView,
	nets: readonly NetView[] = [],
	room = 25,
	/** What holds a reading still long enough to read, for the labels that are readings. */
	settle: Settle = AS_MEASURED,
): SceneItem[] {
	const brightness = new Map(
		solution?.elementCurrents
			.filter((current) => current.brightness !== undefined)
			.map((current) => [current.elementId, current.brightness as number]),
	)

	const segments = new Map(solution?.digits.map((digit) => [digit.elementId, digit.segments]))

	// A servo's horn is where the run has turned it (C-90).
	const angles = new Map(solution?.angles.map((entry) => [entry.elementId, entry.degrees]))

	const warmths = new Map(solution?.heat.map((heat) => [heat.elementId, warmth(heat, room)]))

	// A relay's contact is thrown by its coil, not by the user, so where it is
	// drawn comes from the solution rather than the element (C-64).
	const energised = new Set(solution?.energised ?? [])
	// And which way a turning motor turns (C-91).
	const reversed = new Set(solution?.reversed ?? [])

	return elements.map((element) => {
		const symbol = symbolFor(element.kind, {
			closed:
				element.closed ?? (DRAWN_BY_RUN.has(element.kind) ? energised.has(element.id) : undefined),
			wiper: element.wiper ?? undefined,
			choice: element.choice ?? undefined,
			angle: angles.get(element.id),
			reversed: reversed.has(element.id),
			blown: element.blown ?? undefined,
		})
		const lit = segments.get(element.id)
		return {
			id: element.id,
			kind: element.kind,
			position: { x: element.x, y: element.y },
			symbol,
			turns: element.turns,
			// A probe's label is its reading, which the core does not write: it
			// changes every frame and the element views do not (C-21). Everything
			// else is labelled by what it is, which does — but for a passive
			// buzzer, labelled by the note it is playing, likewise (C-85).
			label:
				probeLabel(element, nets, solution, settle) ??
				toneLabel(element, solution, settle) ??
				angleLabel(element, solution, settle) ??
				batteryLabel(element, solution, settle) ??
				(element.label || undefined),
			// An RGB LED is one lamp, as bright as its brightest die (C-84).
			brightness:
				brightness.get(element.id) ?? (symbol.dice && lit ? Math.max(0, ...lit) : undefined),
			colour: element.colour ?? undefined,
			closed: element.closed ?? undefined,
			wiper: element.wiper ?? undefined,
			segments: lit,
			warmth: warmths.get(element.id),
		}
	})
}

/** A net, ready to draw. */
export interface SceneWire {
	readonly id: string
	/**
	 * The colour of its voltage, when wires are being drawn by voltage
	 * (C-134, `voltage.ts`). Absent, it is drawn in the plain wire colour.
	 */
	readonly ink?: string
	readonly segments: readonly Segment[]
	/** Where this net's pins are, so junctions can tell a bend from a joint. */
	readonly pins: readonly Point[]
	/** Points the wire has been bent through, in order (ADR 0007). */
	readonly waypoints: readonly Point[]
	/**
	 * The current along each run of the wire, in the direction it goes.
	 *
	 * One per run rather than one for the wire (C-73). A net joining three pins
	 * is drawn as a tree, and on a tree the current in each run is whatever
	 * the pins on one side of it put in — so where a battery feeds two
	 * branches, each branch's run carries its own share, and a meter's stub
	 * carries nothing and is not animated: current flowing into an instrument
	 * whose whole point is that none does would teach the opposite (C-21).
	 * Runs carrying nothing are left out. Absent when the wire's current
	 * cannot be known, which is the state where no dots are better than dots
	 * nobody checked.
	 */
	readonly flows?: readonly Flow[]
}

/** A stretch of wire and the current along it, in the order of its segments. */
export interface Flow {
	readonly segments: readonly Segment[]
	/** Amps, always positive: the direction is the segments'. */
	readonly amps: number
}

/**
 * Turns nets into wires by looking up where each pin is drawn.
 *
 * The core knows what is connected and the app knows where pins are, so this is
 * the one place the two facts meet. A pin whose element is not on the canvas is
 * skipped rather than defaulting to the origin, which would draw a wire to
 * nowhere.
 */
export function toWires(
	nets: readonly NetView[],
	items: readonly SceneItem[],
	currents: readonly PinCurrent[] = [],
	/** Routes from the frame before, to reuse where nothing they depend on moved. */
	routes?: RouteCache,
): SceneWire[] {
	const byId = new Map(items.map((item) => [item.id, item]))
	const key = (netId: string, pin: PinView) => `${netId} ${pin.elementId} ${pin.pin}`
	const pushed = new Map(currents.map((current) => [key(current.netId, current.pin), current.amps]))
	// Every pin on the canvas, wired or not: a wire drawn over any pin that is
	// not its own reads as joined to it (C-61).
	const everyPin = items.flatMap((item) => item.symbol.pins.map((pin) => pinPosition(item, pin)))
	const previous = routes?.begin(everyPin)

	return nets
		.map((net) => {
			// The pins kept alongside their positions, because the core names
			// the ends of a current by pin and the routing works in indices.
			const placed = net.pins.flatMap((pin) => {
				const item = byId.get(pin.elementId)
				const symbolPin = item?.symbol.pins.find((candidate) => candidate.name === pin.pin)
				return item && symbolPin
					? [
							{
								pin,
								at: pinPosition(item, symbolPin),
								lead: leadOf(symbolPin, item.turns),
							},
						]
					: []
			})
			const points = placed.map((entry) => entry.at)
			const waypoints = net.waypoints ?? []
			const leads = placed.map((entry) => entry.lead)
			const route = () => {
				const own = new Set(points.map((point) => `${point.x},${point.y}`))
				return routeRuns(
					points,
					waypoints,
					leads,
					everyPin.filter((point) => !own.has(`${point.x},${point.y}`)),
				)
			}
			const runs =
				routes && previous ? routes.reuse(previous, [points, waypoints, leads], route) : route()

			// A pin not on the canvas is current the tree cannot place.
			const flows =
				placed.length === net.pins.length
					? flowsOf(
							runs,
							placed.map((entry) => pushed.get(key(net.id, entry.pin))),
						)
					: undefined

			return {
				id: net.id,
				segments: runs.flatMap((run) => [...run.segments]),
				pins: points,
				waypoints,
				flows,
			}
		})
		.filter((wire) => wire.segments.length > 0)
}

/**
 * The routes of the last frame, kept for the next.
 *
 * Routing is the dear half of drawing a wire: each run tries its candidate
 * bends against every pin on the canvas. What it depends on — where the
 * net's pins are and which way they face, its waypoints, and every other pin
 * — changes with an edit, not with a frame, while the currents along it
 * change every frame. So a route is kept under those inputs, and only what
 * flows along it is worked out again.
 *
 * A move anywhere changes every pin there is, and starts again; between
 * edits, each frame keeps exactly the routes it used, so nothing piles up.
 */
export class RouteCache {
	#obstacles = ''
	#kept = new Map<string, readonly Run[]>()

	/** Starts a frame: what the last one routed, if the canvas has not moved since. */
	begin(everyPin: readonly Point[]): Map<string, readonly Run[]> {
		const obstacles = everyPin.map((point) => `${point.x},${point.y}`).join(' ')
		const previous = obstacles === this.#obstacles ? this.#kept : new Map<string, readonly Run[]>()
		this.#obstacles = obstacles
		this.#kept = new Map()
		return previous
	}

	/** One net's runs: the last frame's, if its inputs are the same, else routed now. */
	reuse(
		previous: Map<string, readonly Run[]>,
		inputs: unknown,
		route: () => readonly Run[],
	): readonly Run[] {
		const key = JSON.stringify(inputs)
		const runs = previous.get(key) ?? route()
		this.#kept.set(key, runs)
		return runs
	}
}

/**
 * The current along every run of a tree, from what each pin puts into it.
 *
 * The runs form a tree over the pins — that is what a minimum spanning tree
 * is — and cutting a run splits the pins in two. Whatever the pins on one side
 * put in has to cross that run to reach the other, so each run's current is
 * one sum, and the whole wire is known from its pins alone.
 *
 * One pin the core could not say — a transistor's, a chip's — is still known:
 * what enters a wire leaves it, so it is whatever the others leave over. Two
 * such pins, or runs that are not a tree, and nothing is claimed.
 */
function flowsOf(
	runs: readonly Run[],
	pushed: readonly (number | undefined)[],
): Flow[] | undefined {
	const count = pushed.length
	if (count < 2 || runs.length !== count - 1) return undefined

	const unknown = pushed.flatMap((amps, index) => (amps === undefined ? [index] : []))
	if (unknown.length > 1) return undefined
	const into = pushed.map((amps) => amps ?? 0)
	if (unknown.length === 1) into[unknown[0]] = -into.reduce((sum, amps) => sum + amps, 0)

	// Walked from the first pin, so every other pin has one run towards it.
	const towards = new Map<number, Run>()
	const order = [0]
	const seen = new Set([0])
	for (let next = 0; next < order.length; next++) {
		const at = order[next]
		for (const run of runs) {
			const [left, right] = run.ends
			const other = left === at ? right : right === at ? left : undefined
			if (other === undefined || seen.has(other)) continue
			seen.add(other)
			towards.set(other, run)
			order.push(other)
		}
	}
	if (seen.size !== count) return undefined

	// Leaves first: each pin's total is its own and everything beyond it, and
	// that total is what crosses the run back towards the first pin.
	const beyond = [...into]
	const flows: Flow[] = []
	for (const pin of order.slice(1).reverse()) {
		const run = towards.get(pin) as Run
		const [left, right] = run.ends
		const parent = left === pin ? right : left
		const amps = beyond[pin]
		beyond[parent] += amps
		// Nothing crossing is nothing to draw, which is what a meter's stub
		// and a reversed diode's branch carry.
		if (Math.abs(amps) < 1e-12) continue
		// The run's segments go from its first end to its second.
		const along = (left === pin) === amps > 0
		flows.push({
			segments: along ? run.segments : reversed(run.segments),
			amps: Math.abs(amps),
		})
	}
	return flows
}

/** The same run, walked the other way. */
function reversed(segments: readonly Segment[]): Segment[] {
	return segments.map((segment) => ({ from: segment.to, to: segment.from })).reverse()
}

/** Every junction dot across every wire. */
export function wireJunctions(wires: readonly SceneWire[]): Point[] {
	return wires.flatMap((wire) => junctions(wire.segments, wire.pins))
}

export type Hit =
	| { readonly kind: 'pin'; readonly item: SceneItem; readonly pin: SymbolPin }
	| { readonly kind: 'element'; readonly item: SceneItem }
	| { readonly kind: 'wire'; readonly wire: SceneWire }

/**
 * A hit on something the circuit is made of, rather than on a wire between two
 * of them. What `hitTest` can return, and what wiring and moving work on.
 */
export type ElementHit = Extract<Hit, { kind: 'element' | 'pin' }>

/**
 * How close, in screen pixels, counts as being on a pin.
 *
 * Screen pixels rather than world units, because the tolerance has to mean the
 * same thing to the hand holding the mouse at every zoom. In world units a pin
 * becomes a single pixel when zoomed out and simply cannot be hit.
 */
export const PIN_PICK_RADIUS = 8

/** The same tolerance in world units, for a given zoom. */
export function pinRadiusFor(zoom: number): number {
	return PIN_PICK_RADIUS / zoom
}

/**
 * How close, in screen pixels, counts as being on a wire.
 *
 * Tighter than a pin's, because a wire is a long target and a generous
 * tolerance around every segment would swallow the empty space a drag needs in
 * order to pan.
 */
export const WIRE_PICK_RADIUS = 5

/** The same tolerance in world units, for a given zoom. */
export function wireRadiusFor(zoom: number): number {
	return WIRE_PICK_RADIUS / zoom
}

/**
 * Which wire is under a world point, or nothing.
 *
 * Elements win over wires everywhere they overlap, so this is asked only once
 * nothing else claimed the point — a wire runs under the symbols it joins, and
 * clicking a resistor should never select the wire crossing behind it.
 */
export function hitWire(
	wires: readonly SceneWire[],
	world: Point,
	tolerance: number = WIRE_PICK_RADIUS,
): (Hit & { kind: 'wire' }) | undefined {
	for (let index = wires.length - 1; index >= 0; index--) {
		const wire = wires[index]
		for (const segment of wire.segments) {
			if (distanceToSegment(world, segment.from, segment.to) <= tolerance) {
				return { kind: 'wire', wire }
			}
		}
	}

	return undefined
}

/**
 * The waypoint list a wire should have after being dragged at a point (ADR 0007).
 *
 * Two gestures, told apart by what was grabbed: a drag that starts on an
 * existing bend moves that bend, and a drag anywhere else on the wire adds one.
 * A new bend goes between the waypoints on either side of the segment that was
 * grabbed, so the wire bends where it was pulled rather than folding over
 * itself.
 */
export function bendWire(
	wire: SceneWire,
	grabbedAt: Point,
	to: Point,
	tolerance: number = WIRE_PICK_RADIUS,
): Point[] {
	const held = wire.waypoints.findIndex((waypoint) => distance(waypoint, grabbedAt) <= tolerance)
	if (held >= 0) {
		return wire.waypoints.map((waypoint, index) => (index === held ? to : waypoint))
	}

	const at = insertionFor(wire, grabbedAt)
	return [...wire.waypoints.slice(0, at), to, ...wire.waypoints.slice(at)]
}

/**
 * Where in the waypoint list a bend grabbed at a point belongs.
 *
 * The run is pin → waypoints → pin, so the answer is how many waypoints lie
 * before the grabbed spot along it: the first stretch belongs at 0, the one
 * after the first waypoint at 1, and so on.
 */
function insertionFor(wire: SceneWire, grabbedAt: Point): number {
	const [from, to] = wire.pins
	if (!from || !to) return wire.waypoints.length

	const stops = [from, ...wire.waypoints, to]
	for (let index = 0; index + 1 < stops.length; index++) {
		const stretch = routeBetween(stops[index], stops[index + 1])
		if (
			stretch.some(
				(segment) => distanceToSegment(grabbedAt, segment.from, segment.to) <= WIRE_PICK_RADIUS,
			)
		) {
			return index
		}
	}

	return wire.waypoints.length
}

/** How far a point lies from a line segment, not from the infinite line through it. */
function distanceToSegment(point: Point, from: Point, to: Point): number {
	const run = { x: to.x - from.x, y: to.y - from.y }
	const lengthSquared = run.x * run.x + run.y * run.y
	if (lengthSquared === 0) return distance(point, from)

	// Where the foot of the perpendicular falls along the segment, clamped to
	// its ends so a point beyond one end measures to that end rather than to
	// the line it is nowhere near.
	const along = Math.max(
		0,
		Math.min(1, ((point.x - from.x) * run.x + (point.y - from.y) * run.y) / lengthSquared),
	)

	return distance(point, { x: from.x + along * run.x, y: from.y + along * run.y })
}

/**
 * What is under a world point, or nothing.
 *
 * Two rules, both of which matter for wiring:
 *
 * - A pin wins over the body it belongs to. Pins sit at the end of the leads,
 *   inside the symbol's bounds, so every pin hit is also a body hit and wiring
 *   would be impossible the other way round.
 * - Later items win over earlier ones, matching the order they are drawn in.
 */
export function hitTest(
	items: readonly SceneItem[],
	world: Point,
	pinRadius: number = PIN_PICK_RADIUS,
): ElementHit | undefined {
	for (let index = items.length - 1; index >= 0; index--) {
		const item = items[index]
		for (const pin of item.symbol.pins) {
			if (distance(pinPosition(item, pin), world) <= pinRadius) {
				return { kind: 'pin', item, pin }
			}
		}
	}

	for (let index = items.length - 1; index >= 0; index--) {
		const item = items[index]
		const halfSize = halfSizeOf(item.symbol, item.turns)
		if (
			Math.abs(world.x - item.position.x) <= halfSize.x &&
			Math.abs(world.y - item.position.y) <= halfSize.y
		) {
			return { kind: 'element', item }
		}
	}

	return undefined
}

/** Absolute world position of a pin, wherever the component is facing. */
export function pinPosition(item: SceneItem, pin: SymbolPin): Point {
	const at = turned(pin.at, item.turns)
	return { x: item.position.x + at.x, y: item.position.y + at.y }
}

function distance(a: Point, b: Point): number {
	return Math.hypot(a.x - b.x, a.y - b.y)
}

/**
 * The rectangle every item on the canvas fits inside, or nothing for an empty
 * one.
 *
 * Each symbol reaches the same distance from its own origin whichever way it is
 * turned — C-28 holds every one of them to that — so the extent of the drawing
 * is the extent of the positions, grown by one symbol. Used to fit the view to
 * a circuit that arrived from a file rather than from the pointer (C-47).
 */
export function boundsOf(items: readonly SceneItem[]): Bounds | undefined {
	if (items.length === 0) return undefined

	let min = { x: Infinity, y: Infinity }
	let max = { x: -Infinity, y: -Infinity }
	for (const item of items) {
		const half = halfSizeOf(item.symbol, item.turns)
		min = {
			x: Math.min(min.x, item.position.x - half.x),
			y: Math.min(min.y, item.position.y - half.y),
		}
		max = {
			x: Math.max(max.x, item.position.x + half.x),
			y: Math.max(max.y, item.position.y + half.y),
		}
	}
	return { min, max }
}

/**
 * The elements a rectangle touches, in the order they are drawn.
 *
 * Touches rather than contains: a selection rectangle is dragged by someone
 * who has just learnt what a resistor is, and asking them to enclose every
 * symbol whole — its leads included — would have it miss the part they
 * plainly meant. Each symbol is its drawn extent, the one `boundsOf` fits the
 * view to, so what looks inside is inside.
 */
export function itemsIn(items: readonly SceneItem[], area: Bounds): string[] {
	const low = { x: Math.min(area.min.x, area.max.x), y: Math.min(area.min.y, area.max.y) }
	const high = { x: Math.max(area.min.x, area.max.x), y: Math.max(area.min.y, area.max.y) }
	return items
		.filter((item) => {
			const half = halfSizeOf(item.symbol, item.turns)
			return (
				item.position.x + half.x >= low.x &&
				item.position.x - half.x <= high.x &&
				item.position.y + half.y >= low.y &&
				item.position.y - half.y <= high.y
			)
		})
		.map((item) => item.id)
}
