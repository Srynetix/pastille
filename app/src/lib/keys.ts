// SPDX-License-Identifier: MIT OR Apache-2.0

// What the app answers to, as data.
//
// C-42. The finding behind this file was not a missing shortcut but a missing
// list: delete, undo and redo were bound and unannounced, so a user could not
// tell "there is no shortcut for this" from "I have not guessed it yet"
// (the author's test, M-22).
//
// A list drawn from a table that nothing else reads would go stale the first
// time a binding changed, and a wrong list is worse than none. So the table is
// the behaviour: the shell dispatches key presses through `actionFor`, and the
// panel renders the same entries. A binding that is not here does not work, and
// one that works is shown.

/** Something a key press asks the shell to do. */
export type KeyAction =
	| 'delete'
	| 'rotate'
	| 'undo'
	| 'redo'
	| 'selectAll'
	| 'zoomIn'
	| 'zoomOut'
	| 'showAll'
	| 'cancel'
	| 'help'

export interface KeyBinding {
	readonly action: KeyAction
	/** What it does, in the user's words. */
	readonly does: string
	/** How to press it, spelled for the keyboard in front of them. */
	readonly label: string
	/** Whether this press is that press. */
	readonly matches: (event: KeyboardEvent) => boolean
}

/** Something done with a pointer, which has no key to press and still has to be found. */
export interface Gesture {
	readonly does: string
	readonly label: string
}

/**
 * Whether to spell the modifier the Apple way.
 *
 * The user agent is the only thing on offer and it is a guess, not a fact — but
 * the cost of guessing wrong is a label that says Ctrl to someone holding ⌘,
 * who will try ⌘ anyway because that is what every other app does.
 */
export function isApple(): boolean {
	if (typeof navigator === 'undefined') return false
	return /mac|iphone|ipad|ipod/i.test(navigator.platform || navigator.userAgent)
}

/** Every key the app answers to, spelled for this keyboard. */
export function keyBindings(apple: boolean = isApple()): readonly KeyBinding[] {
	const modifier = (event: KeyboardEvent) => event.metaKey || event.ctrlKey
	const isZ = (event: KeyboardEvent) => modifier(event) && event.key.toLowerCase() === 'z'

	return [
		{
			action: 'delete',
			does: 'Remove the selection',
			// Apple labels the key "delete" even though it is a backspace, and
			// both it and a real Delete are bound, so one word covers every
			// keyboard.
			label: 'Delete',
			matches: (event) => event.key === 'Delete' || event.key === 'Backspace',
		},
		{
			action: 'rotate',
			does: 'Turn it 90°',
			label: 'R',
			// Bare, with no modifier: it is the one edit a user reaches for
			// repeatedly, and every schematic editor binds it here.
			matches: (event) => !modifier(event) && event.key.toLowerCase() === 'r',
		},
		{
			action: 'selectAll',
			does: 'Select everything',
			label: apple ? '⌘A' : 'Ctrl+A',
			matches: (event) => modifier(event) && !event.shiftKey && event.key.toLowerCase() === 'a',
		},
		// Bare, like R: the browser keeps ⌘+ and ⌘− for zooming the page, and
		// taking them would make the page itself unzoomable. = is + unshifted.
		{
			action: 'zoomIn',
			does: 'Zoom in',
			label: '+',
			matches: (event) => !modifier(event) && (event.key === '+' || event.key === '='),
		},
		{
			action: 'zoomOut',
			does: 'Zoom out',
			label: '−',
			matches: (event) => !modifier(event) && (event.key === '-' || event.key === '_'),
		},
		{
			action: 'showAll',
			does: 'Show the whole circuit',
			label: '0',
			matches: (event) => !modifier(event) && event.key === '0',
		},
		{
			action: 'cancel',
			// A wire half drawn, an area half picked, a part carried from the
			// palette: let go of it. With nothing under way, let go of the
			// selection (C-103).
			does: 'Cancel, or deselect',
			label: 'Esc',
			matches: (event) => event.key === 'Escape',
		},
		{
			action: 'undo',
			does: 'Undo',
			label: apple ? '⌘Z' : 'Ctrl+Z',
			matches: (event) => isZ(event) && !event.shiftKey,
		},
		{
			action: 'redo',
			does: 'Redo',
			label: apple ? '⇧⌘Z' : 'Ctrl+Shift+Z',
			matches: (event) => isZ(event) && event.shiftKey,
		},
		{
			action: 'help',
			// The list moved behind the header's ? (C-127), so the key that
			// opens it is in it: whoever has found it once can find it again
			// without the mouse.
			does: 'Show these shortcuts',
			label: '?',
			matches: (event) => !modifier(event) && event.key === '?',
		},
	]
}

/** Everything done with a pointer that a user would otherwise have to discover. */
export const GESTURES: readonly Gesture[] = [
	{ does: 'Wire two pins', label: 'Drag pin to pin' },
	{ does: 'Bend a wire', label: 'Drag the wire' },
	// C-19. A switch that nothing tells you to click is a switch nobody flips,
	// and the finding behind this panel was exactly that: a user cannot tell
	// "there is no way to do this" from "I have not guessed it yet".
	{ does: 'Flip a switch', label: 'Click it' },
	{ does: 'Press a button', label: 'Hold it down' },
	{ does: 'Add to the selection', label: 'Shift-click' },
	{ does: 'Pick an area', label: 'Shift-drag the grid' },
	{ does: 'Zoom', label: 'Scroll' },
	{ does: 'Move around', label: 'Drag the grid' },
]

/**
 * Everything done with a finger, for a device with no keyboard and no mouse
 * (C-99): the same panel, telling a phone what it can do rather than listing
 * keys it does not have and a scroll wheel it cannot turn.
 */
export const TOUCH_GESTURES: readonly Gesture[] = [
	{ does: 'Add a part', label: 'Tap it in the list' },
	// C-133: the hover readout, for a screen with no hover.
	{ does: 'See what a part is doing', label: 'Rest a finger on it' },
	{ does: 'Wire two pins', label: 'Drag pin to pin' },
	{ does: 'Bend a wire', label: 'Drag the wire' },
	{ does: 'Flip a switch', label: 'Tap it' },
	{ does: 'Press a button', label: 'Hold it down' },
	{ does: 'Turn or delete it', label: 'In the panel that opens' },
	{ does: 'Undo, redo', label: 'Buttons at the top' },
	// C-138: no Shift key, so a hold first, as on a part.
	{ does: 'Pick an area', label: 'Hold the grid, then drag' },
	{ does: 'Zoom', label: 'Pinch, or + and −' },
	{ does: 'Move around', label: 'Drag the grid' },
]

/**
 * Whether the only way in is a finger: a coarse pointer and no fine one.
 * A tablet with a keyboard and a trackpad still gets the keys.
 */
export function touchOnly(): boolean {
	if (typeof matchMedia === 'undefined') return false
	return matchMedia('(pointer: coarse)').matches && !matchMedia('(any-pointer: fine)').matches
}

/** What this press asks for, or nothing if the app does not answer to it. */
export function actionFor(
	event: KeyboardEvent,
	bindings: readonly KeyBinding[] = keyBindings(),
): KeyAction | undefined {
	return bindings.find((binding) => binding.matches(event))?.action
}
