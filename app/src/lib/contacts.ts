// SPDX-License-Identifier: MIT OR Apache-2.0

// What a press and a release mean for a switch and for a push-button.
//
// C-19. The core holds one electrical fact — a contact is made or it is not —
// and the entire difference between the two components is here: a switch
// latches where it is put, a button springs back the moment it is let go.
//
// A pure function of the element and the gesture rather than a branch in the
// shell, so it can be read in one place and tested without a canvas. The canvas
// itself learns nothing: it reports that a pointer went down on something and
// came up again, exactly as it already reports a drag.

import type { ElementView } from './core'

/**
 * The kinds that are pressed and let go: a push-button, whose contact springs
 * back, and a PIR module, held while someone moves in front of it (C-116).
 * Every other contact latches.
 */
const MOMENTARY = new Set(['PushButton', 'Pir'])

/**
 * A contact a hand does not work: a reed switch's is its magnet's to make
 * (C-115), so a click on it is a click on any other part.
 */
const UNTOUCHED = new Set(['Reed'])

/**
 * What a contact should be once the pointer is down on it, or nothing if this
 * press has nothing to do with a contact.
 *
 * Only a button acts on the way down. A switch waits for the release, because
 * until then the gesture may turn out to be a drag.
 */
export function afterPress(element: ElementView | undefined): boolean | undefined {
	return element && MOMENTARY.has(element.kind) ? true : undefined
}

/**
 * What a contact should be once the pointer comes up again, or nothing.
 *
 * `moved` is why a release has to carry anything at all: a switch is flipped by
 * a click, and a drag that carried it across the canvas was somebody moving it.
 * Without that distinction a switch could not be repositioned without also
 * being flipped, which is the frustration C-23 was about arriving by the back
 * door. A button is released either way — it is being let go, whatever else the
 * hand was doing.
 */
export function afterRelease(
	element: ElementView | undefined,
	moved: boolean,
): boolean | undefined {
	if (!element || element.closed === undefined || UNTOUCHED.has(element.kind)) return undefined
	if (MOMENTARY.has(element.kind)) return false
	return moved ? undefined : !element.closed
}
