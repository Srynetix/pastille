// SPDX-License-Identifier: MIT OR Apache-2.0

import { describe, expect, it } from 'vitest'

import type { ElementView, NetView, SolutionView } from './core'
import { noteName, readingFor, toneLabel } from './readings'

function resistor(id = 'r1'): ElementView {
	return {
		id,
		kind: 'Resistor',
		x: 0,
		y: 0,
		turns: 0,
		label: '220 Ω',
		value: '220',
		unit: 'Ω',
		colour: undefined,
		seriesOhms: undefined,
		hertz: undefined,
		initialCondition: undefined,
		initialUnit: '',
		closed: undefined,
		blown: undefined,
		wiper: undefined,
		choice: undefined,
		choices: [],
		basedOn: undefined,
		program: undefined,
	}
}

/** A resistor wired between two nets, the way the solver would report it. */
function solved(overrides: Partial<SolutionView> = {}): SolutionView {
	return {
		netVoltages: [
			{ netId: 'n1', volts: 9 },
			{ netId: 'n2', volts: 7.5 },
		],
		elementCurrents: [{ elementId: 'r1', amps: 0.0068, brightness: undefined }],
		netCurrents: [],
		pinCurrents: [],
		digits: [],
		failure: undefined,
		time: 1,
		findings: [],
		energised: [],
		heat: [],
		tones: [],
		angles: [],
		batteries: [],
		reversed: [],
		serial: [],
		...overrides,
	}
}

const WIRED: NetView[] = [
	{ id: 'n1', waypoints: [], pins: [{ elementId: 'r1', pin: 'a' }] },
	{ id: 'n2', waypoints: [], pins: [{ elementId: 'r1', pin: 'b' }] },
]

describe('what one component is doing', () => {
	it('takes the difference between the nets its pins are on', () => {
		expect(readingFor(resistor(), WIRED, solved()).volts).toBeCloseTo(1.5)
	})

	it('reports the current through it', () => {
		expect(readingFor(resistor(), WIRED, solved()).amps).toBeCloseTo(0.0068)
	})

	it('gives a magnitude whichever way round the component sits', () => {
		// Which way round it is belongs to the symbol and the findings. A minus
		// sign in the readout reads as an error, which it is not.
		const reversed: NetView[] = [
			{ id: 'n2', waypoints: [], pins: [{ elementId: 'r1', pin: 'a' }] },
			{ id: 'n1', waypoints: [], pins: [{ elementId: 'r1', pin: 'b' }] },
		]

		expect(readingFor(resistor(), reversed, solved()).volts).toBeCloseTo(1.5)
	})

	it('says nothing about a pin that is not wired to anything', () => {
		// Half-connected is not zero volts across. It is not knowing, and the
		// app should not invent the difference.
		const half: NetView[] = [{ id: 'n1', waypoints: [], pins: [{ elementId: 'r1', pin: 'a' }] }]

		expect(readingFor(resistor(), half, solved()).volts).toBeUndefined()
	})

	it('says nothing at all when the circuit did not solve', () => {
		const reading = readingFor(
			resistor(),
			WIRED,
			solved({ failure: { code: 'nothing-wired', detail: 'nothing is wired up yet' } }),
		)

		expect(reading).toEqual({})
	})

	it('says nothing before the first solution', () => {
		expect(readingFor(resistor(), WIRED, undefined)).toEqual({})
	})

	it('has no voltage across a component with one pin', () => {
		// A ground has nothing to be across. The current still means something.
		const ground: ElementView = { ...resistor('g1'), kind: 'Ground' }
		const nets: NetView[] = [{ id: 'n1', waypoints: [], pins: [{ elementId: 'g1', pin: 'gnd' }] }]

		expect(readingFor(ground, nets, solved()).volts).toBeUndefined()
	})
})

describe('the note a passive buzzer is playing', () => {
	const buzzer = { ...resistor(), id: 'z1', kind: 'PassiveBuzzer' } as ElementView

	it('is named as a tuner names it', () => {
		expect(noteName(440)).toBe('A4')
		expect(noteName(261.6)).toBe('C4')
		expect(noteName(880)).toBe('A5')
		expect(noteName(55)).toBe('A1')
		expect(noteName(466.2)).toBe('A♯4')
		// A few hertz flat is still the note it is trying to be.
		expect(noteName(435)).toBe('A4')
	})

	it('is written beside it while it sounds, and not while it does not', () => {
		expect(toneLabel(buzzer, solved({ tones: [{ elementId: 'z1', hertz: 439.6 }] }))).toBe(
			'440 Hz · A4',
		)
		expect(toneLabel(buzzer, solved())).toBeUndefined()
		// And only for a passive buzzer.
		expect(
			toneLabel(resistor(), solved({ tones: [{ elementId: 'r1', hertz: 440 }] })),
		).toBeUndefined()
	})
})
