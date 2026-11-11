// SPDX-License-Identifier: MIT OR Apache-2.0

// A component's symbol as a small SVG, for the palette (C-128).
//
// The palette used to be words alone, and nothing connected "Resistor" in the
// list with the zigzag on the board. The connection only teaches if it is the
// same zigzag, so this reads the very shapes `render.ts` draws on the canvas
// (`symbols.ts`) and writes them out as SVG instead of canvas calls — one
// drawing, two surfaces, and a symbol that changes changes in both.
//
// The pins are drawn as the dots the board draws, because where to wire is
// half of what a symbol says.

import type { SymbolDefinition, SymbolShape } from './symbols'
import type { Point } from './viewport'

export interface GlyphShape {
	/** SVG path data, in the symbol's own coordinates. */
	readonly d: string
	/** Whether it is filled, as an LED's triangle is. */
	readonly filled: boolean
}

export interface Glyph {
	/** Framing every shape and pin, with room for the stroke. */
	readonly viewBox: string
	readonly shapes: readonly GlyphShape[]
	readonly pins: readonly Point[]
}

/** Room around the drawing, in symbol units, so a stroke on its edge is not cut. */
const MARGIN = 3

function pathOf(shape: SymbolShape): string {
	if (shape.kind === 'circle') {
		const { centre, radius } = shape
		// Two arcs, since one arc cannot end where it starts.
		return (
			`M${centre.x - radius} ${centre.y}` +
			`a${radius} ${radius} 0 1 0 ${2 * radius} 0` +
			`a${radius} ${radius} 0 1 0 ${-2 * radius} 0Z`
		)
	}
	const [first, ...rest] = shape.points
	const line = `M${first.x} ${first.y}` + rest.map((point) => `L${point.x} ${point.y}`).join('')
	return shape.kind === 'polygon' ? `${line}Z` : line
}

function extentOf(shape: SymbolShape): Point[] {
	if (shape.kind !== 'circle') return [...shape.points]
	const { centre, radius } = shape
	return [
		{ x: centre.x - radius, y: centre.y - radius },
		{ x: centre.x + radius, y: centre.y + radius },
	]
}

/** The symbol's drawing, unturned, as SVG paths framed to fit. */
export function glyphOf(symbol: SymbolDefinition): Glyph {
	// A digit's segments and an LCD's glass are part of what it looks like,
	// drawn dark as on an unpowered board.
	const all: SymbolShape[] = [...symbol.shapes, ...(symbol.segments ?? [])]
	if (symbol.glass) all.push({ kind: 'polygon', points: symbol.glass, filled: false })

	const points = [...all.flatMap(extentOf), ...symbol.pins.map((pin) => pin.at)]
	const xs = points.map((point) => point.x)
	const ys = points.map((point) => point.y)
	const left = Math.min(...xs) - MARGIN
	const top = Math.min(...ys) - MARGIN
	const width = Math.max(...xs) - Math.min(...xs) + 2 * MARGIN
	const height = Math.max(...ys) - Math.min(...ys) + 2 * MARGIN

	return {
		viewBox: `${left} ${top} ${width} ${height}`,
		shapes: all.map((shape) => ({
			d: pathOf(shape),
			filled: shape.kind === 'polygon' && shape.filled,
		})),
		pins: symbol.pins.map((pin) => pin.at),
	}
}
