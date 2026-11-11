// SPDX-License-Identifier: MIT OR Apache-2.0

// The camera over the circuit: where we are looking and how closely.
//
// World coordinates are what the circuit is stored in and never change with the
// view. Screen coordinates are pixels in the canvas. Everything that has to
// survive a zoom — element positions, wire routes, hit tests — stays in world
// coordinates, and this module is the only place the two meet.

export interface Point {
	readonly x: number
	readonly y: number
}

/** World units between grid lines. Components snap to multiples of this. */
export const GRID_SPACING = 10

/**
 * Zoom bounds.
 *
 * The lower bound stops the grid becoming a grey wash; the upper stops a
 * component filling the screen with nothing around it to wire to.
 */
export const MIN_ZOOM = 0.25
export const MAX_ZOOM = 8

/** A camera position. Immutable: every change returns a new one. */
export interface Viewport {
	/** World point drawn at the canvas origin. */
	readonly pan: Point
	readonly zoom: number
}

export const INITIAL_VIEWPORT: Viewport = { pan: { x: 0, y: 0 }, zoom: 1 }

export function toWorld(viewport: Viewport, screen: Point): Point {
	return {
		x: screen.x / viewport.zoom + viewport.pan.x,
		y: screen.y / viewport.zoom + viewport.pan.y,
	}
}

export function toScreen(viewport: Viewport, world: Point): Point {
	return {
		x: (world.x - viewport.pan.x) * viewport.zoom,
		y: (world.y - viewport.pan.y) * viewport.zoom,
	}
}

export function panBy(viewport: Viewport, dxScreen: number, dyScreen: number): Viewport {
	return {
		...viewport,
		pan: {
			x: viewport.pan.x - dxScreen / viewport.zoom,
			y: viewport.pan.y - dyScreen / viewport.zoom,
		},
	}
}

/**
 * Zooms about a point on screen, leaving whatever is under it exactly where it
 * is.
 *
 * Anchoring to the cursor rather than to the canvas centre is the difference
 * between zooming feeling like a magnifier and feeling like the drawing ran
 * away. It is also where the arithmetic usually goes wrong, hence the test.
 */
export function zoomAt(viewport: Viewport, screen: Point, factor: number): Viewport {
	const zoom = clamp(viewport.zoom * factor, MIN_ZOOM, MAX_ZOOM)
	if (zoom === viewport.zoom) return viewport

	// Keep the world point under the cursor fixed: solve toWorld(new) == anchor.
	const anchor = toWorld(viewport, screen)
	return {
		zoom,
		pan: {
			x: anchor.x - screen.x / zoom,
			y: anchor.y - screen.y / zoom,
		},
	}
}

/** A rectangle in world coordinates. */
export interface Bounds {
	readonly min: Point
	readonly max: Point
}

/**
 * Room left around a circuit when the view is fitted to it, in world units.
 *
 * Two grid cells, which is exactly the reach of a pin: a circuit fitted with
 * none of this would have its outermost pins on the edge of the canvas, with
 * nowhere to drag a wire to.
 */
const MARGIN = 2 * GRID_SPACING

/**
 * A camera that shows all of `bounds` inside a canvas of `size`.
 *
 * What was missing when a circuit arrived from somewhere other than the
 * pointer (C-47). Opening a file left the camera exactly where it was, so a
 * circuit drawn anywhere but the top-left corner opened off screen and the
 * canvas looked empty — bad luck for a file the user saved while panned away,
 * and a certainty for a bundled example.
 *
 * Never zooms *in* past 1:1. A circuit of three components fitted to a large
 * canvas would otherwise arrive magnified to the point of being unrecognisable,
 * and "as big as it will go" is not the same request as "let me see it".
 */
export function fitTo(bounds: Bounds, size: { width: number; height: number }): Viewport {
	const width = bounds.max.x - bounds.min.x + 2 * MARGIN
	const height = bounds.max.y - bounds.min.y + 2 * MARGIN
	if (size.width <= 0 || size.height <= 0 || width <= 0 || height <= 0) {
		return INITIAL_VIEWPORT
	}

	const zoom = clamp(Math.min(size.width / width, size.height / height), MIN_ZOOM, 1)
	// Centred: the leftover room is split either side rather than all trailing.
	const centre = {
		x: (bounds.min.x + bounds.max.x) / 2,
		y: (bounds.min.y + bounds.max.y) / 2,
	}

	return {
		zoom,
		pan: {
			x: centre.x - size.width / (2 * zoom),
			y: centre.y - size.height / (2 * zoom),
		},
	}
}

/** Nearest grid intersection to a world point. */
export function snapToGrid(world: Point): Point {
	return {
		x: Math.round(world.x / GRID_SPACING) * GRID_SPACING,
		y: Math.round(world.y / GRID_SPACING) * GRID_SPACING,
	}
}

/** Grid cell coordinates — what a person would call "3 across, 2 down". */
export function toGrid(world: Point): Point {
	return {
		x: Math.round(world.x / GRID_SPACING),
		y: Math.round(world.y / GRID_SPACING),
	}
}

function clamp(value: number, low: number, high: number): number {
	return Math.min(Math.max(value, low), high)
}
