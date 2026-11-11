<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

<script lang="ts">
	import { ACCURACY } from '../lib/accuracy'

	// The engine's state, rendered. The component asks for nothing and computes
	// nothing — whoever owns the simulation passes it down.
	let {
		status,
		detail,
		title,
		position,
		findings = [],
		spoken,
	}: {
		status: 'loading' | 'ready' | 'failed'
		detail: string
		/**
		 * What a screen reader is told, when it is not `detail` word for word.
		 *
		 * The line is for the eye and changes ten times a second while a
		 * circuit runs ("1.2 s simulated"); read aloud as it changed, it was a
		 * clock nobody could stop and that drowned whatever mattered. So what
		 * is announced is the line without what ticks: that it is running, or
		 * why it is not.
		 */
		spoken?: string
		/**
		 * The solver's own words behind `detail`, when there are some.
		 *
		 * A tooltip rather than the line itself: "matrix is singular at column
		 * 1" is the right sentence for exactly one reader, and they are not
		 * the audience of §4 (C-25).
		 */
		title?: string
		/** Grid cell under the cursor, when the pointer is over the canvas. */
		position?: { x: number; y: number }
		/** What the app noticed about the circuit, most important first. */
		findings?: { severity: string; message: string }[]
	} = $props()

	const shown = $derived(findings.find((finding) => finding.severity === 'warning') ?? findings[0])
</script>

<footer class="status">
	<!-- The only part of the line that is announced, and only when it says
	     something new. The rest — the clock, the pointer's position — would
	     be read out on every frame and every mouse move. -->
	<span class="spoken" role="status">{shown?.message ?? spoken ?? detail}</span>
	<!-- A finding takes the line over: it is the most useful thing the app can
	     say, and it says it quietly rather than in a dialog — B-07 is where that
	     choice is made properly, and until then quiet is the reversible one. -->
	{#if shown}
		<!-- A warning before a note: something that will not work matters
		     more than something worth knowing (C-102), and a note is drawn in
		     its own colour so it does not read as a mistake. -->
		<span class="dot {shown.severity === 'note' ? 'note' : 'warning'}" aria-hidden="true"></span>
		<span class="finding">{shown.message}</span>
	{:else}
		<span class="dot {status}" aria-hidden="true"></span>
		<span {title}>{detail}</span>
	{/if}
	{#if position}
		<span class="position">{position.x}, {position.y}</span>
	{/if}
	<!-- Said here and in the saved file, and nowhere else: `lib/accuracy.ts`
	     has the decision and the reasons (C-26). Outside the conditional above,
	     so a finding takes over the line without taking this with it. -->
	<span class="disclaimer">{ACCURACY}</span>
</footer>

<style>
	.spoken {
		position: absolute;
		width: 1px;
		height: 1px;
		overflow: hidden;
		clip-path: inset(50%);
		white-space: nowrap;
	}

	.status {
		grid-area: status;
		border-top: 1px solid var(--edge);
		padding: 0.375rem 1rem;
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.75rem;
		color: var(--muted);
	}

	.dot {
		width: 0.5rem;
		height: 0.5rem;
		border-radius: 50%;
		background: var(--muted);
		flex: none;
	}

	.dot.ready {
		background: var(--ok);
	}
	.dot.failed {
		background: var(--failed);
	}

	.dot.warning {
		background: var(--warning);
	}

	.dot.note {
		background: var(--accent);
	}

	.finding {
		color: var(--fg);
	}

	.position {
		font-variant-numeric: tabular-nums;
		padding-left: 0.5rem;
		border-left: 1px solid var(--edge);
	}

	.disclaimer {
		margin-left: auto;
		text-align: right;
	}
</style>
