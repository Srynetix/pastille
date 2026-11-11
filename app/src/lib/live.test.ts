// SPDX-License-Identifier: MIT OR Apache-2.0

// The live loop, seen from the app side.

import { describe, expect, it } from 'vitest'

import { createCircuit, type Circuit } from './core'

/** Battery, 470 Ω, red LED, wired into a loop. */
async function milestone(): Promise<{ circuit: Circuit; ids: Record<string, string> }> {
	const circuit = await createCircuit()
	const battery = circuit.add('DcSource', 0, 0)
	const resistor = circuit.add('Resistor', 40, 0)
	const led = circuit.add('Led', 80, 0)

	circuit.connect(battery, '+', resistor, 'a')
	circuit.connect(resistor, 'b', led, 'anode')
	circuit.connect(led, 'cathode', battery, '-')

	return { circuit, ids: { battery, resistor, led } }
}

function currentOf(circuit: Circuit, id: string): number {
	const found = circuit.state().elementCurrents.find((current) => current.elementId === id)
	if (!found) throw new Error(`no current for ${id}`)
	return found.amps
}

describe('the live loop', () => {
	it('solves without being started', async () => {
		const { circuit, ids } = await milestone()

		circuit.advance(0.016)

		expect(circuit.state().failure).toBeUndefined()
		expect(currentOf(circuit, ids.led)).toBeCloseTo(0.0153482714, 9)
	})

	it('reflects a value edit on the next frame', async () => {
		const { circuit, ids } = await milestone()
		circuit.advance(0.016)
		const bright = currentOf(circuit, ids.led)

		circuit.setValue(ids.resistor, '2k2')
		circuit.advance(0.016)

		expect(currentOf(circuit, ids.led)).toBeLessThan(bright / 4)
	})

	it('reports brightness for the LED and nothing else', async () => {
		const { circuit, ids } = await milestone()
		circuit.advance(0.016)

		const currents = circuit.state().elementCurrents
		const led = currents.find((current) => current.elementId === ids.led)
		const battery = currents.find((current) => current.elementId === ids.battery)

		expect(led?.brightness).toBeGreaterThan(0)
		expect(battery?.brightness).toBeUndefined()
	})

	it('keeps running while a circuit is half-built', async () => {
		// Frames keep arriving whether or not the circuit makes sense, and the
		// loop has to survive every intermediate state a person passes through.
		const circuit = await createCircuit()

		const resistor = circuit.add('Resistor', 0, 0)
		circuit.advance(0.016)
		expect(circuit.state().failure).toBeDefined()

		const battery = circuit.add('DcSource', 40, 0)
		circuit.advance(0.016)
		expect(circuit.state().failure).toBeDefined()

		circuit.connect(battery, '+', resistor, 'a')
		circuit.connect(resistor, 'b', battery, '-')
		circuit.advance(0.016)

		expect(circuit.state().failure).toBeUndefined()
		// Negative: current leaves the battery by its `+` rather than entering
		// it, which is the one sign convention every element reports under.
		expect(currentOf(circuit, battery)).toBeCloseTo(-9 / 470, 9)
	})

	it('survives a hundred frames with an edit in every one', async () => {
		// The acceptance's "editing while solving never deadlocks": the loop and
		// the edits interleave for as long as anyone would keep typing.
		const { circuit, ids } = await milestone()

		for (let frame = 0; frame < 100; frame++) {
			circuit.setValue(ids.resistor, `${220 + frame}`)
			circuit.advance(0.016)
			expect(circuit.state().failure).toBeUndefined()
		}

		expect(currentOf(circuit, ids.led)).toBeGreaterThan(0)
	})

	it('advances simulated time across frames', async () => {
		const { circuit } = await milestone()

		for (let frame = 0; frame < 10; frame++) circuit.advance(0.016)

		expect(circuit.state().time).toBeCloseTo(0.16, 2)
	})

	it('does not stall after a long gap between frames', async () => {
		// A backgrounded tab reports a huge elapsed time. The loop makes up what
		// it reasonably can and carries on rather than freezing.
		const { circuit, ids } = await milestone()

		circuit.advance(600)
		circuit.advance(0.016)

		expect(circuit.state().failure).toBeUndefined()
		expect(currentOf(circuit, ids.led)).toBeGreaterThan(0)
	})

	it('reports a net voltage for every net', async () => {
		const { circuit } = await milestone()
		circuit.advance(0.016)

		const voltages = circuit.state().netVoltages
		expect(voltages).toHaveLength(3)
		expect(voltages.some((net) => Math.abs(net.volts - 9) < 1e-9)).toBe(true)
		expect(voltages.some((net) => net.volts === 0)).toBe(true)
	})
})

describe('what the status line says', () => {
	// The note itself lives in the shell, so these check the facts it reads.

	it('has nothing to report on an empty canvas', async () => {
		const circuit = await createCircuit()
		circuit.advance(0.016)

		expect(circuit.elements()).toHaveLength(0)
	})

	it('reports no nets while nothing is wired', async () => {
		// The solver's own answer here is "no source to take a reference from",
		// which to someone who has just placed a battery reads as the app not
		// having noticed it. The shell says something true instead, and this is
		// the fact it keys off.
		const circuit = await createCircuit()
		circuit.add('DcSource', 0, 0)
		circuit.advance(0.016)

		expect(circuit.nets()).toHaveLength(0)
		expect(circuit.state().failure).toBeDefined()
	})

	it('has nets once anything is wired', async () => {
		const { circuit } = await milestone()
		circuit.advance(0.016)

		expect(circuit.nets().length).toBeGreaterThan(0)
		expect(circuit.state().failure).toBeUndefined()
	})
})

describe('several circuits on one canvas', () => {
	it('solves two circuits that share no wire', async () => {
		// A beginner putting two experiments side by side is normal, and before
		// every circuit had its own reference the second left the matrix
		// singular and neither of them worked.
		const circuit = await createCircuit()

		const first = circuit.add('DcSource', 0, 0)
		const firstLoad = circuit.add('Resistor', 80, 0)
		circuit.connect(first, '+', firstLoad, 'a')
		circuit.connect(firstLoad, 'b', first, '-')

		const second = circuit.add('DcSource', 0, 200)
		const secondLoad = circuit.add('Resistor', 80, 200)
		circuit.setValue(second, '5')
		circuit.setValue(secondLoad, '1k')
		circuit.connect(second, '+', secondLoad, 'a')
		circuit.connect(secondLoad, 'b', second, '-')

		circuit.advance(0.016)

		expect(circuit.state().failure).toBeUndefined()
		expect(currentOf(circuit, first)).toBeCloseTo(-9 / 470, 9)
		expect(currentOf(circuit, second)).toBeCloseTo(-5 / 1000, 9)
	})

	it('lets a stray component sit there without breaking the rest', async () => {
		// Half-placed components are normal while building, and one of them
		// should not stop the circuit that is finished from working.
		const { circuit, ids } = await milestone()
		const stray = circuit.add('Resistor', 0, 300)
		const strayNet = circuit.add('Resistor', 80, 300)
		circuit.connect(stray, 'a', strayNet, 'b')

		circuit.advance(0.016)

		expect(circuit.state().failure).toBeUndefined()
		expect(currentOf(circuit, ids.led)).toBeGreaterThan(0)
	})
})
