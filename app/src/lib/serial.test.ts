// SPDX-License-Identifier: MIT OR Apache-2.0

import { describe, expect, it } from 'vitest'

import { MOST_KEPT, SerialLog, baudsMatch } from './serial.svelte'

describe('the serial monitor’s transcript (A-06)', () => {
	it('decodes UTF-8 across the frames it arrives in', () => {
		// "°" is two bytes; the core may hand them over a frame apart.
		const log = new SerialLog()
		log.add('uno', new Uint8Array([0x32, 0x31, 0x20, 0xc2]))
		log.add('uno', new Uint8Array([0xb0, 0x43, 0x0d, 0x0a]))
		expect(log.text.uno).toBe('21 °C\r\n')
	})

	it('shows bytes that are not text as the replacement character', () => {
		const log = new SerialLog()
		log.add('uno', new Uint8Array([0xff, 0x41]))
		expect(log.text.uno).toBe('�A')
	})

	it('keeps the latest when it grows past what it keeps', () => {
		const log = new SerialLog()
		log.add('uno', new TextEncoder().encode('x'.repeat(MOST_KEPT) + 'end'))
		expect(log.text.uno.length).toBe(MOST_KEPT)
		expect(log.text.uno.endsWith('end')).toBe(true)
		log.clear('uno')
		expect(log.text.uno).toBe('')
	})

	it('reads the Arduino’s 9615 and 117647 as 9600 and 115200, not 9600 as 115200', () => {
		expect(baudsMatch(9615, 9600)).toBe(true)
		expect(baudsMatch(117647, 115200)).toBe(true)
		expect(baudsMatch(117647, 9600)).toBe(false)
	})
})
