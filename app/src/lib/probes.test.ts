// SPDX-License-Identifier: MIT OR Apache-2.0

// The voltmeter and the ammeter, from the app's side of the boundary (C-21).
//
// What the core owns is that an ideal meter changes nothing; what the app owns
// is the number written beside it. These run against the real bundle, because
// the reading is the two halves put together and testing either alone would
// prove the wrong thing.

import { describe, expect, it } from 'vitest'

import { createCircuit, type Circuit } from './core'
import { probeLabel } from './readings'
import { toScene, toWires } from './scene'
import { SYMBOLS } from './symbols'

const TOP = 1_000
const BOTTOM = 2_000

/** 9 V across 1 kΩ and 2 kΩ: the tap sits at 6 V and 3 mA flows. */
async function divider() {
	const circuit = await createCircuit()
	const battery = circuit.add('DcSource', 0, 0)
	const upper = circuit.add('Resistor', 80, 0)
	const lower = circuit.add('Resistor', 160, 0)
	circuit.setValue(upper, String(TOP))
	circuit.setValue(lower, String(BOTTOM))
	circuit.connect(battery, '+', upper, 'a')
	circuit.connect(upper, 'b', lower, 'a')
	circuit.connect(lower, 'b', battery, '-')
	return { circuit, battery, upper, lower }
}

function labelOf(circuit: Circuit, id: string): string | undefined {
	const element = circuit.elements().find((candidate) => candidate.id === id)
	if (!element) throw new Error(`no element ${id}`)
	return probeLabel(element, circuit.nets(), circuit.state())
}

describe('a voltmeter', () => {
	it('reads the drop across what it is put over', async () => {
		const { circuit, lower } = await divider()
		const meter = circuit.add('Voltmeter', 160, 80)
		circuit.connect(meter, '+', lower, 'a')
		circuit.connect(meter, '-', lower, 'b')

		circuit.advance(0.016)

		expect(labelOf(circuit, meter)).toBe('6.00 V')
	})

	it('reads a negative number when it is on backwards', async () => {
		// The half of the answer a beginner is least sure of, and the reason the
		// reading is signed where the hover readout is a magnitude.
		const { circuit, lower } = await divider()
		const meter = circuit.add('Voltmeter', 160, 80)
		circuit.connect(meter, '+', lower, 'b')
		circuit.connect(meter, '-', lower, 'a')

		circuit.advance(0.016)

		expect(labelOf(circuit, meter)).toBe('-6.00 V')
	})

	it('does not change the circuit it is put across', async () => {
		// The property that makes it an instrument. Not "nearly the same" —
		// the same, because an ideal voltmeter is never stamped at all.
		const plain = await divider()
		plain.circuit.advance(0.016)
		const before = plain.circuit.state().netVoltages.map((entry) => entry.volts)

		const { circuit, lower } = await divider()
		const meter = circuit.add('Voltmeter', 160, 80)
		circuit.connect(meter, '+', lower, 'a')
		circuit.connect(meter, '-', lower, 'b')
		circuit.advance(0.016)

		expect(circuit.state().netVoltages.map((entry) => entry.volts)).toEqual(before)
	})

	it('shows a dash rather than a number when it is not across anything', async () => {
		// What a real one does with its leads in the air. A bare circle with
		// nothing beside it would not even say what it was for.
		const { circuit } = await divider()
		const meter = circuit.add('Voltmeter', 160, 80)

		circuit.advance(0.016)

		expect(labelOf(circuit, meter)).toBe('— V')
	})
})

describe('an ammeter', () => {
	/** The same divider with a meter in line between the two resistors. */
	async function measured() {
		const { circuit, upper, lower } = await divider()
		const meter = circuit.add('Ammeter', 120, 0)
		// Cut the wire it goes into: a meter in line replaces a piece of wire.
		const middle = circuit
			.nets()
			.find((net) => net.pins.some((pin) => pin.elementId === upper && pin.pin === 'b'))
		circuit.removeNet(middle!.id)
		circuit.connect(upper, 'b', meter, '+')
		circuit.connect(meter, '-', lower, 'a')
		return { circuit, meter }
	}

	it('reads the current running through it', async () => {
		const { circuit, meter } = await measured()

		circuit.advance(0.016)

		expect(labelOf(circuit, meter)).toBe('3.00 mA')
	})

	it('does not get in the way of the current it reads', async () => {
		// Ideal means no resistance, so the loop carries what it carried before
		// the meter was put in it.
		const plain = await divider()
		plain.circuit.advance(0.016)
		const before = plain.circuit
			.state()
			.elementCurrents.find((entry) => entry.elementId === plain.upper)!.amps

		const { circuit, meter } = await measured()
		circuit.advance(0.016)
		const reading = circuit.state().elementCurrents.find((entry) => entry.elementId === meter)!.amps

		expect(Math.abs(reading)).toBeCloseTo(Math.abs(before), 12)
	})
})

describe('a probe on the canvas', () => {
	it('is labelled with what it reads, and everything else with what it is', async () => {
		const { circuit, lower } = await divider()
		const meter = circuit.add('Voltmeter', 160, 80)
		circuit.connect(meter, '+', lower, 'a')
		circuit.connect(meter, '-', lower, 'b')
		circuit.advance(0.016)

		const scene = toScene(circuit.elements(), circuit.state(), circuit.nets())

		expect(scene.find((item) => item.id === meter)?.label).toBe('6.00 V')
		expect(scene.find((item) => item.id === lower)?.label).toBe('2 kΩ')
	})

	it('never runs the dots into the meter itself', async () => {
		// The stub down to the meter is drawn, because you have to see what it
		// is measuring — and it is not animated, because nothing goes down it.
		// A meter that looked like it drew current would teach the opposite of
		// the one thing it is for.
		const { circuit, lower } = await divider()
		const meter = circuit.add('Voltmeter', 160, 80)
		circuit.connect(meter, '+', lower, 'a')
		circuit.connect(meter, '-', lower, 'b')
		circuit.advance(0.016)

		const scene = toScene(circuit.elements(), circuit.state(), circuit.nets())
		const wires = toWires(circuit.nets(), scene, circuit.state().pinCurrents)

		const dial = scene.find((item) => item.id === meter)!
		const terminals = SYMBOLS.Voltmeter.pins.map((pin) => ({
			x: dial.position.x + pin.at.x,
			y: dial.position.y + pin.at.y,
		}))
		const animated = wires.flatMap((wire) => (wire.flows ?? []).flatMap((flow) => flow.segments))
		expect(animated.length).toBeGreaterThan(0)
		for (const segment of animated) {
			for (const end of [segment.from, segment.to]) {
				expect(terminals).not.toContainEqual(end)
			}
		}

		// The drawing still goes there, though: both stubs are wire.
		const drawn = wires.flatMap((wire) => wire.segments)
		for (const terminal of terminals) {
			expect(
				drawn.some((segment) => segment.to.x === terminal.x && segment.to.y === terminal.y),
			).toBe(true)
		}
	})

	it('keeps the current dots running on the wire it is tapped onto', async () => {
		// Three pins meet on that wire once the meter is on it, and a wire
		// joining three carries no single current — because the current
		// divides. It does not divide at a voltmeter, and the arrow stays.
		const { circuit, lower } = await divider()
		circuit.advance(0.016)
		const before = circuit.state().netCurrents.length

		const meter = circuit.add('Voltmeter', 160, 80)
		circuit.connect(meter, '+', lower, 'a')
		circuit.connect(meter, '-', lower, 'b')
		circuit.advance(0.016)

		expect(circuit.state().netCurrents).toHaveLength(before)
	})
})
