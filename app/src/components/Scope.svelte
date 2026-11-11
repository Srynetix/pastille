<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

<script lang="ts">
	import { viewPlot, type Circuit, type ElementView } from '../lib/core'
	import {
		DEFAULT_TIME_BASE,
		TIME_BASES,
		choicesFor,
		extentFor,
		fit,
		labelReached,
		reached,
		labelTimeBase,
		lanesFor,
		toY,
		type Extent,
	} from '../lib/scope'
	import { formatQuantity } from '../lib/units'

	// C-22. One lane per probe, sharing a time base: a probe is already the user
	// saying "this is the thing I want to know about" (C-21), so there is nothing
	// else to choose and no trace nobody asked for.
	//
	// A lane each rather than one axis for all of them, because a voltmeter and
	// an ammeter do not share a unit and drawing volts against amps would be a
	// picture of nothing. Each lane carries its own vertical scale for the same
	// reason.
	let {
		circuit,
		elements = [],
		time = 0,
	}: {
		/** The core's handle: the buffer lives there and is read, not copied. */
		circuit?: Circuit
		elements?: readonly ElementView[]
		/** Simulated seconds, which is what makes this redraw every frame. */
		time?: number
	} = $props()

	const lanes = $derived(lanesFor(elements))

	let span = $state(DEFAULT_TIME_BASE)
	/** Full scale per lane, by handle. Absent means "fit to what it is doing". */
	let pinned = $state<Record<string, number>>({})

	/** Lane height in CSS pixels. Enough to read a curve, cheap enough to stack. */
	const LANE_HEIGHT = 56

	/**
	 * Each lane's ink, in the order the lanes come (C-130). A colour per
	 * lane rather than per family: every probe is a meter, so a family's
	 * colour would draw them all alike — what the colour has to say is which
	 * trace is which name, and the swatch beside the name says it.
	 */
	const INKS = [
		'--family-lights',
		'--family-passive',
		'--family-power',
		'--family-switches',
		'--family-chips',
		'--family-logic',
	] as const
	const inkOf = (index: number) => INKS[index % INKS.length]

	/** Folded down to its header, for when the board wants the room (C-130). */
	let folded = $state(false)
	/** Where each trace is now, held as the meters' own labels are. */
	let latest = $state<Record<string, number | undefined>>({})

	let canvases = $state<Record<string, HTMLCanvasElement | undefined>>({})
	/** What each lane's trace actually reached, for its label (C-69). */
	let readings = $state<Record<string, Extent | undefined>>({})
	/**
	 * What each lane *would* show if it were not pinned.
	 *
	 * Kept apart from `extents` because the scales on offer come from this one:
	 * built from a pinned extent they would all be larger than the pin, and the
	 * setting in force would drop out of its own list.
	 */
	let fitted = $state<Record<string, Extent>>({})

	// Redrawn whenever the clock moves, which is every frame the simulation
	// advances — the same trick the canvas uses to animate the current dots.
	$effect(() => {
		// Named so the effect depends on them, and so the dependency is obvious
		// to whoever reads it next.
		void time
		void span
		void pinned
		void lanes

		if (!circuit || folded) return
		lanes.forEach((lane, index) => draw(lane.id, inkOf(index)))
	})

	function draw(id: string, inkToken: string) {
		const canvas = canvases[id]
		if (!canvas || !circuit) return

		const ratio = window.devicePixelRatio || 1
		const width = Math.max(1, Math.round(canvas.clientWidth))
		if (canvas.width !== width * ratio || canvas.height !== LANE_HEIGHT * ratio) {
			canvas.width = width * ratio
			canvas.height = LANE_HEIGHT * ratio
		}

		const context = canvas.getContext('2d')
		if (!context) return
		context.setTransform(ratio, 0, 0, ratio, 0, 0)
		context.clearRect(0, 0, width, LANE_HEIGHT)

		// Filled and then read in place: the numbers never cross the boundary
		// (ADR 0002), and the view must be taken after the fill and used before
		// anything else touches the core.
		const written = circuit.plotTrace(id, span, width)
		if (written === 0) {
			readings[id] = undefined
			latest[id] = undefined
			fitted[id] = { low: 0, high: 1 }
			return
		}
		const block = viewPlot(circuit, written)

		const fit_ = fit(block)
		const extent = extentFor(fit_, pinned[id])
		fitted[id] = fit_
		readings[id] = reached(block)
		latest[id] = lastOf(block, written)

		const ink = readColour(canvas, inkToken)
		context.strokeStyle = readColour(canvas, '--edge')
		context.lineWidth = 1
		// Zero, when the lane spans it: the line a trace is measured from.
		if (extent.low < 0 && extent.high > 0) {
			const zero = Math.round(toY(0, extent, LANE_HEIGHT)) + 0.5
			context.beginPath()
			context.moveTo(0, zero)
			context.lineTo(width, zero)
			context.stroke()
		}

		context.strokeStyle = ink
		context.lineWidth = 1.5
		context.beginPath()
		let drawing = false
		for (let column = 0; column < written / 2; column++) {
			const low = block[column * 2]
			const high = block[column * 2 + 1]
			// The core leaves a column the trace had not reached as NaN, which
			// is a gap rather than a zero: the pen lifts.
			if (!Number.isFinite(low) || !Number.isFinite(high)) {
				drawing = false
				continue
			}

			const x = column + 0.5
			const top = toY(high, extent, LANE_HEIGHT)
			const bottom = toY(low, extent, LANE_HEIGHT)
			if (!drawing) {
				context.moveTo(x, bottom)
				drawing = true
			} else {
				context.lineTo(x, bottom)
			}
			// The column's whole span, so a fast circuit reads as a band rather
			// than as whatever one sample happened to catch.
			if (bottom - top > 0.5) context.lineTo(x, top)
		}
		context.stroke()
	}

	/** The middle of the last column the trace reached, or nothing if it reached none. */
	function lastOf(
		block: Float64Array | Float32Array | number[],
		written: number,
	): number | undefined {
		for (let column = written / 2 - 1; column >= 0; column--) {
			const low = block[column * 2]
			const high = block[column * 2 + 1]
			if (Number.isFinite(low) && Number.isFinite(high)) return (low + high) / 2
		}
		return undefined
	}

	/** One theme token, resolved where the lane actually sits. */
	function readColour(element: HTMLElement, token: string): string {
		const value = getComputedStyle(element).getPropertyValue(token).trim()
		return value === '' ? '#888888' : value
	}

	function pin(id: string, choice: string) {
		const next = { ...pinned }
		if (choice === 'fit') delete next[id]
		else next[id] = Number(choice)
		pinned = next
	}
</script>

<aside class="scope" class:folded aria-label="Scope">
	<header>
		<svg class="icon" viewBox="0 0 24 24" aria-hidden="true"
			><path d="M3 12h4l3-8 4 16 3-8h4" /></svg
		>
		<h2>Scope</h2>
		<!-- What is being watched, so a folded scope still says it is there. -->
		<span class="probes">
			{lanes.length === 1 ? '1 probe' : `${lanes.length} probes`}{folded
				? ` · last ${labelTimeBase(span)}`
				: ''}
		</span>
		{#if !folded}
			<!-- Five choices, all on show: a menu hid the one setting that
			     decides whether a 555's swing is a line or a blur. -->
			<div class="bases" role="radiogroup" aria-label="Time base">
				{#each TIME_BASES as base (base)}
					<button
						type="button"
						role="radio"
						aria-checked={base === span}
						onclick={() => (span = base)}
					>
						{labelTimeBase(base)}
					</button>
				{/each}
			</div>
		{/if}
		<button
			type="button"
			class="fold"
			aria-label={folded ? 'Show the scope' : 'Fold the scope'}
			aria-expanded={!folded}
			onclick={() => (folded = !folded)}
		>
			<svg viewBox="0 0 24 24" aria-hidden="true"
				><path d={folded ? 'M6 15l6-6 6 6' : 'M6 9l6 6 6-6'} /></svg
			>
		</button>
	</header>

	{#if !folded}
		<div class="lanes">
			{#each lanes as lane, index (lane.id)}
				<div class="lane" style:--ink="var({inkOf(index)})">
					<div class="head">
						<span class="swatch" aria-hidden="true"></span>
						<span class="name">{lane.label}</span>
						<span class="now"
							>{latest[lane.id] === undefined
								? '—'
								: formatQuantity(latest[lane.id]!, lane.unit)}</span
						>
						<!-- The window's lowest and highest, when that says more than
						     the value beside it: a flat trace is one number, once. -->
						<span class="range" title="Lowest and highest in the window">
							{labelReached(readings[lane.id], lane.unit).includes('→')
								? labelReached(readings[lane.id], lane.unit)
								: ''}
						</span>
						<select
							aria-label="{lane.label} scale"
							value={pinned[lane.id] === undefined ? 'fit' : String(pinned[lane.id])}
							onchange={(event) => pin(lane.id, event.currentTarget.value)}
						>
							<option value="fit">Fit</option>
							{#each choicesFor(fitted[lane.id] ?? { low: 0, high: 1 }, pinned[lane.id]) as choice (choice)}
								<option value={String(choice)}>{formatQuantity(choice, lane.unit)}</option>
							{/each}
						</select>
					</div>
					<canvas bind:this={canvases[lane.id]} style="height: {LANE_HEIGHT}px"></canvas>
				</div>
			{/each}
		</div>
	{/if}
</aside>

<style>
	.scope {
		grid-area: scope;
		border-top: 1px solid var(--edge);
		padding: 0.5rem 1rem 0.75rem;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		min-height: 0;
	}

	.scope.folded {
		padding-bottom: 0.5rem;
	}

	header {
		display: flex;
		align-items: center;
		gap: 0.625rem;
	}

	.icon {
		width: 1rem;
		height: 1rem;
		color: var(--muted);
	}

	svg {
		fill: none;
		stroke: currentColor;
		stroke-width: 1.8;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	h2 {
		font-size: 0.875rem;
		font-weight: 750;
		margin: 0;
	}

	.probes {
		flex: 1;
		min-width: 0;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		font-size: 0.75rem;
		color: var(--muted);
	}

	.bases {
		display: flex;
		padding: 2px;
		background: var(--sunken);
		border: 1px solid var(--edge);
		border-radius: 8px;
	}

	.bases button {
		height: 1.5rem;
		padding: 0 0.5rem;
		white-space: nowrap;
		background: none;
		border: none;
		border-radius: 6px;
		color: var(--muted);
		font-family: var(--font-mono);
		font-variation-settings: var(--mono-axes);
		font-size: 0.75rem;
		cursor: pointer;
	}

	.bases button[aria-checked='true'] {
		background: var(--raised);
		color: var(--fg);
	}

	.bases button:focus-visible {
		color: var(--fg);
	}

	@media (hover: hover) {
		.bases button:hover {
			color: var(--fg);
		}
	}

	.fold {
		width: 1.75rem;
		height: 1.75rem;
		display: flex;
		align-items: center;
		justify-content: center;
		background: none;
		border: none;
		border-radius: 6px;
		color: var(--muted);
		cursor: pointer;
	}

	.fold:focus-visible {
		color: var(--fg);
		background: var(--raised);
	}

	@media (hover: hover) {
		.fold:hover {
			color: var(--fg);
			background: var(--raised);
		}
	}

	.fold svg {
		width: 1rem;
		height: 1rem;
	}

	select {
		background: var(--sunken);
		border: 1px solid var(--edge);
		border-radius: 4px;
		color: inherit;
		font: inherit;
		font-size: 0.75rem;
		padding: 0.0625rem 0.25rem;
	}

	/* Stacked, and scrollable rather than unbounded: six probes should not push
	   the canvas off the screen. */
	.lanes {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		overflow-y: auto;
		min-height: 0;
	}

	.head {
		display: flex;
		align-items: baseline;
		gap: 0.5rem;
		margin-bottom: 0.25rem;
		font-size: 0.75rem;
		color: var(--muted);
	}

	.swatch {
		width: 0.625rem;
		height: 0.1875rem;
		align-self: center;
		border-radius: 2px;
		background: var(--ink);
	}

	.name {
		color: var(--fg);
	}

	/* Where it is now, large and in the mono face: the number most often
	   wanted, read while it moves. */
	.now {
		font-family: var(--font-mono);
		font-variation-settings: var(--mono-axes);
		font-size: 1rem;
		font-weight: 500;
		color: var(--fg);
	}

	.range {
		flex: 1;
		font-family: var(--font-mono);
		font-variation-settings: var(--mono-axes);
	}

	canvas {
		display: block;
		width: 100%;
		background: var(--sunken);
		border: 1px solid var(--edge);
		border-radius: 6px;
	}

	/* On a phone the five time bases are a row of their own under the title,
	   each a 44 px target: beside it they wrapped onto two lines (C-138). */
	@media (width < 60rem) {
		header {
			flex-wrap: wrap;
		}

		.bases {
			order: 1;
			flex-basis: 100%;
		}

		.bases button {
			flex: 1;
			height: 2.75rem;
		}

		.fold {
			width: 2.75rem;
			height: 2.75rem;
		}
	}
</style>
