// SPDX-License-Identifier: MIT OR Apache-2.0

// How hot a part runs, as the app says it and draws it (C-74).

import { describe, expect, it } from 'vitest'

import type { ElementHeat } from './core'
import { HEAT_RAMP, heatHeading, heatInk, heatReading, heatWarning, warmth } from './heat'

function heat(overrides: Partial<ElementHeat> = {}): ElementHeat {
	return {
		elementId: 'r1',
		watts: 0.9,
		celsius: 85.2,
		settlesAt: 85.4,
		ratedWatts: 1,
		maxCelsius: 125,
		...overrides,
	}
}

describe('what the inspector says about a part that warms', () => {
	it('gives the power and the temperature in one line', () => {
		expect(heatReading(heat())).toBe('900 mW, about 85 °C')
	})

	it('says nothing is wrong inside its limits', () => {
		expect(heatWarning(heat())).toBeUndefined()
	})

	it('says so past its rated power', () => {
		expect(heatWarning(heat({ watts: 1.2 }))).toBe('More than the 1 W it is rated for.')
	})

	it('puts a temperature past its limit first', () => {
		expect(heatWarning(heat({ watts: 1.2, celsius: 130 }))).toBe(
			'Hotter than the 125 °C it is made to survive.',
		)
	})

	it('says where it is heading while it is on its way, and not once it is there', () => {
		expect(heatHeading(heat({ celsius: 40, settlesAt: 150 }))).toBe('Heading for about 150 °C.')
		expect(heatHeading(heat({ celsius: 90, settlesAt: 25 }))).toBe('Cooling towards about 25 °C.')
		expect(heatHeading(heat())).toBeUndefined()
	})
})

describe('how a part that warms is drawn', () => {
	it('is measured from the room to its limit', () => {
		expect(warmth(heat({ celsius: 25 }), 25)).toBe(0)
		expect(warmth(heat({ celsius: 75 }), 25)).toBe(0.5)
		expect(warmth(heat({ celsius: 150 }), 25)).toBeGreaterThan(1)
	})

	it('is left alone near the room', () => {
		expect(heatInk('#8b919b', 0)).toBeUndefined()
		expect(heatInk('#8b919b', 0.02)).toBeUndefined()
	})

	it('turns amber half way and red at its limit', () => {
		expect(heatInk('#8b919b', 0.5)).toBe(HEAT_RAMP.warm)
		expect(heatInk('#8b919b', 1)).toBe(HEAT_RAMP.hot)
		expect(heatInk('#8b919b', 3)).toBe(HEAT_RAMP.hot)
	})

	it('passes between the two', () => {
		const between = heatInk('#8b919b', 0.25)
		expect(between).toMatch(/^#[0-9a-f]{6}$/)
		expect(between).not.toBe('#8b919b')
		expect(between).not.toBe(HEAT_RAMP.warm)
	})
})
