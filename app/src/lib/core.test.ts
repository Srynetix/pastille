// SPDX-License-Identifier: MIT OR Apache-2.0

// Runs in real Chromium, not in Node: the point is that the bundle loads and
// the boundary behaves in the environment the app actually ships to.

import { describe, expect, it } from 'vitest'
import {
	coreVersion,
	echoProbe,
	initCore,
	SolverFrame,
	validateProbe,
	viewFrame,
	type BoundaryProbe,
} from './core'

describe('wasm boundary', () => {
	it('loads the bundle and reports the crate version', async () => {
		await expect(coreVersion()).resolves.toMatch(/^\d+\.\d+\.\d+$/)
	})

	it('instantiates once however many callers ask', async () => {
		const [first, second] = await Promise.all([initCore(), initCore()])
		expect(first).toBe(second)
	})

	describe('control plane', () => {
		it('rebuilds a structured value on the Rust side', async () => {
			const sent: BoundaryProbe = { label: 'divider', nodeVoltages: [9, 4.5] }
			const back = await echoProbe(sent, 2)

			// Rewritten label and scaled voltages: the value was deserialized and
			// rebuilt, not handed straight back.
			expect(back.label).toBe('divider (via rust)')
			expect(back.nodeVoltages).toEqual([18, 9])
			expect(sent.nodeVoltages).toEqual([9, 4.5])
		})

		it('preserves floats exactly across the round trip', async () => {
			const value = 1 / 3
			const back = await echoProbe({ label: 'f64', nodeVoltages: [value] }, 1)
			expect(back.nodeVoltages[0]).toBe(value)
		})

		it('surfaces a core error as a thrown Error', async () => {
			await expect(validateProbe({ label: 'empty', nodeVoltages: [] })).rejects.toThrow(
				'a probe needs at least one node voltage',
			)
		})
	})

	describe('data plane', () => {
		it('views wasm memory without copying it', async () => {
			await initCore()
			const frame = new SolverFrame(3, 1.5)

			expect(Array.from(viewFrame(frame))).toEqual([0, 1.5, 3])

			// Mutating in Rust changes what the view sees, with nothing returned
			// across the boundary — this is the property the render loop depends on.
			frame.shift(0.5)
			expect(Array.from(viewFrame(frame))).toEqual([0.5, 2, 3.5])

			frame.free()
		})

		it('hands out a view backed by wasm memory, not a copy', async () => {
			const wasm = await initCore()
			const frame = new SolverFrame(2, 1)

			expect(viewFrame(frame).buffer).toBe(wasm.memory.buffer)

			frame.free()
		})
	})
})
