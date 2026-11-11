// SPDX-License-Identifier: MIT OR Apache-2.0

// What one component is doing right now, from what the solver said about the
// whole circuit.
//
// The core reports potentials per net and current per element, because that is
// what the solve produces. "1.5 V across this resistor" is a different question
// — it needs to know which nets the element's two pins landed on, and where
// pins are is the app's business (ADR 0006). So the arithmetic lives here, one
// step away from the boundary, rather than in the component that draws it.

import type { ElementView, NetView, SolutionView } from './core'
import { SYMBOLS } from './symbols'
import { formatQuantity } from './units'
import { AS_MEASURED, type Settle } from './steady'

/** What can be said about a component at this instant. */
export interface Reading {
	/**
	 * Volts across it, absent unless both its pins are on a net with a known
	 * potential.
	 *
	 * A magnitude. Which way round it sits is a question the symbol, the dots
	 * and the findings answer between them; a minus sign here would be a fourth
	 * answer in the units panel, where it reads as an error.
	 */
	readonly volts?: number
	/** Amps through it, as a magnitude, absent when the solution reports none. */
	readonly amps?: number
}

/**
 * What the solution says about one element.
 *
 * Empty when there is nothing true to say — no solution yet, a circuit that did
 * not solve, an element with a pin still dangling. Silence is the right answer
 * to a question the app cannot answer; a zero would be a claim.
 */
export function readingFor(
	element: ElementView,
	nets: readonly NetView[],
	solution: SolutionView | undefined,
): Reading {
	if (!solution || solution.failure) return {}

	const current = solution.elementCurrents.find((entry) => entry.elementId === element.id)
	const amps = current ? Math.abs(current.amps) : undefined

	// Across what? A one-pin element has no "across", and a three-pin one has
	// three of them — neither is a single number, so neither gets one.
	const [first, second] = potentialsAt(element, nets, solution)
	if (first === undefined || second === undefined) return { amps }

	return { volts: Math.abs(first - second), amps }
}

/**
 * The potential at each pin of a two-pin element, in pin order.
 *
 * Empty for anything with a different number of pins, and `undefined` in place
 * of a pin whose net the solution says nothing about — a dangling one, or one
 * on a net that is not a node.
 */
function potentialsAt(
	element: ElementView,
	nets: readonly NetView[],
	solution: SolutionView,
): (number | undefined)[] {
	const symbol = SYMBOLS[element.kind]
	if (!symbol || symbol.pins.length !== 2) return []

	return symbol.pins.map((pin) => {
		const net = nets.find((candidate) =>
			candidate.pins.some((end) => end.elementId === element.id && end.pin === pin.name),
		)
		return net ? solution.netVoltages.find((voltage) => voltage.netId === net.id)?.volts : undefined
	})
}

/** The kinds that are instruments, and read rather than do (C-21). */
const PROBES = ['Voltmeter', 'Ammeter']

/**
 * What a probe reads, written the way a meter writes it — or nothing at all
 * for a component that is not one.
 *
 * Signed, unlike the hover readout above, and that is the point of placing a
 * meter rather than pointing at something: a voltmeter on backwards reads −9 V,
 * and an ammeter reads negative when the current runs in at its `-`. A
 * magnitude would throw away the half of the answer a beginner is least sure
 * of.
 *
 * A meter with nothing to say shows a dash and its unit, which is what a real
 * one does with its leads in the air. Silence would leave a bare circle on the
 * canvas with no clue what it is for.
 */
export function probeLabel(
	element: ElementView,
	nets: readonly NetView[],
	solution: SolutionView | undefined,
	/** What holds the number still long enough to read (`steady.ts`). */
	settle: Settle = AS_MEASURED,
): string | undefined {
	if (!PROBES.includes(element.kind)) return undefined

	const reading = settle(`${element.id}:probe`, probeReading(element, nets, solution))
	return reading === undefined ? `— ${element.unit}` : formatQuantity(reading, element.unit)
}

/** The twelve notes of an octave, from C, as a keyboard names them. */
const NOTES = ['C', 'C♯', 'D', 'D♯', 'E', 'F', 'F♯', 'G', 'G♯', 'A', 'A♯', 'B']

/**
 * The nearest note to a frequency, with its octave: 440 Hz is A4, 262 Hz C4.
 *
 * Equal temperament from A4 at 440 Hz, which is what an electronic tuner
 * assumes. Nearest, so a buzzer a few hertz flat still reads as the note it
 * is trying to be; the hertz beside it say how far off it is.
 */
export function noteName(hertz: number): string {
	const fromA4 = Math.round(12 * Math.log2(hertz / 440))
	const fromC0 = fromA4 + 9 + 4 * 12
	return `${NOTES[((fromC0 % 12) + 12) % 12]}${Math.floor(fromC0 / 12)}`
}

/**
 * What a passive buzzer is playing, written beside it as a meter's reading is
 * (C-85): "440 Hz · A4". The note is the run's, so like a reading it moves
 * with the circuit and belongs to the solution. Nothing while it is silent,
 * and for any other part.
 */
export function toneLabel(
	element: ElementView,
	solution: SolutionView | undefined,
	settle: Settle = AS_MEASURED,
): string | undefined {
	if (element.kind !== 'PassiveBuzzer' || !solution || solution.failure) return undefined
	const hertz = settle(
		`${element.id}:tone`,
		solution.tones.find((entry) => entry.elementId === element.id)?.hertz,
	)
	return hertz === undefined ? undefined : `${Math.round(hertz)} Hz · ${noteName(hertz)}`
}

/**
 * Where a servo's horn points, written beside it as a reading is (C-90):
 * "90°". The run's, held still long enough to read like every other number.
 */
export function angleLabel(
	element: ElementView,
	solution: SolutionView | undefined,
	settle: Settle = AS_MEASURED,
): string | undefined {
	if (element.kind !== 'Servo' || !solution || solution.failure) return undefined
	const degrees = settle(
		`${element.id}:angle`,
		solution.angles.find((entry) => entry.elementId === element.id)?.degrees,
	)
	return degrees === undefined ? undefined : `${Math.round(degrees)}°`
}

/**
 * How long a real battery has left, written beside it (C-121): "9 V · 62 %,
 * 18 h left". Hours at the current it is giving now, so it changes as the
 * load does — which is the point of it, since real time is far too slow to
 * watch one empty.
 */
export function batteryLabel(
	element: ElementView,
	solution: SolutionView | undefined,
	settle: Settle = AS_MEASURED,
): string | undefined {
	if (element.kind !== 'Cell' || !solution || solution.failure) return undefined
	const entry = solution.batteries.find((battery) => battery.elementId === element.id)
	if (!entry) return undefined
	const left = `${element.label} · ${Math.round(entry.left * 100)} %`
	const hours = settle(`${element.id}:hours`, entry.hours ?? undefined)
	if (entry.left <= 0) return `${element.label} · flat`
	if (hours === undefined) return left
	return `${left}, ${lasting(hours)} left`
}

/** A length of time a person would say: minutes, hours, days or years. */
export function lasting(hours: number): string {
	if (hours < 1) return `${Math.max(1, Math.round(hours * 60))} min`
	if (hours < 48) return `${hours < 10 ? hours.toFixed(1) : Math.round(hours)} h`
	const days = hours / 24
	if (days < 730) return `${Math.round(days)} days`
	return `${Math.round(days / 365)} years`
}

function probeReading(
	element: ElementView,
	nets: readonly NetView[],
	solution: SolutionView | undefined,
): number | undefined {
	if (!solution || solution.failure) return undefined

	if (element.kind === 'Ammeter') {
		// The core reports the current into an element's first pin, which for a
		// meter is its `+` — the same convention the instrument is marked with.
		return solution.elementCurrents.find((entry) => entry.elementId === element.id)?.amps
	}

	const [plus, minus] = potentialsAt(element, nets, solution)
	return plus !== undefined && minus !== undefined ? plus - minus : undefined
}
