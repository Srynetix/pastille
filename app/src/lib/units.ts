// SPDX-License-Identifier: MIT OR Apache-2.0

// Numbers, written the way a multimeter writes them.
//
// A circuit spans twelve orders of magnitude — nine volts and twenty-seven
// thousandths of an amp in the same breath — and "0.0273 A" is a number a
// beginner has to decode before it means anything. "27.3 mA" is the same fact
// already read out.
//
// The core formats the values it owns (an element's label, "220 Ω"); this
// formats the ones the app computes from a solution, which the core has no
// opinion about.

/** SI prefixes, largest first. */
const PREFIXES: readonly { readonly exponent: number; readonly symbol: string }[] = [
	{ exponent: 9, symbol: 'G' },
	{ exponent: 6, symbol: 'M' },
	{ exponent: 3, symbol: 'k' },
	{ exponent: 0, symbol: '' },
	{ exponent: -3, symbol: 'm' },
	{ exponent: -6, symbol: 'µ' },
	{ exponent: -9, symbol: 'n' },
	{ exponent: -12, symbol: 'p' },
]

/** Below this, a reading is noise from the solver rather than a quantity. */
const NEGLIGIBLE = 1e-12

/**
 * A value with its unit, in engineering notation and three significant figures.
 *
 * Three because that is what the instrument this imitates shows, and because a
 * fourth digit of a simulation this approximate would be a lie (design doc §3:
 * educational, not accurate enough to design from).
 */
export function formatQuantity(value: number, unit: string): string {
	if (!Number.isFinite(value)) return `— ${unit}`

	const magnitude = Math.abs(value)
	if (magnitude < NEGLIGIBLE) return `0 ${unit}`

	let chosen = PREFIXES[PREFIXES.length - 1]
	for (const prefix of PREFIXES) {
		if (magnitude >= 10 ** prefix.exponent) {
			chosen = prefix
			break
		}
	}

	let scaled = value / 10 ** chosen.exponent
	// Rounding can push a value over the top of its own band — 999.7 mA is
	// 1.00 A once it is down to three figures, and "1000 mA" is the reading
	// nobody writes. Promote it rather than print it.
	const larger = PREFIXES.find((prefix) => prefix.exponent === chosen.exponent + 3)
	if (Math.abs(scaled) >= 999.5 && larger) {
		chosen = larger
		scaled = value / 10 ** chosen.exponent
	}

	return `${scaled.toPrecision(3)} ${chosen.symbol}${unit}`
}

/**
 * A value as it is typed — "27k", "470", "4.7k" — said with its unit: "27 kΩ".
 *
 * For a fix's button (C-135, B-04): the core hands over the value the way a
 * field takes it, so pressing the button and typing it are the same edit, and
 * this is only how the button reads it out.
 */
export function spokenValue(typed: string, unit: string): string {
	const match = /^([\d.]+)\s*([GMkmuµnp]?)$/.exec(typed.trim())
	if (!match) return `${typed} ${unit}`.trim()
	const prefix = match[2] === 'u' ? 'µ' : match[2]
	return `${match[1]} ${prefix}${unit}`.trim()
}
