// SPDX-License-Identifier: MIT OR Apache-2.0

import { describe, expect, it } from 'vitest'

import { replacedSentence } from './replace'

describe('replacing a part (C-137)', () => {
	it('says which wires it kept, on which pins, and which it let go', () => {
		expect(
			replacedSentence('Potentiometer', {
				kept: [
					['a', 'a'],
					['b', 'b'],
				],
				dropped: [],
			}),
		).toBe('Potentiometer in its place. Kept its wires on a and b.')
		expect(replacedSentence('MOSFET', { kept: [['b', 'g']], dropped: ['c', 'e'] })).toBe(
			'MOSFET in its place. Kept its wires on b → g. Let go the wires on c and e.',
		)
		expect(replacedSentence('555 timer', { kept: [], dropped: ['a'] })).toBe(
			'555 timer in its place. Let go the wire on a.',
		)
	})
})
