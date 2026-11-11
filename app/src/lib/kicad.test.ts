// SPDX-License-Identifier: MIT OR Apache-2.0

import { describe, expect, it } from 'vitest'

import { createCircuit } from './core'
import milestone from '../examples/milestone.json?raw'

describe('the circuit as a KiCad netlist (B-16)', () => {
	it('is a file KiCad’s PCB editor imports, each part with its footprint', async () => {
		const circuit = await createCircuit()
		circuit.load(milestone)
		const view = circuit.kicadNetlist()
		expect(view.unmapped).toEqual([])
		expect(view.netlist).toContain('(export (version "E")')
		expect(view.netlist).toContain('(footprint "LED_THT:LED_D5.0mm")')
		// And as a schematic its editor opens (B-21).
		expect(view.schematic).toContain('(kicad_sch (version 20231120)')
		expect(view.schematic).toContain('(lib_id "pastille:LED")')
	})

	it('writes a part added since v1, a chip by its package (B-22)', async () => {
		const circuit = await createCircuit()
		circuit.add('Gate', 0, 0)
		const view = circuit.kicadNetlist()
		expect(view.unmapped).toEqual([])
		expect(view.netlist).toContain('(libsource (lib "74xx") (part "74HC00")')
		expect(view.netlist).toContain('(footprint "Package_DIP:DIP-14_W7.62mm")')
	})
})
