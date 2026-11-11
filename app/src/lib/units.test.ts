// SPDX-License-Identifier: MIT OR Apache-2.0

import { describe, expect, it } from 'vitest'

import { formatQuantity, spokenValue } from './units'

describe('reading a quantity out', () => {
	it('keeps three significant figures, trailing zeros and all', () => {
		// "9 V" and "9.00 V" say different things about how hard the app looked.
		expect(formatQuantity(9, 'V')).toBe('9.00 V')
		expect(formatQuantity(1.5, 'V')).toBe('1.50 V')
	})

	it('uses the prefix a multimeter would', () => {
		expect(formatQuantity(0.0273, 'A')).toBe('27.3 mA')
		expect(formatQuantity(0.0000018, 'A')).toBe('1.80 µA')
		expect(formatQuantity(4700, 'Ω')).toBe('4.70 kΩ')
	})

	it('stays in the unit itself between one and a thousand', () => {
		expect(formatQuantity(220, 'Ω')).toBe('220 Ω')
	})

	it('promotes a value that rounding pushes out of its band', () => {
		// 999.7 mA is 1.00 A at three figures. "1000 mA" is a reading nobody
		// writes, and it is what the naive version prints.
		expect(formatQuantity(0.9997, 'A')).toBe('1.00 A')
	})

	it('carries the sign', () => {
		expect(formatQuantity(-0.005, 'A')).toBe('-5.00 mA')
	})

	it('says zero rather than a prefix nobody has heard of', () => {
		// Solver noise is not a picoamp of current, and printing it as one
		// would have the app claiming a precision it does not have.
		expect(formatQuantity(0, 'A')).toBe('0 A')
		expect(formatQuantity(1e-18, 'A')).toBe('0 A')
	})

	it('does not pretend to a number it does not have', () => {
		expect(formatQuantity(Number.NaN, 'V')).toBe('— V')
	})
})

describe('saying a typed value', () => {
	it('puts the prefix with the unit', () => {
		expect(spokenValue('27k', 'Ω')).toBe('27 kΩ')
		expect(spokenValue('4.7k', 'Ω')).toBe('4.7 kΩ')
		expect(spokenValue('470', 'Ω')).toBe('470 Ω')
		expect(spokenValue('10u', 'F')).toBe('10 µF')
	})
})
