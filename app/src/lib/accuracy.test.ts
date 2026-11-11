// SPDX-License-Identifier: MIT OR Apache-2.0

// C-26: the claim, and the places it is made.

import { describe, expect, it } from 'vitest'

import { ACCURACY } from './accuracy'
import { accuracyNote, createCircuit } from './core'

describe('what the app says this simulation is', () => {
	it('says what the core says, word for word', async () => {
		// Two copies of one sentence, which is the arrangement ADR 0008 uses for
		// everything the app and the core both know. The copy exists so the
		// status bar does not depend on the bundle loading; this is what stops
		// the two drifting.
		expect(ACCURACY).toBe(await accuracyNote())
	})

	it('says it in a way a beginner can act on', () => {
		// Not "the numerical accuracy of this solver is not warranted". The
		// audience of §4 has never built a circuit, and the claim is only worth
		// making if they can tell what it means for them.
		expect(ACCURACY).toContain('not accurate enough to design from')
	})
})

describe('a circuit saved to a file', () => {
	it('carries the claim with it', async () => {
		// The file is read where the status bar is not. C-26's second placement,
		// and the whole reason the core owns the sentence.
		const circuit = await createCircuit()
		circuit.add('Resistor', 0, 0)

		expect((JSON.parse(circuit.save()) as { note: string }).note).toBe(ACCURACY)
	})

	it('still opens one written before the claim was in it', async () => {
		const circuit = await createCircuit()
		circuit.load(
			'{"schema":1,"elements":[{"kind":{"Resistor":{"ohms":220.0}},"x":0.0,"y":0.0,"turns":0}],"nets":[]}',
		)

		expect(circuit.elements()).toHaveLength(1)
	})
})
