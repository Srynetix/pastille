// SPDX-License-Identifier: MIT OR Apache-2.0

import { describe, expect, it } from 'vitest'

import '../app.css'
import type { SolutionView } from './core'
import type { SceneWire } from './scene'
import { inkFor, paintVoltages, scaleOf } from './voltage'

const STOPS: [string, string, string] = ['#000000', '#808080', '#ffffff']

function solution(volts: Record<string, number>, failure?: unknown): SolutionView {
	return {
		netVoltages: Object.entries(volts).map(([netId, v]) => ({ netId, volts: v })),
		failure,
	} as unknown as SolutionView
}

function wire(id: string): SceneWire {
	return { id, segments: [], pins: [], waypoints: [] }
}

describe('a wire’s colour from its voltage (C-134)', () => {
	it('spans the circuit’s own lowest to highest net, from 0 V', () => {
		expect(scaleOf(solution({ a: 9, b: 2, c: 0 }))).toEqual({ low: 0, high: 9 })
		expect(scaleOf(solution({ a: 5, b: -5 }))).toEqual({ low: -5, high: 5 })
	})

	it('is never narrower than a volt, so a circuit at rest sits at the bottom', () => {
		expect(scaleOf(solution({ a: 0, b: 0 }))).toEqual({ low: 0, high: 1 })
		expect(inkFor(0, { low: 0, high: 1 }, STOPS)).toBe('#000000')
	})

	it('says nothing for a circuit that did not solve', () => {
		expect(scaleOf(solution({ a: 9 }, { code: 'Singular' }))).toBeUndefined()
		expect(scaleOf(undefined)).toBeUndefined()
	})

	it('runs through its three stops', () => {
		const scale = { low: 0, high: 10 }
		expect(inkFor(0, scale, STOPS)).toBe('#000000')
		expect(inkFor(5, scale, STOPS)).toBe('#808080')
		expect(inkFor(10, scale, STOPS)).toBe('#ffffff')
		expect(inkFor(20, scale, STOPS), 'clamped').toBe('#ffffff')
	})

	it('paints each wire its net’s colour, and leaves one the solution does not know', () => {
		const painted = paintVoltages(
			[wire('a'), wire('b'), wire('dangling')],
			solution({ a: 9, b: 0 }),
			STOPS,
		)
		expect(painted.map((each) => each.ink)).toEqual(['#ffffff', '#000000', undefined])
	})

	it('changes in lightness along the scale, not in hue alone', () => {
		// Read from the theme in force, whichever it is: either way the three
		// stops must run steadily lighter or steadily darker.
		const styles = getComputedStyle(document.documentElement)
		const lightness = ['--volt-low', '--volt-mid', '--volt-high'].map((token) => {
			const hex = styles.getPropertyValue(token).trim()
			const [r, g, b] = [1, 3, 5].map((at) => parseInt(hex.slice(at, at + 2), 16) / 255)
			return 0.2126 * r + 0.7152 * g + 0.0722 * b
		})
		const rising = lightness[0] < lightness[1] && lightness[1] < lightness[2]
		const falling = lightness[0] > lightness[1] && lightness[1] > lightness[2]
		expect(rising || falling, lightness.join(', ')).toBe(true)
	})
})
