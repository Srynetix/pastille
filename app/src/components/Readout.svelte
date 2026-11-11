<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

<script lang="ts">
	import type { ComponentInfo } from '../lib/palette'
	import type { Reading } from '../lib/readings'
	import { formatQuantity } from '../lib/units'

	// On-demand help (design doc §14): nothing appears until the pointer asks,
	// and what appears answers the two questions a beginner has about the thing
	// under their hand — what is it, and what is it doing.
	//
	// It overlaps C-21, which places permanent probes on the canvas. This is the
	// cheap version that is always available and never has to be tidied away.
	let {
		info,
		reading,
		at,
	}: {
		/** What the app can say about the kind. */
		info: ComponentInfo
		/** What it is doing, which may be nothing the app can state. */
		reading: Reading
		/** Where the pointer is, in client coordinates. */
		at: { x: number; y: number }
	} = $props()

	const OFFSET = 14
	const WIDTH = 15 * 16

	// Beside the pointer, and on the other side of it near the right edge: a
	// readout that runs off the screen answers nothing.
	const flipped = $derived(at.x + OFFSET + WIDTH > window.innerWidth)
	const style = $derived(
		`left: ${at.x + (flipped ? -OFFSET : OFFSET)}px; top: ${at.y}px;` +
			(flipped ? ' transform: translate(-100%, -50%);' : ''),
	)

	const measurements = $derived(
		[
			reading.volts !== undefined ? `${formatQuantity(reading.volts, 'V')} across` : undefined,
			reading.amps !== undefined ? `${formatQuantity(reading.amps, 'A')} through` : undefined,
		].filter((measurement) => measurement !== undefined),
	)
</script>

<!-- Never intercepts the pointer whose position it is reporting. -->
<div class="readout" role="status" aria-live="off" {style}>
	<p class="label">{info.label}</p>
	<p class="hint">{info.hint}</p>
	{#if measurements.length > 0}
		<p class="measurements">{measurements.join(' · ')}</p>
	{/if}
</div>

<style>
	.readout {
		position: fixed;
		transform: translate(0, -50%);
		pointer-events: none;
		z-index: 1;
		width: max-content;
		max-width: 15rem;
		padding: 0.375rem 0.5rem;
		border: 1px solid var(--edge);
		border-radius: 6px;
		background: var(--raised);
		box-shadow: 0 2px 8px rgb(0 0 0 / 0.25);
	}

	p {
		margin: 0;
	}

	.label {
		font-size: 0.8125rem;
		font-weight: 500;
	}

	.hint {
		font-size: 0.75rem;
		color: var(--muted);
	}

	.measurements {
		margin-top: 0.25rem;
		font-size: 0.8125rem;
		font-variant-numeric: tabular-nums;
		color: var(--ok);
	}
</style>
