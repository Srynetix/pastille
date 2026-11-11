// SPDX-License-Identifier: MIT OR Apache-2.0

// The scope, from the app's side of the boundary (C-22).
//
// Two halves again: the arithmetic, which needs no core; and the read itself,
// which is the whole point of the design — the plotted block stays in wasm
// memory and is viewed rather than copied, once per trace per frame.

import { describe, expect, it } from 'vitest'

import { createCircuit, viewPlot, type Circuit } from './core'
import {
	DEFAULT_TIME_BASE,
	TIME_BASES,
	choicesFor,
	extentFor,
	fit,
	labelReached,
	reached,
	lanesFor,
	scaleChoices,
	toY,
} from './scope'

describe('fitting a lane', () => {
	it('leaves a little air above and below the trace', () => {
		const extent = fit([0, 10])

		expect(extent.low).toBeLessThan(0)
		expect(extent.high).toBeGreaterThan(10)
	})

	it('gives a flat trace a range of its own', () => {
		// A steady nine volts and a steady nothing would otherwise be drawn on
		// the same row of pixels, and look alike.
		const nine = fit([9, 9, 9])
		const nothing = fit([0, 0])

		expect(nine.low).toBeLessThan(9)
		expect(nine.high).toBeGreaterThan(9)
		expect(nothing.high).toBeGreaterThan(nothing.low)
	})

	it('skips the columns the trace had not reached', () => {
		// The core writes NaN for a column before the trace starts. Reading one
		// as zero would stretch every lane down to nothing.
		const extent = fit([NaN, NaN, 5, 6])

		expect(extent.low).toBeGreaterThan(0)
	})

	it('still has a height when there is nothing at all', () => {
		const extent = fit([NaN, NaN])

		expect(extent.high).toBeGreaterThan(extent.low)
	})
})

describe('the vertical scale a lane can be pinned to', () => {
	it('offers the right order of magnitude for what is being measured', () => {
		// A milliamp trace is never offered ten amps, which is what a fixed
		// ladder of volts-per-division would have done.
		const amps = scaleChoices({ low: 0, high: 0.0033 })

		expect(amps[0]).toBeCloseTo(0.005, 10)
		expect(amps).toHaveLength(3)
		for (const choice of amps) expect(choice).toBeGreaterThan(0.0033)
	})

	it('never offers a scale that would change nothing', () => {
		const choices = scaleChoices({ low: 0, high: 10 })

		for (const choice of choices) expect(choice).toBeGreaterThan(10)
	})

	it('has nothing to offer for a trace with no magnitude', () => {
		expect(scaleChoices({ low: 0, high: 0 })).toEqual([])
	})

	it('keeps offering the scale it is already pinned to', () => {
		// Pinning a lane to ten volts makes its extent ten volts, and a ladder
		// built from that starts at twenty — so the setting in force would drop
		// out of its own list and the control would go blank.
		const fitted = { low: 0, high: 6.5 }

		expect(choicesFor(fitted, 10)).toContain(10)
		// And it comes back even once the trace has outgrown it, because a
		// control that forgets its own setting is worse than one offering a
		// scale that no longer fits.
		expect(choicesFor({ low: 0, high: 400 }, 10)[0]).toBe(10)
	})

	it('offers nothing extra when nothing is pinned', () => {
		const fitted = { low: 0, high: 6.5 }

		expect(choicesFor(fitted, undefined)).toEqual(scaleChoices(fitted))
	})

	it('keeps the side of zero the trace is on', () => {
		// A charging curve runs from nothing upwards. Giving it the negative
		// half of the lane too would spend half the height on a region the
		// trace never enters.
		const rising = extentFor({ low: 0, high: 6 }, 10)
		const falling = extentFor({ low: -6, high: 0 }, 10)
		const both = extentFor({ low: -3, high: 4 }, 10)

		expect(rising).toEqual({ low: 0, high: 10 })
		expect(falling).toEqual({ low: -10, high: 0 })
		expect(both).toEqual({ low: -10, high: 10 })
	})

	it('falls back to the fit when nothing is pinned', () => {
		const fitted = { low: 1, high: 2 }

		expect(extentFor(fitted, undefined)).toBe(fitted)
	})
})

describe('placing a value in a lane', () => {
	it('puts the top of the extent at the top', () => {
		expect(toY(10, { low: 0, high: 10 }, 50)).toBeCloseTo(0, 10)
		expect(toY(0, { low: 0, high: 10 }, 50)).toBeCloseTo(50, 10)
		expect(toY(5, { low: 0, high: 10 }, 50)).toBeCloseTo(25, 10)
	})

	it('puts a value in the middle when the lane has no range', () => {
		expect(toY(3, { low: 1, high: 1 }, 50)).toBe(25)
	})

	it('labels a lane in its own unit', () => {
		expect(labelReached({ low: 0, high: 0.0033 }, 'A')).toContain('mA')
	})
})

describe('what a lane says it read', () => {
	it('says what the trace reached, not where the lane was drawn to (C-69)', () => {
		// "A kick from the coil": a trace from 0.06 V to 34.1 V was labelled
		// −2.67 V → 36.8 V, the drawing's air read out as a measurement — a
		// negative voltage on a circuit that has none.
		const block = [0.06, 0.06, 34.1, 9]

		expect(reached(block)).toEqual({ low: 0.06, high: 34.1 })
		expect(labelReached(reached(block), 'V')).not.toContain('−')
		expect(labelReached(reached(block), 'V')).not.toContain('-')
		// The drawing keeps its air all the same.
		expect(fit(block).low).toBeLessThan(0.06)
	})

	it('reads a steady trace as the one value it is', () => {
		// Not the range the lane invents around it so it can be drawn.
		expect(labelReached(reached([9, 9, 9]), 'V')).toBe(labelReached({ low: 9, high: 9 }, 'V'))
		expect(labelReached(reached([9, 9, 9]), 'V')).not.toContain('→')
	})

	it('skips the columns the trace had not reached', () => {
		expect(reached([NaN, NaN, 5, 6])).toEqual({ low: 5, high: 6 })
	})

	it('says nothing rather than something for an empty trace', () => {
		expect(reached([NaN, NaN])).toBeUndefined()
		expect(labelReached(undefined, 'V')).toBe('—')
	})
})

describe('what the scope shows', () => {
	it('offers a decade of time bases, with ten seconds the default', () => {
		expect(TIME_BASES).toContain(DEFAULT_TIME_BASE)
		for (let index = 1; index < TIME_BASES.length; index++) {
			expect(TIME_BASES[index]).toBeGreaterThan(TIME_BASES[index - 1])
		}
	})

	it('is a lane per probe and nothing else', async () => {
		const circuit = await createCircuit()
		circuit.add('Resistor', 0, 0)
		const meter = circuit.add('Voltmeter', 40, 0)
		const ammeter = circuit.add('Ammeter', 80, 0)

		const lanes = lanesFor(circuit.elements())

		expect(lanes.map((lane) => lane.id)).toEqual([meter, ammeter])
		expect(lanes.map((lane) => lane.unit)).toEqual(['V', 'A'])
		expect(lanes[0].label).toBe('Voltmeter')
	})
})

describe('reading a trace out of the core', () => {
	/** 10 kΩ and 1000 µF — τ of ten seconds — watched by a voltmeter. */
	async function charging() {
		const circuit = await createCircuit()
		const battery = circuit.add('DcSource', 0, 0)
		const resistor = circuit.add('Resistor', 80, 0)
		const capacitor = circuit.add('Capacitor', 160, 0)
		const meter = circuit.add('Voltmeter', 160, 80)
		circuit.setValue(resistor, '10k')
		circuit.setValue(capacitor, '1000u')
		circuit.connect(battery, '+', resistor, 'a')
		circuit.connect(resistor, 'b', capacitor, 'a')
		circuit.connect(capacitor, 'b', battery, '-')
		circuit.connect(meter, '+', capacitor, 'a')
		circuit.connect(meter, '-', capacitor, 'b')
		return { circuit, meter }
	}

	function run(circuit: Circuit, seconds: number) {
		for (let frame = 0; frame < Math.round(seconds * 60); frame++) {
			circuit.advance(1 / 60)
		}
	}

	it('views the block in place rather than copying it', async () => {
		const wasm = await (await import('./core')).initCore()
		const { circuit, meter } = await charging()
		run(circuit, 1)

		const written = circuit.plotTrace(meter, 1, 40)
		const block = viewPlot(circuit, written)

		expect(written).toBe(80)
		expect(block.buffer).toBe(wasm.memory.buffer)
	})

	it('reads back the curve the probe was watching', async () => {
		const { circuit, meter } = await charging()
		run(circuit, 5)

		const written = circuit.plotTrace(meter, 5, 50)
		const block = Array.from(viewPlot(circuit, written))

		// Rising across the window, and at half a time constant it is where the
		// exponential says: 9 × (1 − e^−0.5).
		expect(block[1]).toBeLessThan(1)
		const expected = 9 * (1 - Math.exp(-0.5))
		expect(block[block.length - 1]).toBeCloseTo(expected, 1)
	})

	it('records far more than one sample a frame', async () => {
		// The reason the buffer is in the core at all. A buffer the app kept
		// would hold one sample per frame, and anything faster than about
		// thirty hertz would not exist.
		const { circuit, meter } = await charging()
		run(circuit, 1 / 60)

		expect(circuit.traceDepth(meter)).toBeGreaterThan(1)
	})

	it('has nothing to show for a probe that is not watching anything', async () => {
		const circuit = await createCircuit()
		const meter = circuit.add('Voltmeter', 0, 0)
		circuit.advance(1 / 60)

		expect(circuit.plotTrace(meter, 1, 40)).toBe(0)
		// A getter, not a call: the clock is a property of the traces.
		expect(circuit.traceTime).toBe(0)
	})
})
