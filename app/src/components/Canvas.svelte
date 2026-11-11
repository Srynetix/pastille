<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

<script lang="ts">
	import type { Snippet } from 'svelte'

	import { glyphOf } from '../lib/glyph'
	import { CATALOGUE, familyOf, STARTERS } from '../lib/palette'
	import { drawScene, type Theme } from '../lib/render'

	/** What the canvas is drawn in before CSS has been read, or where it says nothing. */
	const FALLBACK_THEME: Theme = {
		grid: '#22253a',
		wire: '#9aa0be',
		current: '#ffc23d',
		selection: '#5cc8ff',
		body: '#1d2026',
		outline: '#8b919b',
		pin: '#e6e6e6',
		text: '#e6e6e6',
		font: 'system-ui, sans-serif',
	}
	import { routeBetween } from '../lib/routing'
	import { symbolFor } from '../lib/symbols'
	import {
		bendWire,
		hitTest,
		boundsOf,
		hitWire,
		itemsIn,
		pinPosition,
		pinRadiusFor,
		wireRadiusFor,
		type ElementHit,
		type Hit,
		type SceneItem,
		type SceneWire,
	} from '../lib/scene'
	import {
		fitTo,
		INITIAL_VIEWPORT,
		snapToGrid,
		toGrid,
		toWorld,
		panBy,
		zoomAt,
		type Bounds,
		type Point,
		type Viewport,
	} from '../lib/viewport'

	// The canvas draws a scene and reports what the pointer is over. It owns the
	// camera and nothing else: no netlist, no solver, no idea what a resistor is.
	let {
		children,
		whenEmpty,
		items = [],
		wires = [],
		time = 0,
		selected = new Set<string>(),
		selectedNets = new Set<string>(),
		onbeginmove,
		onmove,
		onendmove,
		onbend,
		onhover,
		onhold,
		onleave,
		onpick,
		onarea,
		onpress,
		onrelease,
		onviewport,
		onwire,
	}: {
		/**
		 * Anything to draw over the canvas — a note about what is on it.
		 *
		 * The canvas owns the only element the circuit is drawn in, so anything
		 * anchored to the circuit has to live inside it. What that thing says is
		 * none of its business.
		 */
		children?: Snippet
		/**
		 * Anything to add to the note shown while there is nothing on the
		 * canvas (C-47).
		 *
		 * A snippet rather than a list of anything, because the canvas knows
		 * that it is empty and nothing else about what is worth saying then.
		 */
		whenEmpty?: Snippet
		items?: readonly SceneItem[]
		wires?: readonly SceneWire[]
		/** Simulated seconds — what the current animation runs on. */
		time?: number
		onhover?: (grid: Point, hit: ElementHit | undefined) => void
		/**
		 * A finger held still on a part, and where on screen (C-133): what a
		 * pointer gets by hovering, for a screen that has no hover. Called
		 * again with nothing when the finger lifts or moves away.
		 */
		onhold?: (hit: ElementHit | undefined, at: Point) => void
		/** The pointer left the canvas, so nothing is under it any more. */
		onleave?: () => void
		/** Something was clicked. `additive` is true when shift was held. */
		onpick?: (hit: Hit | undefined, additive: boolean) => void
		/** A rectangle was dragged out, and these elements are the ones it touched. */
		onarea?: (ids: string[]) => void
		/** Reports the camera so a drop can be turned into world coordinates. */
		onviewport?: (viewport: Viewport) => void
		/** A wire was drawn from one pin to another. */
		onwire?: (from: Hit & { kind: 'pin' }, to: Hit & { kind: 'pin' }) => void
		/** The pointer went down on an element's body. */
		onpress?: (id: string) => void
		/**
		 * The pointer came up again, and whether the element was dragged
		 * anywhere in between.
		 *
		 * The pair is what a push-button is held down by and what tells a click
		 * on a switch from a drag of one (C-19). The canvas knows neither of
		 * those things — it reports the gesture and the shell decides.
		 */
		onrelease?: (id: string, moved: boolean) => void

		/** A wire was dragged into a new shape (C-43). */
		onbend?: (netId: string, waypoints: readonly Point[]) => void
		/** Ids of the selected elements, drawn heavier. */
		selected?: ReadonlySet<string>
		/** Ids of the selected nets, drawn in the selection colour. */
		selectedNets?: ReadonlySet<string>
		/** A drag is about to move things, so history can mark one step. */
		onbeginmove?: () => void
		/** An element was dragged to a new place. */
		onmove?: (id: string, x: number, y: number) => void
		/** The drag finished, so the history step is complete. */
		onendmove?: () => void
	} = $props()

	export function bounds(): DOMRect | undefined {
		return surface?.getBoundingClientRect()
	}

	/**
	 * Points the camera at a rectangle of the circuit, so that all of it is on
	 * screen (C-47).
	 *
	 * Imperative, like `bounds`, and for the same reason: the camera belongs to
	 * the canvas — it is the one thing here that knows how big it is — and this
	 * is a thing that *happens* rather than a state to be in. A circuit arriving
	 * from a file is an event; where the user has panned since is theirs.
	 */
	export function show(target: Bounds | undefined): void {
		if (!target) return
		// Measured now rather than taken from the last resize: the circuit
		// that asked for this can have changed the layout in the same update
		// — a probe brings the scope's row in under the canvas — and the
		// observer only says so after the camera has been pointed with the
		// old height, leaving the bottom of the circuit under the scope.
		if (frame) {
			const measured = frame.getBoundingClientRect()
			if (measured.width > 0) size = { width: measured.width, height: measured.height }
		}
		// Not measured yet — a canvas in a tab the browser has not laid out,
		// or one asked the moment it appeared — so the fit waits for the size
		// rather than being dropped, which is how an example used to open at
		// 1:1 with half of it off the edge.
		if (size.width === 0) {
			pendingFit = target
			return
		}
		viewport = fitTo(target, size)
	}

	/** The whole circuit on screen, or nothing to do on an empty canvas. */
	export function showAll(): void {
		show(boundsOf(items))
	}

	/**
	 * Zooms in (a factor over 1) or out, about the middle of the canvas — the
	 * buttons' and the keys' zoom, which have no pointer to zoom about.
	 */
	/**
	 * Lets go of a wire being drawn or an area being picked (C-103). Returns
	 * whether there was one, so Escape can clear the selection instead when
	 * nothing was under way. A part being moved is not undone here: it is
	 * where the hand put it, and Undo is how to take that back.
	 */
	export function cancel(): boolean {
		const underway = wiringFrom !== undefined || area !== undefined
		wiringFrom = undefined
		wiringTo = undefined
		area = undefined
		return underway
	}

	export function zoomBy(factor: number): void {
		if (size.width === 0) return
		viewport = zoomAt(viewport, { x: size.width / 2, y: size.height / 2 }, factor)
	}

	/** How far one press of a zoom button or key goes: a quarter. */
	const ZOOM_STEP = 1.25

	let surface: HTMLCanvasElement
	let frame: HTMLElement
	let viewport = $state<Viewport>(INITIAL_VIEWPORT)
	let size = $state({ width: 0, height: 0 })
	/** A fit asked for before the canvas knew its own size. */
	let pendingFit: Bounds | undefined

	/** Set while a drag is panning, holding the last pointer position. */
	let panningFrom: Point | undefined
	/**
	 * Set while a selection rectangle is being dragged out, in screen
	 * coordinates: where it started and where the pointer is now.
	 *
	 * Held down with shift, like the click it extends. A plain drag on the grid
	 * still moves the view, which is how everybody who has used the app so far
	 * gets around it, and shift already means "add to the selection". A finger
	 * has no shift: it holds still on the grid first, as it does on a part to
	 * ask what the part is doing (C-138).
	 */
	let area = $state<{ from: Point; to: Point } | undefined>()
	/** A finger resting on the grid, timed to become an area, and where it went down. */
	let gathering: { timer: ReturnType<typeof setTimeout>; at: Point } | undefined

	/**
	 * Every finger on the board and where it is now, in screen coordinates
	 * (C-138). Two of them are a pinch: the board zooms about the point
	 * between them by how far apart they have moved, and pans with it.
	 */
	// Gesture bookkeeping, read only by the handlers: nothing is drawn from it.
	// eslint-disable-next-line svelte/prefer-svelte-reactivity
	const touches = new Map<number, Point>()
	/** Set while two fingers are pinching: the camera and the two as they were when it began. */
	let pinch: { start: Viewport; apart: number; middle: Point } | undefined
	/**
	 * Fingers left on the board when a pinch ends. Each does nothing until it
	 * lifts: taking up the gesture it began before would jump the view to
	 * where that gesture had got to.
	 */
	// Gesture bookkeeping, read only by the handlers: nothing is drawn from it.
	// eslint-disable-next-line svelte/prefer-svelte-reactivity
	const spent = new Set<number>()
	/** The element under the pointer, whose pin names are worth showing. */
	let hovered = $state<string | undefined>()
	/**
	 * Set while elements are being dragged, holding where each one started.
	 *
	 * `pressed` is the one the pointer actually went down on, and `moved` says
	 * whether the drag ever took anything off its starting intersection — the
	 * two halves of telling a click from a drag (C-19).
	 */
	let moving:
		{ from: Point; origins: Map<string, Point>; pressed: string; moved: boolean } | undefined

	/** Set while a wire is being bent, holding which one and where it was grabbed. */
	let bending: { wire: SceneWire; grabbedAt: Point } | undefined
	/** Set while a wire is being drawn out of a pin. */
	let wiringFrom = $state<(Hit & { kind: 'pin' }) | undefined>()
	/** Where the wire being drawn currently ends, in world coordinates. */
	let wiringTo = $state<Point | undefined>()

	// The wire under construction is drawn like any other, so what the user sees
	// while dragging is what they will get when they let go.
	const pending = $derived.by(() => {
		if (!wiringFrom || !wiringTo) return []
		const from = pinPosition(wiringFrom.item, wiringFrom.pin)
		return [
			{
				id: 'pending',
				segments: routeBetween(from, wiringTo),
				pins: [from, wiringTo],
				waypoints: [],
			},
		]
	})

	// Redraw whenever the camera, the scene or the canvas size changes. Svelte's
	// effect graph is the invalidation mechanism, so there is no always-on
	// animation loop burning frames while nothing moves.
	//
	// This runs every frame while anything moves, so it does only drawing:
	// the backing store is reallocated only when the size really changed
	// (assigning `width` reallocates and clears it even to the same value),
	// and the colours are read from CSS only when the theme could have changed.
	$effect(() => {
		const context = surface?.getContext('2d')
		if (!context || size.width === 0) return

		const ratio = window.devicePixelRatio || 1
		const width = Math.round(size.width * ratio)
		const height = Math.round(size.height * ratio)
		if (surface.width !== width || surface.height !== height) {
			surface.width = width
			surface.height = height
		}
		context.setTransform(ratio, 0, 0, ratio, 0, 0)

		drawScene(
			context,
			items,
			viewport,
			theme,
			size,
			[...wires, ...pending],
			time,
			selected,
			new Set([...selected, ...(hovered ? [hovered] : [])]),
			selectedNets,
			reducedMotion,
		)
	})

	/** Whether the system has been asked for less motion; followed if it changes. */
	let reducedMotion = $state(false)
	$effect(() => {
		const query = matchMedia('(prefers-reduced-motion: reduce)')
		const read = () => (reducedMotion = query.matches)
		read()
		query.addEventListener('change', read)
		return () => query.removeEventListener('change', read)
	})

	$effect(() => {
		onviewport?.(viewport)
	})

	/**
	 * The canvas's colours, as CSS has them. Read again when the system theme
	 * flips or the page's own `data-theme` changes — the only two ways a token
	 * changes under a canvas that is already drawn.
	 */
	let theme = $state<Theme>(FALLBACK_THEME)
	$effect(() => {
		if (!frame) return
		const read = () => (theme = themeFromCss(frame))
		read()
		const scheme = matchMedia('(prefers-color-scheme: dark)')
		scheme.addEventListener('change', read)
		const observer = new MutationObserver(read)
		observer.observe(document.documentElement, {
			attributes: true,
			attributeFilter: ['data-theme'],
		})
		return () => {
			scheme.removeEventListener('change', read)
			observer.disconnect()
		}
	})

	$effect(() => {
		if (!frame) return
		const observer = new ResizeObserver(([entry]) => {
			size = {
				width: entry.contentRect.width,
				height: entry.contentRect.height,
			}
			if (pendingFit && size.width > 0) {
				viewport = fitTo(pendingFit, size)
				pendingFit = undefined
			}
		})
		observer.observe(frame)
		return () => observer.disconnect()
	})

	function pointerAt(event: PointerEvent | WheelEvent): Point {
		const bounds = surface.getBoundingClientRect()
		return { x: event.clientX - bounds.left, y: event.clientY - bounds.top }
	}

	function handleWheel(event: WheelEvent) {
		event.preventDefault()
		// A trackpad reports many small deltas and a mouse a few large ones;
		// exponentiating keeps both feeling like the same gesture.
		viewport = zoomAt(viewport, pointerAt(event), Math.exp(-event.deltaY / 400))
	}

	/**
	 * How long a finger rests on a part before it counts as asking what the
	 * part is doing (C-133): long enough that a tap or the start of a drag is
	 * never taken for it, short enough not to feel like waiting.
	 */
	const HOLD_MS = 450
	/** How far a finger may wander and still be holding still, in CSS pixels. */
	const HOLD_SLACK = 8
	/** The hold being timed, or held, and where it started on screen. */
	let hold:
		{ timer: ReturnType<typeof setTimeout> | undefined; at: Point; showing: boolean } | undefined

	function letGo() {
		if (gathering) {
			clearTimeout(gathering.timer)
			gathering = undefined
		}
		if (!hold) return
		clearTimeout(hold.timer)
		const { showing, at } = hold
		hold = undefined
		if (showing) onhold?.(undefined, at)
	}

	/**
	 * Captures a pointer to the canvas, so a drag that leaves it still ends
	 * here. One the browser has already taken back cannot be, and nothing is
	 * lost by not trying.
	 */
	function capture(pointer: number) {
		try {
			surface.setPointerCapture(pointer)
		} catch {
			// Not an active pointer any more.
		}
	}

	function release(pointer: number) {
		if (surface.hasPointerCapture(pointer)) surface.releasePointerCapture(pointer)
	}

	/**
	 * Takes back whatever the first finger had begun when a second one comes
	 * down, so a pinch never also moves a part, draws a wire or picks an area
	 * (C-138). A part goes back where it was, and its press ends as a drag —
	 * a switch is not flipped by a pinch that started on it.
	 */
	function abandon() {
		letGo()
		if (moving) {
			const { pressed, origins } = moving
			moving = undefined
			for (const [id, origin] of origins) onmove?.(id, origin.x, origin.y)
			onrelease?.(pressed, true)
			onendmove?.()
		}
		if (bending) {
			bending = undefined
			onendmove?.()
		}
		wiringFrom = undefined
		wiringTo = undefined
		area = undefined
		panningFrom = undefined
	}

	function between(a: Point, b: Point): Point {
		return { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 }
	}

	function handlePointerDown(event: PointerEvent) {
		if (event.pointerType === 'touch') {
			touches.set(event.pointerId, pointerAt(event))
			if (touches.size > 1) {
				capture(event.pointerId)
				if (touches.size === 2 && !pinch) {
					abandon()
					const [a, b] = [...touches.values()]
					pinch = {
						start: viewport,
						apart: Math.max(1, Math.hypot(b.x - a.x, b.y - a.y)),
						middle: between(a, b),
					}
				}
				return
			}
		}
		letGo()
		const screen = pointerAt(event)
		const world = toWorld(viewport, screen)
		// Elements first, then wires: a wire runs under the symbols it joins,
		// and clicking a resistor must never select the wire behind it (C-41).
		const hit =
			hitTest(items, world, pinRadiusFor(viewport.zoom)) ??
			hitWire(wires, world, wireRadiusFor(viewport.zoom))

		if (hit?.kind === 'pin') {
			// Dragging out of a pin draws a wire.
			wiringFrom = hit
			wiringTo = world
			capture(event.pointerId)
		} else if (hit?.kind === 'element') {
			// Dragging a component moves it, along with anything else selected.
			// Which ids are selected is settled by onpick below, which runs
			// first from the caller's point of view.
			const dragging = selected.has(hit.item.id) ? [...selected] : [hit.item.id]
			moving = {
				from: world,
				origins: new Map(
					dragging.flatMap((id) => {
						const item = items.find((candidate) => candidate.id === id)
						return item ? [[id, item.position] as [string, Point]] : []
					}),
				),
				pressed: hit.item.id,
				moved: false,
			}
			onbeginmove?.()
			onpress?.(hit.item.id)
			capture(event.pointerId)
			// A finger has no hover, so resting it on the part is how it asks
			// what the part is doing (C-133). A push-button is pressed all the
			// same: the hold only adds what it is doing to what it does.
			if (event.pointerType === 'touch' && onhold) {
				const at = { x: event.clientX, y: event.clientY }
				const item = hit
				hold = {
					at,
					showing: false,
					timer: setTimeout(() => {
						if (!hold) return
						hold.showing = true
						onhold(item, at)
					}, HOLD_MS),
				}
			}
		} else if (hit?.kind === 'wire') {
			// Dragging a wire bends it (C-43). The route is derived from the pins
			// and whatever bends it already has, so the drag only has to say
			// where this one now goes.
			bending = { wire: hit.wire, grabbedAt: world }
			onbeginmove?.()
			capture(event.pointerId)
		} else if (!hit && event.shiftKey) {
			// Shift-dragging empty space selects what it passes over.
			area = { from: screen, to: screen }
			capture(event.pointerId)
		} else if (!hit) {
			// Dragging empty space pans. A finger held still there first picks
			// an area instead, from where it went down (C-138).
			panningFrom = screen
			capture(event.pointerId)
			if (event.pointerType === 'touch' && onarea) {
				gathering = {
					at: screen,
					timer: setTimeout(() => {
						if (!gathering || !panningFrom) return
						area = { from: gathering.at, to: panningFrom }
						panningFrom = undefined
						gathering = undefined
						navigator.vibrate?.(10)
					}, HOLD_MS),
				}
			}
		}
		onpick?.(hit, event.shiftKey)
	}

	function handlePointerMove(event: PointerEvent) {
		const screen = pointerAt(event)
		if (touches.has(event.pointerId)) touches.set(event.pointerId, screen)
		if (spent.has(event.pointerId)) return
		if (pinch) {
			if (!touches.has(event.pointerId)) return
			const [a, b] = [...touches.values()]
			const middle = between(a, b)
			const panned = panBy(pinch.start, middle.x - pinch.middle.x, middle.y - pinch.middle.y)
			viewport = zoomAt(panned, middle, Math.hypot(b.x - a.x, b.y - a.y) / pinch.apart)
			return
		}
		// A finger that sets off is dragging, not asking: the readout goes.
		if (hold && Math.hypot(event.clientX - hold.at.x, event.clientY - hold.at.y) > HOLD_SLACK)
			letGo()
		// Nor is one that sets off across the grid picking an area: it pans.
		if (gathering && Math.hypot(screen.x - gathering.at.x, screen.y - gathering.at.y) > HOLD_SLACK)
			letGo()

		if (wiringFrom) {
			const world = toWorld(viewport, screen)
			const hit = hitTest(items, world, pinRadiusFor(viewport.zoom))
			// Snap the loose end onto a pin when there is one under the pointer,
			// so the preview shows the connection that will actually be made.
			wiringTo = hit?.kind === 'pin' ? pinPosition(hit.item, hit.pin) : world
			// While wiring, the pin names of whatever is under the pointer are
			// exactly what the user needs to see.
			hovered = hit?.item.id
			onhover?.(toGrid(world), hit)
			return
		}

		if (bending) {
			// Snapped like everything else: a bend off the grid puts the run
			// either side of it off the grid too.
			const to = snapToGrid(toWorld(viewport, screen))
			onbend?.(bending.wire.id, bendWire(bending.wire, bending.grabbedAt, to))
			return
		}

		if (moving) {
			const world = toWorld(viewport, screen)
			const delta = { x: world.x - moving.from.x, y: world.y - moving.from.y }
			for (const [id, origin] of moving.origins) {
				// Snapped, so a moved component lands on the grid exactly as a
				// freshly placed one does and its pins stay wireable.
				const at = snapToGrid({ x: origin.x + delta.x, y: origin.y + delta.y })
				// A pointer that wanders within one grid cell has moved nothing,
				// and a hand is never quite still: this is a drag only once
				// something has actually left where it was.
				if (at.x !== origin.x || at.y !== origin.y) moving.moved = true
				onmove?.(id, at.x, at.y)
			}
			return
		}

		if (area) {
			area = { from: area.from, to: screen }
			return
		}

		if (panningFrom) {
			viewport = panBy(viewport, screen.x - panningFrom.x, screen.y - panningFrom.y)
			panningFrom = screen
			return
		}

		const world = toWorld(viewport, screen)
		const hit = hitTest(items, world, pinRadiusFor(viewport.zoom))
		hovered = hit?.item.id
		onhover?.(toGrid(world), hit)
	}

	function handlePointerUp(event: PointerEvent) {
		touches.delete(event.pointerId)
		if (pinch || spent.has(event.pointerId)) {
			// One finger up ends the pinch; the other is spent until it lifts.
			if (pinch) for (const finger of touches.keys()) spent.add(finger)
			pinch = undefined
			spent.delete(event.pointerId)
			release(event.pointerId)
			return
		}
		letGo()
		if (bending) {
			bending = undefined
			onendmove?.()
			release(event.pointerId)
			return
		}

		if (moving) {
			const { pressed, moved } = moving
			moving = undefined
			// Before the gesture is closed, not after: a press and its release
			// are one thing the hand did, and a release that landed outside
			// would leave a step of history with the button still down — so
			// undoing a press would put it back stuck (C-19).
			onrelease?.(pressed, moved)
			onendmove?.()
			release(event.pointerId)
			return
		}

		if (wiringFrom) {
			const from = wiringFrom
			wiringFrom = undefined
			wiringTo = undefined
			release(event.pointerId)

			const target = hitTest(
				items,
				toWorld(viewport, pointerAt(event)),
				pinRadiusFor(viewport.zoom),
			)
			// Released anywhere but on another pin, nothing is created — the same
			// rule as a cancelled placement (M-16).
			if (
				target?.kind === 'pin' &&
				!(target.item.id === from.item.id && target.pin.name === from.pin.name)
			) {
				onwire?.(from, target)
			}
			return
		}

		if (area) {
			const { from, to } = area
			area = undefined
			release(event.pointerId)
			onarea?.(itemsIn(items, { min: toWorld(viewport, from), max: toWorld(viewport, to) }))
			return
		}

		if (!panningFrom) return
		panningFrom = undefined
		release(event.pointerId)
	}

	// Leaving the canvas is the only way hovering ends: no further move event
	// arrives, so without this the last thing hovered stays hovered for good.
	function handlePointerLeave() {
		if (wiringFrom || moving || bending || panningFrom || area) return
		hovered = undefined
		onleave?.()
	}

	/** Reads the theme from CSS so the canvas follows dark/light like everything else. */
	function themeFromCss(element: HTMLElement): Theme {
		const styles = getComputedStyle(element)
		const token = (name: string, fallback: string) =>
			styles.getPropertyValue(name).trim() || fallback
		return {
			grid: token('--grid', FALLBACK_THEME.grid),
			wire: token('--wire', FALLBACK_THEME.wire),
			current: token('--current', FALLBACK_THEME.current),
			selection: token('--selection', FALLBACK_THEME.selection),
			body: token('--raised', FALLBACK_THEME.body),
			outline: token('--muted', FALLBACK_THEME.outline),
			pin: token('--fg', FALLBACK_THEME.pin),
			text: token('--fg', FALLBACK_THEME.text),
			font: token('--font-canvas', FALLBACK_THEME.font),
		}
	}
</script>

<main class="canvas" aria-label="Circuit" bind:this={frame}>
	{#if items.length === 0}
		<!-- Ambient help (design doc §14): the first thing anyone sees is a blank
		     grid beside a palette, with nothing saying the two are related. It
		     goes the moment the first component lands, so it is never in the way
		     of the thing it was explaining. -->
		<div class="empty">
			<!-- C-131. The first thing anyone sees: what to build, in the
			     order it is built, drawn as the board will draw it. -->
			<h2 class="first">
				Build your <span class="lit">first circuit</span><span class="stop">.</span>
			</h2>
			<p>Drag a component onto the grid to start, then drag from pin to pin to wire them.</p>
			<ol class="steps">
				{#each STARTERS as part, index (part.kind)}
					{@const drawn = glyphOf(symbolFor(part.kind))}
					<li style:--tint="var(--family-{familyOf(part.kind)})">
						<span class="number" aria-hidden="true">{index + 1}</span>
						<svg viewBox={drawn.viewBox} aria-hidden="true">
							{#each drawn.shapes as shape, shapeIndex (shapeIndex)}
								<path d={shape.d} class:filled={shape.filled} />
							{/each}
						</svg>
						<span class="part">{part.label}</span>
						<span class="why">{part.hint}</span>
					</li>
				{/each}
			</ol>
			<p class="aside">A battery, a resistor and an LED will light up.</p>
			{@render whenEmpty?.()}
		</div>
	{/if}
	{@render children?.()}
	{#if area}
		<div
			class="area"
			style="left: {Math.min(area.from.x, area.to.x)}px; top: {Math.min(
				area.from.y,
				area.to.y,
			)}px; width: {Math.abs(area.to.x - area.from.x)}px; height: {Math.abs(
				area.to.y - area.from.y,
			)}px"
		></div>
		<!-- Under a finger the rectangle starts with no size, so a ring says the
		     hold took and the next drag picks rather than pans (C-138). -->
		<div class="started" style="left: {area.from.x}px; top: {area.from.y}px"></div>
	{/if}
	{#if items.length > 0}
		<!-- Zoom for somebody without a wheel, or who has not guessed that the
		     wheel zooms — and the way back to the whole circuit, which nothing
		     else offers once it has been panned away from. -->
		<div class="zoom" role="group" aria-label="Zoom">
			<button
				type="button"
				title="Zoom in (+)"
				aria-label="Zoom in"
				onclick={() => zoomBy(ZOOM_STEP)}>+</button
			>
			<button
				type="button"
				title="Zoom out (−)"
				aria-label="Zoom out"
				onclick={() => zoomBy(1 / ZOOM_STEP)}>−</button
			>
			<button
				type="button"
				title="Show it all (0)"
				aria-label="Show the whole circuit"
				onclick={showAll}>⤢</button
			>
		</div>
	{/if}
	<canvas
		bind:this={surface}
		style="width: {size.width}px; height: {size.height}px"
		onwheel={handleWheel}
		onpointerdown={handlePointerDown}
		onpointermove={handlePointerMove}
		onpointerup={handlePointerUp}
		onpointercancel={handlePointerUp}
		onpointerleave={handlePointerLeave}
	></canvas>
	<!-- The board in words, and the keyboard's way onto it. The canvas is a
	     picture to anything but a pointer: a screen reader found nothing in
	     it, and Tab could not reach one part of it. Each part here is a
	     button that selects it, as a click on it does, and the inspector
	     it opens can turn it, delete it or change it. Out of sight until it
	     holds the focus, and then drawn over the board, so the focus is
	     never somewhere nobody can see. -->
	{#if items.length > 0}
		<ul class="parts-list" aria-label="Parts on the board">
			{#each items as item (item.id)}
				<li>
					<button
						type="button"
						aria-pressed={selected.has(item.id)}
						onclick={() => onpick?.({ kind: 'element', item }, false)}
					>
						{[item.kind ? CATALOGUE[item.kind].label : undefined, item.label]
							.filter(Boolean)
							.join(', ')}
					</button>
				</li>
			{/each}
		</ul>
	{/if}
</main>

<style>
	.parts-list:not(:focus-within) {
		position: absolute;
		width: 1px;
		height: 1px;
		overflow: hidden;
		clip-path: inset(50%);
		white-space: nowrap;
	}

	.parts-list:focus-within {
		position: absolute;
		top: 0.5rem;
		left: 0.5rem;
		z-index: 5;
		max-height: calc(100% - 1rem);
		overflow-y: auto;
		margin: 0;
		padding: 0.25rem;
		list-style: none;
		background: var(--raised);
		border: 1px solid var(--edge);
		border-radius: 6px;
	}

	.parts-list button {
		display: block;
		width: 100%;
		padding: 0.25rem 0.5rem;
		text-align: left;
		background: none;
		border: none;
		border-radius: 4px;
		color: var(--fg);
		font: inherit;
		font-size: 0.8125rem;
	}

	.parts-list button[aria-pressed='true'] {
		background: var(--selection);
		color: var(--on-selection);
	}

	.canvas {
		grid-area: canvas;
		background: var(--sunken);
		overflow: hidden;
		position: relative;
	}

	canvas {
		display: block;
		touch-action: none;
	}

	/* The rectangle being dragged out: the selection colour, faint enough to
	   see the circuit through. Over the canvas, never catching the pointer the
	   drag is being made with. */
	.area {
		position: absolute;
		pointer-events: none;
		border: 1px solid var(--accent);
		background: color-mix(in srgb, var(--accent) 12%, transparent);
	}

	.started {
		position: absolute;
		pointer-events: none;
		width: 2.75rem;
		height: 2.75rem;
		translate: -50% -50%;
		border: 2px solid var(--accent);
		border-radius: 50%;
	}

	.zoom {
		position: absolute;
		right: 0.5rem;
		bottom: 0.5rem;
		display: flex;
		gap: 0.25rem;
		z-index: 1;
	}

	.zoom button {
		width: 2rem;
		height: 2rem;
		padding: 0;
		font-size: 1rem;
		line-height: 1;
	}

	.empty {
		position: absolute;
		inset: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.75rem;
		/* Never in the way: the hint sits over the canvas the drag has to reach. */
		pointer-events: none;
		text-align: center;
		padding: 2.5rem 1.5rem 1rem;
		overflow: hidden;
	}

	.empty p {
		margin: 0;
		max-width: 34rem;
		color: var(--muted);
	}

	.first {
		margin: 0;
		font-size: clamp(1.75rem, 3.2vw, 2.5rem);
		font-weight: 850;
		line-height: 1.1;
		color: var(--fg);
	}

	.lit {
		color: var(--brand);
	}

	.stop {
		color: var(--family-lights);
	}

	.steps {
		list-style: none;
		margin: 0.25rem 0 0;
		padding: 0;
		display: grid;
		grid-template-columns: repeat(3, minmax(0, 11rem));
		gap: 0.625rem;
	}

	.steps li {
		position: relative;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.375rem;
		padding: 0.875rem 0.75rem 0.75rem;
		background: color-mix(in srgb, var(--tint) 8%, var(--bg));
		border: 1px solid color-mix(in srgb, var(--tint) 35%, transparent);
		border-radius: 14px;
		color: var(--tint);
	}

	.steps svg {
		width: 3.5rem;
		height: 1.75rem;
		overflow: visible;
	}

	.steps path {
		fill: none;
		stroke: currentColor;
		stroke-width: 1.6;
		stroke-linecap: round;
		stroke-linejoin: round;
		vector-effect: non-scaling-stroke;
	}

	.steps path.filled {
		fill: currentColor;
	}

	.number {
		position: absolute;
		left: 0.5rem;
		top: 0.5rem;
		width: 1.375rem;
		height: 1.375rem;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 50%;
		background: var(--tint);
		color: var(--on-brand);
		font-family: var(--font-display);
		font-variation-settings: var(--display-axes);
		font-size: 0.75rem;
		font-weight: 850;
	}

	.part {
		font-family: var(--font-display);
		font-variation-settings: var(--display-axes);
		font-weight: 750;
		color: var(--fg);
	}

	.why {
		font-size: 0.75rem;
		line-height: 1.35;
		color: var(--muted);
	}

	.empty .aside {
		font-size: 0.8125rem;
		opacity: 0.8;
	}

	/* On a phone the board is short, and the examples need the room more
	   than the steps need their sentences: the names and the symbols say
	   enough, and the palette says the rest. C-132 gives the phone a board
	   of its own. */
	@media (width < 60rem) {
		/* Its foot clear of the thumb bar, which would otherwise sit over the
		   examples it ends with (C-138): the bar's 3.25rem and its margins. */
		.empty {
			padding: 1.25rem 1rem 4.75rem;
			gap: 0.5rem;
		}

		.first {
			font-size: 1.625rem;
		}

		.steps li {
			padding: 0.625rem 0.5rem 0.5rem;
		}

		.why,
		.empty .aside {
			display: none;
		}
	}
</style>
