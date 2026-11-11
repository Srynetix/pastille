// SPDX-License-Identifier: MIT OR Apache-2.0

// Where a wire is drawn.
//
// The core says what is connected; this decides what that looks like. Schematics
// are drawn with orthogonal wires because a right angle is easy to follow with
// an eye and a diagonal crossing two others is not.

import { GRID_SPACING, type Point } from './viewport'

export interface Segment {
	readonly from: Point
	readonly to: Point
}

/**
 * An L-shaped run between two points.
 *
 * The longer axis goes first, so a mostly-horizontal wire leaves its pin
 * horizontally — which is the direction the pin's lead already points.
 */
export function routeBetween(from: Point, to: Point): Segment[] {
	if (from.x === to.x || from.y === to.y) {
		return same(from, to) ? [] : [{ from, to }]
	}

	const corner =
		Math.abs(to.x - from.x) >= Math.abs(to.y - from.y)
			? { x: to.x, y: from.y }
			: { x: from.x, y: to.y }

	return [
		{ from, to: corner },
		{ from: corner, to },
	]
}

/**
 * How far a wire runs straight out of a pin before it may turn: one cell.
 *
 * Enough to clear the column or row a pin shares with its neighbours on the
 * same edge of a part — an op-amp's two inputs, a 555's DIS, THR and TRIG, one
 * cell apart — so a wire that has to go up or down does it out there, not along
 * the pins, where it would read as joining them.
 */
const LEAD_OUT = GRID_SPACING

/**
 * A run between two pins that leaves each along the way its lead points.
 *
 * `fromLead` and `toLead` are unit steps outward from each part, along the
 * edge the pin sits on; with neither known it is the plain L of
 * [`routeBetween`]. Otherwise the wire goes one cell straight out of each pin
 * and the two ends are joined by the cheapest of a handful of candidates — the
 * two Ls, the two Zs with a middle leg half way, and detours a few cells out to
 * either side — among those that neither double back on themselves nor touch a
 * point in `avoid`: the pins of other nets, which a wire running over would
 * appear to join (C-61).
 *
 * Two pins in line and facing each other keep their straight wire, as a
 * resistor chained to a resistor always had, unless it would cross a pin.
 */
export function routeLeaving(
	from: Point,
	fromLead: Point | undefined,
	to: Point,
	toLead: Point | undefined,
	avoid: readonly Point[] = [],
): Segment[] {
	if (same(from, to)) return []
	if (!fromLead && !toLead && avoid.length === 0) return routeBetween(from, to)

	const blocked = (path: readonly Point[]) =>
		avoid.some((point) => segmentsOf(path).some((segment) => onSegment(segment, point)))

	const facing = (at: Point, lead: Point | undefined, other: Point) =>
		!lead ||
		(lead.x !== 0
			? at.y === other.y && (other.x - at.x) * lead.x > 0
			: at.x === other.x && (other.y - at.y) * lead.y > 0)
	const straight = [from, to]
	if (
		(from.x === to.x || from.y === to.y) &&
		facing(from, fromLead, to) &&
		facing(to, toLead, from) &&
		!blocked(straight)
	) {
		return segmentsOf(straight)
	}

	const out = (at: Point, lead: Point | undefined): Point =>
		lead ? { x: at.x + lead.x * LEAD_OUT, y: at.y + lead.y * LEAD_OUT } : at
	const start = out(from, fromLead)
	const end = out(to, toLead)

	// In order of preference, for a tie on cost: carry on the way the first
	// lead points, then turn, then the Zs, then the detours outwards.
	const horizontalFirst = fromLead
		? fromLead.x !== 0
		: Math.abs(end.x - start.x) >= Math.abs(end.y - start.y)
	const along = [{ x: end.x, y: start.y }]
	const across = [{ x: start.x, y: end.y }]
	const middle = { x: onGrid((start.x + end.x) / 2), y: onGrid((start.y + end.y) / 2) }
	const zColumn = [
		{ x: middle.x, y: start.y },
		{ x: middle.x, y: end.y },
	]
	const zRow = [
		{ x: start.x, y: middle.y },
		{ x: end.x, y: middle.y },
	]
	const bends: Point[][] = horizontalFirst
		? [along, across, zColumn, zRow]
		: [across, along, zRow, zColumn]
	for (const reach of DETOURS) {
		for (const row of [start.y + reach, start.y - reach, end.y + reach, end.y - reach]) {
			bends.push([
				{ x: start.x, y: row },
				{ x: end.x, y: row },
			])
		}
		for (const column of [start.x + reach, start.x - reach, end.x + reach, end.x - reach]) {
			bends.push([
				{ x: column, y: start.y },
				{ x: column, y: end.y },
			])
		}
	}

	let best: { path: Point[]; cost: number } | undefined
	let fallback: { path: Point[]; cost: number } | undefined
	for (const through of bends) {
		const path = tidy([from, start, ...through, end, to])
		const cost = lengthOf(path) + BEND_COST * (path.length - 2)
		if (!fallback || cost < fallback.cost) fallback = { path, cost }
		if (doublesBack(path) || blocked(path)) continue
		if (!best || cost < best.cost) best = { path, cost }
	}
	return segmentsOf((best ?? fallback)!.path)
}

/**
 * How far out a detour goes, in world units: three, five and eight cells —
 * clear of a part's own body, whose pins reach two.
 */
const DETOURS = [3, 5, 8].map((cells) => cells * GRID_SPACING)

/** What a bend costs against length, so a straighter route wins a near tie. */
const BEND_COST = GRID_SPACING

function onGrid(value: number): number {
	return Math.round(value / GRID_SPACING) * GRID_SPACING
}

function lengthOf(points: readonly Point[]): number {
	let length = 0
	for (let index = 1; index < points.length; index++) {
		length +=
			Math.abs(points[index].x - points[index - 1].x) +
			Math.abs(points[index].y - points[index - 1].y)
	}
	return length
}

/** Whether a point lies on a segment, its ends included. */
function onSegment(segment: Segment, point: Point): boolean {
	return same(segment.from, point) || same(segment.to, point) || passesThrough(segment, point)
}

/** The points with repeats and straight-through points dropped. */
function tidy(points: readonly Point[]): Point[] {
	const kept: Point[] = []
	for (const point of points) {
		const last = kept.at(-1)
		if (last && same(last, point)) continue
		if (
			kept.length >= 2 &&
			collinear(kept[kept.length - 2], last!, point) &&
			!reverses(kept[kept.length - 2], last!, point)
		) {
			kept[kept.length - 1] = point
			continue
		}
		kept.push(point)
	}
	return kept
}

/** Whether going a → b → c turns right round, back along itself. */
function reverses(a: Point, b: Point, c: Point): boolean {
	return (b.x - a.x) * (c.x - b.x) + (b.y - a.y) * (c.y - b.y) < 0
}

/** Whether a path doubles back anywhere, which draws a wire over itself. */
function doublesBack(points: readonly Point[]): boolean {
	for (let index = 2; index < points.length; index++) {
		const [a, b, c] = [points[index - 2], points[index - 1], points[index]]
		if (collinear(a, b, c) && reverses(a, b, c)) return true
	}
	return false
}

function segmentsOf(points: readonly Point[]): Segment[] {
	const segments: Segment[] = []
	for (let index = 0; index + 1 < points.length; index++) {
		segments.push({ from: points[index], to: points[index + 1] })
	}
	return segments
}

/** One run of a net: which two pins it joins, and the wire between them. */
export interface Run {
	/** Indices into the pin list the net was routed from. */
	readonly ends: readonly [number, number]
	/** The segments, in order from the first end to the second. */
	readonly segments: readonly Segment[]
}

/**
 * Every run of a net, each one saying which pair of pins it joins.
 *
 * The pairing is what tells one stretch of a wire from another once a net joins
 * more than two pins — which stretch a current crosses, and which is a stub to
 * something that draws none (C-21). [`routeNet`] is this, flattened, for
 * everything that only wants to draw it.
 */
export function routeRuns(
	pins: readonly Point[],
	waypoints: readonly Point[] = [],
	/**
	 * Which way each pin's lead points, outward from its part, in the same
	 * order as `pins` — or nothing for a caller that does not know. With it,
	 * every derived run leaves its pins along their leads ([`routeLeaving`]).
	 * The bent run still goes exactly through its bends (ADR 0007).
	 */
	leads: readonly (Point | undefined)[] = [],
	/** Pins that are not this net's, for its wires to stay off (C-61). */
	avoid: readonly Point[] = [],
): Run[] {
	if (pins.length < 2) return []

	const bent = waypoints.length > 0
	return spanningEdges(pins, bent).map(([from, to], index) => ({
		ends: [from, to] as const,
		segments:
			bent && index === 0
				? routeThrough([pins[from], ...waypoints, pins[to]])
				: routeLeaving(pins[from], leads[from], pins[to], leads[to], avoid),
	}))
}

/**
 * Every segment of a net.
 *
 * Each pin after the first runs back to the first, which gives a net with three
 * or more pins a shared trunk and therefore a real junction rather than a
 * chain that happens to touch.
 */
export function routeNet(
	pins: readonly Point[],
	/**
	 * Points the wire has been bent through, in order (ADR 0007).
	 *
	 * They belong to the run between the first two pins, which is why that run
	 * is kept whatever the geometry says.
	 */
	waypoints: readonly Point[] = [],
): Segment[] {
	return routeRuns(pins, waypoints).flatMap((run) => [...run.segments])
}

/**
 * Which pins to join, and in what order, to draw a net with the least wire.
 *
 * A net says which pins share a potential, never which pair the user dragged
 * between — connecting three pins is one fact, not three. Something has to
 * choose what to draw, and the choice used to be a star from whichever pin the
 * netlist happened to list first. That is insertion order, not geometry: two
 * LEDs wired in parallel and then joined to a battery were drawn as two runs
 * from the battery, one exactly on top of the other, with no wire at all
 * between the LEDs the user had just linked.
 *
 * A minimum spanning tree over Manhattan distances picks the same wires a
 * person would: each pin joins its nearest neighbour, so the short link between
 * two components stays short and nothing is drawn twice. It depends only on
 * where the pins are, so the same circuit always draws the same way.
 */
function spanningEdges(
	pins: readonly Point[],
	/** Keep the run between the first two pins: it is the one that carries the bends. */
	keepFirstRun: boolean,
): [number, number][] {
	const joined = new Set<number>([0])
	const edges: [number, number][] = []

	if (keepFirstRun && pins.length > 1) {
		joined.add(1)
		edges.push([0, 1])
	}

	while (joined.size < pins.length) {
		let best: { from: number; to: number; span: number } | undefined

		for (const from of joined) {
			for (let to = 0; to < pins.length; to++) {
				if (joined.has(to)) continue
				const span = manhattan(pins[from], pins[to])
				// Ties go to the lower index, so the drawing does not depend on
				// the order a Set happens to iterate in.
				if (!best || span < best.span) best = { from, to, span }
			}
		}

		if (!best) break
		joined.add(best.to)
		edges.push([best.from, best.to])
	}

	return edges
}

/** Wire length between two points, which is what an orthogonal run costs. */
function manhattan(a: Point, b: Point): number {
	return Math.abs(a.x - b.x) + Math.abs(a.y - b.y)
}

/** An orthogonal run visiting each point in turn. */
export function routeThrough(points: readonly Point[]): Segment[] {
	const segments: Segment[] = []
	for (let index = 0; index + 1 < points.length; index++) {
		segments.push(...routeBetween(points[index], points[index + 1]))
	}
	return segments
}

/**
 * The waypoints of a wire, with any that no longer bend it dropped.
 *
 * A point in line with its neighbours changes nothing about the route and
 * cannot be seen, so leaving it there would mean a wire straightened by hand
 * keeping invisible bends — and the next drag picking one of them up instead of
 * making the bend the user asked for.
 */
export function withoutRedundant(from: Point, waypoints: readonly Point[], to: Point): Point[] {
	const kept: Point[] = []
	for (let index = 0; index < waypoints.length; index++) {
		const before = kept.at(-1) ?? from
		const after = waypoints[index + 1] ?? to
		if (!collinear(before, waypoints[index], after)) kept.push(waypoints[index])
	}
	return kept
}

function collinear(a: Point, b: Point, c: Point): boolean {
	return (a.x === b.x && b.x === c.x) || (a.y === b.y && b.y === c.y)
}

/**
 * Points that need a dot.
 *
 * A dot means "these are connected"; its absence means two wires merely cross.
 * Getting it wrong makes a schematic actively misleading, so the rule counts
 * everything arriving at a point:
 *
 * - a segment ending there counts once;
 * - a segment passing straight through counts twice, because it arrives from
 *   both sides;
 * - a pin counts as one more thing joined, which is why a pin with two wires
 *   leaving it is a junction while a bend in a wire is not.
 */
export function junctions(segments: readonly Segment[], pins: readonly Point[] = []): Point[] {
	const candidates = new Map<string, Point>()
	for (const segment of segments) {
		candidates.set(key(segment.from), segment.from)
		candidates.set(key(segment.to), segment.to)
	}

	const pinKeys = new Set(pins.map(key))
	const found: Point[] = []

	for (const [at, point] of candidates) {
		let arriving = pinKeys.has(at) ? 1 : 0
		for (const segment of segments) {
			if (same(segment.from, point) || same(segment.to, point)) arriving += 1
			else if (passesThrough(segment, point)) arriving += 2
		}
		if (arriving >= 3) found.push(point)
	}

	return found
}

/** Whether a point lies strictly between a segment's ends. */
function passesThrough(segment: Segment, point: Point): boolean {
	if (same(segment.from, point) || same(segment.to, point)) return false

	const horizontal = segment.from.y === segment.to.y && point.y === segment.from.y
	const vertical = segment.from.x === segment.to.x && point.x === segment.from.x

	if (horizontal) return between(point.x, segment.from.x, segment.to.x)
	if (vertical) return between(point.y, segment.from.y, segment.to.y)
	return false
}

function between(value: number, a: number, b: number): boolean {
	return value > Math.min(a, b) && value < Math.max(a, b)
}

function same(a: Point, b: Point): boolean {
	return a.x === b.x && a.y === b.y
}

function key(point: Point): string {
	return `${point.x},${point.y}`
}
