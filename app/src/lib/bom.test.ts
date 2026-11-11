// SPDX-License-Identifier: MIT OR Apache-2.0

import { describe, expect, it } from 'vitest'

import { billToCsv, billToMarkdown } from './bom'
import { createCircuit, type BillLine } from './core'
import milestone from '../examples/milestone.json?raw'

const LINES: BillLine[] = [
	{
		kind: 'Resistor',
		value: '220 Ω',
		part: 'Yageo CFR-25',
		datasheet: 'https://example.org/cfr25.pdf',
		quantity: 2,
		elements: ['1', '2'],
		notes: ['Set to 217 Ω; 220 Ω is the nearest E12, 1.4 % more.'],
	},
	{
		kind: 'Led',
		value: 'Red',
		part: 'WP7113ID',
		datasheet: undefined,
		quantity: 1,
		elements: ['3'],
		notes: ['This red LED carries 33 mA; it is rated for 20 mA and will burn out.'],
	},
]

describe('the parts list as a spreadsheet (B-11)', () => {
	it('has a row a line, named as the palette names it, and quotes what needs it', () => {
		const rows = billToCsv(LINES).split('\r\n')
		expect(rows[0]).toBe('Quantity,Part,Value,Part number,Datasheet,Notes')
		// The resistor's note has a comma in it, so it is quoted; the LED's has
		// none, and is not.
		expect(rows[1]).toBe(
			'2,Resistor,220 Ω,Yageo CFR-25,https://example.org/cfr25.pdf,"Set to 217 Ω; 220 Ω is the nearest E12, 1.4 % more."',
		)
		expect(rows[2]).toBe(
			'1,LED,Red,WP7113ID,,This red LED carries 33 mA; it is rated for 20 mA and will burn out.',
		)
	})
})

describe('the parts list as a page (B-11)', () => {
	it('is a table, with each line’s notes as sentences under it', () => {
		const page = billToMarkdown(LINES, 'A lit LED')
		expect(page).toContain('# A lit LED')
		expect(page).toContain(
			'| 2 | Resistor | 220 Ω | [Yageo CFR-25](https://example.org/cfr25.pdf) |',
		)
		expect(page).toContain('| 1 | LED | Red | WP7113ID |')
		expect(page).toContain('**LED, Red**')
		expect(page).toContain('- This red LED carries 33 mA')
	})

	it('says so when there is nothing to buy', () => {
		expect(billToMarkdown([], '')).toContain('Nothing to buy yet.')
	})
})

describe('the parts list of a real circuit', () => {
	it('is the milestone: a battery, a resistor and an LED, by part number', async () => {
		const circuit = await createCircuit()
		circuit.load(milestone)
		circuit.advance(1 / 60)
		const rows = billToCsv(circuit.bill()).split('\r\n')
		expect(rows.slice(1, 4).map((row) => row.split(',').slice(0, 3).join(','))).toEqual([
			'1,Battery,9 V',
			'1,Resistor,470 Ω',
			'1,LED,Red',
		])
		expect(rows[3]).toContain('WP7113ID')
	})
})
