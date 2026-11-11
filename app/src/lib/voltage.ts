// SPDX-License-Identifier: MIT OR Apache-2.0

// A wire's colour, from its voltage (C-134).
//
// The idea beginners find hardest is that voltage is *where you are* and
// current is *what moves*. The dots already show what moves; this shows where
// each wire is, on one scale from the circuit's lowest net to its highest, so
// the battery's + side and everything joined straight to it read alike, and
// the drop across a resistor is a change of colour along the loop.
//
// The scale is the circuit's own rather than fixed: a 3 V circuit and a 12 V one
// both use the whole of it, which is what lets a drop be seen at all. The key on
// the board says what its two ends are.
//
// Its three stops are theme tokens (`--volt-low`, `--volt-mid`, `--volt-high`),
// which change in lightness as well as in hue, so the scale survives being
// seen without colour.

import type { SolutionView } from './core'
import type { SceneWire } from './scene'

/** The circuit's lowest and highest net, which are the ends of the scale. */
export interface VoltageScale {
	readonly low: number
	readonly high: number
}

/**
 * The scale for a solution: from its lowest net to its highest, and never
 * narrower than a volt, so a circuit sitting at 0 V is drawn at the bottom of
 * the scale rather than wherever rounding puts it. Nothing for a solution that
 * failed, or has no nets: no colour is better than a colour nobody checked.
 */
export function scaleOf(solution: SolutionView | undefined): VoltageScale | undefined {
	if (!solution || solution.failure || solution.netVoltages.length === 0) return undefined
	const volts = solution.netVoltages.map((net) => net.volts).filter(Number.isFinite)
	if (volts.length === 0) return undefined
	const low = Math.min(0, ...volts)
	const high = Math.max(low + 1, ...volts)
	return { low, high }
}

/** `#rrggbb` as three channels, or nothing for anything else. */
function channels(hex: string): [number, number, number] | undefined {
	const match = /^#([0-9a-f]{6})$/i.exec(hex.trim())
	if (!match) return undefined
	const value = parseInt(match[1], 16)
	return [value >> 16, (value >> 8) & 255, value & 255]
}

function mix(from: string, to: string, t: number): string {
	const a = channels(from)
	const b = channels(to)
	if (!a || !b) return t < 0.5 ? from : to
	const channel = (index: number) =>
		Math.round(a[index] + (b[index] - a[index]) * t)
			.toString(16)
			.padStart(2, '0')
	return `#${channel(0)}${channel(1)}${channel(2)}`
}

/** The colour for a voltage on a scale, through its three stops. */
export function inkFor(
	volts: number,
	scale: VoltageScale,
	stops: readonly [string, string, string],
): string {
	const t = Math.min(1, Math.max(0, (volts - scale.low) / (scale.high - scale.low)))
	return t < 0.5 ? mix(stops[0], stops[1], t / 0.5) : mix(stops[1], stops[2], (t - 0.5) / 0.5)
}

/**
 * The wires, each given its net's colour. A wire whose net the solution says
 * nothing about — dangling, or not a node — keeps the plain wire colour.
 */
export function paintVoltages(
	wires: readonly SceneWire[],
	solution: SolutionView | undefined,
	stops: readonly [string, string, string],
): SceneWire[] {
	const scale = scaleOf(solution)
	if (!scale || !solution) return [...wires]
	const byNet = new Map(solution.netVoltages.map((net) => [net.netId, net.volts]))
	return wires.map((wire) => {
		const volts = byNet.get(wire.id)
		return volts === undefined || !Number.isFinite(volts)
			? wire
			: { ...wire, ink: inkFor(volts, scale, stops) }
	})
}
