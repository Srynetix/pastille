// SPDX-License-Identifier: MIT OR Apache-2.0

// What each Uno's serial monitor has shown (A-06).
//
// The core hears the program's TX line at the monitor's baud and hands over
// only the bytes that are new (`takeSerial`); the transcript is kept here,
// decoded as UTF-8 the way the Arduino IDE's monitor does — so a mismatched
// baud shows as the replacement characters and stray letters it shows there,
// and "°C" arrives whole even when its two bytes come in different frames.

import type { Circuit, SolutionView } from './core'

/** The rates the Arduino IDE's monitor offers. */
export const BAUDS = [
	300, 1200, 2400, 4800, 9600, 19200, 38400, 57600, 74880, 115200, 230400, 250000,
] as const

/** What is added to a line typed and sent, as the IDE's monitor offers it. */
export const LINE_ENDINGS = [
	{ label: 'Newline', ending: '\n' },
	{ label: 'No line ending', ending: '' },
	{ label: 'Carriage return', ending: '\r' },
	{ label: 'Both NL & CR', ending: '\r\n' },
] as const

/**
 * Characters kept per monitor: enough to scroll back through, not so many a
 * program printing in a loop all afternoon slows the page. The oldest go.
 */
export const MOST_KEPT = 20_000

/**
 * Whether two bauds are close enough for a UART to read one at the other:
 * within the few percent the sampling in the middle of a bit allows. The
 * Arduino's 9600 is really 9615, and it reads cleanly.
 */
export function baudsMatch(program: number, monitor: number): boolean {
	return Math.abs(program - monitor) / monitor < 0.03
}

export class SerialLog {
	/** Each Uno's transcript, by handle. */
	text = $state<Record<string, string>>({})
	// Private state between drains; only `text` is shown.
	// eslint-disable-next-line svelte/prefer-svelte-reactivity
	#decoders = new Map<string, TextDecoder>()

	/** Takes what each running program's monitor has heard since last time. */
	drain(circuit: Circuit, solution: SolutionView | undefined): void {
		for (const port of solution?.serial ?? []) {
			const bytes = circuit.takeSerial(port.elementId)
			if (bytes.length === 0) continue
			this.add(port.elementId, bytes)
		}
	}

	/** Adds bytes heard on `id`'s monitor to its transcript. */
	add(id: string, bytes: Uint8Array): void {
		let decoder = this.#decoders.get(id)
		if (!decoder) {
			decoder = new TextDecoder('utf-8')
			this.#decoders.set(id, decoder)
		}
		const more = decoder.decode(bytes, { stream: true })
		if (more === '') return
		const text = (this.text[id] ?? '') + more
		this.text[id] = text.length > MOST_KEPT ? text.slice(text.length - MOST_KEPT) : text
	}

	/** Empties a monitor, as its Clear button does. */
	clear(id: string): void {
		this.text[id] = ''
		this.#decoders.delete(id)
	}

	/** Forgets every monitor: the circuit was cleared, or another opened. */
	forget(): void {
		this.text = {}
		this.#decoders.clear()
	}
}
