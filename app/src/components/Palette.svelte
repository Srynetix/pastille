<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

<script lang="ts">
	import { tick } from 'svelte'
	import { glyphOf } from '../lib/glyph'
	import {
		answersTo,
		PALETTE_CATEGORIES,
		STARTERS,
		type PaletteCategory,
		type PaletteEntry,
	} from '../lib/palette'
	import { symbolFor } from '../lib/symbols'

	// The palette reports intent; the shell decides what to do with it. It knows
	// nothing about the canvas, the circuit or where a component ends up —
	// except whether the board is empty, which is when it offers the first
	// three parts in the order a first circuit is built (C-128).
	let {
		onpick,
		ondragstart,
		onclose,
		start = false,
		replacing,
		onreplace,
	}: {
		onpick?: (entry: PaletteEntry) => void
		/**
		 * The part selected on the board, when exactly one is (C-137): each
		 * entry of another kind then offers to take its place.
		 */
		replacing?: { kind: string; label: string }
		/** An entry was chosen to take the selected part's place, keeping what wires it can. */
		onreplace?: (entry: PaletteEntry) => void
		ondragstart?: (entry: PaletteEntry, at: { x: number; y: number }) => void
		/** Whether to offer "Start here": the board has nothing on it yet. */
		start?: boolean
		/** Put the phone's parts sheet away (C-132). */
		onclose?: () => void
	} = $props()

	// Categories that have been folded away (C-59). All open to begin with:
	// the empty canvas tells a beginner to reach for a battery, a resistor and
	// an LED, and none of the three may be behind a click they do not know to
	// make. Folding is for somebody who knows what they are looking for.
	let folded = $state(new Set<string>())

	/** What has been typed into the search, which narrows the list as it goes (C-128). */
	let query = $state('')
	const searching = $derived(query.trim().length > 0)
	const shown = $derived(
		PALETTE_CATEGORIES.map((category) => ({
			category,
			entries: searching
				? category.entries.filter((entry) => answersTo(entry, query))
				: category.entries,
		})).filter(({ entries }) => entries.length > 0),
	)

	/** Each part's symbol, drawn from the shapes the board draws (C-128). */
	const glyphs = new Map(
		[...PALETTE_CATEGORIES.flatMap((category) => category.entries), ...STARTERS].map((entry) => [
			entry.kind,
			glyphOf(symbolFor(entry.kind)),
		]),
	)
	/** Which family each part belongs to, for its colour. */
	const familyOf = new Map(
		PALETTE_CATEGORIES.flatMap((category) =>
			category.entries.map((entry) => [entry.kind, category.family]),
		),
	)

	let list = $state<HTMLElement | undefined>()

	function toggle(name: string) {
		// A copy, handed back whole: `folded` is replaced, never mutated.
		// eslint-disable-next-line svelte/prefer-svelte-reactivity
		const next = new Set(folded)
		if (next.has(name)) next.delete(name)
		else next.add(name)
		folded = next
	}

	/** A family's chip: its heading, open, at the top of the list. */
	async function jumpTo(category: PaletteCategory) {
		query = ''
		if (folded.has(category.name)) toggle(category.name)
		await tick()
		list
			?.querySelector(`[data-family="${category.family}"]`)
			?.scrollIntoView({ block: 'start', behavior: 'smooth' })
	}
</script>

{#snippet glyph(entry: PaletteEntry)}
	{@const drawn = glyphs.get(entry.kind)}
	<span class="glyph" style:--tint="var(--family-{familyOf.get(entry.kind)})" aria-hidden="true">
		{#if drawn}
			<svg viewBox={drawn.viewBox} preserveAspectRatio="xMidYMid meet">
				{#each drawn.shapes as shape, index (index)}
					<path d={shape.d} class:filled={shape.filled} />
				{/each}
				{#each drawn.pins as pin, index (index)}
					<circle class="pin" cx={pin.x} cy={pin.y} r="1.6" />
				{/each}
			</svg>
		{/if}
	</span>
{/snippet}

{#snippet part(entry: PaletteEntry, kind: 'entry' | 'starter', step?: number)}
	<button
		type="button"
		class={kind}
		class:beside={replacing && onreplace && entry.kind !== replacing.kind}
		style:--tint="var(--family-{familyOf.get(entry.kind)})"
		onclick={() => onpick?.(entry)}
		onpointerdown={(event) => ondragstart?.(entry, { x: event.clientX, y: event.clientY })}
	>
		{@render glyph(entry)}
		{#if step}<span class="step" aria-hidden="true">{step}</span>{/if}
		<span class="words">
			<span class="label">{entry.label}</span>
			<span class="hint">{entry.hint}</span>
		</span>
	</button>
	{#if replacing && onreplace && entry.kind !== replacing.kind}
		<!-- C-137. Beside the entry, not in it: a button cannot hold one. -->
		<button
			type="button"
			class="replace"
			aria-label="Replace the {replacing.label.toLowerCase()} with this {entry.label.toLowerCase()}"
			title="Put this in place of the selected {replacing.label.toLowerCase()}, keeping the wires that still make sense"
			onclick={() => onreplace?.(entry)}
		>
			<!-- Two arrows, one each way: this one in, that one out. -->
			<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 8h14l-4-4M20 16H6l4 4" /></svg>
		</button>
	{/if}
{/snippet}

<aside class="palette" aria-label="Components">
	<h2>Components</h2>
	<label class="search">
		<svg viewBox="0 0 24 24" aria-hidden="true"
			><path d="M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16zM21 21l-4.3-4.3" /></svg
		>
		<input
			type="search"
			aria-label="Find a part"
			placeholder="Find a part — LED, 555, relay…"
			bind:value={query}
		/>
	</label>
	<nav class="families" aria-label="Families">
		{#each PALETTE_CATEGORIES as category (category.name)}
			<button
				type="button"
				class="chip"
				style:--tint="var(--family-{category.family})"
				title={category.name}
				onclick={() => jumpTo(category)}
			>
				<span class="dot" aria-hidden="true"></span>{category.short}
			</button>
		{/each}
	</nav>

	<div class="list" bind:this={list}>
		{#if start && !searching}
			<!-- The first circuit, in the order it is built. Not a second copy
			     of the list: these three are also in their families below. -->
			<section class="starters" aria-label="Start here">
				<h3 class="starters-heading">Start here</h3>
				<ul>
					{#each STARTERS as entry, index (entry.kind)}
						<li>{@render part(entry, 'starter', index + 1)}</li>
					{/each}
				</ul>
			</section>
		{/if}

		{#each shown as { category, entries } (category.name)}
			{@const open = searching || !folded.has(category.name)}
			<section
				class="category"
				data-family={category.family}
				style:--tint="var(--family-{category.family})"
			>
				<h3>
					<button
						type="button"
						class="heading"
						aria-expanded={open}
						onclick={() => toggle(category.name)}
					>
						<span class="chevron" aria-hidden="true">{open ? '▾' : '▸'}</span>
						{category.name}
					</button>
				</h3>
				{#if open}
					<ul>
						{#each entries as entry (entry.kind)}
							<li>{@render part(entry, 'entry')}</li>
						{/each}
					</ul>
				{/if}
			</section>
		{:else}
			<p class="none">
				Nothing called that. Try a word for what it does — “light”, “sound”, “switch”.
			</p>
		{/each}
	</div>
	<!-- Last, so it is not the panel's first control; drawn in its corner. -->
	{#if onclose}
		<button type="button" class="sheet-close" aria-label="Close" onclick={() => onclose()}>
			<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M18 6L6 18M6 6l12 12" /></svg>
		</button>
	{/if}
</aside>

<style>
	.palette {
		grid-area: palette;
		border-right: 1px solid var(--edge);
		padding: 1rem 0.75rem 0;
		display: flex;
		flex-direction: column;
		gap: 0.625rem;
		min-height: 0;
	}

	h2 {
		font-size: 0.75rem;
		text-transform: uppercase;
		letter-spacing: 0.08em;
		color: var(--muted);
		margin: 0 0.25rem;
	}

	.search {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		height: 2.25rem;
		padding: 0 0.625rem;
		background: var(--sunken);
		border: 1px solid var(--edge);
		border-radius: 9px;
		color: var(--muted);
	}

	.search:focus-within {
		border-color: var(--selection);
	}

	.search svg {
		width: 1rem;
		height: 1rem;
		flex: none;
		fill: none;
		stroke: currentColor;
		stroke-width: 1.8;
		stroke-linecap: round;
	}

	/* The text face, not the mono one app.css gives inputs: what is typed
	   here is a word, not a value. */
	.search input {
		flex: 1;
		min-width: 0;
		background: none;
		border: none;
		outline: none;
		color: var(--fg);
		font-family: var(--font-text);
		font-variation-settings: normal;
		font-size: 0.8125rem;
	}

	.families {
		display: flex;
		flex-wrap: wrap;
		gap: 0.25rem;
	}

	.chip {
		display: inline-flex;
		align-items: center;
		gap: 0.3125rem;
		height: 1.5rem;
		padding: 0 0.5rem;
		background: none;
		border: 1px solid var(--edge);
		border-radius: 0.75rem;
		color: var(--muted);
		font: inherit;
		font-size: 0.6875rem;
		cursor: pointer;
	}

	.chip:focus-visible {
		color: var(--fg);
		border-color: var(--tint);
	}

	@media (hover: hover) {
		.chip:hover {
			color: var(--fg);
			border-color: var(--tint);
		}
	}

	.dot {
		width: 0.4375rem;
		height: 0.4375rem;
		border-radius: 50%;
		background: var(--tint);
	}

	.list {
		flex: 1;
		min-height: 0;
		overflow-y: auto;
		padding-bottom: 1rem;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	h3 {
		margin: 0 0 0.125rem;
	}

	.heading {
		display: flex;
		align-items: center;
		gap: 0.375rem;
		width: 100%;
		padding: 0.25rem 0.375rem;
		background: none;
		border: none;
		font: inherit;
		font-family: var(--font-display);
		font-variation-settings: var(--display-axes);
		font-size: 0.6875rem;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--tint);
		cursor: pointer;
	}

	.heading:focus-visible {
		filter: brightness(1.2);
	}

	@media (hover: hover) {
		.heading:hover {
			filter: brightness(1.2);
		}
	}

	.chevron {
		width: 0.75em;
		display: inline-block;
		color: var(--muted);
	}

	.starters-heading {
		padding: 0.25rem 0.375rem;
		font-size: 0.6875rem;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--fg);
	}

	ul {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 0.125rem;
	}

	.entry,
	.starter {
		position: relative;
		width: 100%;
		display: flex;
		align-items: flex-start;
		gap: 0.625rem;
		padding: 0.4375rem 0.375rem;
		text-align: left;
		background: none;
		border: 1px solid transparent;
		border-radius: 8px;
		color: inherit;
		font: inherit;
		cursor: pointer;
	}

	li {
		position: relative;
	}

	/* On the entry's right edge, over the end of its hint, while a part is
	   selected (C-137). Small and quiet: placing a part is still what the
	   list is for. */
	.replace {
		position: absolute;
		right: 0.375rem;
		top: 50%;
		translate: 0 -50%;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 1.5rem;
		height: 1.5rem;
		padding: 0;
		background: var(--raised);
		border: 1px solid var(--edge);
		border-radius: 999px;
		color: var(--muted);
		cursor: pointer;
	}

	.replace svg {
		width: 0.875rem;
		height: 0.875rem;
		fill: none;
		stroke: currentColor;
		stroke-width: 2;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	/* Room for it, so the hint wraps short of it rather than under it. */
	.beside {
		padding-right: 2.25rem;
	}

	.replace:focus-visible {
		color: var(--fg);
		border-color: var(--tint, var(--muted));
	}

	@media (hover: hover) {
		.replace:hover {
			color: var(--fg);
			border-color: var(--tint, var(--muted));
		}
	}

	.entry:focus-visible,
	.starter:focus-visible {
		background: var(--raised);
		border-color: var(--edge);
	}

	@media (hover: hover) {
		.entry:hover,
		.starter:hover {
			background: var(--raised);
			border-color: var(--edge);
		}
	}

	.starter {
		border-color: color-mix(in srgb, var(--tint, var(--brand)) 35%, transparent);
	}

	/* The symbol, in its family's colour on a wash of it. */
	.glyph {
		width: 2.75rem;
		height: 1.75rem;
		flex: none;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 0.1875rem;
		border-radius: 6px;
		background: color-mix(in srgb, var(--tint) 10%, transparent);
		color: var(--tint);
	}

	.glyph svg {
		width: 100%;
		height: 100%;
		overflow: visible;
	}

	.glyph path {
		fill: none;
		stroke: currentColor;
		stroke-width: 1.6;
		stroke-linecap: round;
		stroke-linejoin: round;
		vector-effect: non-scaling-stroke;
	}

	.glyph path.filled {
		fill: currentColor;
	}

	.glyph .pin {
		fill: currentColor;
	}

	.step {
		position: absolute;
		left: 0;
		top: 0;
		width: 1.125rem;
		height: 1.125rem;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 50%;
		background: var(--brand);
		color: var(--on-brand);
		font-size: 0.6875rem;
		font-weight: 700;
	}

	.words {
		display: flex;
		flex-direction: column;
		gap: 0.0625rem;
		min-width: 0;
	}

	.label {
		font-weight: 500;
		font-size: 0.875rem;
	}

	.hint {
		font-size: 0.75rem;
		color: var(--muted);
	}

	.none {
		margin: 0.5rem 0.375rem;
		font-size: 0.8125rem;
		color: var(--muted);
	}

	/* A phone's sheet closes from here (C-132); beside columns there is
	   nothing to close. */
	.sheet-close {
		display: none;
	}

	@media (width < 60rem) {
		.sheet-close {
			position: absolute;
			right: 0.75rem;
			top: 0.625rem;
			width: 2.75rem;
			height: 2.75rem;
			display: flex;
			align-items: center;
			justify-content: center;
			padding: 0;
			background: var(--raised);
			border: none;
			border-radius: 50%;
			color: var(--muted);
			cursor: pointer;
			z-index: 1;
		}

		.sheet-close svg {
			width: 1.125rem;
			height: 1.125rem;
			fill: none;
			stroke: currentColor;
			stroke-width: 2;
			stroke-linecap: round;
		}
	}
</style>
