<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

<script lang="ts">
	import { GESTURES, keyBindings, TOUCH_GESTURES, touchOnly } from '../lib/keys'

	// C-42. Ambient help in the sense of §14: always there, never in the way,
	// nothing to dismiss. It is deliberately the *whole* list — the finding was
	// that a user could not tell "there is no shortcut for this" from "I have
	// not guessed it yet", and a partial list answers neither.
	//
	// It renders the same table the shell dispatches through, so it cannot drift
	// from what the app actually does.
	//
	// On a device that is only a finger, the keys it does not have are left
	// out and what it can do instead is said (C-99).
	const finger = touchOnly()
	const bindings = finger ? [] : keyBindings()
	const gestures = finger ? TOUCH_GESTURES : GESTURES
</script>

<aside class="keys" aria-label="Keys">
	<h2>{finger ? 'Touch' : 'Keys'}</h2>
	<dl>
		{#each bindings as binding (binding.action)}
			<div class="row">
				<dt>{binding.does}</dt>
				<dd><kbd>{binding.label}</kbd></dd>
			</div>
		{/each}
		{#each gestures as gesture (gesture.does)}
			<div class="row">
				<dt>{gesture.does}</dt>
				<dd>{gesture.label}</dd>
			</div>
		{/each}
	</dl>
</aside>

<style>
	.keys {
		border-right: 1px solid var(--edge);
		border-top: 1px solid var(--edge);
		padding: 0.75rem 1rem 1rem;
	}

	h2 {
		font-size: 0.75rem;
		text-transform: uppercase;
		letter-spacing: 0.08em;
		color: var(--muted);
		margin: 0 0 0.5rem;
	}

	dl {
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.row {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 0.5rem;
	}

	dt,
	dd {
		margin: 0;
		font-size: 0.75rem;
		color: var(--muted);
	}

	dd {
		flex: none;
		text-align: right;
	}

	kbd {
		font: inherit;
		color: var(--fg);
		background: var(--sunken);
		border: 1px solid var(--edge);
		border-radius: 4px;
		padding: 0.0625rem 0.3125rem;
		white-space: nowrap;
	}
</style>
