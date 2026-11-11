// SPDX-License-Identifier: MIT OR Apache-2.0

import type { Replacement } from './core'

/**
 * What to say once a part has been replaced by another kind (C-137): which
 * of its wires it kept, on which pins now, and which it let go. Said every
 * time, because a wire let go is a change to the circuit the person did not
 * make with their own hands.
 */
export function replacedSentence(label: string, replacement: Replacement): string {
	const kept = replacement.kept.map(([was, now]) => (was === now ? was : `${was} → ${now}`))
	// No article and no lower case: "an LED", "a MOSFET" are the label's to get right.
	const parts = [`${label} in its place.`]
	if (kept.length > 0) parts.push(`Kept its wires on ${list(kept)}.`)
	if (replacement.dropped.length > 0) {
		parts.push(
			`Let go the ${replacement.dropped.length > 1 ? 'wires' : 'wire'} on ${list(replacement.dropped)}.`,
		)
	}
	if (kept.length === 0 && replacement.dropped.length === 0) parts.push('It had no wires to keep.')
	return parts.join(' ')
}

function list(items: readonly string[]): string {
	return items.length > 1 ? `${items.slice(0, -1).join(', ')} and ${items.at(-1)}` : items[0]
}
