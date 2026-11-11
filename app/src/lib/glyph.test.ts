// SPDX-License-Identifier: MIT OR Apache-2.0

import { describe, expect, it } from 'vitest'

import { glyphOf } from './glyph'
import { PALETTE } from './palette'
import { symbolFor } from './symbols'

describe('a part’s symbol in the palette (C-128)', () => {
	it('is drawn for every part on offer', () => {
		for (const entry of PALETTE) {
			const glyph = glyphOf(symbolFor(entry.kind))
			expect(glyph.shapes.length, entry.label).toBeGreaterThan(0)
			expect(
				glyph.shapes.every((shape) => shape.d.startsWith('M')),
				entry.label,
			).toBe(true)
		}
	})

	it('is the board’s drawing: one path per shape, and every pin', () => {
		// The point is that it is the same zigzag, so nothing is redrawn here.
		const resistor = symbolFor('Resistor')
		const glyph = glyphOf(resistor)
		expect(glyph.shapes).toHaveLength(resistor.shapes.length)
		expect(glyph.pins).toEqual(resistor.pins.map((pin) => pin.at))
	})

	it('frames everything it draws, pins included', () => {
		for (const entry of PALETTE) {
			const glyph = glyphOf(symbolFor(entry.kind))
			const [left, top, width, height] = glyph.viewBox.split(' ').map(Number)
			for (const pin of glyph.pins) {
				expect(pin.x, entry.label).toBeGreaterThan(left)
				expect(pin.x, entry.label).toBeLessThan(left + width)
				expect(pin.y, entry.label).toBeGreaterThan(top)
				expect(pin.y, entry.label).toBeLessThan(top + height)
			}
		}
	})

	it('fills what the board fills, as an LED’s triangle', () => {
		expect(glyphOf(symbolFor('Led')).shapes.some((shape) => shape.filled)).toBe(true)
		expect(glyphOf(symbolFor('Resistor')).shapes.some((shape) => shape.filled)).toBe(false)
	})
})
