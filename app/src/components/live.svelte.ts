// SPDX-License-Identifier: MIT OR Apache-2.0

// For tests: props a mounted component follows, as it follows its parent's.
// A plain object passed to `mount` is read once; this one is reactive, so a
// test can hand the component a new value the way the app does.

/** `initial`, made reactive: assigning to one of its fields updates the component. */
export function live<T extends object>(initial: T): T {
	const props = $state(initial)
	return props
}
