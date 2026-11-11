// SPDX-License-Identifier: MIT OR Apache-2.0

import { describe, expect, it } from 'vitest'

import { createCircuit, type FailureCode } from './core'
import { detailOf, explain, splitRefusal } from './failures'

/** Every code the core can produce. The table below has to cover all of them. */
const CODES: FailureCode[] = ['nothing-wired', 'no-limit', 'unsettled', 'internal']

describe('saying why a circuit did not solve', () => {
	it('has words for every failure the core can report', () => {
		// TypeScript already refuses a table with a code missing; this catches
		// the other half — a code present with nothing useful behind it.
		for (const code of CODES) {
			const words = explain({ code, detail: '' })
			expect(words, code).toBeTruthy()
			expect(words!.length, code).toBeGreaterThan(20)
		}
	})

	it('names the thing to go and look at', () => {
		// Reactive help waits until it has something specific to say (§14).
		// "Something went wrong" is not that.
		expect(explain({ code: 'no-limit', detail: '' })).toContain('resistor')
		expect(explain({ code: 'nothing-wired', detail: '' })).toContain('pin')
	})

	it('never shows the solver its own words', () => {
		for (const code of CODES) {
			const words = explain({ code, detail: 'matrix is singular at column 1' })!
			expect(words, code).not.toContain('matrix')
			expect(words, code).not.toContain('column')
		}
	})

	it('keeps the solver its own words, out of the way', () => {
		// Kept for whoever is debugging the solver, and not in the status line.
		expect(detailOf({ code: 'no-limit', detail: 'matrix is singular at column 1' })).toBe(
			'matrix is singular at column 1',
		)
	})

	it('says nothing when nothing failed', () => {
		expect(explain(undefined)).toBeUndefined()
		expect(detailOf(undefined)).toBeUndefined()
	})
})

describe('what the core actually reports', () => {
	it('calls an unwired canvas nothing-wired', async () => {
		const circuit = await createCircuit()
		circuit.add('DcSource', 0, 0)
		circuit.advance(0.016)

		expect(circuit.state().failure?.code).toBe('nothing-wired')
	})

	it('calls an LED straight across a battery no-limit', async () => {
		// The circuit from the second M-22 session, and the whole reason this
		// ticket exists: it used to put "matrix is singular at column 1" in
		// front of somebody who had wired two things together.
		const circuit = await createCircuit()
		const battery = circuit.add('DcSource', 0, 0)
		const led = circuit.add('Led', 80, 0)
		circuit.connect(battery, '+', led, 'anode')
		circuit.connect(led, 'cathode', battery, '-')
		circuit.advance(0.016)

		const failure = circuit.state().failure
		expect(failure?.code).toBe('no-limit')
		expect(explain(failure)).toContain('resistor')
	})

	it('reports nothing once the circuit works', async () => {
		const circuit = await createCircuit()
		const battery = circuit.add('DcSource', 0, 0)
		const resistor = circuit.add('Resistor', 80, 0)
		const led = circuit.add('Led', 160, 0)
		circuit.connect(battery, '+', resistor, 'a')
		circuit.connect(resistor, 'b', led, 'anode')
		circuit.connect(led, 'cathode', battery, '-')
		circuit.advance(0.016)

		expect(circuit.state().failure).toBeUndefined()
	})
})

describe('a file that will not open, said to a person (C-100)', () => {
	it('says what to do, and keeps the parser for a tooltip', async () => {
		const circuit = await createCircuit()
		let thrown: unknown
		try {
			circuit.open('{"schema":1,"nets":[]}')
		} catch (error) {
			thrown = error
		}
		const { sentence, detail } = splitRefusal(thrown)
		expect(sentence).toBe('This file is not a circuit saved by this app, or part of it is missing.')
		expect(detail).toContain('elements')
	})

	it('tells a file from a newer version to be opened there', async () => {
		const circuit = await createCircuit()
		let thrown: unknown
		try {
			circuit.open('{"schema":99,"elements":[],"nets":[]}')
		} catch (error) {
			thrown = error
		}
		const { sentence, detail } = splitRefusal(thrown)
		expect(sentence).toContain('newer version')
		expect(detail).toContain('format 99')
	})

	it('has no detail for a refusal that is only a sentence', () => {
		expect(splitRefusal(new Error('A resistance has to be more than 0 Ω.'))).toEqual({
			sentence: 'A resistance has to be more than 0 Ω.',
			detail: undefined,
		})
	})
})
