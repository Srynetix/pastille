<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

<script lang="ts">
	import { tick } from 'svelte'

	import type { Circuit, ElementSerial } from '../lib/core'
	import { BAUDS, LINE_ENDINGS, baudsMatch, type SerialLog } from '../lib/serial.svelte'

	// A-06. What a program prints with `Serial`, line by line, under the
	// board, as the Arduino IDE's serial monitor shows it — at the baud the
	// monitor is set to, so a mismatch reads as the garbage it is. A line
	// typed here is sent to the program at that baud too.
	let {
		circuit,
		ports = [],
		log,
	}: {
		circuit?: Circuit
		/** Each running program's serial port, from the solution. */
		ports?: readonly ElementSerial[]
		log: SerialLog
	} = $props()

	/** The Uno shown, when there is more than one. */
	let chosen = $state<string | undefined>()
	const port = $derived(ports.find((entry) => entry.elementId === chosen) ?? ports[0])
	const text = $derived(port ? (log.text[port.elementId] ?? '') : '')

	let folded = $state(false)
	let typed = $state('')
	let ending = $state<string>(LINE_ENDINGS[0].ending)
	let output = $state<HTMLPreElement | undefined>()
	/** Follows what arrives, unless the reader has scrolled up to look. */
	let following = $state(true)

	$effect(() => {
		void text
		if (!following || !output) return
		const pre = output
		void tick().then(() => (pre.scrollTop = pre.scrollHeight))
	})

	function scrolled() {
		if (!output) return
		following = output.scrollHeight - output.scrollTop - output.clientHeight < 8
	}

	/** What the panel says about the two ends of the line. */
	const status = $derived.by(() => {
		if (!port) return ''
		if (port.programBaud == null) return 'The program has not begun Serial.'
		if (baudsMatch(port.programBaud, port.monitorBaud))
			return `The program sends at ${port.programBaud} baud.`
		return `The program sends at ${port.programBaud} baud, the monitor listens at ${port.monitorBaud}: garbage. Set the monitor to match.`
	})
	const mismatched = $derived(
		port?.programBaud != null && !baudsMatch(port.programBaud, port.monitorBaud),
	)

	function send(event: SubmitEvent) {
		event.preventDefault()
		if (!port || !circuit) return
		circuit.sendSerial(port.elementId, typed + ending)
		typed = ''
	}
</script>

{#if port}
	<aside class="monitor" class:folded aria-label="Serial monitor">
		<header>
			<svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
				<path d="M4 6h16v12H4zM7 10l3 2-3 2M12 15h5" />
			</svg>
			<h2>Serial monitor</h2>
			{#if ports.length > 1}
				<select
					aria-label="Which Uno"
					value={port.elementId}
					onchange={(event) => (chosen = event.currentTarget.value)}
				>
					{#each ports as entry, index (entry.elementId)}
						<option value={entry.elementId}>Uno {index + 1}</option>
					{/each}
				</select>
			{/if}
			<span class="spacer"></span>
			{#if !folded}
				<select
					aria-label="Baud"
					value={String(port.monitorBaud)}
					onchange={(event) =>
						circuit?.setMonitorBaud(port.elementId, Number(event.currentTarget.value))}
				>
					{#each BAUDS as baud (baud)}
						<option value={String(baud)}>{baud} baud</option>
					{/each}
				</select>
				<button type="button" onclick={() => log.clear(port.elementId)}>Clear</button>
			{/if}
			<button
				type="button"
				class="fold"
				aria-label={folded ? 'Show the serial monitor' : 'Fold the serial monitor'}
				aria-expanded={!folded}
				onclick={() => (folded = !folded)}
			>
				<svg viewBox="0 0 24 24" aria-hidden="true"
					><path d={folded ? 'M6 15l6-6 6 6' : 'M6 9l6 6 6-6'} /></svg
				>
			</button>
		</header>
		<p class="status" class:warn={mismatched} role="status">{status}</p>

		{#if !folded}
			<!-- Focusable, because it scrolls: without a stop in the tab order, a
			     keyboard has no way to read back further than fits. A labelled
			     region in the tab order is what WCAG asks of a scrolling box;
			     Svelte's rule against a tabindex on one predates that. -->
			<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
			<pre
				bind:this={output}
				onscroll={scrolled}
				tabindex="0"
				role="region"
				aria-label="What the program printed"
				aria-live="off">{text}</pre>
			<form onsubmit={send}>
				<input
					type="text"
					aria-label="A line to send"
					placeholder="Type a line to send to the program"
					bind:value={typed}
				/>
				<select aria-label="Line ending" bind:value={ending}>
					{#each LINE_ENDINGS as choice (choice.label)}
						<option value={choice.ending}>{choice.label}</option>
					{/each}
				</select>
				<button type="submit">Send</button>
			</form>
		{/if}
	</aside>
{/if}

<style>
	.monitor {
		grid-area: monitor;
		border-top: 1px solid var(--edge);
		padding: 0.5rem 1rem 0.75rem;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		min-height: 0;
	}

	.monitor.folded {
		padding-bottom: 0.5rem;
	}

	header {
		display: flex;
		align-items: center;
		gap: 0.625rem;
		flex-wrap: wrap;
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

	.spacer {
		flex: 1;
	}

	.status {
		margin: -0.25rem 0 0;
		font-size: 0.75rem;
		color: var(--muted);
	}

	.status.warn {
		color: var(--warning, var(--text));
		font-weight: 600;
	}

	.fold {
		width: 1.75rem;
		height: 1.75rem;
		padding: 0;
		display: grid;
		place-items: center;
	}

	.fold svg {
		width: 1rem;
		height: 1rem;
	}

	pre {
		margin: 0;
		height: 6rem;
		overflow: auto;
		padding: 0.5rem 0.625rem;
		background: var(--sunken);
		border: 1px solid var(--edge);
		border-radius: 8px;
		font-family: ui-monospace, 'SF Mono', Menlo, Consolas, monospace;
		font-size: 0.8125rem;
		line-height: 1.4;
		white-space: pre-wrap;
		overflow-wrap: anywhere;
	}

	form {
		display: flex;
		gap: 0.5rem;
	}

	input {
		flex: 1;
		min-width: 0;
	}

	/* Every control a 44 px target on a phone (C-138), and the baud and Clear
	   on a row of their own under the title and the fold, which they pushed
	   onto a line by itself. */
	@media (width < 60rem) {
		header select,
		header button,
		form > * {
			min-height: 2.75rem;
		}

		.fold {
			width: 2.75rem;
		}

		header select[aria-label='Baud'] {
			order: 1;
			/* Wider than what the first row has left, so it starts the second. */
			flex: 1 0 12rem;
		}

		header button:not(.fold) {
			order: 1;
		}
	}
</style>
