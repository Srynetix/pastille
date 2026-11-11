// SPDX-License-Identifier: MIT OR Apache-2.0

// Schematic symbols, described rather than drawn.
//
// A symbol is a list of shapes and a list of pins in symbol-local coordinates,
// origin at the centre. The renderer knows how to draw the shapes; it does not
// know what a resistor looks like. Adding a component means adding an entry
// here — no new draw calls, and the wiring code gets its anchor points for free.
//
// Pins sit two grid cells from the origin, so an item placed on an intersection
// has its pins on intersections too. Orthogonal wire routing (M-17) depends on
// that and it is much harder to retrofit than to decide now.

import type { ComponentKind } from './core'
import { GRID_SPACING, type Point } from './viewport'

/** Distance from origin to a pin: two cells. */
const PIN_REACH = 2 * GRID_SPACING

export type SymbolShape =
	| { readonly kind: 'polyline'; readonly points: readonly Point[] }
	/** Closed, and filled when the component's body is solid — an LED's triangle. */
	| { readonly kind: 'polygon'; readonly points: readonly Point[]; readonly filled: boolean }
	| { readonly kind: 'circle'; readonly centre: Point; readonly radius: number }

export interface SymbolPin {
	/** Must match the core's pin name for that element kind. */
	readonly name: string
	readonly at: Point
}

export interface SymbolDefinition {
	readonly shapes: readonly SymbolShape[]
	readonly pins: readonly SymbolPin[]
	/**
	 * The least half-size of the hit box, when the drawing alone is not enough.
	 *
	 * For a component drawn differently in two states (C-19), so that both are
	 * the same target. A closed switch is very nearly a straight line, and
	 * deriving its box from what it draws would give a strip a few pixels tall
	 * — so the click that shut it could not reach it to open it again.
	 */
	readonly extent?: Point
	/**
	 * Shapes lit one at a time, in the order the core reports their brightness
	 * — a seven-segment digit's a to g and its point (C-75). Drawn dim, over
	 * the body, and in the light's colour as each one is lit.
	 */
	readonly segments?: readonly SymbolShape[]
	/**
	 * For an LED of several dice (C-84): each die's colour, by the core's
	 * name, in the order the core reports their brightness. The glow is their
	 * light added up, so red and green lit together glow yellow.
	 */
	readonly dice?: readonly string[]
	/**
	 * For a digit made of glass rather than light (C-76): the window its
	 * segments sit in, drawn as the glass's own face, and the segments drawn
	 * dark on it as far as each has turned instead of lit.
	 */
	readonly glass?: readonly Point[]
	/**
	 * Whether the body fills the square its pins sit on, so a label in a
	 * corner goes outside it rather than into the room between two leads — a
	 * box has no such room, where an op-amp's triangle does (C-75).
	 */
	readonly boxed?: boolean
	/**
	 * How far out its pins sit, when it is not the two cells every other
	 * symbol's do (C-77). A chip with sixteen pins cannot fit them on a square
	 * two cells out, so it declares a larger one; the rules are the same on
	 * it — every pin on its edge, nothing drawn past it.
	 */
	readonly reach?: number
	/**
	 * Whether its label is written inside its body, as a chip's name is on a
	 * schematic — for a symbol with pins on every side and in every corner,
	 * where no outside place is free of wires (C-77).
	 */
	readonly labelInside?: boolean
}

/** The kinds the palette offers, and the core knows. */
export type SymbolKind = ComponentKind

const BATTERY: SymbolDefinition = {
	shapes: [
		// Leads.
		{ kind: 'polyline', points: [{ x: -PIN_REACH, y: 0 }, { x: -5, y: 0 }] },
		{ kind: 'polyline', points: [{ x: 5, y: 0 }, { x: PIN_REACH, y: 0 }] },
		// Long plate is the positive terminal, short plate the negative: the one
		// piece of schematic convention a beginner can read without being told.
		{ kind: 'polyline', points: [{ x: -5, y: -9 }, { x: -5, y: 9 }] },
		{ kind: 'polyline', points: [{ x: 5, y: -5 }, { x: 5, y: 5 }] },
	],
	pins: [
		{ name: '+', at: { x: -PIN_REACH, y: 0 } },
		{ name: '-', at: { x: PIN_REACH, y: 0 } },
	],
}

/**
 * A real battery: two cells in a row, which is the schematic sign for a
 * battery of several, beside the ideal source's one (C-121). The first long
 * plate is still its +.
 */
const REAL_BATTERY: SymbolDefinition = {
	shapes: [
		{ kind: 'polyline', points: [{ x: -PIN_REACH, y: 0 }, { x: -9, y: 0 }] },
		{ kind: 'polyline', points: [{ x: 9, y: 0 }, { x: PIN_REACH, y: 0 }] },
		{ kind: 'polyline', points: [{ x: -9, y: -9 }, { x: -9, y: 9 }] },
		{ kind: 'polyline', points: [{ x: -4, y: -5 }, { x: -4, y: 5 }] },
		{ kind: 'polyline', points: [{ x: 4, y: -9 }, { x: 4, y: 9 }] },
		{ kind: 'polyline', points: [{ x: 9, y: -5 }, { x: 9, y: 5 }] },
		{ kind: 'polyline', points: [{ x: -4, y: 0 }, { x: 4, y: 0 }] },
	],
	pins: BATTERY.pins,
}

/**
 * A solar cell: the battery's cell, with light arriving at it — the LDR's
 * two arrows, because light is what it runs on (C-117). The long plate is
 * still its +.
 */
const SOLAR_CELL: SymbolDefinition = {
	shapes: [
		...BATTERY.shapes,
		{ kind: 'polyline', points: [{ x: -14, y: -19 }, { x: -9, y: -13 }] },
		arrowhead({ x: -14, y: -19 }, { x: -7, y: -10 }, 4),
		{ kind: 'polyline', points: [{ x: -5, y: -19 }, { x: 0, y: -13 }] },
		arrowhead({ x: -5, y: -19 }, { x: 2, y: -10 }, 4),
	],
	pins: BATTERY.pins,
}

const RESISTOR: SymbolDefinition = {
	shapes: [
		{
			kind: 'polyline',
			// Zigzag rather than the IEC rectangle: this is the shape the Arduino
			// tutorials the audience is already reading use.
			points: [
				{ x: -PIN_REACH, y: 0 },
				{ x: -10, y: 0 },
				{ x: -8, y: -6 },
				{ x: -4, y: 6 },
				{ x: 0, y: -6 },
				{ x: 4, y: 6 },
				{ x: 8, y: -6 },
				{ x: 10, y: 0 },
				{ x: PIN_REACH, y: 0 },
			],
		},
	],
	pins: [
		{ name: 'a', at: { x: -PIN_REACH, y: 0 } },
		{ name: 'b', at: { x: PIN_REACH, y: 0 } },
	],
}

/**
 * A light sensor: the resistor, with light arriving at it — the LED's two
 * arrows turned round, because it takes in what an LED gives out (C-65).
 */
const LDR: SymbolDefinition = {
	shapes: [
		...RESISTOR.shapes,
		{ kind: 'polyline', points: [{ x: -9, y: -16 }, { x: -4, y: -9 }] },
		arrowhead({ x: -9, y: -16 }, { x: -2, y: -6 }, 4),
		{ kind: 'polyline', points: [{ x: -2, y: -16 }, { x: 3, y: -9 }] },
		arrowhead({ x: -2, y: -16 }, { x: 5, y: -6 }, 4),
	],
	pins: RESISTOR.pins,
}

/**
 * A thermistor: the resistor with a slash through it and a foot on the
 * slash, the schematic sign for a resistance that follows something outside
 * the circuit — here, the temperature (C-65).
 */
const THERMISTOR: SymbolDefinition = {
	shapes: [
		...RESISTOR.shapes,
		{
			kind: 'polyline',
			points: [
				{ x: -14, y: 10 },
				{ x: -10, y: 10 },
				{ x: 10, y: -10 },
			],
		},
	],
	pins: RESISTOR.pins,
}


/** A fuse's body: the IEC box, a cell and a half long, with the leads. */
const FUSE_BODY: readonly SymbolShape[] = [
	{ kind: 'polyline', points: [{ x: -PIN_REACH, y: 0 }, { x: -11, y: 0 }] },
	{ kind: 'polyline', points: [{ x: 11, y: 0 }, { x: PIN_REACH, y: 0 }] },
	{
		kind: 'polygon',
		filled: false,
		points: [
			{ x: -11, y: -5 },
			{ x: 11, y: -5 },
			{ x: 11, y: 5 },
			{ x: -11, y: 5 },
		],
	},
]

/** The same box around every state of it, so it is one target (C-19's rule). */
const FUSE_EXTENT: Point = { x: PIN_REACH, y: 6 }

/**
 * A fuse: the IEC box with its wire running through it (C-120). Whole, the
 * wire is unbroken; blown, it is two stubs with a gap between, which is what
 * a beginner holding a blown glass fuse up to the light sees.
 */
const FUSE: SymbolDefinition = {
	shapes: [...FUSE_BODY, { kind: 'polyline', points: [{ x: -11, y: 0 }, { x: 11, y: 0 }] }],
	pins: RESISTOR.pins,
	extent: FUSE_EXTENT,
}

const FUSE_BLOWN: SymbolDefinition = {
	shapes: [
		...FUSE_BODY,
		{ kind: 'polyline', points: [{ x: -11, y: 0 }, { x: -5, y: 0 }, { x: -3, y: -2 }] },
		{ kind: 'polyline', points: [{ x: 3, y: 2 }, { x: 5, y: 0 }, { x: 11, y: 0 }] },
	],
	pins: RESISTOR.pins,
	extent: FUSE_EXTENT,
}

/**
 * A polyfuse: the box with the thermistor's slash and foot across it — a
 * resistance that follows its own heat, which is what it is (C-120).
 */
const POLYFUSE: SymbolDefinition = {
	shapes: [
		...FUSE_BODY,
		{ kind: 'polyline', points: [{ x: -11, y: 0 }, { x: 11, y: 0 }] },
		{
			kind: 'polyline',
			points: [
				{ x: -14, y: 9 },
				{ x: -10, y: 9 },
				{ x: 10, y: -9 },
			],
		},
	],
	pins: RESISTOR.pins,
	extent: FUSE_EXTENT,
}

/**
 * The junction both diodes are drawn from: leads, a triangle and a bar.
 *
 * Shared rather than drawn twice, because the two components really are the
 * same part with one difference — an LED is this with light coming off it, and
 * the symbols say so by being the same shape (C-50).
 */
const JUNCTION: readonly SymbolShape[] = [
	{ kind: 'polyline', points: [{ x: -PIN_REACH, y: 0 }, { x: -6, y: 0 }] },
	{ kind: 'polyline', points: [{ x: 6, y: 0 }, { x: PIN_REACH, y: 0 }] },
	// Triangle points from anode to cathode — the direction current flows.
	{
		kind: 'polygon',
		filled: true,
		points: [
			{ x: -6, y: -7 },
			{ x: 6, y: 0 },
			{ x: -6, y: 7 },
		],
	},
	// Cathode bar: the one thing that says which way round it is, and the
	// reason neither of these two needs a word beside it.
	{ kind: 'polyline', points: [{ x: 6, y: -7 }, { x: 6, y: 7 }] },
]

/** The pins of a junction, which both of them share. */
const JUNCTION_PINS: readonly SymbolPin[] = [
	{ name: 'anode', at: { x: -PIN_REACH, y: 0 } },
	{ name: 'cathode', at: { x: PIN_REACH, y: 0 } },
]

const DIODE: SymbolDefinition = {
	shapes: JUNCTION,
	pins: JUNCTION_PINS,
}

/**
 * A Zener: the diode's triangle, with its cathode bar bent at both ends (C-62).
 *
 * The bent bar is the schematic sign, and it is the whole difference: the part
 * is a diode until it is reversed hard enough, and then it holds.
 */
const ZENER: SymbolDefinition = {
	shapes: [
		...JUNCTION.slice(0, 3),
		{
			kind: 'polyline',
			points: [
				{ x: 2, y: -9 },
				{ x: 6, y: -7 },
				{ x: 6, y: 7 },
				{ x: 10, y: 9 },
			],
		},
	],
	pins: JUNCTION_PINS,
}

/**
 * A Schottky: the diode's triangle, its cathode bar hooked at both ends into
 * the S that marks a metal-to-silicon barrier (C-114) — where a Zener's
 * bar is bent away, this one's ends curl back over themselves.
 */
const SCHOTTKY: SymbolDefinition = {
	shapes: [
		...JUNCTION.slice(0, 3),
		{
			kind: 'polyline',
			points: [
				{ x: 10, y: -5 },
				{ x: 10, y: -9 },
				{ x: 6, y: -9 },
				{ x: 6, y: 9 },
				{ x: 2, y: 9 },
				{ x: 2, y: 5 },
			],
		},
	],
	pins: JUNCTION_PINS,
}

const LED: SymbolDefinition = {
	shapes: [
		...JUNCTION,
		// The two arrows that make it a light-emitting diode rather than a diode.
		{ kind: 'polyline', points: [{ x: 0, y: -9 }, { x: 5, y: -14 }] },
		{ kind: 'polygon', filled: true, points: [{ x: 5, y: -14 }, { x: 2, y: -12 }, { x: 4, y: -10 }] },
		{ kind: 'polyline', points: [{ x: 5, y: -9 }, { x: 10, y: -14 }] },
		{ kind: 'polygon', filled: true, points: [{ x: 10, y: -14 }, { x: 7, y: -12 }, { x: 9, y: -10 }] },
	],
	pins: JUNCTION_PINS,
}

/**
 * An RGB LED: three small LEDs stacked, one per die, their shared ends joined
 * and led out to the common (C-84).
 *
 * R, G and B down the left in the order the name spells them, the common on
 * the right. Which way the triangles point is the choice: towards the common
 * on a common-cathode part, away from it on a common-anode one — so the
 * drawing says which end is shared, where the digit leaves it to its part
 * number (C-75), because here there is no part number to say it.
 */
function rgbLed(sharedCathode: boolean): SymbolDefinition {
	const rows = [-GRID_SPACING, 0, GRID_SPACING]
	const [base, point] = sharedCathode ? [-4, 4] : [4, -4]
	const shared = 8
	return {
		shapes: [
			...rows.flatMap((y): SymbolShape[] => [
				{ kind: 'polyline', points: [{ x: -PIN_REACH, y }, { x: -4, y }] },
				{
					kind: 'polygon',
					filled: true,
					points: [
						{ x: base, y: y - 3.5 },
						{ x: point, y },
						{ x: base, y: y + 3.5 },
					],
				},
				// Its cathode bar, at the point of the triangle.
				{ kind: 'polyline', points: [{ x: point, y: y - 3.5 }, { x: point, y: y + 3.5 }] },
				{ kind: 'polyline', points: [{ x: 4, y }, { x: shared, y }] },
			]),
			// The shared end, and its lead out.
			{ kind: 'polyline', points: [{ x: shared, y: -GRID_SPACING }, { x: shared, y: GRID_SPACING }] },
			{ kind: 'polyline', points: [{ x: shared, y: 0 }, { x: PIN_REACH, y: 0 }] },
			// The light's two arrows, once for the three: it is one lamp.
			{ kind: 'polyline', points: [{ x: 0, y: -15 }, { x: 5, y: -20 }] },
			{ kind: 'polygon', filled: true, points: [{ x: 5, y: -20 }, { x: 2, y: -18 }, { x: 4, y: -16 }] },
			{ kind: 'polyline', points: [{ x: 5, y: -15 }, { x: 10, y: -20 }] },
			{ kind: 'polygon', filled: true, points: [{ x: 10, y: -20 }, { x: 7, y: -18 }, { x: 9, y: -16 }] },
		],
		dice: ['Red', 'Green', 'Blue'],
		// The core's order and the core's names.
		pins: [
			{ name: 'R', at: { x: -PIN_REACH, y: -GRID_SPACING } },
			{ name: 'G', at: { x: -PIN_REACH, y: 0 } },
			{ name: 'B', at: { x: -PIN_REACH, y: GRID_SPACING } },
			{ name: 'com', at: { x: PIN_REACH, y: 0 } },
		],
	}
}

const RGB_CATHODE = rgbLed(true)
const RGB_ANODE = rgbLed(false)

const CAPACITOR: SymbolDefinition = {
	shapes: [
		// Leads up to the plates, and a gap between them: the gap is the whole
		// idea, and drawing it narrow is what says no current passes through.
		{ kind: 'polyline', points: [{ x: -PIN_REACH, y: 0 }, { x: -3, y: 0 }] },
		{ kind: 'polyline', points: [{ x: 3, y: 0 }, { x: PIN_REACH, y: 0 }] },
		{ kind: 'polyline', points: [{ x: -3, y: -9 }, { x: -3, y: 9 }] },
		// Two straight plates, not one curved: the curved one means polarised,
		// and nothing in v1 is (the pins are `a` and `b`, not `+` and `−`).
		{ kind: 'polyline', points: [{ x: 3, y: -9 }, { x: 3, y: 9 }] },
	],
	pins: [
		{ name: 'a', at: { x: -PIN_REACH, y: 0 } },
		{ name: 'b', at: { x: PIN_REACH, y: 0 } },
	],
}

/**
 * An electrolytic capacitor: the capacitor's straight plate on the + side,
 * a curved one on the −, and a + beside the straight one (C-86).
 *
 * The curved plate is the schematic sign for polarised, which is why the
 * plain capacitor keeps two straight ones; the + is there because a
 * beginner reads it where they would not yet read the curve.
 */
const ELECTROLYTIC: SymbolDefinition = {
	shapes: [
		{ kind: 'polyline', points: [{ x: -PIN_REACH, y: 0 }, { x: -3, y: 0 }] },
		{ kind: 'polyline', points: [{ x: 4, y: 0 }, { x: PIN_REACH, y: 0 }] },
		{ kind: 'polyline', points: [{ x: -3, y: -9 }, { x: -3, y: 9 }] },
		{
			kind: 'polyline',
			points: Array.from({ length: 9 }, (_, index) => {
				const y = -9 + (18 * index) / 8
				return { x: 6 - 2 * Math.sqrt(1 - (y / 9) ** 2), y }
			}),
		},
		// The +, above the lead on its side.
		{ kind: 'polyline', points: [{ x: -12, y: -8 }, { x: -6, y: -8 }] },
		{ kind: 'polyline', points: [{ x: -9, y: -11 }, { x: -9, y: -5 }] },
	],
	pins: [
		{ name: '+', at: { x: -PIN_REACH, y: 0 } },
		{ name: '-', at: { x: PIN_REACH, y: 0 } },
	],
}

const INDUCTOR: SymbolDefinition = {
	shapes: [
		{ kind: 'polyline', points: [{ x: -PIN_REACH, y: 0 }, { x: -12, y: 0 }] },
		{ kind: 'polyline', points: [{ x: 12, y: 0 }, { x: PIN_REACH, y: 0 }] },
		// Four bumps, drawn ashalf circles would be if the format had arcs; it has
		// polylines, so each bump is a short run of points. Bumps rather than
		// the IEC rectangle, for the same reason the resistor is a zigzag.
		...[-9, -3, 3, 9].map((centre) => ({
			kind: 'polyline' as const,
			points: [
				{ x: centre - 3, y: 0 },
				{ x: centre - 2, y: -5 },
				{ x: centre, y: -7 },
				{ x: centre + 2, y: -5 },
				{ x: centre + 3, y: 0 },
			],
		})),
	],
	pins: [
		{ name: 'a', at: { x: -PIN_REACH, y: 0 } },
		{ name: 'b', at: { x: PIN_REACH, y: 0 } },
	],
}

/**
 * A transformer: two coils either side of the two lines of their core, and a
 * dot at the end of each winding where the two sines rise together (C-118).
 *
 * The primary on the left, the secondary on the right, each winding's ends a
 * cell above and below the middle. Nothing is drawn across the core, because
 * nothing conducts across it.
 */
const TRANSFORMER: SymbolDefinition = (() => {
	const winding = (side: -1 | 1): SymbolShape[] => {
		const coil = 6 * side
		return [
			{ kind: 'polyline', points: [{ x: PIN_REACH * side, y: -10 }, { x: coil, y: -10 }] },
			{ kind: 'polyline', points: [{ x: PIN_REACH * side, y: 10 }, { x: coil, y: 10 }] },
			// Four bumps along the core, bulging away from it.
			...[-7.5, -2.5, 2.5, 7.5].map((centre) => ({
				kind: 'polyline' as const,
				points: [
					{ x: coil, y: centre - 2.5 },
					{ x: coil + 3 * side, y: centre - 1.8 },
					{ x: coil + 4 * side, y: centre },
					{ x: coil + 3 * side, y: centre + 1.8 },
					{ x: coil, y: centre + 2.5 },
				],
			})),
			// The dot, by the winding's first pin.
			{ kind: 'circle', centre: { x: 14 * side, y: -15 }, radius: 1.5 },
		]
	}
	return {
		shapes: [
			...winding(-1),
			...winding(1),
			{ kind: 'polyline', points: [{ x: -1.5, y: -13 }, { x: -1.5, y: 13 }] },
			{ kind: 'polyline', points: [{ x: 1.5, y: -13 }, { x: 1.5, y: 13 }] },
		],
		pins: [
			{ name: 'p1', at: { x: -PIN_REACH, y: -10 } },
			{ name: 'p2', at: { x: -PIN_REACH, y: 10 } },
			{ name: 's1', at: { x: PIN_REACH, y: -10 } },
			{ name: 's2', at: { x: PIN_REACH, y: 10 } },
		],
	}
})()

/**
 * A bridge rectifier: four diodes round a diamond, AC in at its left and
 * right corners, + out of the top and − out of the bottom (C-118).
 *
 * Every diode points up the diamond, which is the whole of how it works: from
 * whichever AC pin is higher up to +, and up from − to whichever is lower.
 */
const BRIDGE: SymbolDefinition = (() => {
	const corner = 12
	const diode = (from: Point, to: Point): SymbolShape[] => {
		const length = Math.hypot(to.x - from.x, to.y - from.y)
		const along = { x: (to.x - from.x) / length, y: (to.y - from.y) / length }
		const across = { x: -along.y, y: along.x }
		const middle = { x: (from.x + to.x) / 2, y: (from.y + to.y) / 2 }
		const at = (a: number, b: number): Point => ({
			x: middle.x + along.x * a + across.x * b,
			y: middle.y + along.y * a + across.y * b,
		})
		return [
			{ kind: 'polyline', points: [from, to] },
			{ kind: 'polygon', filled: true, points: [at(-2.5, -3), at(2.5, 0), at(-2.5, 3)] },
			{ kind: 'polyline', points: [at(2.5, -3), at(2.5, 3)] },
		]
	}
	const left = { x: -corner, y: 0 }
	const right = { x: corner, y: 0 }
	const top = { x: 0, y: -corner }
	const bottom = { x: 0, y: corner }
	return {
		shapes: [
			...diode(left, top),
			...diode(right, top),
			...diode(bottom, left),
			...diode(bottom, right),
			{ kind: 'polyline', points: [{ x: -PIN_REACH, y: 0 }, left] },
			{ kind: 'polyline', points: [right, { x: PIN_REACH, y: 0 }] },
			{ kind: 'polyline', points: [{ x: 0, y: -PIN_REACH }, top] },
			{ kind: 'polyline', points: [bottom, { x: 0, y: PIN_REACH }] },
		],
		pins: [
			{ name: 'ac1', at: { x: -PIN_REACH, y: 0 } },
			{ name: 'ac2', at: { x: PIN_REACH, y: 0 } },
			{ name: '+', at: { x: 0, y: -PIN_REACH } },
			{ name: '-', at: { x: 0, y: PIN_REACH } },
		],
	}
})()

/** How far along the track the wiper can be drawn, either side of centre. */
const TRACK_REACH = 10

/**
 * A potentiometer at one wiper position.
 *
 * The track is the resistor's own zigzag, because that is what it is, and the
 * arrow is where the wiper has got to. Built per position rather than drawn
 * once: seeing the arrow move is the feedback that the knob did something, and
 * the pins do not move with it, so nothing about the wiring notices (C-20).
 */
function potentiometer(wiper = 0.5): SymbolDefinition {
	const along = -TRACK_REACH + 2 * TRACK_REACH * Math.min(1, Math.max(0, wiper))
	return {
		shapes: [
			...RESISTOR.shapes,
			{ kind: 'polyline', points: [{ x: 0, y: -PIN_REACH }, { x: along, y: -13 }] },
			// An arrowhead on the track, which is the one piece of schematic
			// convention that says "tapped here" rather than "joined here".
			{
				kind: 'polygon',
				filled: true,
				points: [
					{ x: along, y: -5 },
					{ x: along - 3, y: -13 },
					{ x: along + 3, y: -13 },
				],
			},
		],
		pins: [
			{ name: 'a', at: { x: -PIN_REACH, y: 0 } },
			{ name: 'wiper', at: { x: 0, y: -PIN_REACH } },
			{ name: 'b', at: { x: PIN_REACH, y: 0 } },
		],
	}
}

/** Where a contact's two terminals sit, either side of the gap. */
const CONTACT = 8
/** How big a dot marks a terminal. */
const CONTACT_DOT = 2
/**
 * The hit box both states of a contact share.
 *
 * Declared rather than derived, because a shut contact draws almost nothing
 * above the axis and a hit box that shrinks when the switch closes is a switch
 * that cannot be opened again.
 */
const CONTACT_EXTENT: Point = { x: PIN_REACH, y: 13 }

/** Leads and terminals, which are the same whichever way the contact is. */
const CONTACT_BODY: readonly SymbolShape[] = [
	{ kind: 'polyline', points: [{ x: -PIN_REACH, y: 0 }, { x: -CONTACT, y: 0 }] },
	{ kind: 'polyline', points: [{ x: CONTACT, y: 0 }, { x: PIN_REACH, y: 0 }] },
	{ kind: 'circle', centre: { x: -CONTACT, y: 0 }, radius: CONTACT_DOT },
	{ kind: 'circle', centre: { x: CONTACT, y: 0 }, radius: CONTACT_DOT },
]

const CONTACT_PINS: readonly SymbolPin[] = [
	{ name: 'a', at: { x: -PIN_REACH, y: 0 } },
	{ name: 'b', at: { x: PIN_REACH, y: 0 } },
]

// The knife switch, hinged on its left terminal: lifted off is open, lying
// across both terminals is closed. Two drawings of one component, and the only
// thing on the canvas that says which way a switch is — there is no label
// (C-19), because the picture says it in every language.
const SWITCH_OPEN: SymbolDefinition = {
	shapes: [...CONTACT_BODY, { kind: 'polyline', points: [{ x: -CONTACT, y: 0 }, { x: 6, y: -11 }] }],
	pins: CONTACT_PINS,
	extent: CONTACT_EXTENT,
}

const SWITCH_CLOSED: SymbolDefinition = {
	shapes: [
		...CONTACT_BODY,
		{ kind: 'polyline', points: [{ x: -CONTACT, y: 0 }, { x: CONTACT, y: 0 }] },
	],
	pins: CONTACT_PINS,
	extent: CONTACT_EXTENT,
}

/** The bar, the plunger and the cap, at whatever height the button is at. */
function button(bar: number): readonly SymbolShape[] {
	return [
		...CONTACT_BODY,
		// The bar is what bridges the two terminals, so pressed it sits on them
		// and released it hovers above them. The travel is the whole symbol.
		{ kind: 'polyline', points: [{ x: -11, y: bar }, { x: 11, y: bar }] },
		{ kind: 'polyline', points: [{ x: 0, y: bar }, { x: 0, y: bar - 7 }] },
		{ kind: 'polyline', points: [{ x: -7, y: bar - 7 }, { x: 7, y: bar - 7 }] },
	]
}

const BUTTON_UP: SymbolDefinition = {
	shapes: button(-6),
	pins: CONTACT_PINS,
	extent: CONTACT_EXTENT,
}

const BUTTON_DOWN: SymbolDefinition = {
	shapes: button(0),
	pins: CONTACT_PINS,
	extent: CONTACT_EXTENT,
}

/**
 * A ball tilt switch, drawn as what it is: a little can standing on its two
 * legs with a ball inside (C-88).
 *
 * Upright, the ball rests at the bottom across both contacts and the circuit
 * is shut; tipped, the can leans over and the ball has rolled to its far end,
 * away from them. Which way up it is is the state, so the drawing says it
 * with no word beside it, as the switch's lifted blade does (C-19).
 */
function tilt(upright: boolean): SymbolDefinition {
	// The can, about its foot at the centre, leaning 40° when tipped.
	const lean = upright ? 0 : (40 * Math.PI) / 180
	const at = (x: number, y: number): Point => ({
		x: x * Math.cos(lean) - y * Math.sin(lean),
		y: x * Math.sin(lean) + y * Math.cos(lean),
	})
	const can: Point[] = [at(-6, 1), at(-6, -15), at(6, -15), at(6, 1)]
	return {
		shapes: [
			...CONTACT_BODY,
			{ kind: 'polygon', filled: false, points: can },
			// The ball: across the contacts, or rolled to the top of the can.
			{ kind: 'circle', centre: upright ? { x: 0, y: -3 } : at(0, -11), radius: 3.5 },
		],
		pins: CONTACT_PINS,
		extent: { x: PIN_REACH, y: 16 },
	}
}

const TILT_TIPPED: SymbolDefinition = tilt(false)
const TILT_UPRIGHT: SymbolDefinition = tilt(true)

/**
 * A relay: its coil on the left, its contact on the right (C-64).
 *
 * The coil is a box between its two pins; the contact a blade pivoting up from
 * COM that rests against NC and, while the coil is energised, is drawn against
 * NO instead — the one symbol here whose drawing follows the run rather than
 * the user, because nobody flips a relay: its coil does.
 */
function relay(energised: boolean): SymbolDefinition {
	const pivot = { x: 6, y: 10 }
	const normallyClosed = { x: 12, y: 0 }
	const normallyOpen = { x: 10, y: -10 }
	return {
		shapes: [
			// The coil and its leads.
			{
				kind: 'polyline',
				points: [
					{ x: -PIN_REACH, y: -GRID_SPACING },
					{ x: -10, y: -GRID_SPACING },
					{ x: -10, y: -6 },
				],
			},
			{
				kind: 'polyline',
				points: [
					{ x: -PIN_REACH, y: GRID_SPACING },
					{ x: -10, y: GRID_SPACING },
					{ x: -10, y: 6 },
				],
			},
			{
				kind: 'polygon',
				filled: false,
				points: [
					{ x: -14, y: -6 },
					{ x: -6, y: -6 },
					{ x: -6, y: 6 },
					{ x: -14, y: 6 },
				],
			},
			// What the coil pulls on.
			{ kind: 'polyline', points: [{ x: -6, y: 0 }, { x: 2, y: 0 }] },
			// The contacts, and COM's lead up to the pivot.
			{ kind: 'polyline', points: [{ x: 0, y: PIN_REACH }, pivot] },
			{ kind: 'polyline', points: [normallyClosed, { x: PIN_REACH, y: 0 }] },
			{ kind: 'polyline', points: [normallyOpen, { x: PIN_REACH, y: -GRID_SPACING }] },
			// The blade, against whichever it is touching.
			{ kind: 'polyline', points: [pivot, energised ? normallyOpen : normallyClosed] },
		],
		// Coil, then COM, NO, NC — the core's order and the core's names.
		pins: [
			{ name: 'coil+', at: { x: -PIN_REACH, y: -GRID_SPACING } },
			{ name: 'coil-', at: { x: -PIN_REACH, y: GRID_SPACING } },
			{ name: 'com', at: { x: 0, y: PIN_REACH } },
			{ name: 'no', at: { x: PIN_REACH, y: -GRID_SPACING } },
			{ name: 'nc', at: { x: PIN_REACH, y: 0 } },
		],
	}
}

const RELAY_AT_REST: SymbolDefinition = relay(false)
const RELAY_ENERGISED: SymbolDefinition = relay(true)

/**
 * A buzzer: a dome on a bar, and two arcs of sound above it while it sounds
 * (C-66). Its + is on the left, the leg a buzzer marks.
 *
 * A passive one has a wave drawn in its dome (C-85): it plays the wave it is
 * fed and has none of its own, which is the one thing to know before swapping
 * it for the active kind.
 */
function buzzer(sounding: boolean, passive = false): SymbolDefinition {
	const dome: Point[] = Array.from({ length: 9 }, (_, index) => {
		const angle = Math.PI * (index / 8)
		return { x: -10 * Math.cos(angle), y: -10 * Math.sin(angle) }
	})
	const arc = (radius: number): SymbolShape => ({
		kind: 'polyline',
		points: Array.from({ length: 5 }, (_, index) => {
			const angle = Math.PI * (0.3 + 0.4 * (index / 4))
			return { x: -radius * Math.cos(angle), y: -radius * Math.sin(angle) }
		}),
	})
	return {
		shapes: [
			{ kind: 'polyline', points: [{ x: -PIN_REACH, y: 0 }, { x: -10, y: 0 }] },
			{ kind: 'polyline', points: [{ x: 10, y: 0 }, { x: PIN_REACH, y: 0 }] },
			{ kind: 'polyline', points: [{ x: -10, y: 0 }, { x: 10, y: 0 }] },
			{ kind: 'polyline', points: dome },
			...(passive
				? [
						{
							kind: 'polyline' as const,
							points: Array.from({ length: 9 }, (_, index) => ({
								x: -5 + (10 * index) / 8,
								y: -4 - 2 * Math.sin((2 * Math.PI * index) / 8),
							})),
						},
					]
				: []),
			...(sounding ? [arc(14), arc(18)] : []),
		],
		pins: [
			{ name: '+', at: { x: -PIN_REACH, y: 0 } },
			{ name: '-', at: { x: PIN_REACH, y: 0 } },
		],
	}
}

const BUZZER_QUIET: SymbolDefinition = buzzer(false)
const BUZZER_SOUNDING: SymbolDefinition = buzzer(true)
const PASSIVE_QUIET: SymbolDefinition = buzzer(false, true)
const PASSIVE_SOUNDING: SymbolDefinition = buzzer(true, true)

/**
 * A motor: a dial with an M in it, and a curved arrow over it while it turns
 * (C-66) — pointing the other way while it turns backwards (C-91).
 */
function motor(turning: boolean, backwards = false): SymbolDefinition {
	const way = backwards ? -1 : 1
	const turn: Point[] = Array.from({ length: 7 }, (_, index) => {
		const angle = Math.PI * (0.25 + 0.5 * (index / 6))
		return { x: -14 * way * Math.cos(angle), y: -14 * Math.sin(angle) }
	})
	return {
		shapes: [
			{ kind: 'polyline', points: [{ x: -PIN_REACH, y: 0 }, { x: -9, y: 0 }] },
			{ kind: 'polyline', points: [{ x: 9, y: 0 }, { x: PIN_REACH, y: 0 }] },
			{ kind: 'circle', centre: { x: 0, y: 0 }, radius: 9 },
			{
				kind: 'polyline',
				points: [
					{ x: -4, y: 4 },
					{ x: -4, y: -4 },
					{ x: 0, y: 1 },
					{ x: 4, y: -4 },
					{ x: 4, y: 4 },
				],
			},
			...(turning
				? [
						{ kind: 'polyline', points: turn } as SymbolShape,
						arrowhead(turn[turn.length - 2], turn[turn.length - 1], 4),
					]
				: []),
		],
		pins: [
			{ name: 'a', at: { x: -PIN_REACH, y: 0 } },
			{ name: 'b', at: { x: PIN_REACH, y: 0 } },
		],
	}
}

const MOTOR_STILL: SymbolDefinition = motor(false)
const MOTOR_TURNING: SymbolDefinition = motor(true)
const MOTOR_BACKWARDS: SymbolDefinition = motor(true, true)

/** Radius of a meter's dial. */
const DIAL = 9

/**
 * A meter: a dial with a letter in it, and leads either side.
 *
 * The letter is drawn rather than written, because a symbol is a list of
 * shapes and the format has no text in it — which is no hardship for a V and
 * an A, and keeps every symbol one kind of thing.
 */
function meter(letter: readonly SymbolShape[]): SymbolDefinition {
	return {
		shapes: [
			{ kind: 'polyline', points: [{ x: -PIN_REACH, y: 0 }, { x: -DIAL, y: 0 }] },
			{ kind: 'polyline', points: [{ x: DIAL, y: 0 }, { x: PIN_REACH, y: 0 }] },
			{ kind: 'circle', centre: { x: 0, y: 0 }, radius: DIAL },
			...letter,
		],
		// `+` and `-`: a meter put on backwards reads a negative number rather
		// than quietly agreeing, which is the whole of what polarity means.
		pins: [
			{ name: '+', at: { x: -PIN_REACH, y: 0 } },
			{ name: '-', at: { x: PIN_REACH, y: 0 } },
		],
	}
}

const VOLTMETER: SymbolDefinition = meter([
	{ kind: 'polyline', points: [{ x: -4, y: -4 }, { x: 0, y: 4 }, { x: 4, y: -4 }] },
])

const AMMETER: SymbolDefinition = meter([
	{ kind: 'polyline', points: [{ x: -4, y: 4 }, { x: 0, y: -4 }, { x: 4, y: 4 }] },
	{ kind: 'polyline', points: [{ x: -2, y: 1 }, { x: 2, y: 1 }] },
])

/**
 * A signal generator: a dial, like the meters, with its waveform drawn in it
 * (ADR 0011).
 *
 * The circle with a wave in it is the schematic sign for a source that
 * alternates, and drawing the actual shape — rather than always a sine —
 * means the canvas says which one is set without a word beside it. Its pins
 * are the meters' `+` and `−`, which are a battery's too.
 */
const SIGNAL_SQUARE: SymbolDefinition = meter([
	{
		kind: 'polyline',
		points: [
			{ x: -5, y: 3 },
			{ x: -5, y: -3 },
			{ x: 0, y: -3 },
			{ x: 0, y: 3 },
			{ x: 5, y: 3 },
			{ x: 5, y: -3 },
		],
	},
])

const SIGNAL_SINE: SymbolDefinition = meter([
	{
		kind: 'polyline',
		points: Array.from({ length: 13 }, (_, index) => {
			const x = -6 + index
			return { x, y: -3.5 * Math.sin((Math.PI * x) / 6) }
		}),
	},
])

const SIGNAL_TRIANGLE: SymbolDefinition = meter([
	{
		kind: 'polyline',
		points: [
			{ x: -6, y: 0 },
			{ x: -3, y: -3.5 },
			{ x: 3, y: 3.5 },
			{ x: 6, y: 0 },
		],
	},
])

/** Where the base bar stands, and how far up and down it runs. */
const BASE_BAR = { x: -8, y: 9 }

/** Where the two slanted leads meet the bar, and where they straighten out. */
const LEAD_JOINT = 4
const BEND = 12

/**
 * A filled arrowhead sitting on a lead, pointing along it.
 *
 * The one piece of a transistor that carries meaning rather than shape: which
 * way it points is the whole difference between an NPN and a PNP, and it points
 * the way conventional current runs through the emitter.
 */
function arrowhead(from: Point, to: Point, size = 6): SymbolShape {
	const run = { x: to.x - from.x, y: to.y - from.y }
	const length = Math.hypot(run.x, run.y)
	const along = { x: run.x / length, y: run.y / length }
	const across = { x: -along.y, y: along.x }
	// Two thirds of the way along the lead rather than at its end: at the end
	// it would sit on the bend, where it reads as a kink in the wire.
	const tip = { x: from.x + along.x * length * 0.72, y: from.y + along.y * length * 0.72 }
	const back = { x: tip.x - along.x * size, y: tip.y - along.y * size }

	return {
		kind: 'polygon',
		filled: true,
		points: [
			tip,
			{ x: back.x + across.x * size * 0.4, y: back.y + across.y * size * 0.4 },
			{ x: back.x - across.x * size * 0.4, y: back.y - across.y * size * 0.4 },
		],
	}
}

/**
 * A bipolar transistor, base to the left, collector up and emitter down.
 *
 * Drawn without a circle around it, unlike the meters: the circle is optional
 * in every standard and leaving it off keeps a transistor from reading as
 * another instrument at a glance. Falstad's, the UX benchmark of the prior-art study,
 * has none either.
 *
 * `outward` is what makes it an NPN — the arrow leaves the base, because that
 * is the way current runs through the emitter. A PNP is the same drawing with
 * the arrow turned round, which is exactly how the two differ in the part.
 */
function transistor(outward: boolean): SymbolDefinition {
	const base = { x: BASE_BAR.x, y: 0 }
	const collectorJoint = { x: BASE_BAR.x, y: -LEAD_JOINT }
	const emitterJoint = { x: BASE_BAR.x, y: LEAD_JOINT }
	const collectorBend = { x: 0, y: -BEND }
	const emitterBend = { x: 0, y: BEND }

	return {
		shapes: [
			// Base lead, and the bar it lands on.
			{ kind: 'polyline', points: [{ x: -PIN_REACH, y: 0 }, base] },
			{
				kind: 'polyline',
				points: [
					{ x: BASE_BAR.x, y: -BASE_BAR.y },
					{ x: BASE_BAR.x, y: BASE_BAR.y },
				],
			},
			// Collector: slanted off the bar, then straight up to its pin.
			{ kind: 'polyline', points: [collectorJoint, collectorBend] },
			{ kind: 'polyline', points: [collectorBend, { x: 0, y: -PIN_REACH }] },
			// Emitter: the same, down, and carrying the arrow.
			{ kind: 'polyline', points: [emitterJoint, emitterBend] },
			{ kind: 'polyline', points: [emitterBend, { x: 0, y: PIN_REACH }] },
			outward
				? arrowhead(emitterJoint, emitterBend)
				: arrowhead(emitterBend, emitterJoint),
		],
		// Base, collector, emitter — the core's order and the core's names.
		pins: [
			{ name: 'b', at: { x: -PIN_REACH, y: 0 } },
			{ name: 'c', at: { x: 0, y: -PIN_REACH } },
			{ name: 'e', at: { x: 0, y: PIN_REACH } },
		],
	}
}

/**
 * A phototransistor: the NPN, with no base lead and two arrows of light
 * falling on its base instead (C-89).
 *
 * Its base is there and is not a pin — light is what drives it — so the
 * drawing keeps the bar the light lands on and drops the wire to it, which
 * is also how a beginner tells it from the transistor beside it in a kit.
 */
const PHOTOTRANSISTOR: SymbolDefinition = (() => {
	const npn = transistor(true)
	const towards = (from: Point, to: Point): SymbolShape[] => [
		{ kind: 'polyline', points: [from, to] },
		arrowhead(from, to, 4),
	]
	return {
		// The transistor's shapes but its base lead, which is the first.
		shapes: [
			...npn.shapes.slice(1),
			...towards({ x: -19, y: -13 }, { x: -10, y: -5 }),
			...towards({ x: -19, y: -4 }, { x: -10, y: 4 }),
		],
		// Collector, emitter — the core's order and the core's names.
		pins: npn.pins.filter((pin) => pin.name !== 'b'),
	}
})()

/**
 * An enhancement MOSFET: the gate a plate apart from the channel, gate to the
 * left, drain up and source down, where a transistor has its base, collector
 * and emitter (C-63).
 *
 * The gap between the gate and the channel is the point: nothing crosses it,
 * which is why the gate takes no current. The channel is drawn broken in three,
 * the sign of a part that is off until its gate turns it on. The arrow on the
 * middle piece points into the channel for N and out of it for P, and is the
 * one difference between the two drawings, as it is between NPN and PNP.
 */
function mosfet(inward: boolean): SymbolDefinition {
	const plate = -8
	const channel = -4
	return {
		shapes: [
			// Gate lead and plate.
			{ kind: 'polyline', points: [{ x: -PIN_REACH, y: 0 }, { x: plate, y: 0 }] },
			{ kind: 'polyline', points: [{ x: plate, y: -9 }, { x: plate, y: 9 }] },
			// The channel, broken in three.
			{ kind: 'polyline', points: [{ x: channel, y: -10 }, { x: channel, y: -5 }] },
			{ kind: 'polyline', points: [{ x: channel, y: -2 }, { x: channel, y: 2 }] },
			{ kind: 'polyline', points: [{ x: channel, y: 5 }, { x: channel, y: 10 }] },
			// Drain and source leave the ends of the channel.
			{ kind: 'polyline', points: [{ x: channel, y: -8 }, { x: 0, y: -8 }, { x: 0, y: -PIN_REACH }] },
			{ kind: 'polyline', points: [{ x: channel, y: 8 }, { x: 0, y: 8 }, { x: 0, y: PIN_REACH }] },
			// The body, tied to the source, and its arrow.
			{ kind: 'polyline', points: [{ x: channel, y: 0 }, { x: 0, y: 0 }, { x: 0, y: 8 }] },
			inward
				? arrowhead({ x: 2, y: 0 }, { x: channel, y: 0 }, 4)
				: arrowhead({ x: channel, y: 0 }, { x: 2, y: 0 }, 4),
		],
		// Gate, drain, source — the core's order and the core's names.
		pins: [
			{ name: 'g', at: { x: -PIN_REACH, y: 0 } },
			{ name: 'd', at: { x: 0, y: -PIN_REACH } },
			{ name: 's', at: { x: 0, y: PIN_REACH } },
		],
	}
}

/** Half the width and the height of a servo's body. */
const SERVO_BODY = { x: 12, y: 13 }

/**
 * A hobby servo: its box, its three wires on the left in the order its plug
 * has them, and its horn on top, drawn at the angle it has turned to (C-90).
 *
 * The horn sweeps the upper half of a circle about its shaft: 0° points
 * right, 90° straight up, 180° left — one end of its travel, the middle, the
 * other. Drawn from the run, as a potentiometer's wiper is drawn from its
 * knob, so it is seen turning at its own speed rather than told.
 */
function servo(angle = 90): SymbolDefinition {
	const { x: wide, y: tall } = SERVO_BODY
	const radians = (Math.min(180, Math.max(0, angle)) * Math.PI) / 180
	const shaft = { x: 2, y: -2 }
	const reach = 9
	const tip = { x: shaft.x + reach * Math.cos(radians), y: shaft.y - reach * Math.sin(radians) }
	const across = { x: Math.sin(radians) * 2, y: Math.cos(radians) * 2 }
	return {
		shapes: [
			{
				kind: 'polygon',
				filled: false,
				points: [
					{ x: -wide, y: -tall },
					{ x: wide, y: -tall },
					{ x: wide, y: tall },
					{ x: -wide, y: tall },
				],
			},
			...[-GRID_SPACING, 0, GRID_SPACING].map(
				(y): SymbolShape => ({ kind: 'polyline', points: [{ x: -PIN_REACH, y }, { x: -wide, y }] }),
			),
			// The horn: tapered from its shaft to its tip, with the shaft's hub.
			{
				kind: 'polygon',
				filled: true,
				points: [
					{ x: shaft.x + across.x, y: shaft.y + across.y },
					tip,
					{ x: shaft.x - across.x, y: shaft.y - across.y },
				],
			},
			{ kind: 'circle', centre: shaft, radius: 2.5 },
		],
		boxed: true,
		// Signal, +5V, GND — the core's order and the core's names.
		pins: [
			{ name: 'SIG', at: { x: -PIN_REACH, y: -GRID_SPACING } },
			{ name: '+5V', at: { x: -PIN_REACH, y: 0 } },
			{ name: 'GND', at: { x: -PIN_REACH, y: GRID_SPACING } },
		],
	}
}

const NMOS: SymbolDefinition = mosfet(true)
const PMOS: SymbolDefinition = mosfet(false)

/**
 * A power MOSFET: the N-channel, and beside it the body diode every power
 * part's datasheet draws — source to drain, pointing up — which its model
 * has and the small ones' do not (C-112).
 */
const POWER_NMOS: SymbolDefinition = {
	...NMOS,
	shapes: [
		...NMOS.shapes,
		{ kind: 'polyline', points: [{ x: 0, y: 8 }, { x: 8, y: 8 }, { x: 8, y: 3 }] },
		{
			kind: 'polygon',
			filled: false,
			points: [
				{ x: 5, y: 3 },
				{ x: 11, y: 3 },
				{ x: 8, y: -2 },
			],
		},
		{ kind: 'polyline', points: [{ x: 5, y: -2 }, { x: 11, y: -2 }] },
		{ kind: 'polyline', points: [{ x: 8, y: -2 }, { x: 8, y: -8 }, { x: 0, y: -8 }] },
	],
}

const NPN: SymbolDefinition = transistor(true)
const PNP: SymbolDefinition = transistor(false)

/**
 * A Darlington: the NPN, its base fed by a small first transistor whose
 * emitter, with its arrow, lands on the big one's base bar and whose
 * collector joins the big one's — the pair a TIP120 is (C-112).
 */
const DARLINGTON: SymbolDefinition = {
	...NPN,
	shapes: [
		// The NPN's shapes but its base lead, which the first transistor
		// takes over.
		...NPN.shapes.slice(1),
		{ kind: 'polyline', points: [{ x: -PIN_REACH, y: 0 }, { x: -15, y: 0 }] },
		{ kind: 'polyline', points: [{ x: -15, y: -4 }, { x: -15, y: 4 }] },
		{ kind: 'polyline', points: [{ x: -15, y: -2 }, { x: -12, y: -6 }, { x: -12, y: -16 }, { x: 0, y: -16 }] },
		{ kind: 'polyline', points: [{ x: -15, y: 2 }, { x: -11, y: 6 }, { x: BASE_BAR.x, y: 6 }] },
		arrowhead({ x: -15, y: 2 }, { x: -11, y: 6 }, 3),
	],
}

/** Half the width and half the height of a regulator's body. */
const REGULATOR_BODY = { x: 12, y: 8 }

/**
 * A linear regulator: a box, in on the left, out on the right, ground below.
 *
 * The schematic convention for an integrated circuit, and the honest drawing
 * of one: what is inside is not a circuit a beginner could draw, and ADR 0010
 * models it from its datasheet rather than pretending otherwise. Which part it
 * is goes beside it — the core writes the part number there — because the
 * format has no text and a 7805 and an LM1117 are the same shape of box.
 *
 * The pins run the way the current does, left to right and down to ground, so
 * a regulator dropped between a battery and a load reads as a thing in line.
 */
const REGULATOR: SymbolDefinition = {
	shapes: [
		{ kind: 'polyline', points: [{ x: -PIN_REACH, y: 0 }, { x: -REGULATOR_BODY.x, y: 0 }] },
		{ kind: 'polyline', points: [{ x: REGULATOR_BODY.x, y: 0 }, { x: PIN_REACH, y: 0 }] },
		{ kind: 'polyline', points: [{ x: 0, y: REGULATOR_BODY.y }, { x: 0, y: PIN_REACH }] },
		{
			kind: 'polygon',
			filled: false,
			points: [
				{ x: -REGULATOR_BODY.x, y: -REGULATOR_BODY.y },
				{ x: REGULATOR_BODY.x, y: -REGULATOR_BODY.y },
				{ x: REGULATOR_BODY.x, y: REGULATOR_BODY.y },
				{ x: -REGULATOR_BODY.x, y: REGULATOR_BODY.y },
			],
		},
	],
	// In, ground, out — the core's order and the core's names.
	pins: [
		{ name: 'in', at: { x: -PIN_REACH, y: 0 } },
		{ name: 'gnd', at: { x: 0, y: PIN_REACH } },
		{ name: 'out', at: { x: PIN_REACH, y: 0 } },
	],
}

/** Half the width and half the height of a TMP36's body. */
/**
 * An LM317: the regulator's box with the arrow across it that marks anything
 * adjustable, as a variable resistor's does, and its middle leg ADJ rather
 * than a ground (C-113).
 */
const ADJUSTABLE: SymbolDefinition = {
	...REGULATOR,
	shapes: [
		...REGULATOR.shapes,
		{ kind: 'polyline', points: [{ x: -9, y: 7 }, { x: 9, y: -7 }] },
		arrowhead({ x: -9, y: 7 }, { x: 9, y: -7 }, 4),
	],
	pins: REGULATOR.pins.map((pin) => (pin.name === 'gnd' ? { ...pin, name: 'adj' } : pin)),
}

const SENSOR_BODY = { x: 10, y: 10 }

/**
 * A TMP36: the regulator's box, supply above, ground below and the output on
 * the right, with a thermometer drawn in it (C-87).
 *
 * The box because it is a chip; the thermometer because a box alone says
 * nothing about what it does, and the part number beside it is a name, not a
 * clue. The output on the right like every part here whose output goes on
 * to something.
 */
const TMP36: SymbolDefinition = {
	shapes: [
		{ kind: 'polyline', points: [{ x: 0, y: -PIN_REACH }, { x: 0, y: -SENSOR_BODY.y }] },
		{ kind: 'polyline', points: [{ x: 0, y: SENSOR_BODY.y }, { x: 0, y: PIN_REACH }] },
		{ kind: 'polyline', points: [{ x: SENSOR_BODY.x, y: 0 }, { x: PIN_REACH, y: 0 }] },
		{
			kind: 'polygon',
			filled: false,
			points: [
				{ x: -SENSOR_BODY.x, y: -SENSOR_BODY.y },
				{ x: SENSOR_BODY.x, y: -SENSOR_BODY.y },
				{ x: SENSOR_BODY.x, y: SENSOR_BODY.y },
				{ x: -SENSOR_BODY.x, y: SENSOR_BODY.y },
			],
		},
		// The thermometer: a stem, and the bulb it rises from.
		{ kind: 'polyline', points: [{ x: 0, y: -6 }, { x: 0, y: 2 }] },
		{ kind: 'circle', centre: { x: 0, y: 4.5 }, radius: 2.5 },
	],
	boxed: true,
	// +Vs, Vout, GND — the core's order and the core's names.
	pins: [
		{ name: '+Vs', at: { x: 0, y: -PIN_REACH } },
		{ name: 'Vout', at: { x: PIN_REACH, y: 0 } },
		{ name: 'GND', at: { x: 0, y: PIN_REACH } },
	],
}

/**
 * A reed switch: two blades sealed in a glass envelope, overlapping but apart
 * while it is open and lying together once a magnet has drawn them shut
 * (C-115). No hinge and no lever, because nothing outside moves.
 */
function reed(shut: boolean): SymbolDefinition {
	const glass = Array.from({ length: 16 }, (_, step) => {
		const angle = (2 * Math.PI * step) / 16
		return { x: 14 * Math.cos(angle), y: 6 * Math.sin(angle) }
	})
	const lift = shut ? 0 : 2
	return {
		shapes: [
			{ kind: 'polyline', points: [{ x: -PIN_REACH, y: 0 }, { x: -10, y: 0 }, { x: 3, y: -lift }] },
			{ kind: 'polyline', points: [{ x: PIN_REACH, y: 0 }, { x: 10, y: 0 }, { x: -3, y: lift }] },
			{ kind: 'polygon', filled: false, points: glass },
		],
		pins: CONTACT_PINS,
		extent: CONTACT_EXTENT,
	}
}

const REED_OPEN = reed(false)
const REED_SHUT = reed(true)

/**
 * An A3144: the TMP36's box, supply above, ground below and its output on the
 * right, with a Hall plate drawn in it — a square crossed corner to corner, as
 * a schematic draws the element that feels the field (C-115).
 */
const HALL: SymbolDefinition = {
	shapes: [
		...TMP36.shapes.slice(0, 4),
		{
			kind: 'polygon',
			filled: false,
			points: [
				{ x: -5, y: -5 },
				{ x: 5, y: -5 },
				{ x: 5, y: 5 },
				{ x: -5, y: 5 },
			],
		},
		{ kind: 'polyline', points: [{ x: -5, y: -5 }, { x: 5, y: 5 }] },
		{ kind: 'polyline', points: [{ x: 5, y: -5 }, { x: -5, y: 5 }] },
	],
	boxed: true,
	// VCC, GND, OUT — the core's order and the core's names.
	pins: [
		{ name: 'VCC', at: { x: 0, y: -PIN_REACH } },
		{ name: 'GND', at: { x: 0, y: PIN_REACH } },
		{ name: 'OUT', at: { x: PIN_REACH, y: 0 } },
	],
}

/**
 * An HC-SR501: the sensor box, supply above, ground below and its output on
 * the right, with its white Fresnel dome drawn in it (C-116); while someone
 * moves in front of it, two marks of movement beside the dome.
 */
function pir(moving: boolean): SymbolDefinition {
	const dome = Array.from({ length: 9 }, (_, step) => {
		const angle = Math.PI + (Math.PI * step) / 8
		return { x: 5 * Math.cos(angle), y: 3 + 5 * Math.sin(angle) }
	})
	const marks: SymbolShape[] = moving
		? [
				{ kind: 'polyline', points: [{ x: -8, y: -6 }, { x: -6, y: -8 }] },
				{ kind: 'polyline', points: [{ x: 6, y: -8 }, { x: 8, y: -6 }] },
			]
		: []
	return {
		shapes: [...TMP36.shapes.slice(0, 4), { kind: 'polygon', filled: false, points: dome }, ...marks],
		boxed: true,
		// VCC, OUT, GND — the core's order and the core's names.
		pins: [
			{ name: 'VCC', at: { x: 0, y: -PIN_REACH } },
			{ name: 'OUT', at: { x: PIN_REACH, y: 0 } },
			{ name: 'GND', at: { x: 0, y: PIN_REACH } },
		],
	}
}

/** The op-amp's triangle: its back edge, and how far it reaches either way. */
const TRIANGLE = { back: -12, tip: 16, half: 16 }

/** Where the triangle's slanted edge crosses the vertical through the centre. */
const TRIANGLE_AT_CENTRE = TRIANGLE.half * (TRIANGLE.tip / (TRIANGLE.tip - TRIANGLE.back))

/** Where the two inputs meet the triangle, one cell either side of centre. */
const INPUT_OFFSET = GRID_SPACING

/**
 * An amplifier's triangle — inputs on its back, output at its point, supply
 * above and below — with whatever marks the part inside it (C-55).
 *
 * The inverting input on top, marked −, because that is where feedback is
 * drawn from the output in every textbook stage. The supply pins are drawn,
 * which most schematics leave off: they are the pins a beginner forgets, and a
 * drawing that hides them teaches an op-amp that runs on nothing (ADR 0010).
 *
 * Five pins, so two share a side: the inputs sit one cell either side of the
 * centre of the back edge. Still on intersections, still on the edge of the
 * two-cell square — which is what the pin rule is for.
 */
function amplifier(inside: readonly SymbolShape[]): SymbolDefinition {
	return {
	shapes: [
		{
			kind: 'polygon',
			filled: false,
			points: [
				{ x: TRIANGLE.back, y: -TRIANGLE.half },
				{ x: TRIANGLE.tip, y: 0 },
				{ x: TRIANGLE.back, y: TRIANGLE.half },
			],
		},
		// Leads: the two inputs, the output, and the supply either side.
		{
			kind: 'polyline',
			points: [
				{ x: -PIN_REACH, y: -INPUT_OFFSET },
				{ x: TRIANGLE.back, y: -INPUT_OFFSET },
			],
		},
		{
			kind: 'polyline',
			points: [
				{ x: -PIN_REACH, y: INPUT_OFFSET },
				{ x: TRIANGLE.back, y: INPUT_OFFSET },
			],
		},
		{ kind: 'polyline', points: [{ x: TRIANGLE.tip, y: 0 }, { x: PIN_REACH, y: 0 }] },
		{ kind: 'polyline', points: [{ x: 0, y: -PIN_REACH }, { x: 0, y: -TRIANGLE_AT_CENTRE }] },
		{ kind: 'polyline', points: [{ x: 0, y: TRIANGLE_AT_CENTRE }, { x: 0, y: PIN_REACH }] },
		// − beside the inverting input, + beside the other.
		{
			kind: 'polyline',
			points: [
				{ x: -9, y: -INPUT_OFFSET },
				{ x: -5, y: -INPUT_OFFSET },
			],
		},
		{
			kind: 'polyline',
			points: [
				{ x: -9, y: INPUT_OFFSET },
				{ x: -5, y: INPUT_OFFSET },
			],
		},
		{
			kind: 'polyline',
			points: [
				{ x: -7, y: INPUT_OFFSET - 2 },
				{ x: -7, y: INPUT_OFFSET + 2 },
			],
		},
		...inside,
	],
	// The core's order and the core's names.
	pins: [
		{ name: 'in+', at: { x: -PIN_REACH, y: INPUT_OFFSET } },
		{ name: 'in-', at: { x: -PIN_REACH, y: -INPUT_OFFSET } },
		{ name: 'out', at: { x: PIN_REACH, y: 0 } },
		{ name: 'V+', at: { x: 0, y: -PIN_REACH } },
		{ name: 'V-', at: { x: 0, y: PIN_REACH } },
	],
	}
}

const OP_AMP: SymbolDefinition = amplifier([])

/**
 * A comparator: the op-amp's triangle, with a step drawn in it (C-56).
 *
 * The same triangle because it is wired the same way and has the same pins,
 * and the step because it does not amplify: its output is one thing or the
 * other. That little square wave is the mark schematics put on a part that
 * switches, and it keeps the two from being told apart only by their label.
 */
const COMPARATOR: SymbolDefinition = amplifier([
	{
		kind: 'polyline',
		points: [
			{ x: 0, y: 3 },
			{ x: 3, y: 3 },
			{ x: 3, y: -3 },
			{ x: 7, y: -3 },
		],
	},
])

/** Half the width and half the height of a 555's body. */
const TIMER_BODY = { x: 12, y: 16 }

/**
 * A 555: a box, the three pins the timing parts meet on its left, the output
 * on its right, supply above and ground below (ADR 0012).
 *
 * DIS, THR and TRIG are drawn in the order an astable wires them, top to
 * bottom — the resistor chain comes down into DIS and the capacitor sits under
 * THR and TRIG — so the textbook circuit can be drawn without a crossing. The
 * pulse inside says what it makes; which part it is goes beside it, as for
 * every chip.
 */
const TIMER: SymbolDefinition = {
	shapes: [
		{
			kind: 'polygon',
			filled: false,
			points: [
				{ x: -TIMER_BODY.x, y: -TIMER_BODY.y },
				{ x: TIMER_BODY.x, y: -TIMER_BODY.y },
				{ x: TIMER_BODY.x, y: TIMER_BODY.y },
				{ x: -TIMER_BODY.x, y: TIMER_BODY.y },
			],
		},
		...[-GRID_SPACING, 0, GRID_SPACING].map(
			(y): SymbolShape => ({
				kind: 'polyline',
				points: [
					{ x: -PIN_REACH, y },
					{ x: -TIMER_BODY.x, y },
				],
			}),
		),
		{ kind: 'polyline', points: [{ x: TIMER_BODY.x, y: 0 }, { x: PIN_REACH, y: 0 }] },
		{ kind: 'polyline', points: [{ x: 0, y: -PIN_REACH }, { x: 0, y: -TIMER_BODY.y }] },
		{ kind: 'polyline', points: [{ x: 0, y: TIMER_BODY.y }, { x: 0, y: PIN_REACH }] },
		// A pulse, low, high, low.
		{
			kind: 'polyline',
			points: [
				{ x: -6, y: 4 },
				{ x: -2, y: 4 },
				{ x: -2, y: -4 },
				{ x: 3, y: -4 },
				{ x: 3, y: 4 },
				{ x: 7, y: 4 },
			],
		},
	],
	// The core's order and the core's names.
	pins: [
		{ name: 'VCC', at: { x: 0, y: -PIN_REACH } },
		{ name: 'GND', at: { x: 0, y: PIN_REACH } },
		{ name: 'TRIG', at: { x: -PIN_REACH, y: GRID_SPACING } },
		{ name: 'THR', at: { x: -PIN_REACH, y: 0 } },
		{ name: 'DIS', at: { x: -PIN_REACH, y: -GRID_SPACING } },
		{ name: 'OUT', at: { x: PIN_REACH, y: 0 } },
	],
}

const GROUND: SymbolDefinition = {
	shapes: [
		{ kind: 'polyline', points: [{ x: 0, y: -PIN_REACH }, { x: 0, y: 0 }] },
		{ kind: 'polyline', points: [{ x: -9, y: 0 }, { x: 9, y: 0 }] },
		{ kind: 'polyline', points: [{ x: -5, y: 5 }, { x: 5, y: 5 }] },
		{ kind: 'polyline', points: [{ x: -2, y: 10 }, { x: 2, y: 10 }] },
	],
	pins: [{ name: 'gnd', at: { x: 0, y: -PIN_REACH } }],
}

/** Half the width and the height of a seven-segment digit's body. */
const DIGIT_BODY = { x: 14, y: 16 }

/**
 * A seven-segment digit: a box with the digit drawn in it (C-75).
 *
 * Nine pins in the square every symbol's pins sit on, each near the bar it
 * lights, so a wire runs to the segment it feeds: f, a and b along the top, g
 * and e on the left, d and the point along the bottom, the common and c on the
 * right. One corner is left without a pin, the lower left, because every side
 * has one and the part number has to go somewhere no wire leaves from. Not a
 * datasheet's pinout — the breadboard view (B-14) maps the pins onto the ten a
 * 5161AS has, as it will a 555's.
 *
 * The segments are shapes of their own, so each can be lit by itself; the
 * body is everything else. Common cathode or common anode is drawn the same:
 * which way round the LEDs are is inside the package, and the part number
 * beside it says which.
 */
const SEVEN_SEGMENT: SymbolDefinition = (() => {
	const edge = (from: Point, to: Point): SymbolShape => ({ kind: 'polyline', points: [from, to] })
	const { x: wide, y: tall } = DIGIT_BODY
	const [left, right, high, middle, low] = [-6, 6, -11, 0, 11]
	return {
		shapes: [
			{
				kind: 'polygon',
				filled: false,
				points: [
					{ x: -wide, y: -tall },
					{ x: wide, y: -tall },
					{ x: wide, y: tall },
					{ x: -wide, y: tall },
				],
			},
			// The leads, from the body out to the pins.
			...[-GRID_SPACING, 0, GRID_SPACING].map((x) => edge({ x, y: -tall }, { x, y: -PIN_REACH })),
			...[0, GRID_SPACING].map((x) => edge({ x, y: tall }, { x, y: PIN_REACH })),
			...[-GRID_SPACING, 0].map((y) => edge({ x: -wide, y }, { x: -PIN_REACH, y })),
			...[-GRID_SPACING, GRID_SPACING].map((y) => edge({ x: wide, y }, { x: PIN_REACH, y })),
		],
		segments: [
			edge({ x: left + 1.5, y: high }, { x: right - 1.5, y: high }), // a
			edge({ x: right, y: high + 1.5 }, { x: right, y: middle - 1.5 }), // b
			edge({ x: right, y: middle + 1.5 }, { x: right, y: low - 1.5 }), // c
			edge({ x: left + 1.5, y: low }, { x: right - 1.5, y: low }), // d
			edge({ x: left, y: middle + 1.5 }, { x: left, y: low - 1.5 }), // e
			edge({ x: left, y: high + 1.5 }, { x: left, y: middle - 1.5 }), // f
			edge({ x: left + 1.5, y: middle }, { x: right - 1.5, y: middle }), // g
			{ kind: 'circle', centre: { x: right + 4, y: low }, radius: 1 }, // dp
		],
		boxed: true,
		// The core's order and the core's names: the segments, then the common.
		pins: [
			{ name: 'a', at: { x: 0, y: -PIN_REACH } },
			{ name: 'b', at: { x: GRID_SPACING, y: -PIN_REACH } },
			{ name: 'c', at: { x: PIN_REACH, y: GRID_SPACING } },
			{ name: 'd', at: { x: 0, y: PIN_REACH } },
			{ name: 'e', at: { x: -PIN_REACH, y: 0 } },
			{ name: 'f', at: { x: -GRID_SPACING, y: -PIN_REACH } },
			{ name: 'g', at: { x: -PIN_REACH, y: -GRID_SPACING } },
			{ name: 'dp', at: { x: GRID_SPACING, y: PIN_REACH } },
			{ name: 'com', at: { x: PIN_REACH, y: -GRID_SPACING } },
		],
	}
})()

/**
 * A liquid-crystal digit: the LED digit's box, pins and bars, behind a window
 * of glass (C-76).
 *
 * The same pins in the same places, because a digit is lettered the same
 * whatever it is made of and the wiring a beginner learnt on one carries over
 * — COM is where the common was, and is the backplane. What differs is only
 * what is drawn in the box: a grey-green face, and bars that darken on it.
 */
const LCD_DIGIT: SymbolDefinition = {
	...SEVEN_SEGMENT,
	glass: [
		{ x: -11, y: -14 },
		{ x: 11, y: -14 },
		{ x: 11, y: 14 },
		{ x: -11, y: 14 },
	],
}

/** How far out a CD4511's pins sit: four cells, to fit sixteen of them. */
const DECODER_REACH = 4 * GRID_SPACING

/** Half the width and the height of its body. */
const DECODER_BODY = { x: 3 * GRID_SPACING, y: 3.6 * GRID_SPACING }

/**
 * A CD4511: a box with its name in it, the four bits and three controls down
 * its left, the seven segments down its right in the order a digit wants
 * them, supply above and ground below (C-77).
 *
 * Four cells out rather than two, because sixteen pins need the room. The
 * outputs are in the same top-to-bottom order as a digit's, so seven wires
 * run straight across to it.
 */
const DECODER: SymbolDefinition = (() => {
	const edge = (from: Point, to: Point): SymbolShape => ({ kind: 'polyline', points: [from, to] })
	const rows = [-3, -2, -1, 0, 1, 2, 3].map((row) => row * GRID_SPACING)
	const inputs = ['A', 'B', 'C', 'D', 'LT', 'BI', 'LE']
	const outputs = ['a', 'b', 'c', 'd', 'e', 'f', 'g']
	return {
		shapes: [
			{
				kind: 'polygon',
				filled: false,
				points: [
					{ x: -DECODER_BODY.x, y: -DECODER_BODY.y },
					{ x: DECODER_BODY.x, y: -DECODER_BODY.y },
					{ x: DECODER_BODY.x, y: DECODER_BODY.y },
					{ x: -DECODER_BODY.x, y: DECODER_BODY.y },
				],
			},
			...rows.map((y) => edge({ x: -DECODER_REACH, y }, { x: -DECODER_BODY.x, y })),
			...rows.map((y) => edge({ x: DECODER_BODY.x, y }, { x: DECODER_REACH, y })),
			edge({ x: 0, y: -DECODER_REACH }, { x: 0, y: -DECODER_BODY.y }),
			edge({ x: 0, y: DECODER_BODY.y }, { x: 0, y: DECODER_REACH }),
		],
		reach: DECODER_REACH,
		labelInside: true,
		// The core's order and the core's names.
		pins: [
			...inputs.map((name, index) => ({ name, at: { x: -DECODER_REACH, y: rows[index] } })),
			...outputs.map((name, index) => ({ name, at: { x: DECODER_REACH, y: rows[index] } })),
			{ name: 'VDD', at: { x: 0, y: -DECODER_REACH } },
			{ name: 'VSS', at: { x: 0, y: DECODER_REACH } },
		],
	}
})()

/**
 * A CD4026: the CD4511's box and its outputs in the same order, with its own
 * inputs down the left — clock, inhibit, reset, display enable — and its
 * carry out below them, to clock the next digit's (C-78).
 */
const COUNTER: SymbolDefinition = (() => {
	const edge = (from: Point, to: Point): SymbolShape => ({ kind: 'polyline', points: [from, to] })
	const rows = [-3, -2, -1, 0, 1, 2, 3].map((row) => row * GRID_SPACING)
	const left = ['CLK', 'INH', 'RST', 'DEI', 'CO']
	const leftRows = [-2, -1, 0, 1, 2].map((row) => row * GRID_SPACING)
	const outputs = ['a', 'b', 'c', 'd', 'e', 'f', 'g']
	return {
		shapes: [
			{
				kind: 'polygon',
				filled: false,
				points: [
					{ x: -DECODER_BODY.x, y: -DECODER_BODY.y },
					{ x: DECODER_BODY.x, y: -DECODER_BODY.y },
					{ x: DECODER_BODY.x, y: DECODER_BODY.y },
					{ x: -DECODER_BODY.x, y: DECODER_BODY.y },
				],
			},
			...leftRows.map((y) => edge({ x: -DECODER_REACH, y }, { x: -DECODER_BODY.x, y })),
			...rows.map((y) => edge({ x: DECODER_BODY.x, y }, { x: DECODER_REACH, y })),
			edge({ x: 0, y: -DECODER_REACH }, { x: 0, y: -DECODER_BODY.y }),
			edge({ x: 0, y: DECODER_BODY.y }, { x: 0, y: DECODER_REACH }),
		],
		reach: DECODER_REACH,
		labelInside: true,
		// The core's order and the core's names.
		pins: [
			...left.map((name, index) => ({ name, at: { x: -DECODER_REACH, y: leftRows[index] } })),
			...outputs.map((name, index) => ({ name, at: { x: DECODER_REACH, y: rows[index] } })),
			{ name: 'VDD', at: { x: 0, y: -DECODER_REACH } },
			{ name: 'VSS', at: { x: 0, y: DECODER_REACH } },
		],
	}
})()

/**
 * An L293D: the CD4511's box, each pair's enable and inputs down the left and
 * their outputs across from them on the right, the two supplies above and its
 * one GND below (C-91).
 *
 * Each output sits on the row of the input that sets it, so a channel reads
 * straight across the chip, and the two pairs are a row apart, as the two
 * halves of the package are.
 */
const MOTOR_DRIVER: SymbolDefinition = (() => {
	const edge = (from: Point, to: Point): SymbolShape => ({ kind: 'polyline', points: [from, to] })
	const left = ['1,2EN', '1A', '2A', '3,4EN', '3A', '4A']
	const leftRows = [-3, -2, -1, 1, 2, 3].map((row) => row * GRID_SPACING)
	const right = ['1Y', '2Y', '3Y', '4Y']
	const rightRows = [-2, -1, 2, 3].map((row) => row * GRID_SPACING)
	const supplies = [
		{ name: 'VCC2', x: -GRID_SPACING },
		{ name: 'VCC1', x: GRID_SPACING },
	]
	const at = (name: string) => {
		const index = left.indexOf(name)
		if (index >= 0) return { x: -DECODER_REACH, y: leftRows[index] }
		const out = right.indexOf(name)
		if (out >= 0) return { x: DECODER_REACH, y: rightRows[out] }
		const supply = supplies.find((entry) => entry.name === name)
		if (supply) return { x: supply.x, y: -DECODER_REACH }
		return { x: 0, y: DECODER_REACH }
	}
	// The core's order and the core's names.
	const order = ['1,2EN', '1A', '1Y', '2Y', '2A', 'VCC2', '3,4EN', '3A', '3Y', '4Y', '4A', 'VCC1', 'GND']
	return {
		shapes: [
			{
				kind: 'polygon',
				filled: false,
				points: [
					{ x: -DECODER_BODY.x, y: -DECODER_BODY.y },
					{ x: DECODER_BODY.x, y: -DECODER_BODY.y },
					{ x: DECODER_BODY.x, y: DECODER_BODY.y },
					{ x: -DECODER_BODY.x, y: DECODER_BODY.y },
				],
			},
			...leftRows.map((y) => edge({ x: -DECODER_REACH, y }, { x: -DECODER_BODY.x, y })),
			...rightRows.map((y) => edge({ x: DECODER_BODY.x, y }, { x: DECODER_REACH, y })),
			...supplies.map(({ x }) => edge({ x, y: -DECODER_REACH }, { x, y: -DECODER_BODY.y })),
			edge({ x: 0, y: DECODER_BODY.y }, { x: 0, y: DECODER_REACH }),
		],
		reach: DECODER_REACH,
		labelInside: true,
		pins: order.map((name) => ({ name, at: at(name) })),
	}
})()

/**
 * A 4N35: a box with an LED on its left and a transistor on its right, two
 * arrows of light between them and no line (C-92).
 *
 * The gap is the drawing's point, as the MOSFET's is: nothing joins the two
 * halves but the light, so the two circuits on either side need not share a
 * wire. Anode above cathode on the left, collector above emitter on the right.
 */
const OPTOCOUPLER: SymbolDefinition = (() => {
	const line = (...points: Point[]): SymbolShape => ({ kind: 'polyline', points })
	const towards = (from: Point, to: Point): SymbolShape[] => [line(from, to), arrowhead(from, to, 3)]
	return {
		shapes: [
			{
				kind: 'polygon',
				filled: false,
				points: [
					{ x: -15, y: -15 },
					{ x: 15, y: -15 },
					{ x: 15, y: 15 },
					{ x: -15, y: 15 },
				],
			},
			// The LED, pointing down from anode to cathode.
			line({ x: -PIN_REACH, y: -10 }, { x: -9, y: -10 }, { x: -9, y: -4 }),
			{
				kind: 'polygon',
				filled: false,
				points: [
					{ x: -13, y: -4 },
					{ x: -5, y: -4 },
					{ x: -9, y: 3 },
				],
			},
			line({ x: -13, y: 3 }, { x: -5, y: 3 }),
			line({ x: -9, y: 3 }, { x: -9, y: 10 }, { x: -PIN_REACH, y: 10 }),
			// Its light, across the gap.
			...towards({ x: -3, y: -3 }, { x: 3, y: -1 }),
			...towards({ x: -3, y: 2 }, { x: 3, y: 4 }),
			// The transistor: its base the light, its emitter's arrow out.
			line({ x: 6, y: -6 }, { x: 6, y: 6 }),
			line({ x: 6, y: -3 }, { x: 11, y: -8 }, { x: 11, y: -10 }, { x: PIN_REACH, y: -10 }),
			line({ x: 6, y: 3 }, { x: 11, y: 8 }),
			arrowhead({ x: 6, y: 3 }, { x: 11, y: 8 }, 3),
			line({ x: 11, y: 8 }, { x: 11, y: 10 }, { x: PIN_REACH, y: 10 }),
		],
		// The core's order and the core's names.
		pins: [
			{ name: 'A', at: { x: -PIN_REACH, y: -10 } },
			{ name: 'K', at: { x: -PIN_REACH, y: 10 } },
			{ name: 'C', at: { x: PIN_REACH, y: -10 } },
			{ name: 'E', at: { x: PIN_REACH, y: 10 } },
		],
	}
})()

/** How far out a 74HC595's pins sit: five cells, for nine down one side. */
const SHIFT_REACH = 5 * GRID_SPACING

/**
 * A 74HC595: data and its four controls down the left, QA to QH down the
 * right with QH′ under them to chain the next, supply above and ground below
 * (C-93).
 *
 * Five cells out, because eight outputs and the carry are nine rows. The
 * outputs run top to bottom in the order the bits go in, so eight wires fan
 * out to eight LEDs without crossing.
 */
const SHIFT_REGISTER: SymbolDefinition = (() => {
	const edge = (from: Point, to: Point): SymbolShape => ({ kind: 'polyline', points: [from, to] })
	const body = { x: 3 * GRID_SPACING, y: 4.6 * GRID_SPACING }
	const left = ['SER', 'SRCLK', 'RCLK', 'SRCLR', 'OE']
	const leftRows = [-2, -1, 0, 1, 2].map((row) => row * GRID_SPACING)
	const right = ['QA', 'QB', 'QC', 'QD', 'QE', 'QF', 'QG', 'QH', "QH'"]
	const rightRows = [-4, -3, -2, -1, 0, 1, 2, 3, 4].map((row) => row * GRID_SPACING)
	const at = (name: string): Point => {
		const input = left.indexOf(name)
		if (input >= 0) return { x: -SHIFT_REACH, y: leftRows[input] }
		const output = right.indexOf(name)
		if (output >= 0) return { x: SHIFT_REACH, y: rightRows[output] }
		return { x: 0, y: name === 'VCC' ? -SHIFT_REACH : SHIFT_REACH }
	}
	// The core's order and the core's names.
	const order = ['QB', 'QC', 'QD', 'QE', 'QF', 'QG', 'QH', 'GND', "QH'", 'SRCLR', 'SRCLK', 'RCLK', 'OE', 'SER', 'QA', 'VCC']
	return {
		shapes: [
			{
				kind: 'polygon',
				filled: false,
				points: [
					{ x: -body.x, y: -body.y },
					{ x: body.x, y: -body.y },
					{ x: body.x, y: body.y },
					{ x: -body.x, y: body.y },
				],
			},
			...leftRows.map((y) => edge({ x: -SHIFT_REACH, y }, { x: -body.x, y })),
			...rightRows.map((y) => edge({ x: body.x, y }, { x: SHIFT_REACH, y })),
			edge({ x: 0, y: -SHIFT_REACH }, { x: 0, y: -body.y }),
			edge({ x: 0, y: body.y }, { x: 0, y: SHIFT_REACH }),
		],
		reach: SHIFT_REACH,
		labelInside: true,
		pins: order.map((name) => ({ name, at: at(name) })),
	}
})()

/** What each gate of a quad does, by the core's spelling of the choice. */
type Logic = 'NAND' | 'AND' | 'OR' | 'XOR'

/** A quadratic curve from `from` to `to`, bent towards `bend`, as points. */
function curve(from: Point, bend: Point, to: Point, steps = 6): Point[] {
	return Array.from({ length: steps + 1 }, (_, step) => {
		const t = step / steps
		const u = 1 - t
		return {
			x: u * u * from.x + 2 * u * t * bend.x + t * t * to.x,
			y: u * u * from.y + 2 * u * t * bend.y + t * t * to.y,
		}
	})
}

/**
 * One gate's body, centred on `y`: the shapes every truth table is drawn with
 * (C-108). Returns the shapes, where its inputs meet it and where its output
 * leaves.
 */
function gateBody(logic: Logic | 'NOT', y: number): { shapes: SymbolShape[]; back: number; tip: number } {
	const half = 7
	const bubble = (x: number): SymbolShape => ({ kind: 'circle', centre: { x: x + 2.5, y }, radius: 2.5 })
	if (logic === 'NOT') {
		const triangle: SymbolShape = {
			kind: 'polygon',
			filled: false,
			points: [
				{ x: -8, y: y - 4.5 },
				{ x: 3, y },
				{ x: -8, y: y + 4.5 },
			],
		}
		return { shapes: [triangle, bubble(3)], back: -8, tip: 8 }
	}
	if (logic === 'AND' || logic === 'NAND') {
		const arc = Array.from({ length: 9 }, (_, step) => {
			const angle = -Math.PI / 2 + (Math.PI * step) / 8
			return { x: -3 + half * Math.cos(angle), y: y + half * Math.sin(angle) }
		})
		const shapes: SymbolShape[] = [
			{ kind: 'polygon', filled: false, points: [{ x: -12, y: y - half }, ...arc, { x: -12, y: y + half }] },
		]
		if (logic === 'NAND') shapes.push(bubble(4))
		return { shapes, back: -12, tip: logic === 'NAND' ? 9 : 4 }
	}
	// OR and XOR: a hollow back, and two curves meeting in a point.
	const back = logic === 'XOR' ? -9 : -12
	const top = curve({ x: back, y: y - half }, { x: back + 12, y: y - half }, { x: back + 18, y })
	const bottom = curve({ x: back + 18, y }, { x: back + 12, y: y + half }, { x: back, y: y + half })
	const hollow = curve({ x: back, y: y + half }, { x: back + 5, y }, { x: back, y: y - half })
	const shapes: SymbolShape[] = [
		{ kind: 'polygon', filled: false, points: [...top, ...bottom.slice(1), ...hollow.slice(1, -1)] },
	]
	// XOR's second back, a little behind the first.
	if (logic === 'XOR') {
		shapes.push({ kind: 'polyline', points: curve({ x: -13, y: y + half }, { x: -8, y }, { x: -13, y: y - half }) })
	}
	// The inputs stop where the hollow back is, a little in from its ends.
	return { shapes, back: back + 1.5, tip: back + 18 }
}

/** How far out a quad's pins sit: five cells, for eight inputs down one side. */
const QUAD_REACH = 5 * GRID_SPACING

/**
 * A 74HC00, 08, 32 or 86: each gate's two inputs on the left, its output
 * across from its first, and the gate itself drawn between them, so what the
 * chip does is on it rather than in its number (C-108). The two pairs are a
 * row apart, as the two halves of the package are, and the part number sits
 * in the gap.
 */
function quad(logic: Logic): SymbolDefinition {
	const edge = (...points: Point[]): SymbolShape => ({ kind: 'polyline', points })
	const body = { x: 3 * GRID_SPACING, y: 4.6 * GRID_SPACING }
	// The row of each gate's A; its B is the row under it.
	const rows = [-4, -2, 1, 3].map((row) => row * GRID_SPACING)
	const shapes: SymbolShape[] = [
		{
			kind: 'polygon',
			filled: false,
			points: [
				{ x: -body.x, y: -body.y },
				{ x: body.x, y: -body.y },
				{ x: body.x, y: body.y },
				{ x: -body.x, y: body.y },
			],
		},
		edge({ x: 0, y: -QUAD_REACH }, { x: 0, y: -body.y }),
		edge({ x: 0, y: body.y }, { x: 0, y: QUAD_REACH }),
	]
	const at: Record<string, Point> = { VCC: { x: 0, y: -QUAD_REACH }, GND: { x: 0, y: QUAD_REACH } }
	rows.forEach((a, index) => {
		const b = a + GRID_SPACING
		const middle = (a + b) / 2
		const drawn = gateBody(logic, middle)
		shapes.push(
			...drawn.shapes,
			edge({ x: -QUAD_REACH, y: a }, { x: drawn.back, y: a }),
			edge({ x: -QUAD_REACH, y: b }, { x: drawn.back, y: b }),
			edge({ x: drawn.tip, y: middle }, { x: 16, y: middle }, { x: 16, y: a }, { x: QUAD_REACH, y: a }),
		)
		const gate = index + 1
		at[`${gate}A`] = { x: -QUAD_REACH, y: a }
		at[`${gate}B`] = { x: -QUAD_REACH, y: b }
		at[`${gate}Y`] = { x: QUAD_REACH, y: a }
	})
	// The core's order and the core's names.
	const order = ['1A', '1B', '1Y', '2A', '2B', '2Y', 'GND', '3Y', '3A', '3B', '4Y', '4A', '4B', 'VCC']
	return {
		shapes,
		reach: QUAD_REACH,
		labelInside: true,
		pins: order.map((name) => ({ name, at: at[name] })),
	}
}

const QUADS: Record<Logic, SymbolDefinition> = {
	NAND: quad('NAND'),
	AND: quad('AND'),
	OR: quad('OR'),
	XOR: quad('XOR'),
}

/**
 * A 74HC04: six inverters, each straight across from its input to its
 * output, three above the part number and three below (C-108). A 74HC14 is
 * the same with the mark of a Schmitt trigger in each triangle — two steps,
 * one on top of the other, for the two thresholds (C-109).
 */
function hex(schmitt: boolean): SymbolDefinition {
	const edge = (...points: Point[]): SymbolShape => ({ kind: 'polyline', points })
	const body = { x: 2.5 * GRID_SPACING, y: 3.6 * GRID_SPACING }
	const rows = [-3, -2, -1, 1, 2, 3].map((row) => row * GRID_SPACING)
	const shapes: SymbolShape[] = [
		{
			kind: 'polygon',
			filled: false,
			points: [
				{ x: -body.x, y: -body.y },
				{ x: body.x, y: -body.y },
				{ x: body.x, y: body.y },
				{ x: -body.x, y: body.y },
			],
		},
		edge({ x: 0, y: -DECODER_REACH }, { x: 0, y: -body.y }),
		edge({ x: 0, y: body.y }, { x: 0, y: DECODER_REACH }),
	]
	const at: Record<string, Point> = { VCC: { x: 0, y: -DECODER_REACH }, GND: { x: 0, y: DECODER_REACH } }
	rows.forEach((y, index) => {
		const drawn = gateBody('NOT', y)
		if (schmitt) {
			shapes.push(
				edge({ x: -6.5, y: y + 1.2 }, { x: -3, y: y + 1.2 }, { x: -3, y: y - 1.2 }),
				edge({ x: -5, y: y + 1.2 }, { x: -5, y: y - 1.2 }, { x: -1.5, y: y - 1.2 }),
			)
		}
		shapes.push(
			...drawn.shapes,
			edge({ x: -DECODER_REACH, y }, { x: drawn.back, y }),
			edge({ x: drawn.tip, y }, { x: DECODER_REACH, y }),
		)
		at[`${index + 1}A`] = { x: -DECODER_REACH, y }
		at[`${index + 1}Y`] = { x: DECODER_REACH, y }
	})
	// The core's order and the core's names.
	const order = ['1A', '1Y', '2A', '2Y', '3A', '3Y', 'GND', '4Y', '4A', '5Y', '5A', '6Y', '6A', 'VCC']
	return {
		shapes,
		reach: DECODER_REACH,
		labelInside: true,
		pins: order.map((name) => ({ name, at: at[name] })),
	}
}

/** How far out a CD4017's pins sit: six cells, for eleven down one side. */
const DECADE_REACH = 6 * GRID_SPACING

/**
 * A CD4017: clock, inhibit and reset down the left, Q0 to Q9 down the right
 * in counting order with the carry under them, supply above and ground
 * below (C-110). The outputs in order, so ten LEDs in a column light from
 * top to bottom as it counts.
 */
const DECADE: SymbolDefinition = (() => {
	const edge = (from: Point, to: Point): SymbolShape => ({ kind: 'polyline', points: [from, to] })
	const body = { x: 3 * GRID_SPACING, y: 5.6 * GRID_SPACING }
	const left = ['CLK', 'INH', 'RST']
	const leftRows = [-1, 0, 1].map((row) => row * GRID_SPACING)
	const right = ['Q0', 'Q1', 'Q2', 'Q3', 'Q4', 'Q5', 'Q6', 'Q7', 'Q8', 'Q9', 'CO']
	const rightRows = [-5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5].map((row) => row * GRID_SPACING)
	const at = (name: string): Point => {
		const input = left.indexOf(name)
		if (input >= 0) return { x: -DECADE_REACH, y: leftRows[input] }
		const output = right.indexOf(name)
		if (output >= 0) return { x: DECADE_REACH, y: rightRows[output] }
		return { x: 0, y: name === 'VDD' ? -DECADE_REACH : DECADE_REACH }
	}
	// The core's order and the core's names.
	const order = ['Q5', 'Q1', 'Q0', 'Q2', 'Q6', 'Q7', 'Q3', 'VSS', 'Q8', 'Q4', 'Q9', 'CO', 'INH', 'CLK', 'RST', 'VDD']
	return {
		shapes: [
			{
				kind: 'polygon',
				filled: false,
				points: [
					{ x: -body.x, y: -body.y },
					{ x: body.x, y: -body.y },
					{ x: body.x, y: body.y },
					{ x: -body.x, y: body.y },
				],
			},
			...leftRows.map((y) => edge({ x: -DECADE_REACH, y }, { x: -body.x, y })),
			...rightRows.map((y) => edge({ x: body.x, y }, { x: DECADE_REACH, y })),
			edge({ x: 0, y: -DECADE_REACH }, { x: 0, y: -body.y }),
			edge({ x: 0, y: body.y }, { x: 0, y: DECADE_REACH }),
		],
		reach: DECADE_REACH,
		labelInside: true,
		pins: order.map((name) => ({ name, at: at(name) })),
	}
})()

/** How far out a CD4013's pins sit: five cells, for four a flip-flop. */
const FLIP_FLOP_REACH = 5 * GRID_SPACING

/**
 * A CD4013: its two flip-flops one above the other, the part number in the
 * row between them, each one's D, CLK, SET and RST down the left and its Q
 * and /Q across on the right, a wedge on each clock as a schematic marks one
 * (C-111).
 */
const FLIP_FLOP: SymbolDefinition = (() => {
	const edge = (...points: Point[]): SymbolShape => ({ kind: 'polyline', points })
	const body = { x: 3 * GRID_SPACING, y: 4.6 * GRID_SPACING }
	const at: Record<string, Point> = {
		VDD: { x: 0, y: -FLIP_FLOP_REACH },
		VSS: { x: 0, y: FLIP_FLOP_REACH },
	}
	const shapes: SymbolShape[] = [
		{
			kind: 'polygon',
			filled: false,
			points: [
				{ x: -body.x, y: -body.y },
				{ x: body.x, y: -body.y },
				{ x: body.x, y: body.y },
				{ x: -body.x, y: body.y },
			],
		},
		edge({ x: 0, y: -FLIP_FLOP_REACH }, { x: 0, y: -body.y }),
		edge({ x: 0, y: body.y }, { x: 0, y: FLIP_FLOP_REACH }),
	]
	for (const [half, top] of [
		[1, -4],
		[2, 1],
	]) {
		const row = (offset: number) => (top + offset) * GRID_SPACING
		;['D', 'CLK', 'SET', 'RST'].forEach((name, offset) => {
			at[`${name}${half}`] = { x: -FLIP_FLOP_REACH, y: row(offset) }
			shapes.push(edge({ x: -FLIP_FLOP_REACH, y: row(offset) }, { x: -body.x, y: row(offset) }))
		})
		const clock = row(1)
		shapes.push(edge({ x: -body.x, y: clock - 3 }, { x: -body.x + 5, y: clock }, { x: -body.x, y: clock + 3 }))
		at[`Q${half}`] = { x: FLIP_FLOP_REACH, y: row(0) }
		at[`/Q${half}`] = { x: FLIP_FLOP_REACH, y: row(3) }
		for (const y of [row(0), row(3)]) shapes.push(edge({ x: body.x, y }, { x: FLIP_FLOP_REACH, y }))
	}
	// The core's order and the core's names.
	const order = ['Q1', '/Q1', 'CLK1', 'RST1', 'D1', 'SET1', 'VSS', 'SET2', 'D2', 'RST2', 'CLK2', '/Q2', 'Q2', 'VDD']
	return {
		shapes,
		reach: FLIP_FLOP_REACH,
		labelInside: true,
		pins: order.map((name) => ({ name, at: at[name] })),
	}
})()

/** How far out an LM3914's and an LED bar's pins sit: six cells, for ten down one side. */
const BAR_REACH = 6 * GRID_SPACING

/** Ten rows a cell apart, LED 1 at the top. */
const BAR_ROWS = [-5, -4, -3, -2, -1, 0, 1, 2, 3, 4].map((row) => row * GRID_SPACING)

/** A box of half-size `body`, as a closed outline. */
function box(body: Point): SymbolShape {
	return {
		kind: 'polygon',
		filled: false,
		points: [
			{ x: -body.x, y: -body.y },
			{ x: body.x, y: -body.y },
			{ x: body.x, y: body.y },
			{ x: -body.x, y: body.y },
		],
	}
}

/** How far out an Uno's pins sit: seven cells, for fourteen along its top. */
const UNO_REACH = 7 * GRID_SPACING

/**
 * An Arduino Uno, from above, as it lies on the bench (A-02): the digital
 * header along its top edge, D13 at the left to D0 at the right, as the board
 * prints them; the power header and A0 to A5 along its bottom edge, power
 * first, as the board has them; and its USB socket on the left. Every pin a
 * header pin's lead up or down to the edge of its square, so wires leave the
 * board the way jumpers leave the real one.
 */
const UNO: SymbolDefinition = (() => {
	const edge = (from: Point, to: Point): SymbolShape => ({ kind: 'polyline', points: [from, to] })
	const body = { x: UNO_REACH, y: 4 * GRID_SPACING }
	const top = ['D13', 'D12', 'D11', 'D10', 'D9', 'D8', 'D7', 'D6', 'D5', 'D4', 'D3', 'D2', 'D1', 'D0']
	const bottom: [string, number][] = [
		['3V3', -6],
		['5V', -5],
		['GND', -4],
		['VIN', -3],
		['A0', -1],
		['A1', 0],
		['A2', 1],
		['A3', 2],
		['A4', 3],
		['A5', 4],
	]
	const at = (name: string): Point => {
		const column = top.indexOf(name)
		if (column >= 0) return { x: (column - 7) * GRID_SPACING, y: -UNO_REACH }
		const [, x] = bottom.find(([pin]) => pin === name) ?? ['', 0]
		return { x: x * GRID_SPACING, y: UNO_REACH }
	}
	// The core's order and the core's names.
	const order = [
		...['D0', 'D1', 'D2', 'D3', 'D4', 'D5', 'D6', 'D7', 'D8', 'D9', 'D10', 'D11', 'D12', 'D13'],
		...['A0', 'A1', 'A2', 'A3', 'A4', 'A5', '5V', '3V3', 'VIN', 'GND'],
	]
	const usb: SymbolShape = {
		kind: 'polygon',
		filled: false,
		points: [
			{ x: -body.x, y: -2.5 * GRID_SPACING },
			{ x: -body.x + 1.5 * GRID_SPACING, y: -2.5 * GRID_SPACING },
			{ x: -body.x + 1.5 * GRID_SPACING, y: -0.5 * GRID_SPACING },
			{ x: -body.x, y: -0.5 * GRID_SPACING },
		],
	}
	return {
		shapes: [
			box(body),
			usb,
			...order.map((name) => {
				const pin = at(name)
				return edge(pin, { x: pin.x, y: Math.sign(pin.y) * body.y })
			}),
		],
		reach: UNO_REACH,
		labelInside: true,
		pins: order.map((name) => ({ name, at: at(name) })),
	}
})()

/**
 * An LM3914: SIG and its divider's ends, the reference's two pins and MODE
 * down the left; LED 1 to LED 10 down the right, from the top, so a bar
 * beside it lights from the top down as the input climbs; V+ above, V−
 * below (C-119).
 */
const LM3914: SymbolDefinition = (() => {
	const edge = (from: Point, to: Point): SymbolShape => ({ kind: 'polyline', points: [from, to] })
	const body = { x: 3 * GRID_SPACING, y: 5.6 * GRID_SPACING }
	const left = ['SIG', 'RLO', 'RHI', 'REFOUT', 'REFADJ', 'MODE']
	const leftRows = [-3, -2, -1, 0, 1, 2].map((row) => row * GRID_SPACING)
	const at = (name: string): Point => {
		const input = left.indexOf(name)
		if (input >= 0) return { x: -BAR_REACH, y: leftRows[input] }
		if (name.startsWith('L')) return { x: BAR_REACH, y: BAR_ROWS[Number(name.slice(1)) - 1] }
		return { x: 0, y: name === 'V+' ? -BAR_REACH : BAR_REACH }
	}
	// The core's order and the core's names.
	const order = ['L1', 'V-', 'V+', 'RLO', 'SIG', 'RHI', 'REFOUT', 'REFADJ', 'MODE', 'L10', 'L9', 'L8', 'L7', 'L6', 'L5', 'L4', 'L3', 'L2']
	return {
		shapes: [
			box(body),
			...leftRows.map((y) => edge({ x: -BAR_REACH, y }, { x: -body.x, y })),
			...BAR_ROWS.map((y) => edge({ x: body.x, y }, { x: BAR_REACH, y })),
			edge({ x: 0, y: -BAR_REACH }, { x: 0, y: -body.y }),
			edge({ x: 0, y: body.y }, { x: 0, y: BAR_REACH }),
		],
		reach: BAR_REACH,
		labelInside: true,
		pins: order.map((name) => ({ name, at: at(name) })),
	}
})()

/**
 * A bar of ten LEDs: a column of ten bars in a box, each cathode on the left
 * and its anode opposite on the right, LED 1 at the top (C-119). Cathodes
 * left, because what drives a bar sinks its current — an LM3914 beside it,
 * its outputs facing the bar — and the supply comes in from the far side.
 * The bars light as the digit's segments do (C-75).
 */
const BARGRAPH: SymbolDefinition = (() => {
	const edge = (from: Point, to: Point): SymbolShape => ({ kind: 'polyline', points: [from, to] })
	const body = { x: 1.6 * GRID_SPACING, y: 5.6 * GRID_SPACING }
	const leds = BAR_ROWS.map((_, index) => index + 1)
	return {
		shapes: [
			box(body),
			...BAR_ROWS.map((y) => edge({ x: -BAR_REACH, y }, { x: -body.x, y })),
			...BAR_ROWS.map((y) => edge({ x: body.x, y }, { x: BAR_REACH, y })),
		],
		segments: BAR_ROWS.map((y) => edge({ x: -9, y }, { x: 9, y })),
		boxed: true,
		reach: BAR_REACH,
		// The core's order: the ten anodes, then the ten cathodes.
		pins: [
			...leds.map((led) => ({ name: `a${led}`, at: { x: BAR_REACH, y: BAR_ROWS[led - 1] } })),
			...leds.map((led) => ({ name: `k${led}`, at: { x: -BAR_REACH, y: BAR_ROWS[led - 1] } })),
		],
	}
})()

export const SYMBOLS: Readonly<Record<SymbolKind, SymbolDefinition>> = {
	DcSource: BATTERY,
	SignalGenerator: SIGNAL_SQUARE,
	Cell: REAL_BATTERY,
	SolarCell: SOLAR_CELL,
	Resistor: RESISTOR,
	Capacitor: CAPACITOR,
	Electrolytic: ELECTROLYTIC,
	Inductor: INDUCTOR,
	Transformer: TRANSFORMER,
	Bridge: BRIDGE,
	Led: LED,
	RgbLed: RGB_CATHODE,
	SevenSegment: SEVEN_SEGMENT,
	LcdDigit: LCD_DIGIT,
	Bargraph: BARGRAPH,
	Diode: DIODE,
	Zener: ZENER,
	Switch: SWITCH_OPEN,
	PushButton: BUTTON_UP,
	TiltSwitch: TILT_TIPPED,
	Relay: RELAY_AT_REST,
	Fuse: FUSE,
	Buzzer: BUZZER_QUIET,
	PassiveBuzzer: PASSIVE_QUIET,
	Motor: MOTOR_STILL,
	Servo: servo(),
	Potentiometer: potentiometer(),
	Ldr: LDR,
	Phototransistor: PHOTOTRANSISTOR,
	Thermistor: THERMISTOR,
	Voltmeter: VOLTMETER,
	Ammeter: AMMETER,
	Bjt: NPN,
	Mosfet: NMOS,
	Regulator: REGULATOR,
	OpAmp: OP_AMP,
	Comparator: COMPARATOR,
	Tmp36: TMP36,
	Timer: TIMER,
	Decoder: DECODER,
	Counter: COUNTER,
	MotorDriver: MOTOR_DRIVER,
	Optocoupler: OPTOCOUPLER,
	ShiftRegister: SHIFT_REGISTER,
	Gate: QUADS.NAND,
	Inverter: hex(false),
	Schmitt: hex(true),
	Decade: DECADE,
	Lm3914: LM3914,
	FlipFlop: FLIP_FLOP,
	Adjustable: ADJUSTABLE,
	Schottky: SCHOTTKY,
	Reed: REED_OPEN,
	Hall: HALL,
	Pir: pir(false),
	Uno: UNO,
	Ground: GROUND,
}

/**
 * The other drawing, for the kinds that have two (C-19).
 *
 * Apart from `SYMBOLS` rather than a second entry in it, because that table is
 * keyed by the kinds the core knows and "SwitchClosed" is not one of them — it
 * is a state a switch is in, not a component the netlist can hold.
 */
const SHUT: Partial<Record<SymbolKind, SymbolDefinition>> = {
	Switch: SWITCH_CLOSED,
	PushButton: BUTTON_DOWN,
	// A tilt switch's "shut" is standing upright (C-88).
	TiltSwitch: TILT_UPRIGHT,
	// A relay's "shut" is its coil having pulled the blade over to NO.
	Relay: RELAY_ENERGISED,
	// And a buzzer's is sounding, a motor's turning (C-66).
	Buzzer: BUZZER_SOUNDING,
	PassiveBuzzer: PASSIVE_SOUNDING,
	Motor: MOTOR_TURNING,
	// And a reed's, its magnet near enough (C-115).
	Reed: REED_SHUT,
	// And a PIR's, someone moving in front of it (C-116).
	Pir: pir(true),
}

/** Whatever about a component changes how it is drawn. */
export interface SymbolState {
	/** A contact, made or broken (C-19). */
	readonly closed?: boolean
	/** A wiper's position along its track, 0 to 1 (C-20). */
	readonly wiper?: number
	/**
	 * Which of its alternatives the component is set to (C-46).
	 *
	 * Only a transistor or a gate draws differently by it — an LED's colour changes the
	 * light, not the shape — so this is the core's own spelling of the choice
	 * and the table below is keyed by it.
	 */
	readonly choice?: string
	/** Where a servo's horn points, 0° to 180° (C-90). */
	readonly angle?: number
	/** A motor turning backwards, `b` above `a` (C-91). */
	readonly reversed?: boolean
	/** A fuse that has blown (C-120). */
	readonly blown?: boolean
}

/**
 * Kinds drawn differently depending on which alternative they are set to,
 * keyed by the core's own spelling of it (C-46, ADR 0008).
 *
 * `Npn` and `Pnp`, which is what crosses the boundary — not the `NPN` the core
 * writes *beside* the symbol, which is a label for a person to read. Keyed
 * wrongly it does not fail: `symbolFor` falls back to the kind's one drawing,
 * so every transistor comes out an NPN with the right word underneath it. That
 * is exactly what happened, and it is why the table is exported and held to the
 * core's list by a test rather than merely commented.
 *
 * Not every choice has to be here — an LED's colour is light, not shape — but a
 * kind that is in this table has to cover all of its own.
 */
export const DRAWN_BY_CHOICE: Partial<Record<SymbolKind, Record<string, SymbolDefinition>>> = {
	Bjt: { Npn: NPN, Pnp: PNP, Darlington: DARLINGTON },
	Mosfet: { 'N-channel': NMOS, 'P-channel': PMOS, 'Power N-channel': POWER_NMOS },
	RgbLed: { 'Common cathode': RGB_CATHODE, 'Common anode': RGB_ANODE },
	SignalGenerator: { Square: SIGNAL_SQUARE, Sine: SIGNAL_SINE, Triangle: SIGNAL_TRIANGLE },
	// What each gate does is drawn on the chip (C-108).
	Gate: QUADS,
}

/**
 * How a component of this kind is drawn, in the state it is in.
 *
 * Every field of the state comes from the core's element view and is absent for
 * everything it does not apply to, so a component with one drawing gets it
 * without having to know that states exist. The pins are the same whatever the
 * state, which is what lets a wire survive a switch being flipped or a knob
 * being turned.
 */
export function symbolFor(kind: SymbolKind, state: SymbolState = {}): SymbolDefinition {
	if (kind === 'Potentiometer' && state.wiper !== undefined) {
		return potentiometer(state.wiper)
	}
	if (kind === 'Servo' && state.angle !== undefined) {
		return servo(state.angle)
	}
	if (kind === 'Motor' && state.closed && state.reversed) {
		return MOTOR_BACKWARDS
	}
	if (kind === 'Fuse') {
		if (state.choice === 'Polyfuse') return POLYFUSE
		return state.blown ? FUSE_BLOWN : FUSE
	}
	const chosen = state.choice ? DRAWN_BY_CHOICE[kind]?.[state.choice] : undefined
	return chosen ?? (state.closed ? SHUT[kind] : undefined) ?? SYMBOLS[kind]
}

/**
 * A point in symbol-local coordinates, turned a number of quarter turns
 * clockwise about the symbol's origin.
 *
 * The one place rotation is defined (C-40). Drawing, hit-testing and where a
 * wire starts all go through it, so a turned component cannot end up drawn one
 * way and wired another.
 *
 * Clockwise on screen, where y runs down: a point out to the right goes to the
 * bottom. Quarter turns only, so a pin two grid cells out lands two grid cells
 * out along another axis, never between intersections.
 */
export function turned(point: Point, turns = 0): Point {
	// Negating a zero coordinate gives -0, which every arithmetic use treats as
	// 0 and every identity comparison — `Object.is`, and so every test matcher
	// — treats as a different number. Folded away here rather than worked
	// around at each call.
	const flat = (value: number) => (value === 0 ? 0 : value)

	switch (((turns % 4) + 4) % 4) {
		case 1:
			return { x: flat(-point.y), y: point.x }
		case 2:
			return { x: flat(-point.x), y: flat(-point.y) }
		case 3:
			return { x: point.y, y: flat(-point.x) }
		default:
			return point
	}
}

/**
 * Half-width and half-height of everything the symbol draws, as it is turned.
 *
 * Derived rather than declared, so a symbol cannot end up with a hit box that
 * disagrees with its drawing.
 */
export function halfSizeOf(symbol: SymbolDefinition, turns = 0): Point {
	let x = 0
	let y = 0
	const stretch = (point: Point) => {
		x = Math.max(x, Math.abs(point.x))
		y = Math.max(y, Math.abs(point.y))
	}

	for (const shape of symbol.shapes) {
		if (shape.kind === 'circle') {
			stretch({ x: shape.centre.x + shape.radius, y: shape.centre.y + shape.radius })
			stretch({ x: shape.centre.x - shape.radius, y: shape.centre.y - shape.radius })
		} else {
			for (const point of shape.points) stretch(point)
		}
	}
	for (const pin of symbol.pins) stretch(pin.at)
	// A symbol that is drawn differently in two states says what box it wants,
	// so both states are the same target however little one of them draws.
	if (symbol.extent) stretch(symbol.extent)

	// A quarter or three-quarter turn swaps the two; a half turn is symmetric
	// about the origin and changes nothing, because the extents are measured
	// either side of it.
	return turns % 2 === 0 ? { x, y } : { x: y, y: x }
}

/**
 * Which way a pin's lead points, as a unit step outward from its part.
 *
 * A pin sits on the edge of the two-cell square its symbol fits in, so the
 * edge it is on is the axis it is furthest out along; turned with the part.
 * The router leaves a pin this way before it turns (C-61), which is what keeps
 * a wire off the column a pin shares with its neighbours.
 */
export function leadOf(pin: SymbolPin, turns = 0): Point {
	const at = turned(pin.at, turns)
	return Math.abs(at.x) >= Math.abs(at.y)
		? { x: Math.sign(at.x), y: 0 }
		: { x: 0, y: Math.sign(at.y) }
}

/** A side of a symbol, as it sits on the canvas. */
export type Side = 'below' | 'above' | 'right' | 'left'

/** A corner of a symbol, for when every side has a pin. */
export type Corner = 'below-right' | 'above-right' | 'below-left' | 'above-left'

/**
 * Which side of a symbol its label goes on, as it is turned.
 *
 * Below by default, where a label has always gone. But a pin pointing down
 * has a wire leaving it downwards, and a label written there is written on
 * the wire: a regulator's ground lead ran straight through its part number
 * (C-54), and a transistor's emitter through its NPN. So the first side with
 * no pin on it, in the order a reader looks — below, above, then beside.
 * Every side has a pin only for a symbol with four, which nothing has; below
 * is the answer then, as it was before.
 */
export function labelSide(symbol: SymbolDefinition, turns = 0): Side | Corner {
	const occupied = new Set<Side | Corner>()
	for (const pin of symbol.pins) {
		const at = turned(pin.at, turns)
		if (Math.abs(at.y) > Math.abs(at.x)) occupied.add(at.y > 0 ? 'below' : 'above')
		else if (Math.abs(at.x) > Math.abs(at.y)) occupied.add(at.x > 0 ? 'right' : 'left')
		// A pin off both axes is in a corner as well as on a side.
		if (at.x !== 0 && at.y !== 0) {
			occupied.add(`${at.y > 0 ? 'below' : 'above'}-${at.x > 0 ? 'right' : 'left'}` as Corner)
		}
	}
	// An op-amp has a pin on every side (C-55), so after the sides come the
	// corners: the room between two leads, where no wire can run.
	const places: (Side | Corner)[] = [
		'below',
		'above',
		'right',
		'left',
		'below-right',
		'above-right',
		'below-left',
		'above-left',
	]
	return places.find((place) => !occupied.has(place)) ?? 'below'
}

