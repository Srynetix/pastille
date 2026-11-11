// SPDX-License-Identifier: MIT OR Apache-2.0

import { describe, expect, it } from 'vitest'

import { actionFor, GESTURES, keyBindings } from './keys'

function press(key: string, modifiers: Partial<KeyboardEventInit> = {}): KeyboardEvent {
	return new KeyboardEvent('keydown', { key, ...modifiers })
}

describe('what the app answers to', () => {
	it('deletes on either delete key', () => {
		// A Mac's "delete" is a backspace; a full keyboard has both.
		expect(actionFor(press('Delete'))).toBe('delete')
		expect(actionFor(press('Backspace'))).toBe('delete')
	})

	it('undoes with the platform modifier, either one', () => {
		expect(actionFor(press('z', { metaKey: true }))).toBe('undo')
		expect(actionFor(press('z', { ctrlKey: true }))).toBe('undo')
	})

	it('redoes when shift is held', () => {
		expect(actionFor(press('z', { metaKey: true, shiftKey: true }))).toBe('redo')
	})

	it('answers to a capital Z, which is what shift produces', () => {
		expect(actionFor(press('Z', { metaKey: true, shiftKey: true }))).toBe('redo')
	})

	it('selects everything with the platform modifier and A', () => {
		expect(actionFor(press('a', { metaKey: true }))).toBe('selectAll')
		expect(actionFor(press('a', { ctrlKey: true }))).toBe('selectAll')
		// A bare A is a letter, and nothing is bound to it.
		expect(actionFor(press('a'))).toBeUndefined()
	})

	it('zooms on bare keys, and leaves the page’s own zoom alone', () => {
		expect(actionFor(press('+'))).toBe('zoomIn')
		expect(actionFor(press('='))).toBe('zoomIn')
		expect(actionFor(press('-'))).toBe('zoomOut')
		expect(actionFor(press('0'))).toBe('showAll')
		// ⌘+ and ⌘− zoom the page itself, which has to stay possible.
		expect(actionFor(press('=', { metaKey: true }))).toBeUndefined()
		expect(actionFor(press('-', { ctrlKey: true }))).toBeUndefined()
	})

	it('turns on a bare R', () => {
		expect(actionFor(press('r'))).toBe('rotate')
		expect(actionFor(press('R'))).toBe('rotate')
	})

	it('shows its own list on a question mark', () => {
		expect(actionFor(press('?', { shiftKey: true }))).toBe('help')
		expect(actionFor(press('?', { metaKey: true }))).toBeUndefined()
	})

	it('leaves the browser its own shortcuts', () => {
		// ⌘R reloads the page. Rotating the circuit as it goes would be a
		// memorable way to lose work.
		expect(actionFor(press('r', { metaKey: true }))).toBeUndefined()
		expect(actionFor(press('r', { ctrlKey: true }))).toBeUndefined()
	})

	it('says nothing about a key it does not answer to', () => {
		expect(actionFor(press('z'))).toBeUndefined()
		expect(actionFor(press('b', { metaKey: true }))).toBeUndefined()
	})
})

describe('spelling a shortcut', () => {
	it('uses the symbols on the keyboard in front of them', () => {
		expect(keyBindings(true).map((binding) => binding.label)).toEqual([
			'Delete',
			'R',
			'⌘A',
			'+',
			'−',
			'0',
			'Esc',
			'⌘Z',
			'⇧⌘Z',
			'?',
		])
		expect(keyBindings(false).map((binding) => binding.label)).toEqual([
			'Delete',
			'R',
			'Ctrl+A',
			'+',
			'−',
			'0',
			'Esc',
			'Ctrl+Z',
			'Ctrl+Shift+Z',
			'?',
		])
	})

	it('describes every binding and every gesture in words', () => {
		// The list is the answer to "is there a shortcut for this?", so an entry
		// with nothing to say would leave the question open.
		for (const entry of [...keyBindings(true), ...GESTURES]) {
			expect(entry.does.length, entry.label).toBeGreaterThan(0)
			expect(entry.label.length, entry.does).toBeGreaterThan(0)
		}
	})
})
