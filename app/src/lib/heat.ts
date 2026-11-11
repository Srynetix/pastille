// SPDX-License-Identifier: MIT OR Apache-2.0

// How hot a part runs, in words and in colour (C-74).
//
// The core carries each part's temperature and says what it is rated for; what
// that looks like on the canvas and how it is said in the inspector are the
// app's (ADR 0008).

import type { ElementHeat } from './core'
import { formatQuantity } from './units'

/**
 * How far a part has come from the room towards the hottest it may run: 0 at
 * the room's temperature, 1 at its limit, more past it.
 */
export function warmth(heat: ElementHeat, room: number): number {
	const span = heat.maxCelsius - room
	if (span <= 0) return 1
	return Math.max(0, (heat.celsius - room) / span)
}

/** Below this much of the way to its limit, a part is drawn as it always was. */
const UNNOTICED = 0.05

/**
 * The colours a part passes through as it warms: amber half way to its limit,
 * red at it. Fixed rather than themed, like an LED's light: hot looks the same
 * on paper of either colour.
 */
export const HEAT_RAMP = { warm: '#f59e0b', hot: '#ef4444' } as const

/**
 * The ink a part is drawn in at this warmth, or nothing for a part too close
 * to the room to be worth colouring — a circuit whose every resistor had
 * turned faintly orange would say nothing about any of them.
 */
export function heatInk(outline: string, amount: number): string | undefined {
	if (!(amount >= UNNOTICED)) return undefined
	if (amount >= 1) return HEAT_RAMP.hot
	return amount < 0.5
		? mix(outline, HEAT_RAMP.warm, amount / 0.5)
		: mix(HEAT_RAMP.warm, HEAT_RAMP.hot, (amount - 0.5) / 0.5)
}

/** "368 mW, about 104 °C": what it is dissipating, and where that has got it. */
export function heatReading(heat: ElementHeat): string {
	return `${formatQuantity(heat.watts, 'W')}, about ${Math.round(heat.celsius)} °C`
}

/**
 * What a beginner should be told about a part past its limits, or nothing for
 * one inside them. Temperature first: a part past its hottest is the more
 * urgent news, and it is also what a power past its rating leads to.
 */
export function heatWarning(heat: ElementHeat): string | undefined {
	if (heat.celsius > heat.maxCelsius) {
		return `Hotter than the ${Math.round(heat.maxCelsius)} °C it is made to survive.`
	}
	if (heat.ratedWatts != null && heat.watts > heat.ratedWatts) {
		// The rating as it is printed on the part and in the catalogue —
		// "0.25 W" — rather than as a measurement.
		return `More than the ${heat.ratedWatts} W it is rated for.`
	}
	return undefined
}

/**
 * Where a part is heading, when it is still on its way: "Heading for about
 * 150 °C". Nothing once it is within a degree of there.
 */
export function heatHeading(heat: ElementHeat): string | undefined {
	if (Math.abs(heat.settlesAt - heat.celsius) < 1) return undefined
	return `${heat.settlesAt > heat.celsius ? 'Heading for' : 'Cooling towards'} about ${Math.round(heat.settlesAt)} °C.`
}

/** Two `#rrggbb` colours `t` of the way from one to the other. */
function mix(from: string, to: string, t: number): string {
	const a = rgb(from)
	const b = rgb(to)
	if (!a || !b) return t < 0.5 ? from : to
	const channel = (index: number) =>
		Math.round(a[index] + (b[index] - a[index]) * t)
			.toString(16)
			.padStart(2, '0')
	return `#${channel(0)}${channel(1)}${channel(2)}`
}

function rgb(colour: string): [number, number, number] | undefined {
	const match = /^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i.exec(colour.trim())
	if (!match) return undefined
	return [parseInt(match[1], 16), parseInt(match[2], 16), parseInt(match[3], 16)]
}
