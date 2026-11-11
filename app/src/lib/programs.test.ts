// SPDX-License-Identifier: MIT OR Apache-2.0

import { describe, expect, it } from 'vitest'

import { EXAMPLES } from './examples'
import { PROGRAMS } from './programs'

describe('the programs an Uno comes with (A-07)', () => {
	it('each runs in an example, as built', () => {
		// An example carries its program in the file; it must be the build in
		// programs/, or a rebuilt sketch would leave its example running the
		// old one.
		const carried = EXAMPLES.flatMap((example) =>
			(
				JSON.parse(example.document) as { elements: { program?: { name: string; hex: string } }[] }
			).elements
				.map((element) => element.program)
				.filter((program) => program !== undefined),
		)
		for (const program of PROGRAMS) {
			const inExamples = carried.filter((other) => other.name === program.name)
			expect(inExamples.length, `${program.name} has an example`).toBeGreaterThan(0)
			for (const other of inExamples) expect(other.hex, program.name).toBe(program.hex)
		}
	})
})
