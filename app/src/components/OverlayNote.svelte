<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

<script lang="ts">
	// C-39. The dots travelling along the wires are the app's main channel for
	// "current is flowing, this way round", and they arrive unannounced: the
	// first time a circuit works, something starts moving and nothing says what
	// it is. This is the sentence that says it.
	//
	// Said once, when there is finally a current to point at — explaining an
	// animation that is not running would be worse than saying nothing. Whoever
	// mounts this decides when; the note itself only knows how to be read and
	// how to get out of the way.
	let { ondismiss }: { ondismiss?: () => void } = $props()
</script>

<div class="note" role="note">
	<p>The moving dots are the current, flowing from + all the way round to −.</p>
	<button type="button" onclick={() => ondismiss?.()}>Got it</button>
</div>

<style>
	.note {
		position: absolute;
		left: 50%;
		bottom: 1rem;
		transform: translateX(-50%);
		max-width: min(28rem, calc(100% - 2rem));
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.5rem 0.5rem 0.5rem 0.75rem;
		border: 1px solid var(--edge);
		border-radius: 8px;
		background: var(--raised);
		box-shadow: 0 2px 12px rgb(0 0 0 / 0.3);
	}

	p {
		margin: 0;
		font-size: 0.8125rem;
	}

	button {
		flex: none;
		background: none;
		border: 1px solid var(--edge);
		border-radius: 5px;
		padding: 0.25rem 0.5rem;
		color: var(--muted);
		font: inherit;
		font-size: 0.75rem;
		cursor: pointer;
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

	/* On a phone, across the board above its thumb bar and the row over it,
	   where an opened example's notice goes: at the foot it lay under the
	   thumb bar (C-138). */
	@media (width < 60rem) {
		.note {
			left: 0.75rem;
			right: 0.75rem;
			bottom: 8.25rem;
			transform: none;
			max-width: none;
		}

		button {
			min-height: 2.75rem;
		}
	}
</style>
