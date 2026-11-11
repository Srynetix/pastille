<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

<script lang="ts">
	import Keys from './Keys.svelte'

	// C-127. The list C-42 put in the rail, behind the header's ?: it took half
	// the rail whether anyone was reading it or not, and the board is the
	// thing the screen is for. Still the whole list, still drawn from the one
	// table, and one press away — of this button or of ?, which is in it.
	//
	// What C-42's finding was about — nobody could guess how to delete or
	// turn — is answered beside those actions as well, where the keys are
	// written on the buttons themselves (App.svelte, `.selection-actions`).
	let { open = $bindable(false) }: { open?: boolean } = $props()

	let wrapper = $state<HTMLElement | undefined>()

	// A press anywhere else puts it away, as any popover does; one on the
	// button itself is the button's to toggle.
	$effect(() => {
		if (!open) return
		const away = (event: PointerEvent) => {
			if (wrapper && !wrapper.contains(event.target as Node)) open = false
		}
		window.addEventListener('pointerdown', away, true)
		return () => window.removeEventListener('pointerdown', away, true)
	})

	// Escape closes it and does nothing else: without this it would also
	// reach the shell and drop the selection the user was about to act on.
	function handleKey(event: KeyboardEvent) {
		if (open && event.key === 'Escape') {
			event.stopPropagation()
			open = false
		}
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="shortcuts" bind:this={wrapper} onkeydown={handleKey}>
	<button
		type="button"
		class="icon"
		aria-label="Shortcuts"
		aria-expanded={open}
		title="Shortcuts (?)"
		onclick={() => (open = !open)}
	>
		<svg viewBox="0 0 24 24" aria-hidden="true">
			<path
				d="M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20zM9.1 9a3 3 0 0 1 5.8 1c0 2-3 3-3 3M12 17h.01"
			/>
		</svg>
	</button>
	{#if open}
		<div class="panel" role="dialog" aria-label="Shortcuts">
			<Keys />
		</div>
	{/if}
</div>

<style>
	.shortcuts {
		position: relative;
	}

	/* The same as the header's other icon buttons, which live in App.svelte. */
	.icon {
		width: 2.25rem;
		height: 2.25rem;
		display: flex;
		align-items: center;
		justify-content: center;
		background: none;
		border: none;
		border-radius: 8px;
		color: var(--muted);
		cursor: pointer;
	}

	.icon:focus-visible,
	.icon[aria-expanded='true'] {
		color: var(--fg);
		background: var(--raised);
	}

	@media (hover: hover) {
		.icon:hover {
			color: var(--fg);
			background: var(--raised);
		}
	}

	svg {
		width: 1.125rem;
		height: 1.125rem;
		fill: none;
		stroke: currentColor;
		stroke-width: 1.8;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.panel {
		position: absolute;
		right: 0;
		top: calc(100% + 0.5rem);
		width: 18rem;
		max-height: calc(100dvh - 5rem);
		overflow-y: auto;
		background: var(--bg);
		border: 1px solid var(--edge);
		border-radius: 12px;
		box-shadow: 0 12px 32px rgb(0 0 0 / 0.35);
		z-index: 10;
	}

	/* The panel's own frame replaces the rail's borders it was drawn with. */
	.panel :global(.keys) {
		border: none;
		padding: 0.875rem 1rem 1rem;
	}
</style>
