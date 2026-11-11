// SPDX-License-Identifier: MIT OR Apache-2.0

// Dragging a component out of the palette and onto the canvas.
//
// A small state machine rather than the browser's drag-and-drop: native DnD has
// its own coordinate model and its own idea of what a drop target is, neither of
// which matches a canvas. This is also testable by dispatching pointer events,
// which native DnD is not.

import { halfSizeOf, SYMBOLS, type SymbolKind } from './symbols'
import { GRID_SPACING, snapToGrid, toWorld, type Point, type Viewport } from './viewport'

/** A component being carried from the palette to the canvas. */
export interface Drag {
	readonly kind: SymbolKind
	/** Where the pointer is, in client coordinates. */
	readonly at: Point
	/** Whether the pointer is currently over the canvas. */
	readonly overCanvas: boolean
}

export function startDrag(kind: SymbolKind, at: Point): Drag {
	return { kind, at, overCanvas: false }
}

export function moveDrag(drag: Drag, at: Point, canvas: DOMRect | undefined): Drag {
	return { ...drag, at, overCanvas: canvas !== undefined && contains(canvas, at) }
}

/**
 * Where a drop should place the component, or `undefined` if it should not.
 *
 * Returning nothing for a drop outside the canvas is the whole of "cancel leaves
 * the netlist untouched": there is no half-created element to undo, because
 * nothing was created.
 */
export function dropPosition(
	drag: Drag,
	canvas: DOMRect | undefined,
	viewport: Viewport,
): Point | undefined {
	if (!canvas || !contains(canvas, drag.at)) return undefined

	const screen = { x: drag.at.x - canvas.left, y: drag.at.y - canvas.top }
	// Snapped, so a component always lands on an intersection and its pins land
	// on intersections too — which is what makes wiring tractable (M-17).
	return snapToGrid(toWorld(viewport, screen))
}

/** Something already on the canvas, as far as finding room beside it goes. */
export interface Occupant {
	readonly position: Point
	/** Half its width and height, as drawn and turned. */
	readonly half: Point
}

/** Clear space kept between a placed part and its neighbours, in world units: two cells, so a label beside one does not sit on the other. */
const ROOM_AROUND = 2 * GRID_SPACING

/**
 * Where a component clicked in the palette should go: the middle of what is in
 * view, or the nearest spot to it with nothing already there (C-97).
 *
 * Clicking is how a keyboard places a part, and how a finger places one the
 * phone's canvas has scrolled away from — so it lands where the person is
 * looking, never on top of something, and always on the grid.
 */
export function clickPosition(
	kind: SymbolKind,
	canvas: { width: number; height: number },
	viewport: Viewport,
	occupants: readonly Occupant[],
): Point {
	const centre = snapToGrid(toWorld(viewport, { x: canvas.width / 2, y: canvas.height / 2 }))
	const half = halfSizeOf(SYMBOLS[kind])
	const free = (at: Point) =>
		occupants.every(
			(other) =>
				Math.abs(at.x - other.position.x) >= half.x + other.half.x + ROOM_AROUND ||
				Math.abs(at.y - other.position.y) >= half.y + other.half.y + ROOM_AROUND,
		)
	// Outwards in square rings of grid steps, nearest first: a ring at a time
	// keeps the part as close to the middle as there is room for.
	const step = 2 * GRID_SPACING
	for (let ring = 0; ring <= 25; ring++) {
		const around: Point[] = []
		for (let dx = -ring; dx <= ring; dx++) {
			for (let dy = -ring; dy <= ring; dy++) {
				if (Math.max(Math.abs(dx), Math.abs(dy)) !== ring) continue
				around.push({ x: centre.x + dx * step, y: centre.y + dy * step })
			}
		}
		around.sort((a, b) => distance(a, centre) - distance(b, centre))
		const spot = around.find(free)
		if (spot) return spot
	}
	// A canvas so full there is nowhere near: the middle, as a drop would be.
	return centre
}

function distance(a: Point, b: Point): number {
	return Math.hypot(a.x - b.x, a.y - b.y)
}

function contains(rect: DOMRect, point: Point): boolean {
	return (
		point.x >= rect.left && point.x <= rect.right && point.y >= rect.top && point.y <= rect.bottom
	)
}
