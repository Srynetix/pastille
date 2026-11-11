<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

<script lang="ts">
	// What the circuit on the canvas is, in its author's words: the line an
	// example opens with, or whatever somebody wrote about their own. On the
	// canvas, in its corner, because that is where the circuit it describes
	// is — an example's "what to look at" used to vanish with the offer, at
	// the moment it was needed.
	//
	// Folded away rather than closed, so it is one click to read again, and
	// unfolded again whenever the text changes: a new circuit is a new thing
	// to read.
	let { text }: { text: string } = $props()

	let folded = $state(false)
	let tab = $state<HTMLButtonElement>()
	let fold = $state<HTMLButtonElement>()
	/** Where the focus goes once the other button exists, after a press. */
	let focusing = $state<'tab' | 'fold' | undefined>()

	// The button pressed is replaced by the other, so the focus follows it
	// across rather than falling to the page (C-103) — once it is there,
	// which is after the block has swapped and bound it.
	function toggle(to: boolean) {
		folded = to
		focusing = to ? 'tab' : 'fold'
	}

	$effect(() => {
		const target = focusing === 'tab' ? tab : focusing === 'fold' ? fold : undefined
		if (!target) return
		target.focus()
		focusing = undefined
	})
	let shown = ''
	$effect(() => {
		if (text !== shown) {
			shown = text
			folded = false
		}
	})
</script>

{#if folded}
	<button
		type="button"
		class="tab"
		bind:this={tab}
		onclick={() => toggle(false)}
		aria-expanded="false"
	>
		About this circuit
	</button>
{:else}
	<div class="card" role="note" aria-label="About this circuit">
		<p>{text}</p>
		<button
			type="button"
			bind:this={fold}
			onclick={() => toggle(true)}
			aria-expanded="true"
			aria-label="Fold away">−</button
		>
	</div>
{/if}

<style>
	.card,
	.tab {
		position: absolute;
		left: 0.75rem;
		top: 0.75rem;
		border: 1px solid var(--edge);
		border-radius: 8px;
		background: var(--raised);
		box-shadow: 0 2px 12px rgb(0 0 0 / 0.3);
	}

	.card {
		max-width: min(24rem, calc(100% - 1.5rem));
		/* Capped, so a long description cannot cover the parts on a phone's
		   short canvas (C-107); it scrolls past that, and folds away. */
		max-height: min(40%, 8rem);
		overflow: auto;
		display: flex;
		align-items: flex-start;
		gap: 0.5rem;
		padding: 0.5rem 0.5rem 0.5rem 0.75rem;
	}

	p {
		margin: 0;
		font-size: 0.8125rem;
		line-height: 1.45;
		white-space: pre-wrap;
		overflow-wrap: anywhere;
	}

	/* The first line reads as its title when there is more than one. */
	p::first-line {
		font-weight: 600;
	}

	button {
		flex: none;
		background: none;
		border: 1px solid var(--edge);
		border-radius: 5px;
		padding: 0 0.4rem;
		color: var(--muted);
		font: inherit;
		font-size: 0.75rem;
		line-height: 1.4rem;
		cursor: pointer;
	}

	.tab {
		padding: 0.25rem 0.625rem;
		color: var(--muted);
		font-size: 0.75rem;
	}

	button:focus-visible {
		color: var(--fg);
		border-color: var(--muted);
	}

	@media (hover: hover) {
		button:hover {
			color: var(--fg);
			border-color: var(--muted);
		}
	}
</style>
