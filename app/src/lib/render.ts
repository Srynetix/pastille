// SPDX-License-Identifier: MIT OR Apache-2.0

// Drawing a scene onto a 2D context.
//
// Kept out of the component so the render loop can be measured without mounting
// anything — M-14 has a frame budget to meet, and a number nobody can reproduce
// is not a budget.
//
// Nothing here knows what a resistor looks like: an item carries a symbol, and a
// symbol is a list of shapes (M-15). Adding a component adds data, not a branch.

import { heatInk } from './heat'
import { pinPosition, wireJunctions, type SceneItem, type SceneWire } from './scene'
import type { Segment } from './routing'
import { halfSizeOf, labelSide, turned, type SymbolShape } from './symbols'
import { GRID_SPACING, toScreen, type Point, type Viewport } from './viewport'

export interface Theme {
	readonly grid: string
	readonly wire: string
	/** The moving dots that show current.  */
	readonly current: string
	/** Outline of whatever is selected. */
	readonly selection: string
	readonly body: string
	readonly outline: string
	readonly pin: string
	readonly text: string
	/** The family labels are set in, as a CSS font-family list. */
	readonly font: string
}

/** Below this on-screen spacing the grid reads as fog, so it is not drawn. */
const MIN_GRID_PIXELS = 6

/** World units between the dots that travel along a wire. */
const DOT_SPACING = 12

/**
 * How fast a dot moves, in world units per second per amp.
 *
 * Current spans decades — microamps to amps — so a linear speed would be either
 * invisible or a blur. The square root compresses that into a range an eye can
 * read, which is a deliberate lie about magnitude in exchange for being able to
 * see direction at all. The number beside the wire is what says how much.
 */
const DOT_SPEED = 220

/**
 * Current below this does not animate: it would read as a stalled circuit.
 *
 * A microamp, not the nanoamp it was. Dots move as the square root of the
 * current, so under a microamp they crawl a fraction of a pixel a second —
 * dots that are there and do not move, which is the stalled look this floor
 * exists to avoid. And the currents down there are the models' leaks: a
 * relay's open contact and a chip's inputs leak a nanosiemens each (C-64,
 * C-77), a few nanoamps that drew dots along the wire to a motor that was not
 * running (C-87). Every current a kit circuit is about is above it; the
 * readout still gives the ones that are not, an LCD's tens of nanoamps
 * included (C-76).
 */
export const MIN_VISIBLE_AMPS = 1e-6

/** Colours for the light an LED gives off, by the core's colour name. */
/**
 * What each of the core's LED colours looks like.
 *
 * Keyed by the core's own spelling, which is the second place the closed set of
 * colours has to be agreed on (C-46, ADR 0008): the core owns *which* colours
 * exist, the app owns what they look like. A test holds the two together, so a
 * fifth colour cannot arrive and quietly glow white.
 */
export const LED_LIGHT: Record<string, string> = {
	Red: '#ff3b30',
	Green: '#32d74b',
	Blue: '#0a84ff',
	White: '#f5f5f7',
}

/** The screen channel each of an RGB LED's dice lights, by the core's name. */
const PRIMARY: Record<string, number> = { Red: 0, Green: 1, Blue: 2 }

/**
 * The colour an LED of several dice gives off: each die's light, as bright as
 * it is lit, added up (C-84).
 *
 * Each die lights its own channel of the screen, so light adds as it does on
 * a colour wheel: red and green glow yellow, all three white. Not the single
 * LEDs' colours added — their green is partly blue and their red partly
 * green, and three of them sum to mint, where a real diffused RGB lamp is made
 * to give white. Then scaled so the brightest channel is full: how bright the
 * lamp is is the glow's size and strength, as for any LED, and this is only
 * its hue.
 */
export function mixedLight(dice: readonly string[], brightness: readonly number[]): string {
	const sum = [0, 0, 0]
	dice.forEach((colour, index) => {
		const channel = PRIMARY[colour]
		if (channel !== undefined) sum[channel] += brightness[index] ?? 0
	})
	const peak = Math.max(...sum)
	if (peak <= 0) return LED_LIGHT.White
	return `#${sum
		.map((value) =>
			Math.round((255 * value) / peak)
				.toString(16)
				.padStart(2, '0'),
		)
		.join('')}`
}

/** The light an item gives off, when it is lit. */
function lightOf(item: SceneItem): string {
	if (item.symbol.dice) return mixedLight(item.symbol.dice, item.segments ?? [])
	return LED_LIGHT[item.colour ?? ''] ?? LED_LIGHT.White
}

/**
 * An LCD digit's face and its ink: grey-green glass, the colour of a clock
 * radio's or a calculator's, and the near-black of a turned segment (C-76).
 */
const LCD_FACE = '#b7c4a0'
const LCD_INK = '#1f2a1c'

export function drawScene(
	context: CanvasRenderingContext2D,
	items: readonly SceneItem[],
	viewport: Viewport,
	theme: Theme,
	size: { width: number; height: number },
	wires: readonly SceneWire[] = [],
	/** Simulated seconds, which is what the current animation runs on. */
	time = 0,
	selected: ReadonlySet<string> = new Set(),
	/** Items whose pin names should be shown — hovered or selected. */
	labelled: ReadonlySet<string> = new Set(),
	/**
	 * Ids of the selected nets.
	 *
	 * A set of its own rather than sharing `selected`: element and net handles
	 * come from different slotmaps and the same string can name one of each.
	 */
	selectedNets: ReadonlySet<string> = new Set(),
	/**
	 * Whether to draw current without moving it, for someone who has asked
	 * their system for less motion: the same places along the wire, as
	 * chevrons pointing the way it flows, so the direction — the point of
	 * the overlay — is still there to see (WCAG 2.2.2).
	 */
	still = false,
): void {
	context.clearRect(0, 0, size.width, size.height)
	drawGrid(context, viewport, theme, size)

	// Wires first: a symbol drawn over a wire reads as the wire passing behind
	// it, which is what a schematic means.
	drawWires(context, wires, viewport, theme, selectedNets)

	for (const item of items) {
		drawGlow(context, item, viewport)
	}
	for (const item of items) {
		drawItem(context, item, viewport, theme, selected.has(item.id), labelled.has(item.id))
	}

	drawCurrent(context, wires, viewport, theme, time, still)
}

/**
 * The light an LED is giving off.
 *
 * Two layers, because a single fading halo turns out to be hard to read: the
 * eye compares *sizes* far better than it compares transparencies, so the glow
 * grows as well as brightens. A dim LED is a small close halo, a bright one
 * reaches across a grid square.
 *
 * Drawn under the symbols so a bright LED does not wash out its own outline,
 * and as a radial fade because a flat disc reads as a sticker rather than a
 * light.
 */
function drawGlow(context: CanvasRenderingContext2D, item: SceneItem, viewport: Viewport): void {
	if (!item.brightness || item.brightness <= 0) return

	const at = toScreen(viewport, item.position)
	const light = lightOf(item)
	// Even a barely-lit LED is visibly lit; full brightness reaches well past
	// the symbol.
	const radius = (14 + 34 * item.brightness) * viewport.zoom

	context.save()

	const bloom = context.createRadialGradient(at.x, at.y, 0, at.x, at.y, radius)
	bloom.addColorStop(0, light)
	bloom.addColorStop(0.35, light)
	bloom.addColorStop(1, 'transparent')
	context.globalAlpha = 0.25 + 0.65 * item.brightness
	context.fillStyle = bloom
	context.beginPath()
	context.arc(at.x, at.y, radius, 0, Math.PI * 2)
	context.fill()

	// A hot core, so the lamp itself looks lit rather than merely surrounded by
	// colour.
	context.globalAlpha = 0.35 + 0.65 * item.brightness
	context.fillStyle = light
	context.beginPath()
	context.arc(at.x, at.y, 6 * viewport.zoom, 0, Math.PI * 2)
	context.fill()

	context.restore()
}

/**
 * Dots travelling along the wires, in the direction the current flows.
 *
 * Falstad's idiom, and the reason CircuitJS1 reads as alive: current is an
 * animation rather than a number, so direction is something you see rather than
 * something you work out (see the prior-art study, §1).
 */
function drawCurrent(
	context: CanvasRenderingContext2D,
	wires: readonly SceneWire[],
	viewport: Viewport,
	theme: Theme,
	time: number,
	still: boolean,
): void {
	context.save()
	context.fillStyle = theme.current

	for (const wire of wires) {
		// Each run at its own current, in the direction it runs: where a wire
		// splits, each branch moves at its own share (C-73), and a stub that
		// carries nothing is not in the list at all (C-21).
		for (const flow of wire.flows ?? []) {
			if (flow.amps < MIN_VISIBLE_AMPS) continue

			const travelled = still ? DOT_SPACING / 2 : time * DOT_SPEED * Math.sqrt(flow.amps)
			const phase = ((travelled % DOT_SPACING) + DOT_SPACING) % DOT_SPACING

			for (const segment of flow.segments) {
				drawDots(context, segment, phase, viewport, still)
			}
		}
	}

	context.restore()
}

function drawDots(
	context: CanvasRenderingContext2D,
	segment: Segment,
	phase: number,
	viewport: Viewport,
	/** Chevrons along the segment's direction, rather than round dots. */
	pointed = false,
): void {
	const span = Math.hypot(segment.to.x - segment.from.x, segment.to.y - segment.from.y)
	if (span === 0) return

	const step = {
		x: (segment.to.x - segment.from.x) / span,
		y: (segment.to.y - segment.from.y) / span,
	}
	const radius = Math.max(1.5, 2 * viewport.zoom)

	for (let along = phase; along < span; along += DOT_SPACING) {
		const at = toScreen(viewport, {
			x: segment.from.x + step.x * along,
			y: segment.from.y + step.y * along,
		})
		context.beginPath()
		if (pointed) {
			// A small arrowhead: its tip ahead along the run, its back corners
			// either side of where a dot would have been.
			const reach = radius * 1.8
			const side = { x: -step.y, y: step.x }
			context.moveTo(at.x + step.x * reach, at.y + step.y * reach)
			context.lineTo(at.x - step.x * reach + side.x * reach, at.y - step.y * reach + side.y * reach)
			context.lineTo(at.x - step.x * reach - side.x * reach, at.y - step.y * reach - side.y * reach)
			context.closePath()
		} else {
			context.arc(at.x, at.y, radius, 0, Math.PI * 2)
		}
		context.fill()
	}
}

function drawWires(
	context: CanvasRenderingContext2D,
	wires: readonly SceneWire[],
	viewport: Viewport,
	theme: Theme,
	selectedNets: ReadonlySet<string> = new Set(),
): void {
	if (wires.length === 0) return

	context.save()
	context.strokeStyle = theme.wire
	context.fillStyle = theme.wire
	context.lineWidth = Math.max(1, 1.5 * viewport.zoom)
	context.lineCap = 'round'

	// Two passes, because a selected wire is a different colour and a canvas
	// path has one. Plain wires first: a selected one drawn over its neighbours
	// is easier to follow than one hidden under them.
	for (const chosen of [false, true]) {
		const batch = wires.filter((wire) => selectedNets.has(wire.id) === chosen)
		if (batch.length === 0) continue

		context.lineWidth = Math.max(1, (chosen ? 2.5 : 1.5) * viewport.zoom)

		// One path per colour: every wire in the plain colour unless wires
		// are drawn by voltage (C-134), when each net has its own.
		const byInk = new Map<string, SceneWire[]>()
		for (const wire of batch) {
			const ink = chosen ? theme.selection : (wire.ink ?? theme.wire)
			byInk.set(ink, [...(byInk.get(ink) ?? []), wire])
		}
		for (const [ink, inked] of byInk) {
			context.strokeStyle = ink
			// Heavier when it carries a voltage: the colour is the message,
			// and a hairline under the current dots does not deliver it.
			if (!chosen && inked[0].ink) context.lineWidth = Math.max(1.5, 2.5 * viewport.zoom)
			else context.lineWidth = Math.max(1, (chosen ? 2.5 : 1.5) * viewport.zoom)
			context.beginPath()
			for (const wire of inked) {
				for (const segment of wire.segments) {
					const from = toScreen(viewport, segment.from)
					const to = toScreen(viewport, segment.to)
					context.moveTo(from.x, from.y)
					context.lineTo(to.x, to.y)
				}
			}
			context.stroke()
		}
	}

	// A dot means connected; its absence means two wires merely cross. In
	// its wire's colour, so a joint reads as part of the net it joins.
	for (const wire of wires) {
		context.fillStyle = selectedNets.has(wire.id) ? theme.selection : (wire.ink ?? theme.wire)
		for (const junction of wireJunctions([wire])) {
			const at = toScreen(viewport, junction)
			context.beginPath()
			context.arc(at.x, at.y, Math.max(2, 3 * viewport.zoom), 0, Math.PI * 2)
			context.fill()
		}
	}

	context.restore()
}

function drawGrid(
	context: CanvasRenderingContext2D,
	viewport: Viewport,
	theme: Theme,
	size: { width: number; height: number },
): void {
	const spacing = GRID_SPACING * viewport.zoom
	if (spacing < MIN_GRID_PIXELS) return

	// Only the lines inside the viewport: drawing the whole plane and letting the
	// canvas clip is what makes a zoomed-out grid cost a frame.
	const first: Point = {
		x: Math.floor(viewport.pan.x / GRID_SPACING) * GRID_SPACING,
		y: Math.floor(viewport.pan.y / GRID_SPACING) * GRID_SPACING,
	}

	context.save()
	context.strokeStyle = theme.grid
	context.lineWidth = 1
	context.beginPath()

	for (let x = first.x; toScreen(viewport, { x, y: 0 }).x <= size.width; x += GRID_SPACING) {
		const at = Math.round(toScreen(viewport, { x, y: 0 }).x) + 0.5
		context.moveTo(at, 0)
		context.lineTo(at, size.height)
	}
	for (let y = first.y; toScreen(viewport, { x: 0, y }).y <= size.height; y += GRID_SPACING) {
		const at = Math.round(toScreen(viewport, { x: 0, y }).y) + 0.5
		context.moveTo(0, at)
		context.lineTo(size.width, at)
	}

	context.stroke()
	context.restore()
}

/**
 * A digit's segments, each dim or lit on its own (C-75).
 *
 * Dim is the outline, faint, because a real digit's unlit bars are visible
 * and that is how a person finds out a digit is there at all. Lit is the
 * segment's own light, thicker, glowing as far as it is bright — the same
 * brightness an LED at that current is given, since each bar is one.
 */
function drawSegments(
	context: CanvasRenderingContext2D,
	item: SceneItem,
	viewport: Viewport,
	theme: Theme,
): void {
	const shapes = item.symbol.segments
	if (!shapes) return
	if (item.symbol.glass) {
		drawGlass(context, item, item.symbol.glass, shapes, viewport)
		return
	}
	const light = LED_LIGHT.Red

	shapes.forEach((shape, index) => {
		const brightness = item.segments?.[index] ?? 0
		const lit = brightness > 0.02
		context.save()
		context.lineCap = 'round'
		context.strokeStyle = lit ? light : theme.outline
		context.fillStyle = context.strokeStyle
		context.globalAlpha = lit ? 0.45 + 0.55 * brightness : 0.25
		// Unlit, the pen every symbol is drawn with; lit, a bar of light.
		context.lineWidth = lit ? Math.max(2, 3 * viewport.zoom) : Math.max(1, 1.5 * viewport.zoom)
		if (lit) {
			context.shadowColor = light
			context.shadowBlur = 12 * brightness * viewport.zoom
		}
		drawShape(context, shape, item.position, viewport, item.turns)
		// The point is a dot, which has to be filled to be seen at all.
		if (shape.kind === 'circle') {
			const at = toScreen(viewport, {
				x: item.position.x + turned(shape.centre, item.turns).x,
				y: item.position.y + turned(shape.centre, item.turns).y,
			})
			context.beginPath()
			context.arc(at.x, at.y, shape.radius * 1.5 * viewport.zoom, 0, Math.PI * 2)
			context.fill()
		}
		context.restore()
	})
}

/**
 * The face of an LCD digit and its segments, each as dark as the glass has
 * turned (C-76).
 *
 * The glass's own colours whatever the theme, because it is a thing with a
 * colour of its own, as an LED's light is. A clear segment is nearly
 * invisible, as it is on a real one held to the light, and a dark one is
 * ink, with no glow: glass shows by blocking light, not by giving it.
 */
function drawGlass(
	context: CanvasRenderingContext2D,
	item: SceneItem,
	window: readonly Point[],
	shapes: readonly SymbolShape[],
	viewport: Viewport,
): void {
	context.save()
	context.fillStyle = LCD_FACE
	context.strokeStyle = LCD_FACE
	context.lineWidth = Math.max(1, 1.5 * viewport.zoom)
	drawShape(
		context,
		{ kind: 'polygon', filled: true, points: [...window] },
		item.position,
		viewport,
		item.turns,
	)

	shapes.forEach((shape, index) => {
		const dark = item.segments?.[index] ?? 0
		context.lineCap = 'round'
		context.strokeStyle = LCD_INK
		context.fillStyle = LCD_INK
		context.globalAlpha = 0.08 + 0.92 * dark
		// Clear, the pen every symbol is drawn with; turned, a bar of ink.
		context.lineWidth =
			dark > 0.02 ? Math.max(2, 3 * viewport.zoom) : Math.max(1, 1.5 * viewport.zoom)
		drawShape(context, shape, item.position, viewport, item.turns)
		if (shape.kind === 'circle') {
			const at = toScreen(viewport, {
				x: item.position.x + turned(shape.centre, item.turns).x,
				y: item.position.y + turned(shape.centre, item.turns).y,
			})
			context.beginPath()
			context.arc(at.x, at.y, shape.radius * 1.5 * viewport.zoom, 0, Math.PI * 2)
			context.fill()
		}
	})
	context.restore()
}

function drawItem(
	context: CanvasRenderingContext2D,
	item: SceneItem,
	viewport: Viewport,
	theme: Theme,
	selected = false,
	labelled = false,
): void {
	// A lit LED is drawn in its own colour: the outline is the first thing the
	// eye lands on, and leaving it grey makes a lit lamp look like an unlit one
	// standing in front of a light.
	const lit = item.brightness && item.brightness > 0.02 ? lightOf(item) : undefined
	// A part that has warmed is tinted by how far it has come towards its
	// limit, amber on the way and red at it (C-74).
	const warm = item.warmth === undefined ? undefined : heatInk(theme.outline, item.warmth)
	const ink = selected ? theme.selection : (lit ?? warm ?? theme.outline)

	context.save()
	context.strokeStyle = ink
	context.fillStyle = ink
	// Selected components are drawn heavier as well as coloured, so the
	// selection survives being looked at in greyscale or by someone who cannot
	// tell the two colours apart.
	context.lineWidth = Math.max(1, (selected ? 2.5 : 1.5) * viewport.zoom)
	context.lineJoin = 'round'
	context.lineCap = 'round'

	for (const shape of item.symbol.shapes) {
		drawShape(context, shape, item.position, viewport, item.turns)
	}
	drawSegments(context, item, viewport, theme)

	context.fillStyle = theme.pin
	for (const pin of item.symbol.pins) {
		const at = toScreen(viewport, pinPosition(item, pin))
		context.beginPath()
		context.arc(at.x, at.y, Math.max(1.5, 2 * viewport.zoom), 0, Math.PI * 2)
		context.fill()
	}

	if (item.label && item.symbol.labelInside) {
		// A chip's name, in the middle of its body, upright however it is
		// turned (C-77).
		const origin = toScreen(viewport, item.position)
		context.fillStyle = theme.text
		context.font = `${Math.max(8, 9 * viewport.zoom)}px ${theme.font}`
		context.textAlign = 'center'
		context.textBaseline = 'middle'
		context.fillText(item.label, origin.x, origin.y)
	} else if (item.label) {
		const origin = toScreen(viewport, item.position)
		const half = halfSizeOf(item.symbol, item.turns)
		context.fillStyle = theme.text
		context.font = `${Math.max(8, 9 * viewport.zoom)}px ${theme.font}`
		// Clear of the symbol however it is turned, rather than a fixed drop that
		// a rotated component grows past — and on a side no wire leaves from.
		const gap = 3 * viewport.zoom
		const place = labelSide(item.symbol, item.turns)
		switch (place) {
			case 'below':
				context.textAlign = 'center'
				context.textBaseline = 'top'
				context.fillText(item.label, origin.x, origin.y + half.y * viewport.zoom + gap)
				break
			case 'above':
				context.textAlign = 'center'
				context.textBaseline = 'bottom'
				context.fillText(item.label, origin.x, origin.y - half.y * viewport.zoom - gap)
				break
			case 'right':
				context.textAlign = 'left'
				context.textBaseline = 'middle'
				context.fillText(item.label, origin.x + half.x * viewport.zoom + gap, origin.y)
				break
			case 'left':
				context.textAlign = 'right'
				context.textBaseline = 'middle'
				context.fillText(item.label, origin.x - half.x * viewport.zoom - gap, origin.y)
				break
			// In the room between two leads, a little way out from the centre
			// on both axes: clear of the lead on each, and of the body, which
			// is narrowest there.
			//
			// A box has no such room — its body fills the corner — so there the
			// label goes outside it, level with the corner (C-75).
			default: {
				const [vertical, horizontal] = place.split('-')
				const down = vertical === 'below' ? 1 : -1
				const across = horizontal === 'right' ? 1 : -1
				context.textAlign = across > 0 ? 'left' : 'right'
				context.textBaseline = down > 0 ? 'top' : 'bottom'
				const [out, level] = item.symbol.boxed
					? [half.x * viewport.zoom + gap, half.y * viewport.zoom]
					: [0.2 * half.x * viewport.zoom + gap, 0.55 * half.y * viewport.zoom]
				context.fillText(item.label, origin.x + across * out, origin.y + down * level)
			}
		}
	}

	// Pin names, shown only while the component is hovered or selected. Polarity
	// is one of the first real ideas in electronics and the symbols carry it —
	// the long plate, the triangle and the bar — but a beginner cannot read
	// those yet (design doc §14, on-demand help).
	if (labelled) {
		context.fillStyle = theme.text
		context.font = `${Math.max(8, 8 * viewport.zoom)}px ${theme.font}`
		context.textBaseline = 'bottom'

		const offsets = item.symbol.pins.map((pin) => turned(pin.at, item.turns))
		for (const [index, pin] of item.symbol.pins.entries()) {
			const at = toScreen(viewport, pinPosition(item, pin))
			// Away from the body, so the name never sits on top of the symbol —
			// which side that is depends on where the pin ended up once the
			// component was turned, not on where the symbol drew it.
			const offset = offsets[index]
			// A pin pointing up or down with a neighbour a cell away along the
			// same edge — a header, as on an Uno (A-02) — has no room for its
			// name across: it is written along the pin, reading upwards, as a
			// board's silkscreen is.
			const upright = Math.abs(offset.y) >= Math.abs(offset.x)
			const crowded =
				upright &&
				offsets.some(
					(other, at) =>
						at !== index && other.y === offset.y && Math.abs(other.x - offset.x) <= GRID_SPACING,
				)
			if (crowded) {
				const pointsDown = offset.y > 0
				context.save()
				context.translate(at.x, at.y + (pointsDown ? 5 : -5) * viewport.zoom)
				context.rotate(-Math.PI / 2)
				context.textAlign = pointsDown ? 'right' : 'left'
				context.textBaseline = 'middle'
				context.fillText(pin.name, 0, 0)
				context.restore()
				continue
			}
			context.textAlign = offset.x < 0 ? 'right' : offset.x > 0 ? 'left' : 'center'
			const nudge = offset.x === 0 ? 0 : Math.sign(offset.x) * 4 * viewport.zoom
			// Below a pin that points down, which is outside the body; above
			// every other one. Always above put a bottom pin's name on the
			// body it leaves — on a digit, over the digit (C-75).
			const pointsDown = offset.y > 0 && Math.abs(offset.y) > Math.abs(offset.x)
			context.textBaseline = pointsDown ? 'top' : 'bottom'
			const lift = (pointsDown ? 5 : -5) * viewport.zoom
			context.fillText(pin.name, at.x + nudge, at.y + lift)
		}
	}

	context.restore()
}

/** Draws one shape of a symbol, in world coordinates offset by the item. */
function drawShape(
	context: CanvasRenderingContext2D,
	shape: SymbolShape,
	origin: Point,
	viewport: Viewport,
	turns = 0,
): void {
	// Turned in world coordinates rather than by transforming the canvas, so
	// the drawing goes through the same rule as the hit box and the pin
	// positions (C-40) — and so text drawn afterwards stays upright.
	const at = (point: Point) => {
		const local = turned(point, turns)
		return toScreen(viewport, { x: origin.x + local.x, y: origin.y + local.y })
	}

	context.beginPath()

	if (shape.kind === 'circle') {
		const centre = at(shape.centre)
		context.arc(centre.x, centre.y, shape.radius * viewport.zoom, 0, Math.PI * 2)
		context.stroke()
		return
	}

	const [first, ...rest] = shape.points
	const start = at(first)
	context.moveTo(start.x, start.y)
	for (const point of rest) {
		const next = at(point)
		context.lineTo(next.x, next.y)
	}

	if (shape.kind === 'polygon') {
		context.closePath()
		if (shape.filled) context.fill()
	}
	context.stroke()
}
