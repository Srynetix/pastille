<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

<script lang="ts">
	// B-07. A finding, said beside the part it is about: neither an
	// interruption nor a line in the status bar as far from the part as the
	// screen allows (design doc §12, decided after M-22). It points at the
	// part, says what is wrong, and steps through the others when there are
	// several. On a phone it is a banner under the header instead, where there
	// is no room beside anything (App.svelte's CSS).
	let {
		severity,
		message,
		at,
		side = 'right',
		index = 0,
		count = 1,
		onprevious,
		onnext,
		ondismiss,
		fix,
	}: {
		severity: 'warning' | 'note'
		message: string
		/** Where on the board it points from, in the board's pixels; none, pinned to the top. */
		at?: { x: number; y: number }
		/** Which side of the point it sits on, so it stays on the board. */
		side?: 'left' | 'right'
		index?: number
		count?: number
		onprevious?: () => void
		onnext?: () => void
		ondismiss?: () => void
		/**
		 * The one edit that would make it go, as a button beside it (C-135):
		 * what the button says, and what pressing it does. One step of undo.
		 */
		fix?: { label: string; apply: () => void }
	} = $props()
</script>

<div
	class="callout {severity} {side}"
	class:pinned={!at}
	role="status"
	aria-live="polite"
	style:left={at ? `${at.x}px` : undefined}
	style:top={at ? `${at.y}px` : undefined}
>
	<p class="head">
		<svg viewBox="0 0 24 24" aria-hidden="true">
			{#if severity === 'warning'}
				<path
					d="M10.3 3.9L1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0zM12 9v4M12 17h.01"
				/>
			{:else}
				<path d="M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20zM12 16v-4M12 8h.01" />
			{/if}
		</svg>
		<span class="what">{severity === 'warning' ? 'Check this' : 'Worth knowing'}</span>
		{#if count > 1}
			<span class="steps">
				<button type="button" aria-label="Previous check" onclick={onprevious}>‹</button>
				<span class="of">{index + 1} of {count}</span>
				<button type="button" aria-label="Next check" onclick={onnext}>›</button>
			</span>
		{/if}
		<button type="button" class="dismiss" aria-label="Put this away" onclick={ondismiss}>×</button>
	</p>
	<p class="message">{message}</p>
	{#if fix}
		<p class="actions">
			<button type="button" class="fix" onclick={fix.apply}>{fix.label}</button>
			<span class="undo">Undo takes it back.</span>
		</p>
	{/if}
	<span class="arrow" aria-hidden="true"></span>
</div>

<style>
	.callout {
		--tone: var(--warning);
		position: absolute;
		z-index: 3;
		width: 17rem;
		padding: 0.625rem 0.75rem 0.75rem;
		background: color-mix(in srgb, var(--tone) 12%, var(--bg));
		border: 1px solid color-mix(in srgb, var(--tone) 55%, transparent);
		border-radius: 12px;
		box-shadow: 0 8px 24px rgb(0 0 0 / 0.35);
		transform: translateY(-50%);
	}

	.callout.note {
		--tone: var(--family-passive);
	}

	/* On the left of the part: its right edge at the point. */
	.callout.left {
		transform: translate(-100%, -50%);
	}

	.callout.pinned {
		left: 50%;
		top: 0.75rem;
		transform: translateX(-50%);
	}

	p {
		margin: 0;
	}

	.head {
		display: flex;
		align-items: center;
		gap: 0.375rem;
		margin-bottom: 0.375rem;
		color: var(--tone);
		font-family: var(--font-display);
		font-variation-settings: var(--display-axes);
		font-size: 0.8125rem;
		font-weight: 750;
	}

	svg {
		width: 1rem;
		height: 1rem;
		flex: none;
		fill: none;
		stroke: currentColor;
		stroke-width: 1.8;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.what {
		flex: 1;
	}

	.steps {
		display: inline-flex;
		align-items: center;
		gap: 0.125rem;
		font-family: var(--font-mono);
		font-variation-settings: var(--mono-axes);
		font-size: 0.6875rem;
		font-weight: 500;
		color: var(--muted);
	}

	button {
		width: 1.5rem;
		height: 1.5rem;
		padding: 0;
		background: none;
		border: none;
		border-radius: 6px;
		color: var(--muted);
		font: inherit;
		font-size: 1rem;
		line-height: 1;
		cursor: pointer;
	}

	button:focus-visible {
		color: var(--fg);
		background: var(--raised);
	}

	@media (hover: hover) {
		button:hover {
			color: var(--fg);
			background: var(--raised);
		}
	}

	.actions {
		display: flex;
		align-items: center;
		gap: 0.625rem;
		margin-top: 0.625rem;
	}

	.fix {
		width: auto;
		flex: none;
		white-space: nowrap;
		height: 2rem;
		padding: 0 0.75rem;
		background: var(--brand);
		color: var(--on-brand);
		border-radius: 8px;
		font-family: var(--font-display);
		font-variation-settings: var(--display-axes);
		font-size: 0.8125rem;
		font-weight: 750;
	}

	.fix:focus-visible {
		background: var(--brand);
		color: var(--on-brand);
		filter: brightness(1.08);
	}

	@media (hover: hover) {
		.fix:hover {
			background: var(--brand);
			color: var(--on-brand);
			filter: brightness(1.08);
		}
	}

	.undo {
		font-size: 0.75rem;
		color: var(--muted);
	}

	.message {
		font-size: 0.8125rem;
		line-height: 1.45;
		color: var(--fg);
	}

	/* The point it is about: a corner of the card turned towards the part. */
	.arrow {
		position: absolute;
		top: 50%;
		left: -6px;
		width: 10px;
		height: 10px;
		background: inherit;
		border-left: 1px solid color-mix(in srgb, var(--tone) 55%, transparent);
		border-bottom: 1px solid color-mix(in srgb, var(--tone) 55%, transparent);
		transform: translateY(-50%) rotate(45deg);
	}

	.callout.left .arrow {
		left: auto;
		right: -6px;
		transform: translateY(-50%) rotate(-135deg);
	}

	.callout.pinned .arrow {
		display: none;
	}

	/* On a phone there is no room beside anything: a banner across the top of
	   the board, under the header (B-07's decision). */
	@media (width < 60rem) {
		.callout,
		.callout.left,
		.callout.pinned {
			left: 0.75rem !important;
			right: 0.75rem;
			top: 0.75rem !important;
			width: auto;
			transform: none;
		}

		.arrow {
			display: none;
		}

		button {
			width: 2.75rem;
			height: 2.75rem;
		}

		.fix {
			width: auto;
		}
	}
</style>
