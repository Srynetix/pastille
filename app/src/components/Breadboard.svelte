<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

<!--
	The breadboard view (B-14, design doc §6.3): the same circuit, on the board
	it is built on.

	A second projection of one netlist, as ADR 0006 has it. The core places the
	circuit (B-13) and says where every leg and jumper is; this draws that, and
	the one edit it takes — a part dragged to other holes — goes back to the
	core, which keeps it there and places the rest around it. The schematic is
	never touched from here: what a move can do is make the board build
	something else, and the problems under the board say what.

	Parts are their schematic symbols on a tile over their legs, as decided for
	the first cut; a chip is a DIP straddling the channel with pin 1 marked.
	Units are pitches of 0.1″, drawn ten to the pitch.
-->
<script lang="ts">
	import type { BoardPart, BoardView } from '../lib/core'
	import { glyphOf } from '../lib/glyph'
	import { familyOf } from '../lib/palette'
	import { symbolFor, type SymbolDefinition, type SymbolKind } from '../lib/symbols'

	let {
		board,
		refused,
		selected,
		onpick,
		onmove,
	}: {
		/** What the core says is on the board, or nothing while there is no circuit. */
		board?: BoardView
		/** Why the circuit is not on a board, when it cannot be put on one. */
		refused?: string
		/** The selection, shared with the schematic. */
		selected: ReadonlySet<string>
		/** A part was picked, or the board was, which picks nothing. */
		onpick?: (id: string | undefined) => void
		/** A part was dragged by whole holes. */
		onmove?: (id: string, dx: number, dy: number) => void
	} = $props()

	/** Drawn units to a pitch, and room around the board — more above it,
	 * where a battery sits with its leads in the rails. */
	const P = 10
	const M = 24
	const ABOVE = 64

	const px = (x: number) => M + x * P
	const py = (y: number) => ABOVE + y * P

	const width = $derived(board ? px(board.rows - 1) + M : 0)
	const height = py(17) + M

	/** Holes a problem is in, to be drawn as such. */
	const troubled = $derived(new Set(board?.problems.flatMap((problem) => problem.holes) ?? []))

	/**
	 * Building it, a step at a time (B-15): which step is on, or nothing
	 * while the board is simply looked at.
	 */
	let step = $state<number | undefined>()
	const steps = $derived(board?.steps ?? [])
	/** A board that changed under the steps keeps the step it can. */
	const current = $derived(
		step === undefined ? undefined : Math.min(step, Math.max(steps.length - 1, 0)),
	)
	/** The step each part goes in at, and each jumper. */
	const partStep = $derived(
		new Map(steps.flatMap((each, index) => each.elements.map((id) => [id, index] as const))),
	)
	const jumperStep = $derived(
		steps.flatMap((each, index) => (each.elements.length === 0 ? [index] : [])),
	)
	/** Whether a part or a jumper is on the board yet, at this step. */
	const shown = (at: number | undefined) =>
		current === undefined || at === undefined || at <= current

	/** A part being dragged, and by how many holes so far. */
	let dragging = $state<
		{ id: string; dx: number; dy: number; from: { x: number; y: number }; dip: boolean } | undefined
	>()
	let svg = $state<SVGSVGElement>()

	/** A pointer's position in the drawing's own units. */
	function local(event: PointerEvent): { x: number; y: number } {
		const matrix = svg?.getScreenCTM()
		if (!svg || !matrix) return { x: 0, y: 0 }
		const point = new DOMPoint(event.clientX, event.clientY).matrixTransform(matrix.inverse())
		return { x: point.x, y: point.y }
	}

	function grab(event: PointerEvent, part: BoardPart) {
		event.stopPropagation()
		;(event.currentTarget as Element).setPointerCapture?.(event.pointerId)
		dragging = { id: part.elementId, dx: 0, dy: 0, from: local(event), dip: part.dip != null }
		if (!part.movable) {
			// Picked, not dragged: a battery stays on its rails.
			dragging = undefined
			onpick?.(part.elementId)
		}
	}

	function drag(event: PointerEvent) {
		if (!dragging) return
		const at = local(event)
		const dx = Math.round((at.x - dragging.from.x) / P)
		// A chip slides along the channel and nowhere else.
		const dy = dragging.dip ? 0 : Math.round((at.y - dragging.from.y) / P)
		if (dx !== dragging.dx || dy !== dragging.dy) dragging = { ...dragging, dx, dy }
	}

	function drop() {
		if (!dragging) return
		const { id, dx, dy } = dragging
		dragging = undefined
		if (dx === 0 && dy === 0) onpick?.(id)
		else onmove?.(id, dx, dy)
	}

	/** Where a part is drawn while it is dragged. */
	function shift(part: BoardPart): string {
		return dragging?.id === part.elementId ? `translate(${dragging.dx * P} ${dragging.dy * P})` : ''
	}

	function glyphFor(kind: SymbolKind) {
		const symbol = symbolFor(kind) as SymbolDefinition | undefined
		return symbol ? glyphOf(symbol) : undefined
	}

	/**
	 * Where a part's tile sits: in the middle of its legs, or for a battery,
	 * above the board over the rails its leads go into.
	 */
	function centre(part: BoardPart) {
		const xs = part.legs.map((leg) => leg.x)
		const ys = part.legs.map((leg) => leg.y)
		const x = (Math.min(...xs) + Math.max(...xs)) / 2
		if (!part.movable) return { x: Math.max(x, 1), y: -3.6 }
		const y = (Math.min(...ys) + Math.max(...ys)) / 2
		// A part from one side of a chip to the other would sit on the chip:
		// its tile goes just above the chip instead, its leads still to its
		// holes.
		const under = chips.some(
			(chip) => x >= chip.from - 0.5 && x <= chip.to + 0.5 && y > 6 && y < 11,
		)
		return { x, y: under ? 5.2 : y }
	}

	/** A rectangle, in pitches: what a tile and its label cover, or a chip. */
	type Box = { left: number; right: number; top: number; bottom: number }

	/** Half a tile, and the label's height above it, in pitches. */
	const HALF = 0.8
	const LABEL = 0.75
	/** A label's width, in pitches: the 5-unit font is about 0.28 of a pitch to a character. */
	const labelWidth = (label: string) => label.length * 0.28

	function boxAt(x: number, y: number, label: string): Box {
		const half = Math.max(HALF, labelWidth(label) / 2)
		return { left: x - half, right: x + half, top: y - HALF - LABEL, bottom: y + HALF }
	}

	/**
	 * The moves a tile may make, in pitches, nearest first: half a pitch at a
	 * time, up to 8 up or down and 3 sideways, a sideways step counting as
	 * two, so a tile leaves its column only when it has to. Upward first on a
	 * tie, then leftward.
	 */
	const OFFSETS: [number, number][] = (() => {
		const steps = (reach: number) =>
			Array.from({ length: reach * 4 + 1 }, (_, index) => (index - reach * 2) / 2)
		return steps(3)
			.flatMap((dx) => steps(8).map((dy): [number, number] => [dx, dy]))
			.sort(
				([ax, ay], [bx, by]) =>
					4 * ax * ax + ay * ay - (4 * bx * bx + by * by) || ay - by || ax - bx,
			)
	})()

	const overlaps = (a: Box, b: Box) =>
		a.left < b.right + 0.15 &&
		b.left < a.right + 0.15 &&
		a.top < b.bottom + 0.15 &&
		b.top < a.bottom + 0.15

	/**
	 * Where each part's tile is drawn. Midway between its legs is where it
	 * starts; when that would cover a tile already drawn, a label or a chip,
	 * it moves to the nearest place that is free, half a pitch at a time —
	 * up or down first, sideways only when a crowded board leaves no room
	 * above or below — and its leads still run to its holes. Parts that share
	 * strips, as most do, would otherwise pile their tiles into one spot.
	 *
	 * From left to right, so the result does not depend on the order the core
	 * lists the parts in, and a part keeps its spot while another is dragged.
	 */
	const spots = $derived.by(() => {
		const placed: Box[] = [
			...chips.map((chip) => ({
				left: chip.from - 0.5,
				right: chip.to + 0.5,
				top: 6.4,
				bottom: 10.6,
			})),
			// Every leg's end, so no tile or label hides where a lead goes in.
			...(board?.parts ?? [])
				.filter((part) => !part.dip && part.movable)
				.flatMap((part) => part.legs)
				.map((leg) => ({
					left: leg.x - 0.2,
					right: leg.x + 0.2,
					top: leg.y - 0.2,
					bottom: leg.y + 0.2,
				})),
		]
		// Built afresh on every run and handed back whole: never mutated after.
		// eslint-disable-next-line svelte/prefer-svelte-reactivity
		const result = new Map<string, { x: number; y: number }>()
		const parts = (board?.parts ?? [])
			.filter((part) => !part.dip)
			.map((part) => ({ part, at: centre(part) }))
			.sort((a, b) => a.at.x - b.at.x || a.at.y - b.at.y)
		const last = (board?.rows ?? 1) - 1
		for (const { part, at } of parts) {
			let spot = at
			if (part.movable) {
				const free = OFFSETS.map(([dx, dy]) => ({ x: at.x + dx, y: at.y + dy }))
					.filter(
						(candidate) =>
							candidate.y >= -1 && candidate.y <= 17 && candidate.x >= 0 && candidate.x <= last,
					)
					.find((candidate) =>
						placed.every((box) => !overlaps(boxAt(candidate.x, candidate.y, part.label), box)),
					)
				spot = free ?? at
			}
			placed.push(boxAt(spot.x, spot.y, part.label))
			result.set(part.elementId, spot)
		}
		return result
	})

	/** Where each chip's body is along the board, in pitches. */
	const chips = $derived(
		(board?.parts ?? [])
			.filter((part) => part.dip)
			.map((part) => {
				const from = Math.min(...part.legs.map((leg) => leg.x))
				return { from, to: from + (part.dip ?? 0) / 2 - 1 }
			}),
	)

	/** A jumper's path: a bow, so two along one row do not hide each other. */
	function bow(from: { x: number; y: number }, to: { x: number; y: number }): string {
		const [ax, ay, bx, by] = [px(from.x), py(from.y), px(to.x), py(to.y)]
		const lift = Math.min(12, Math.hypot(bx - ax, by - ay) / 4)
		return `M${ax} ${ay}Q${(ax + bx) / 2} ${(ay + by) / 2 - lift} ${bx} ${by}`
	}
</script>

{#if refused}
	<section class="breadboard" aria-label="Breadboard">
		<p class="problems" role="alert">{refused}</p>
	</section>
{:else if board}
	<section class="breadboard" aria-label="Breadboard">
		<div class="top">
			<p class="caption">
				{board.size === 'half' ? 'Half' : 'Full'} breadboard, {board.holes.length} holes. Drag a part
				to move it on the board; the schematic stays as it is.
			</p>
			{#if steps.length > 0}
				<button
					type="button"
					class="build"
					aria-pressed={current !== undefined}
					onclick={() => (step = step === undefined ? 0 : undefined)}
				>
					{current === undefined ? 'Build it step by step' : 'Show it all'}
				</button>
			{/if}
		</div>
		{#if current !== undefined}
			<!-- B-15. One step, said, and the board showing what is in so far
			     with this step's holes ringed. -->
			<div class="steps" aria-live="polite">
				<p class="count">Step {current + 1} of {steps.length}</p>
				<p class="say">{steps[current].text}</p>
				<p class="go">
					<button type="button" disabled={current === 0} onclick={() => (step = current - 1)}
						>Back</button
					>
					{#if current < steps.length - 1}
						<button type="button" class="next" onclick={() => (step = current + 1)}>Next</button>
					{:else}
						<button type="button" class="next" onclick={() => (step = undefined)}>Done</button>
					{/if}
				</p>
			</div>
		{/if}
		<div class="scroll">
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<svg
				bind:this={svg}
				viewBox="0 0 {width} {height}"
				style:min-width="{board.rows * 14}px"
				role="img"
				aria-label="The circuit on a breadboard"
				onpointermove={drag}
				onpointerup={drop}
				onpointercancel={() => (dragging = undefined)}
				onpointerdown={() => onpick?.(undefined)}
			>
				<rect
					class="body"
					x={M - 14}
					y={ABOVE - 14}
					width={width - 2 * M + 28}
					height={py(17) - ABOVE + 28}
					rx="6"
				/>
				<rect class="channel" x={M - 14} y={py(7.7)} width={width - 2 * M + 28} height={P * 1.6} />
				<!-- The rails' stripes: + red outermost, − blue inside. -->
				<line class="plus" x1={M} x2={width - M} y1={py(-0.7)} y2={py(-0.7)} />
				<line class="minus" x1={M} x2={width - M} y1={py(1.7)} y2={py(1.7)} />
				<line class="minus" x1={M} x2={width - M} y1={py(15.3)} y2={py(15.3)} />
				<line class="plus" x1={M} x2={width - M} y1={py(17.7)} y2={py(17.7)} />
				{#each { length: board.rows } as _, row (row)}
					{#if row === 0 || (row + 1) % 5 === 0}
						<text class="number" x={px(row)} y={py(2.2)}>{row + 1}</text>
					{/if}
				{/each}
				{#each ['a', 'b', 'c', 'd', 'e'] as letter, index (letter)}
					<text class="letter" x={M - 18} y={py(3 + index) + 3}>{letter}</text>
				{/each}
				{#each ['f', 'g', 'h', 'i', 'j'] as letter, index (letter)}
					<text class="letter" x={M - 18} y={py(10 + index) + 3}>{letter}</text>
				{/each}
				{#each board.holes as hole (hole.name + hole.x + hole.y)}
					<rect
						class="hole"
						class:trouble={troubled.has(hole.name)}
						x={px(hole.x) - 2.2}
						y={py(hole.y) - 2.2}
						width="4.4"
						height="4.4"
						rx="0.8"
					/>
				{/each}
				{#each board.jumpers as jumper, index (index)}
					{#if shown(jumperStep[index])}
						<path
							class="jumper"
							class:now={current !== undefined && jumperStep[index] === current}
							d={bow(jumper.from, jumper.to)}
						/>
					{/if}
				{/each}
				{#each board.parts.filter( (part) => shown(partStep.get(part.elementId)) ) as part (part.elementId)}
					{@const tint = `var(--family-${familyOf(part.kind) ?? 'passive'})`}
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<g
						class="part"
						class:selected={selected.has(part.elementId)}
						class:fixed={!part.movable}
						class:dragging={dragging?.id === part.elementId}
						class:now={current !== undefined && partStep.get(part.elementId) === current}
						transform={shift(part)}
						style:--tint={tint}
						onpointerdown={(event) => grab(event, part)}
					>
						<title
							>{part.label} — {part.legs
								.map((leg) => `${leg.pin} in ${leg.hole}`)
								.join(', ')}</title
						>
						{#if part.dip}
							{@const xs = part.legs.map((leg) => leg.x)}
							{@const first = Math.min(...xs)}
							{@const last = first + part.dip / 2 - 1}
							<rect
								class="chip"
								x={px(first) - 5}
								y={py(6.4)}
								width={(last - first) * P + 10}
								height={P * 4.2}
								rx="1.5"
							/>
							<path class="notch" d="M{px(first) - 5} {py(8.1)}a2.6 2.6 0 0 1 0 {P * 0.8}" />
							<circle class="pin-one" cx={px(first)} cy={py(6.9)} r="1.3" />
							<text class="chip-label" x={(px(first) + px(last)) / 2} y={py(8.7)}>{part.label}</text
							>
							{#each part.legs as leg (leg.pin)}
								<rect class="leg" x={px(leg.x) - 1.5} y={py(leg.y) - 1.5} width="3" height="3" />
							{/each}
						{:else}
							{@const c = spots.get(part.elementId) ?? centre(part)}
							{@const glyph = glyphFor(part.kind)}
							{#each part.legs as leg (leg.pin)}
								<line class="lead" x1={px(leg.x)} y1={py(leg.y)} x2={px(c.x)} y2={py(c.y)} />
								<circle class="leg-end" cx={px(leg.x)} cy={py(leg.y)} r="2" />
							{/each}
							<rect class="tile" x={px(c.x) - 8} y={py(c.y) - 8} width="16" height="16" rx="3" />
							{#if glyph}
								<svg
									x={px(c.x) - 6.5}
									y={py(c.y) - 6.5}
									width="13"
									height="13"
									viewBox={glyph.viewBox}
								>
									{#each glyph.shapes as shape, index (index)}
										<path class="glyph" class:filled={shape.filled} d={shape.d} />
									{/each}
								</svg>
							{/if}
							<text class="label" x={px(c.x)} y={py(c.y) - 10}>{part.label}</text>
						{/if}
					</g>
				{/each}
				{#if current !== undefined}
					{#each steps[current].holes as spot, index (index)}
						<circle class="ring" cx={px(spot.x)} cy={py(spot.y)} r="4.5" />
					{/each}
				{/if}
			</svg>
		</div>
		{#if board.problems.length > 0}
			<ul class="problems" role="alert">
				{#each board.problems as problem, index (index)}
					<li>{problem.message}</li>
				{/each}
			</ul>
		{/if}
	</section>
{/if}

<style>
	.breadboard {
		grid-area: canvas;
		z-index: 1;
		display: flex;
		flex-direction: column;
		min-height: 0;
		min-width: 0;
		background: var(--bg);
		padding: 0.75rem 1rem;
		gap: 0.5rem;
	}

	.top {
		display: flex;
		align-items: center;
		gap: 0.75rem;
	}

	.caption {
		flex: 1;
		margin: 0;
		color: var(--muted);
		font-size: 0.8125rem;
	}

	.build {
		flex: none;
	}

	.steps {
		border-left: 3px solid var(--accent);
		background: var(--raised);
		padding: 0.5rem 0.75rem;
	}

	.steps p {
		margin: 0;
	}

	.count {
		color: var(--muted);
		font-size: 0.75rem;
	}

	.say {
		margin: 0.25rem 0 0.5rem;
		font-size: 0.9375rem;
	}

	.go {
		display: flex;
		gap: 0.5rem;
	}

	.ring {
		fill: none;
		stroke: var(--accent);
		stroke-width: 1.4;
		pointer-events: none;
	}

	.jumper.now {
		stroke: var(--accent);
		stroke-width: 2.4;
	}

	.part.now .tile,
	.part.now .chip {
		stroke: var(--accent);
		stroke-width: 2.4;
	}

	.scroll {
		flex: 1;
		min-height: 0;
		overflow: auto;
	}

	svg {
		display: block;
		width: 100%;
		height: auto;
		touch-action: none;
		user-select: none;
	}

	.body {
		fill: var(--raised);
		stroke: var(--edge);
	}

	.channel {
		fill: var(--sunken);
		opacity: 0.6;
	}

	.plus,
	.minus {
		stroke-width: 0.8;
	}

	.plus {
		stroke: var(--failed);
	}

	.minus {
		stroke: var(--accent);
	}

	.number,
	.letter {
		fill: var(--muted);
		font: 5px var(--font-mono);
		text-anchor: middle;
	}

	.hole {
		fill: var(--sunken);
		stroke: var(--edge);
		stroke-width: 0.4;
	}

	.hole.trouble {
		fill: var(--failed);
	}

	.jumper {
		fill: none;
		stroke: var(--wire);
		stroke-width: 1.6;
		stroke-linecap: round;
	}

	.part {
		cursor: grab;
	}

	.part.fixed {
		cursor: pointer;
	}

	.part.dragging {
		cursor: grabbing;
		opacity: 0.8;
	}

	.lead {
		stroke: var(--muted);
		stroke-width: 0.9;
	}

	.leg-end,
	.leg {
		fill: var(--fg);
	}

	.tile {
		fill: var(--bg);
		stroke: var(--tint);
		stroke-width: 1.2;
	}

	.glyph {
		fill: none;
		stroke: var(--fg);
		stroke-width: 2;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.glyph.filled {
		fill: var(--fg);
	}

	.label,
	.chip-label {
		fill: var(--fg);
		font: 5px var(--font-text);
		text-anchor: middle;
	}

	/* Black in either theme, as a DIP is. */
	.chip {
		fill: #1d1d22;
		stroke: var(--edge);
		stroke-width: 0.6;
	}

	.chip-label {
		fill: #e8e8ec;
	}

	.notch {
		fill: var(--raised);
	}

	.pin-one {
		fill: #e8e8ec;
	}

	.part.selected .tile,
	.part.selected .chip {
		stroke: var(--selection);
		stroke-width: 2;
	}

	.problems {
		margin: 0;
		padding: 0.5rem 0.75rem 0.5rem 1.75rem;
		border-left: 3px solid var(--failed);
		background: var(--raised);
		font-size: 0.875rem;
	}
</style>
