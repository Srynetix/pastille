// SPDX-License-Identifier: MIT OR Apache-2.0

// C-47's regression net.
//
// Every example is a real saved document exercising the file format, the
// netlist, the solver and the drawing at once, so loading each one and running
// a frame is close to an end-to-end suite for nothing. It also stops an example
// rotting quietly: the day a format or a default changes under them, this is
// what says so rather than a beginner clicking one and finding it broken.

import { describe, expect, it } from 'vitest'

import { createCircuit, viewPlot, type Circuit, type ElementHeat } from './core'
import { EXAMPLE_GROUPS, EXAMPLES } from './examples'
import { readingFor } from './readings'
import { mixedLight } from './render'
import { boundsOf, toScene } from './scene'

describe('every circuit that ships with the app', () => {
	it.each(EXAMPLES.map((example) => [example.id, example] as const))(
		'%s opens, solves and draws',
		async (_id, example) => {
			const circuit = await createCircuit()
			circuit.load(example.document)

			// A frame, because a circuit that loads and then fails to solve is
			// worse than one that will not load: it opens onto a red line in
			// the status bar with the user's own canvas already thrown away.
			circuit.advance(1 / 60)

			const state = circuit.state()
			expect(state.failure, `${_id} solves`).toBeUndefined()
			expect(circuit.elements().length, `${_id} has components`).toBeGreaterThan(1)
			expect(circuit.nets().length, `${_id} is wired`).toBeGreaterThan(0)

			// And it is drawable: every kind in it has a symbol, which is what
			// `toScene` would throw over if one were missing.
			const scene = toScene(circuit.elements(), state, circuit.nets())
			expect(scene).toHaveLength(circuit.elements().length)
			expect(boundsOf(scene), `${_id} has an extent`).toBeDefined()
		},
	)

	it('carries a reason to open each one', () => {
		// A circuit with nothing said about it teaches nothing (design doc §14):
		// the line is what somebody reads *before* clicking, so it has to say
		// what there is to look at rather than name the parts.
		for (const example of EXAMPLES) {
			expect(example.title.length, example.id).toBeLessThan(24)
			expect(example.look.length, example.id).toBeGreaterThan(20)
			expect(example.look.length, example.id).toBeLessThan(110)
		}
	})

	it('names each one once', () => {
		const ids = EXAMPLES.map((example) => example.id)

		expect(new Set(ids).size).toBe(ids.length)
	})

	it('stays inside what v1 can do', async () => {
		// An example using something the app does not offer is a list of things
		// that do not work yet. The registry is the list of what exists, and
		// what is offered is the list of what a person could have built
		// themselves — which is what an example is supposed to be.
		const { componentRegistry } = await import('./core')
		const offered = new Set(
			(await componentRegistry()).filter((entry) => entry.offered).map((entry) => entry.kind),
		)

		for (const example of EXAMPLES) {
			const circuit = await createCircuit()
			circuit.load(example.document)

			for (const element of circuit.elements()) {
				expect(offered.has(element.kind), `${example.id} uses ${element.kind}`).toBe(true)
			}
		}
	})
})

/**
 * A loaded example, run for a while, and read the way its probes read it.
 *
 * Elements are named by where they sit in the document, which is the order
 * `elements()` returns them in.
 */
class Run {
	constructor(private readonly circuit: Circuit) {}

	/** Advance by `seconds` of simulated time, a frame at a time. */
	for(seconds: number): this {
		for (let frame = 0; frame < Math.round(seconds * 60); frame++) this.circuit.advance(1 / 60)
		return this
	}

	/** What `read` says of the run as it is now, for a claim to chain on. */
	and<T>(read: () => T): T {
		return read()
	}

	/** Advance by `seconds` in one go — finer than a frame, to catch a 50 Hz ripple. */
	step(seconds: number): this {
		this.circuit.advance(seconds)
		return this
	}

	/** Put a new fuse in place of a blown one, as the Inspector does (C-120). */
	replace(index: number): this {
		this.circuit.replaceFuse(this.circuit.elements()[index].id)
		return this
	}

	/** Hours a battery has left at what it gives now, NaN while it gives nothing (C-121). */
	hours(index: number): number {
		const id = this.circuit.elements()[index].id
		return (
			this.circuit.state().batteries.find((entry) => entry.elementId === id)?.hours ?? Number.NaN
		)
	}

	/** Whether a fuse has blown. */
	blown(index: number): boolean {
		return this.circuit.elements()[index].blown === true
	}

	/** Shut or open a contact, as a click or a held button would. */
	press(index: number, closed: boolean): this {
		this.circuit.setClosed(this.circuit.elements()[index].id, closed)
		return this
	}

	/** What the app notes about the circuit, as opposed to warns. */
	notes(): string[] {
		return this.circuit
			.state()
			.findings.filter((finding) => finding.severity === 'note')
			.map((finding) => finding.message)
	}

	/** Type a value into an element, as the Inspector does. */
	set(index: number, text: string): this {
		this.circuit.setValue(this.circuit.elements()[index].id, text)
		return this
	}

	/**
	 * Delete an element. Elements after it move down one, so delete the last
	 * of those a claim still reads.
	 */
	remove(index: number): this {
		this.circuit.remove(this.circuit.elements()[index].id)
		return this
	}

	/** What an Uno's serial monitor has heard since this was last asked (A-06). */
	printed(index: number): string {
		return new TextDecoder().decode(this.circuit.takeSerial(this.circuit.elements()[index].id))
	}

	/** Type a line into an Uno's serial monitor, ending and all. */
	type(index: number, text: string): this {
		this.circuit.sendSerial(this.circuit.elements()[index].id, text)
		return this
	}

	/** Set an Uno's serial monitor to another baud. */
	baud(index: number, baud: number): this {
		this.circuit.setMonitorBaud(this.circuit.elements()[index].id, baud)
		return this
	}

	/** Turn an element's dial — a wiper, the light, the temperature. */
	dial(index: number, position: number): this {
		this.circuit.setWiper(this.circuit.elements()[index].id, position)
		return this
	}

	/** Whether an element is doing what it is for: pulled in, sounding, turning. */
	active(index: number): boolean {
		return this.circuit.state().energised.includes(this.circuit.elements()[index].id)
	}

	/**
	 * The highest a probe read over the last `seconds`, from every step the
	 * run took rather than only the one each frame ended on — which is what
	 * its trace on the scope shows.
	 */
	highest(index: number, seconds: number): number {
		const written = this.circuit.plotTrace(this.circuit.elements()[index].id, seconds, 1)
		return viewPlot(this.circuit, written)[1]
	}

	/** Which of a digit's segments are lit, by letter: "abc" for a seven. */
	lit(index: number): string {
		const id = this.circuit.elements()[index].id
		const digit = this.circuit.state().digits.find((entry) => entry.elementId === id)
		const letters = ['a', 'b', 'c', 'd', 'e', 'f', 'g', '.']
		return (digit?.segments ?? [])
			.map((brightness, segment) => (brightness > 0.1 ? letters[segment] : ''))
			.join('')
	}

	/** How many of an LED bar's ten are lit (C-119). */
	bars(index: number): number {
		const id = this.circuit.elements()[index].id
		const bar = this.circuit.state().digits.find((entry) => entry.elementId === id)
		return (bar?.segments ?? []).filter((brightness) => brightness > 0.1).length
	}

	/** How brightly each of an RGB LED's dice is lit, red, green, blue (C-84). */
	dice(index: number): readonly number[] {
		const id = this.circuit.elements()[index].id
		return this.circuit.state().digits.find((entry) => entry.elementId === id)?.segments ?? []
	}

	/** The note a passive buzzer is sounding, in hertz, or 0 while it is silent (C-85). */
	note(index: number): number {
		const id = this.circuit.elements()[index].id
		return this.circuit.state().tones.find((entry) => entry.elementId === id)?.hertz ?? 0
	}

	/** Whether a motor is turning backwards, `b` above `a` (C-91). */
	reversed(index: number): boolean {
		return this.circuit.state().reversed.includes(this.circuit.elements()[index].id)
	}

	/** Where a servo's horn points, in degrees (C-90). */
	angle(index: number): number {
		const id = this.circuit.elements()[index].id
		return (
			this.circuit.state().angles.find((entry) => entry.elementId === id)?.degrees ?? Number.NaN
		)
	}

	/** How dark each of an LCD digit's segments has turned, a to the point (C-76). */
	shade(index: number): readonly number[] {
		const id = this.circuit.elements()[index].id
		return this.circuit.state().digits.find((entry) => entry.elementId === id)?.segments ?? []
	}

	/** How hot an element is running (C-74). */
	heat(index: number): ElementHeat {
		const id = this.circuit.elements()[index].id
		const heat = this.circuit.state().heat.find((entry) => entry.elementId === id)
		expect(heat, 'it has a temperature').toBeDefined()
		return heat as ElementHeat
	}

	/** Volts across a two-pin element, as a meter on it would read them. */
	volts(index: number): number {
		return this.read(index).volts ?? Number.NaN
	}

	/** Amps through an element, however it is wired. */
	amps(index: number): number {
		return this.read(index).amps ?? Number.NaN
	}

	private read(index: number) {
		const element = this.circuit.elements()[index]
		const state = this.circuit.state()
		expect(state.failure, 'still solving').toBeUndefined()
		return readingFor(element, this.circuit.nets(), state)
	}
}

/** Lit, as far as anybody looking at it could tell. */
const LIT = 5e-3
/** Dark, likewise: a microamp through an LED is nothing to see. */
const DARK = 1e-5
/**
 * Lit by a CMOS output on 5 V: its sheet's 780 Ω behind it gives a red LED
 * through 100 Ω about 3.6 mA, drawn at nearly half brightness (C-124).
 */
const LIT_BY_CMOS = 2e-3

/**
 * Whether an LED is lit for each row of a two-input truth table — off off,
 * off on, on off, on on — with its two switches set and a tenth of a second
 * between rows (C-108).
 */
function table(run: Run, a: number, b: number, led: number): boolean[] {
	return [
		[false, false],
		[false, true],
		[true, false],
		[true, true],
	].map(([first, second]) => run.press(a, first).press(b, second).for(0.1).amps(led) > LIT)
}

/**
 * What each example's line says, checked against the circuit.
 *
 * One entry per example, and a test below requires it: an example whose
 * sentence is not true teaches the thing it says, which is worse than no
 * example at all (C-58).
 */
const CLAIMS: Record<string, (run: Run) => void> = {
	// A-07: the Uno's examples, each running its program from programs/.
	'uno-blink': (run) => {
		// The LED (2) on for the first second, off for the next.
		expect(run.for(0.5).amps(2), 'on').toBeGreaterThan(LIT)
		expect(run.for(1).amps(2), 'off').toBeLessThan(DARK)
	},
	'uno-button': (run) => {
		// Dark until the button (3) is held, lit while it is, dark again.
		expect(run.for(0.1).amps(2), 'dark').toBeLessThan(DARK)
		expect(run.press(3, true).for(0.05).amps(2), 'lit while held').toBeGreaterThan(LIT)
		expect(run.press(3, false).for(0.05).amps(2), 'dark again').toBeLessThan(DARK)
	},
	'uno-traffic': (run) => {
		// Red (2), the middle light (4), green (6): green, then the middle
		// one, then red.
		const lit = () => [2, 4, 6].map((led) => run.amps(led) > LIT)
		expect(run.for(1).and(lit), 'green').toEqual([false, false, true])
		expect(run.for(1.25).and(lit), 'middle').toEqual([false, true, false])
		expect(run.for(1).and(lit), 'red').toEqual([true, false, false])
	},
	'uno-fade': (run) => {
		// Each frame catches the pulses on or off, so brightness is how many
		// frames an LED is lit in: D10's (4) more than D9's (2) at first, and
		// the other way round a second later, when they have swapped.
		const frames = () => {
			let [d9, d10] = [0, 0]
			for (let frame = 0; frame < 12; frame++) {
				run.for(1 / 60)
				if (run.amps(2) > LIT) d9++
				if (run.amps(4) > LIT) d10++
			}
			return [d9, d10]
		}
		const [early9, early10] = frames()
		expect(early10, 'D10 brighter at first').toBeGreaterThan(early9)
		run.for(0.7)
		const [later9, later10] = frames()
		expect(later9, 'D9 brighter later').toBeGreaterThan(later10)
	},
	'uno-smoothed': (run) => {
		// The meter (3) on the capacitor: about 1.25 V, with a ripple on the
		// scope above what a frame shows.
		run.for(0.1)
		const volts = run.volts(3)
		expect(volts).toBeGreaterThan(1.05)
		expect(volts).toBeLessThan(1.45)
		const ripple = run.highest(3, 0.05) - volts
		expect(ripple, 'ripple').toBeGreaterThan(0.05)
		// Ten times the capacitor (2): the same average, a tenth the ripple.
		run.set(2, '10u').for(0.6)
		expect(run.highest(3, 0.05) - run.volts(3), 'smaller ripple').toBeLessThan(ripple / 3)
	},
	'uno-sweep': (run) => {
		// The horn (1) climbs a degree every 10 ms.
		const early = run.for(0.5).angle(1)
		expect(run.for(0.5).angle(1), 'further round').toBeGreaterThan(early + 20)
	},
	// A-04: analogRead.
	'uno-analog-input': (run) => {
		// How often the LED (2) changes in two seconds: a fifth of the way
		// round (3) is 204 ms each way, four fifths 818.
		const changes = (seconds: number) => {
			let count = 0
			let was = run.amps(2) > LIT
			for (let frame = 0; frame < seconds * 60; frame++) {
				const lit = run.for(1 / 60).amps(2) > LIT
				if (lit !== was) count++
				was = lit
			}
			return count
		}
		const fast = changes(2)
		expect(fast, 'fast').toBeGreaterThanOrEqual(8)
		run.dial(3, 0.8).for(1)
		expect(changes(2), 'slow').toBeLessThanOrEqual(3)
	},
	'uno-dimmer': (run) => {
		// Frames that catch D9's pulses on: a quarter of them at a quarter of
		// the way round (3), three quarters at three quarters.
		const lit = () => {
			let count = 0
			for (let frame = 0; frame < 60; frame++) if (run.for(1 / 60).amps(2) > LIT) count++
			return count
		}
		const dim = lit()
		const bright = run.dial(3, 0.75).for(0.1).and(lit)
		expect(dim, 'dim').toBeGreaterThan(5)
		expect(bright, 'bright').toBeGreaterThan(2 * dim)
	},
	'uno-night-light': (run) => {
		// Out in a lit room; the LDR (4) in the dark, and the LED (2) lights;
		// the room lit again, and it goes out.
		expect(run.for(0.5).amps(2), 'out').toBeLessThan(DARK)
		expect(run.dial(4, 0.1).for(1.5).amps(2), 'lit in the dark').toBeGreaterThan(LIT)
		expect(run.dial(4, 0.6).for(1.5).amps(2), 'out again').toBeLessThan(DARK)
	},
	// A-06: Serial.
	'uno-analog-read-serial': (run) => {
		// The pot (1) half way: 511 or 512 on every line; a quarter, 255 or 256.
		const lines = run.for(0.6).printed(0).trim().split('\r\n')
		expect(lines.length, 'lines').toBeGreaterThanOrEqual(4)
		for (const line of lines) expect(['511', '512'], 'half way').toContain(line)
		const later = run.dial(1, 0.25).for(0.4).printed(0).trim().split('\r\n')
		expect(['255', '256'], 'a quarter').toContain(later.at(-1))
	},
	'uno-serial-led': (run) => {
		// Typed "on", D13's LED (2) lights and the program says so; "off".
		expect(run.for(0.2).printed(0)).toBe('Type on or off.\r\n')
		expect(run.type(0, 'on\n').for(0.2).printed(0)).toBe('D13 is on.\r\n')
		expect(run.amps(2), 'lit').toBeGreaterThan(LIT)
		expect(run.type(0, 'off\n').for(0.2).printed(0)).toBe('D13 is off.\r\n')
		expect(run.amps(2), 'dark').toBeLessThan(DARK)
	},
	'uno-serial-thermometer': (run) => {
		// At the monitor's 9600, garbage; at 115200, 22 °C a line.
		const garbage = run.for(1.1).printed(0)
		expect(garbage, 'garbage').not.toContain('°C')
		run.baud(0, 115200).for(0.6).printed(0)
		const lines = run.for(1.1).printed(0).trim().split('\r\n')
		expect(lines.length, 'lines').toBeGreaterThanOrEqual(2)
		for (const line of lines) {
			expect(line).toMatch(/^\d+\.\d °C$/)
			expect(Math.abs(parseFloat(line) - 22)).toBeLessThan(0.3)
		}
	},
	'uno-two-pots': (run) => {
		// A0's pot (1) at 0.3 is 1.5 V, about 307; A1's (2) at 0.8, about 818.
		const last = run.for(0.5).printed(0).trim().split('\r\n').at(-1) ?? ''
		const [, a0, a1] = /^A0 (\d+) {3}A1 (\d+)$/.exec(last) ?? []
		expect(Math.abs(Number(a0) - 307), last).toBeLessThan(3)
		expect(Math.abs(Number(a1) - 818), last).toBeLessThan(3)
	},
	'uno-thermometer': (run) => {
		// The LEDs (2, 4, 6, 8, 10): at 27 °C the first two; the TMP36 (11)
		// at 38 °C, four.
		const lit = () => [2, 4, 6, 8, 10].map((led) => run.amps(led) > LIT)
		expect(run.for(0.3).and(lit), '27 °C').toEqual([true, true, false, false, false])
		expect(
			run
				.dial(11, (38 + 40) / 165)
				.for(0.3)
				.and(lit),
			'38 °C',
		).toEqual([true, true, true, true, false])
	},
	milestone: (run) => {
		// Lit; then a bigger resistor, and the LED (2) dims — its current
		// falls by more than half for 1 kΩ in place of 220 Ω.
		const lit = run.for(0.1).amps(2)
		expect(lit).toBeGreaterThan(LIT)
		const dimmer = run.set(1, '1k').for(0.1).amps(2)
		expect(dimmer, 'dimmer').toBeLessThan(lit / 2)
		expect(dimmer, 'still on').toBeGreaterThan(DARK)
	},
	switched: (run) => {
		// Lit; the switch (1) opened, and nothing anywhere carries current;
		// shut again, lit again.
		expect(run.for(0.1).amps(3)).toBeGreaterThan(LIT)
		run.press(1, false).for(1 / 60)
		for (const index of [0, 2, 3]) expect(run.amps(index), `element ${index}`).toBeLessThan(DARK)
		expect(
			run
				.press(1, true)
				.for(1 / 60)
				.amps(3),
			'lit once shut',
		).toBeGreaterThan(LIT)
	},
	held: (run) => {
		// Dark until the button (1) is held, lit while it is, out at once —
		// the very next frame — when it is let go.
		expect(run.for(0.1).amps(3), 'dark until held').toBeLessThan(DARK)
		expect(run.press(1, true).for(0.1).amps(3), 'lit while held').toBeGreaterThan(LIT)
		expect(
			run
				.press(1, false)
				.for(1 / 60)
				.amps(3),
			'out at once',
		).toBeLessThan(DARK)
	},
	series: (run) => {
		run.for(0.1)
		expect(run.amps(2)).toBeGreaterThan(LIT)
		expect(run.amps(3)).toBeCloseTo(run.amps(2), 9)
	},
	parallel: (run) => {
		run.for(0.1)
		const red = run.amps(2)
		expect(red).toBeGreaterThan(LIT)
		expect(run.amps(4)).toBeGreaterThan(LIT)
		expect(run.amps(0)).toBeCloseTo(red + run.amps(4), 6)
		// The green LED (4) deleted, and the red stays lit, as bright as it was.
		const left = run.remove(4).for(0.1).amps(2)
		expect(Math.abs(left - red) / red, 'just as bright').toBeLessThan(0.01)
	},
	dimmer: (run) => {
		// The knob (1) from one end to the other: the LED's (3) current moves
		// one way all along, and one end is well over half again the other.
		const at = [0, 0.25, 0.5, 0.75, 1].map((wiper) => run.dial(1, wiper).for(0.1).amps(3))
		const rising = at.every((amps, index) => index === 0 || amps > at[index - 1])
		const falling = at.every((amps, index) => index === 0 || amps < at[index - 1])
		expect(rising || falling, `${at.join(', ')}`).toBe(true)
		expect(Math.max(...at) / Math.min(...at), `${at.join(', ')}`).toBeGreaterThan(1.5)
		expect(Math.min(...at), 'lit throughout').toBeGreaterThan(DARK)
	},
	white: (run) => {
		// All three nearly as bright as they get, so the light is white; and
		// the red's larger resistor is what keeps it level with the others.
		const [red, green, blue] = run.for(0.1).dice(1)
		for (const die of [red, green, blue]) expect(die).toBeGreaterThan(0.9)
		expect(Math.abs(red - blue), 'red level with blue').toBeLessThan(0.05)
		expect(mixedLight(['Red', 'Green', 'Blue'], [red, green, blue])).toMatch(
			/^#f[0-9a-f]f[0-9a-f]f[0-9a-f]$/,
		)
	},
	mixing: (run) => {
		// Opens orange: red full, green part-way, blue turned right down.
		const [red, green, blue] = run.for(0.1).dice(1)
		expect(red).toBeGreaterThan(green)
		expect(green).toBeGreaterThan(blue)
		// Each knob is its own colour: blue turned up lights the blue, and
		// leaves the red where it was.
		run.dial(4, 1).for(0.1)
		expect(run.dice(1)[2]).toBeGreaterThan(0.9)
		expect(run.dice(1)[0]).toBeCloseTo(red, 9)
	},
	relay: (run) => {
		run.for(0.1)
		expect(run.amps(6), 'green on NC, at rest').toBeGreaterThan(LIT)
		expect(run.amps(4), 'red on NO, at rest').toBeLessThan(DARK)
		run.press(1, true).for(0.2)
		expect(run.amps(4), 'red, held').toBeGreaterThan(LIT)
		expect(run.amps(6), 'green, held').toBeLessThan(DARK)
		run.press(1, false).for(0.2)
		expect(run.amps(6), 'green again, let go').toBeGreaterThan(LIT)
	},
	buzzer: (run) => {
		expect(run.for(0.1).active(3), 'silent until pressed').toBe(false)
		run.press(1, true).for(0.1)
		expect(run.active(3), 'sounding while held').toBe(true)
		// 9 V over 150 Ω and the buzzer's 200: 25.7 mA, what it is rated for.
		expect(run.amps(3)).toBeCloseTo(9 / 350, 3)
		expect(run.press(1, false).for(0.1).active(3), 'silent again').toBe(false)
	},
	hot: (run) => {
		// 9 V over 150 Ω: 0.54 W, and 340 K/W on top of 25 °C is 209 °C,
		// past the 155 °C it may reach. Thirty seconds is one time constant.
		const heat = run.for(30).heat(1)
		expect(heat.watts, 'half a watt').toBeCloseTo(81 / 150, 3)
		expect(heat.watts, 'past its rating').toBeGreaterThan(heat.ratedWatts ?? Infinity)
		expect(heat.settlesAt, 'heading past its limit').toBeGreaterThan(heat.maxCelsius)
		expect(heat.celsius, 'well on its way').toBeGreaterThan(120)
	},
	divider: (run) => expect(run.for(0.1).volts(4)).toBeCloseTo(6, 3),
	loop: (run) => {
		run.for(0.1)
		expect(run.amps(1)).toBeCloseTo(9 / 3000, 6)
		expect(run.amps(3)).toBeCloseTo(run.amps(1), 9)
		expect(run.amps(5)).toBeCloseTo(run.amps(1), 9)
	},
	sum: (run) => {
		run.for(0.1)
		const [one, two, three] = [run.volts(4), run.volts(5), run.volts(6)]
		expect(one).toBeCloseTo(1.5, 3)
		expect(two).toBeCloseTo(3, 3)
		expect(one + two + three).toBeCloseTo(9, 3)
	},
	thermometer: (run) => {
		// 10 kΩ over a 10 kΩ thermistor at 25 °C: exactly half the battery.
		expect(run.for(0.1).volts(3)).toBeCloseTo(4.5, 2)
		// At 50 °C the B equation gives 3.59 kΩ: 9 · 3.59 / 13.59 = 2.38 V.
		expect(
			run
				.dial(2, 70 / 120)
				.for(0.1)
				.volts(3),
		).toBeCloseTo(2.38, 1)
	},
	nightlight: (run) => {
		expect(run.for(0.1).amps(5), 'dark in a bright room').toBeLessThan(DARK)
		expect(run.dial(2, 0.1).for(0.1).amps(5), 'lit once the room is dark').toBeGreaterThan(LIT)
	},
	nightphoto: (run) => {
		// 1 000 lux saturates it and holds the base down; at a third of a lux
		// the 100 kΩ turns the transistor on (C-89).
		expect(run.for(0.1).amps(5), 'dark in a bright room').toBeLessThan(DARK)
		expect(run.dial(2, 0.1).for(0.1).amps(5), 'lit once the room is dark').toBeGreaterThan(LIT)
	},
	luxmeter: (run) => {
		// Dial 0.6 is 100 lux, 0.8 is 1 000: ten times the light, ten times the
		// reading — where an LDR's divider would not be a straight line.
		const hundred = run.for(0.1).volts(3)
		expect(hundred).toBeCloseTo(0.2, 2)
		expect(run.dial(1, 0.8).for(0.1).volts(3) / hundred).toBeCloseTo(10, 1)
	},
	solarmeter: (run) => {
		// 40 Ω is the sheet's maximum-power load: 4.0 V and 100 mA. At half
		// the sun the load is on the flat of the curve, so the current halves
		// near enough (C-117).
		expect(run.for(0.1).volts(3)).toBeCloseTo(4.0, 2)
		expect(run.amps(1)).toBeCloseTo(0.1, 3)
		expect(run.dial(0, 0.5).for(0.1).amps(1) / 0.1).toBeCloseTo(0.5, 1)
	},
	sunlight: (run) => {
		// A tenth of the sun, 10.5 mA, into 4 700 µF: under half way in half
		// a second, past 3.4 V by ten. Then dark, and the diode keeps the
		// panel — a diode itself, at night — from taking it back: what it
		// loses is the capacitor's own leak, 1 % a second (C-117).
		const early = run.for(0.5).volts(3)
		const day = run.for(9.5).volts(3)
		const night = run.dial(0, 0).for(3).volts(3)
		expect(early).toBeLessThan(day / 2)
		expect(day).toBeGreaterThan(3.4)
		expect(night).toBeGreaterThan(0.95 * day)
	},
	oldsupply: (run) => {
		// 24 V peak halved to 12, less two diodes: the reservoir (7) rides
		// near 9.7 V with some 0.15 V of ripple, refilled 100
		// times a second, and the 7805's 5 V (8) does not move (C-118).
		run.for(1.5)
		let [low, high] = [Infinity, -Infinity]
		let [outLow, outHigh] = [Infinity, -Infinity]
		for (let sample = 0; sample < 200; sample++) {
			run.step(0.5e-3)
			low = Math.min(low, run.volts(7))
			high = Math.max(high, run.volts(7))
			outLow = Math.min(outLow, run.volts(8))
			outHigh = Math.max(outHigh, run.volts(8))
		}
		expect(high).toBeGreaterThan(9.5)
		expect(high - low).toBeGreaterThan(0.05)
		expect(outLow).toBeCloseTo(5, 1)
		expect(outHigh - outLow).toBeLessThan(0.02)
		expect(run.amps(6), 'the LED lit').toBeGreaterThan(0.01)
	},
	gauge: (run) => {
		// The reference set to some 4.85 V full scale by 2.2 kΩ and 5.6 kΩ, a
		// step every 0.49 V; the knob (6) under 8.2 kΩ gives 0 to 4.95 V
		// (C-119). Three quarters up is 3.7 V, seven LEDs; a quarter, two;
		// all the way, the tenth as well.
		run.dial(6, 0.75)
		expect(run.for(0.2).bars(2)).toBe(7)
		expect(run.dial(6, 0.25).for(0.2).bars(2)).toBe(2)
		expect(run.dial(6, 1).for(0.2).bars(2)).toBe(10)
		expect(run.dial(6, 0).for(0.2).bars(2)).toBe(0)
	},
	lightbar: (run) => {
		// The LDR (4) from the 1.25 V reference into 10 kΩ: 100 lux is 2 kΩ
		// and 1.04 V, eight steps of 125 mV; 1 lux, 50 kΩ and 0.21 V, one;
		// a dark room, none (C-119).
		run.dial(4, 0.6)
		expect(run.for(0.2).bars(2)).toBe(8)
		expect(run.dial(4, 0.2).for(0.2).bars(2)).toBe(1)
		expect(run.dial(4, 0).for(0.2).bars(2)).toBe(0)
	},
	fused: (run) => {
		// The LED (3) lit through 470 Ω; the switch (4) shorts it, 9 V across
		// the fuse's 0.21 Ω, and the 500 mA fuse (1) goes in a frame. Opening
		// the switch again mends nothing; a new fuse does (C-120).
		expect(run.for(0.1).amps(3), 'lit').toBeGreaterThan(LIT)
		expect(run.press(4, true).for(0.2).blown(1), 'blown by the short').toBe(true)
		expect(run.press(4, false).for(0.5).amps(3), 'still dark').toBeLessThan(DARK)
		expect(run.blown(1)).toBe(true)
		expect(run.replace(1).for(0.2).amps(3), 'lit with a new fuse').toBeGreaterThan(LIT)
	},
	resetting: (run) => {
		// An MF-R050 (1) instead: the short trips it within its 4 s and it
		// is never blown; with the short gone, the LED's 20 mA cannot keep it
		// hot, and it cools back to conducting by itself (C-120).
		expect(run.for(0.1).amps(3), 'lit').toBeGreaterThan(LIT)
		run.press(4, true).for(4)
		expect(
			run.notes().some((note) => note.includes('tripped')),
			'tripped',
		).toBe(true)
		run.press(4, false).for(20)
		expect(
			run.notes().some((note) => note.includes('tripped')),
			'cooled',
		).toBe(false)
		expect(run.amps(3), 'lit again').toBeGreaterThan(0.015)
		expect(run.blown(1)).toBe(false)
	},
	ninemotor: (run) => {
		// The same motor's 15 Ω on each: the 9 V, behind its 9.6 Ω, gives it
		// 5.6 V; four AAs, behind 0.9 Ω, 5.7 V. At that 370 mA the 9 V has
		// under an hour in it, the AAs several (C-121).
		run.for(0.5)
		expect(run.volts(2)).toBeCloseTo(5.55, 1)
		expect(run.volts(5)).toBeCloseTo(5.7, 1)
		expect(run.hours(0)).toBeLessThan(1)
		expect(run.hours(3)).toBeGreaterThan(3)
	},
	lasting: (run) => {
		// A CR2032 through 150 Ω into a red LED: about 6.5 mA, so its
		// 235 mAh last a day and a half; turned down to a tenth, a tenth of
		// that (C-121).
		const full = run.for(0.5).hours(0)
		expect(full).toBeGreaterThan(30)
		expect(full).toBeLessThan(42)
		expect(run.amps(2), 'lit').toBeGreaterThan(LIT)
		const tenth = run.dial(0, 0.1).for(0.5).hours(0)
		expect(tenth / full).toBeCloseTo(0.1, 1)
	},
	charging: (run) => {
		const early = run.for(0.2).volts(3)
		const later = run.for(3).volts(3)
		expect(early).toBeGreaterThan(0.5)
		expect(later).toBeGreaterThan(early)
		expect(later).toBeLessThan(9)
	},
	fading: (run) => {
		expect(run.for(0.05).amps(2), 'lit at first').toBeGreaterThan(LIT)
		expect(run.for(15).amps(2), 'out once empty').toBeLessThan(1e-3)
	},
	coil: (run) => {
		// 0.15 A at the end, a time constant of one second: 1.5 V, so the
		// 10 Ω stays inside its quarter watt (B-03).
		expect(run.for(0.2).amps(3)).toBeLessThan(0.05)
		expect(run.for(8).amps(3)).toBeGreaterThan(0.14)
	},
	ringing: (run) => {
		// About one swing a second: through zero and out the other side.
		let lowest = Number.POSITIVE_INFINITY
		let highest = 0
		for (let frame = 0; frame < 90; frame++) {
			const volts = run.for(1 / 60).volts(3)
			lowest = Math.min(lowest, volts)
			if (lowest < 0.5) highest = Math.max(highest, volts)
		}
		expect(lowest, 'swings through zero').toBeLessThan(0.5)
		expect(highest, 'and back out').toBeGreaterThan(2)
	},
	blink: (run) => {
		expect(run.for(0.25).amps(2), 'forward half').toBeGreaterThan(LIT)
		expect(run.for(0.5).amps(2), 'backward half').toBeLessThan(DARK)
	},
	turns: (run) => {
		run.for(0.25)
		expect(run.amps(2), 'red, first half').toBeGreaterThan(LIT)
		expect(run.amps(3), 'green, first half').toBeLessThan(DARK)
		run.for(0.5)
		expect(run.amps(2), 'red, second half').toBeLessThan(DARK)
		expect(run.amps(3), 'green, second half').toBeGreaterThan(LIT)
	},
	lifted: (run) => {
		expect(run.for(0.25).volts(4), 'top of the swing').toBeCloseTo(10, 2)
		expect(run.for(0.5).volts(4), 'bottom of the swing').toBeCloseTo(0, 2)
	},
	rounded: (run) => {
		// τ = 0.1 s: most of the way up by a quarter second, and still well
		// above zero three hundredths after the edge that sent it down.
		const climbed = run.for(0.25).volts(3)
		expect(climbed).toBeGreaterThan(4)
		expect(climbed).toBeLessThan(5)
		expect(run.for(0.28).volts(3), 'no jump at the edge').toBeGreaterThan(2)
	},
	rectified: (run) => {
		const peak = run.for(0.25).volts(3)
		expect(peak, 'the top, less a diode drop').toBeGreaterThan(3.9)
		expect(peak).toBeLessThan(4.6)
		expect(run.for(0.5).volts(3), 'the bottom, blocked').toBeLessThan(0.01)
	},
	smoothed: (run) => {
		// 50 Hz, 1 kΩ and 470 µF: τ is 0.47 s against 20 ms between peaks, so
		// over a whole cycle the meter sags by a fifth of a volt, where the
		// same circuit without the capacitor drops to nothing (C-86).
		run.for(1)
		const volts: number[] = []
		for (let frame = 0; frame < 6; frame++) volts.push(run.for(1 / 60).volts(4))
		expect(Math.min(...volts), 'never down to nothing').toBeGreaterThan(3.5)
		expect(Math.max(...volts) - Math.min(...volts), 'barely moving').toBeLessThan(0.4)
	},
	zener: (run) => {
		const volts = run.for(0.1).volts(3)
		expect(volts).toBeGreaterThan(4.95)
		expect(volts).toBeLessThan(5.1)
	},
	clipped: (run) => {
		// Over a whole cycle the meter never passes 5.3 V though the source
		// swings to 9, and at the bottom of the swing it reads a diode drop.
		let highest = 0
		for (let frame = 0; frame < 60; frame++) highest = Math.max(highest, run.for(1 / 60).volts(3))
		expect(highest, 'cut off at the Zener voltage').toBeGreaterThan(4.9)
		expect(highest, 'and no higher').toBeLessThan(5.3)
		expect(run.for(0.75).volts(3), 'the bottom, a diode drop').toBeLessThan(1)
	},
	oneway: (run) => {
		run.for(0.1)
		expect(run.amps(3), 'forwards').toBeGreaterThan(LIT)
		expect(run.amps(6), 'backwards').toBeLessThan(DARK)
	},
	transistor: (run) => {
		run.for(0.1)
		const led = run.amps(5)
		expect(led).toBeGreaterThan(LIT)
		expect(led / run.amps(2), 'a far bigger current').toBeGreaterThan(20)
	},
	seven: (run) => expect(run.for(0.1).lit(1)).toBe('abc'),
	glass: (run) => expect(run.for(0.2).lit(1)).toBe('bc'),
	antiphase: (run) => expect(run.for(0.2).lit(2)).toBe('abc'),
	rms: (run) => {
		// A ±4 V square is 4 V RMS, where the glass is as dark as it goes; a
		// ±4 V sine is 2.8 V RMS, part of the way from its 2 V threshold.
		const [square, sine] = [run.for(0.2).shade(1), run.shade(3)]
		expect(square[1], 'the square’s b').toBeGreaterThan(0.95)
		expect(sine[1], 'the sine’s b').toBeGreaterThan(0.2)
		expect(sine[1], 'the sine’s b').toBeLessThan(0.6)
	},
	counting: (run) => {
		// The digit (element 6) goes up one a tick, about 1.46 a second: in
		// four seconds five ticks at least, each one more than the last and
		// nothing out of turn. It shows 1 from the start, because the 555's
		// output rises the moment it is powered, and the counter counts that
		// rise — as a breadboard does, which is what a power-on reset is for.
		const font = [
			'abcdef',
			'bc',
			'abdeg',
			'abcdg',
			'bcfg',
			'acdfg',
			'acdefg',
			'abc',
			'abcdefg',
			'abcdfg',
		]
		const shown: number[] = []
		for (let frame = 0; frame < 240; frame++) {
			const number = font.indexOf(run.for(1 / 60).lit(6))
			if (shown[shown.length - 1] !== number) shown.push(number)
		}
		// And no more than about one and a half a second either: four
		// seconds, eight numbers at most with the first.
		expect(shown.length, `${shown.join(', ')}`).toBeGreaterThanOrEqual(6)
		expect(shown.length, `${shown.join(', ')}`).toBeLessThanOrEqual(8)
		for (let index = 1; index < shown.length; index++) {
			expect(shown[index], `${shown.join(', ')}`).toBe((shown[index - 1] + 1) % 10)
		}
	},
	hundred: (run) => {
		// The tens digit is element 7 and the ones 8, clocked by a 555 ten
		// times faster than "A digit that counts": about 15 a second. In under
		// eight seconds it passes 99 and goes round, and every change it shows
		// is one more than the last — the tens going up exactly when the ones
		// go round, never a frame early or late.
		const font = [
			'abcdef',
			'bc',
			'abdeg',
			'abcdg',
			'bcfg',
			'acdfg',
			'acdefg',
			'abc',
			'abcdefg',
			'abcdfg',
		]
		const shown: number[] = []
		for (let frame = 0; frame < 460; frame++) {
			run.for(1 / 60)
			const [tens, ones] = [font.indexOf(run.lit(7)), font.indexOf(run.lit(8))]
			expect(tens, `tens at frame ${frame}`).toBeGreaterThanOrEqual(0)
			expect(ones, `ones at frame ${frame}`).toBeGreaterThanOrEqual(0)
			const number = tens * 10 + ones
			if (shown[shown.length - 1] !== number) shown.push(number)
		}
		expect(shown.length, `${shown.join(', ')}`).toBeGreaterThan(100)
		for (let index = 1; index < shown.length; index++) {
			expect(shown[index], `after ${shown.slice(Math.max(0, index - 3), index).join(', ')}`).toBe(
				(shown[index - 1] + 1) % 100,
			)
		}
		expect(shown, 'round past 99').toContain(99)
	},
	decoded: (run) => {
		// The digit is element 2; the switches are 10 (A, worth 1), 12 (B, 2),
		// 14 (C, 4) and 16 (D, 8). A alone is closed to begin with.
		expect(run.for(0.1).lit(2), 'one to begin with').toBe('bc')
		expect(run.press(12, true).for(0.1).lit(2), 'three').toBe('abcdg')
		expect(run.press(10, false).press(12, false).press(16, true).for(0.1).lit(2), 'eight').toBe(
			'abcdefg',
		)
		expect(run.press(10, true).for(0.1).lit(2), 'nine, with no tail').toBe('abcfg')
		expect(run.press(14, true).for(0.1).lit(2), 'thirteen is blank').toBe('')
		expect(run.press(10, false).press(14, false).press(16, false).for(0.1).lit(2), 'zero').toBe(
			'abcdef',
		)
	},
	one: (run) => {
		expect(run.for(0.1).lit(2), 'dark until held').toBe('')
		expect(run.press(1, true).for(0.1).lit(2), 'a one while held').toBe('bc')
		expect(run.press(1, false).for(0.1).lit(2), 'dark again').toBe('')
	},
	fan: (run) => {
		run.for(0.1)
		expect(run.active(6), 'still at 25 °C').toBe(false)
		run.dial(1, 80 / 120).for(0.2)
		expect(run.active(4), 'the relay pulls in at 60 °C').toBe(true)
		expect(run.active(6), 'and the motor turns').toBe(true)
	},
	clicking: (run) => {
		// 1 Hz: the contact is thrown and released at least twice in three s.
		let changes = 0
		let lit = run.for(1 / 60).amps(7) > LIT
		for (let frame = 0; frame < 180; frame++) {
			const now = run.for(1 / 60).amps(7) > LIT
			if (now !== lit) changes++
			lit = now
		}
		expect(changes).toBeGreaterThanOrEqual(4)
	},
	kick: (run) => {
		// Past the first falling edge of the 1 Hz square, at half a second.
		expect(run.for(0.4).highest(7, 0.4), 'nothing while the coil is fed').toBeLessThan(9.5)
		expect(run.for(0.2).highest(7, 0.2), 'far past 9 V as it lets go').toBeGreaterThan(18)
	},
	caught: (run) => {
		run.for(0.4)
		const held = run.highest(8, 0.4)
		expect(held, 'held: the transistor is on').toBeLessThan(1)
		expect(run.for(0.2).highest(8, 0.2), 'a diode’s drop past 9 V at most').toBeLessThan(10)
		expect(run.highest(8, 0.1), 'and off, the meter reads the battery').toBeGreaterThan(8.9)
	},
	gate: (run) => {
		expect(run.for(0.25).amps(5), 'lit while the gate is high').toBeGreaterThan(LIT)
		expect(run.amps(2), 'and the gate takes nothing').toBeLessThan(1e-9)
		expect(run.for(0.5).amps(5), 'dark while it is low').toBeLessThan(DARK)
	},
	remembers: (run) => {
		expect(run.for(0.5).amps(6), 'dark until pressed').toBeLessThan(DARK)
		run.press(1, true).for(0.1).press(1, false)
		// 4.7 MΩ across 1 µF: a time constant of 4.7 s. A second after the
		// press the gate is most of the way up still; by twelve it is under
		// the threshold.
		expect(run.for(1).amps(6), 'still lit a second after').toBeGreaterThan(LIT)
		expect(run.for(12).amps(6), 'out once the charge has gone').toBeLessThan(DARK)
	},
	regulated: (run) => {
		// The meter (2) holds 5 V whatever the battery (0) is, until the
		// battery is too low: on 6 V, a 7805 cannot keep it.
		expect(run.for(0.1).volts(2)).toBeCloseTo(5, 2)
		expect(run.set(0, '12').for(0.1).volts(2), 'on 12 V').toBeCloseTo(5, 2)
		expect(run.set(0, '7.5').for(0.1).volts(2), 'on 7.5 V').toBeCloseTo(5, 1)
		expect(run.set(0, '6').for(0.1).volts(2), 'on 6 V').toBeLessThan(4.8)
	},
	dropout: (run) => {
		expect(run.for(0.1).volts(2)).toBeCloseTo(7, 1)
		// And the note says why: two volts more in than it makes.
		const notes = run.notes()
		expect(
			notes.some((note) => note.includes('needs 14 V in to give 12 V out')),
			`${notes.join(', ')}`,
		).toBe(true)
	},
	threevolts: (run) => expect(run.for(0.1).volts(2)).toBeCloseTo(3.3, 2),
	heatsink: (run) => {
		run.for(0.1)
		expect(run.active(3), 'the motor turns').toBe(true)
		expect(run.volts(2), 'on a steady 5 V').toBeCloseTo(5, 1)
		// 7 V across it at a third of an amp: over two watts, and 50 K/W
		// on top of the room is past the 125 °C its die may reach.
		const heat = run.for(30).heat(1)
		expect(heat.watts, 'over two watts').toBeGreaterThan(2)
		expect(heat.settlesAt, 'heading past its limit').toBeGreaterThan(heat.maxCelsius)
		expect(heat.celsius, 'warm after half a minute').toBeGreaterThan(60)
		expect(heat.celsius, 'not there yet').toBeLessThan(heat.maxCelsius)
	},
	thermometer36: (run) => {
		// 0.5 V and 10 mV a degree, from its sheet (C-87).
		expect(run.for(0.1).volts(2)).toBeCloseTo(0.75, 2)
		expect(
			run
				.dial(1, (50 + 40) / 165)
				.for(0.1)
				.volts(2),
		).toBeCloseTo(1.0, 2)
	},
	hotfan: (run) => {
		// The knob sets about 0.80 V on the comparator's −, which the sensor
		// passes at 30 °C. At 25 °C the fan is still; at 40 °C it runs, and
		// back at 25 °C it stops.
		expect(run.for(0.2).active(9), 'still at 25 °C').toBe(false)
		expect(
			run
				.dial(1, (40 + 40) / 165)
				.for(0.3)
				.active(9),
			'running at 40 °C',
		).toBe(true)
		expect(
			run
				.dial(1, (25 + 40) / 165)
				.for(0.3)
				.active(9),
			'still again',
		).toBe(false)
	},
	threshold: (run) => {
		// Turned slowly from where it starts to the end, the LED (6) is dark
		// and then lit, once, with nothing dim between: it snaps.
		expect(run.for(0.1).amps(6), 'dark until turned').toBeLessThan(DARK)
		const seen: number[] = []
		for (let wiper = 0.3; wiper <= 1.0001; wiper += 0.02)
			seen.push(
				run
					.dial(1, wiper)
					.for(1 / 60)
					.amps(6),
			)
		const lit = seen.map((amps) => amps > LIT)
		const dim = seen.filter((amps) => amps >= DARK && amps <= LIT)
		expect(dim, 'nothing between dark and lit').toEqual([])
		const changes = lit.filter((on, index) => index > 0 && on !== lit[index - 1]).length
		expect(changes, `${seen.join(', ')}`).toBe(1)
		expect(lit.at(-1), 'lit at the end').toBe(true)
	},
	astable: (run) => {
		// 1.46 Hz, "about one and a half times a second": after a second for
		// the capacitor's first charge, four seconds hold between ten and
		// fourteen changes of the LED — 1.25 to 1.75 blinks a second.
		run.for(1)
		let changes = 0
		let lit = run.for(1 / 60).amps(6) > LIT
		for (let frame = 0; frame < 240; frame++) {
			const now = run.for(1 / 60).amps(6) > LIT
			if (now !== lit) changes++
			lit = now
		}
		expect(changes).toBeGreaterThanOrEqual(10)
		expect(changes).toBeLessThanOrEqual(14)
	},
	upright: (run) => {
		expect(run.for(0.1).amps(3), 'lit upright').toBeGreaterThan(LIT)
		expect(run.press(1, false).for(0.1).amps(3), 'dark tipped').toBeLessThan(DARK)
	},
	tilted: (run) => {
		// Upright, the switch holds the transistor's base at 0 V; tipped, the
		// 10 kΩ turns it on and the buzzer sounds; stood up, it stops (C-88).
		expect(run.for(0.1).active(5), 'quiet upright').toBe(false)
		expect(run.press(1, false).for(0.1).active(5), 'sounding tipped').toBe(true)
		expect(run.press(1, true).for(0.1).active(5), 'quiet again').toBe(false)
	},
	servo: (run) => {
		// The knob is the 555's charging resistor, 10 to 20 kΩ with the fixed
		// one, so 1 to 2 ms high; the diode skips the 270 kΩ while charging,
		// and that is the 19 ms low. Turned up, the horn goes one way;
		// turned down, the other (C-90).
		const middle = run.for(0.5).angle(7)
		expect(middle).toBeGreaterThan(50)
		expect(middle).toBeLessThan(130)
		expect(run.dial(3, 1).for(0.5).angle(7), 'knob one way').toBeGreaterThan(middle + 40)
		expect(run.dial(3, 0).for(0.6).angle(7), 'knob the other').toBeLessThan(middle - 40)
	},
	byhand: (run) => {
		// Buttons 2, 4 and 6 are data, shift and latch; the LEDs are 16 (QA)
		// to 23 (QH). Nothing shows until the latch.
		const tap = (button: number) => run.press(button, true).for(0.05).press(button, false).for(0.05)
		const bit = (one: boolean) => {
			run.press(2, one).for(0.05)
			tap(4)
		}
		const lit = () => [16, 17, 18, 19, 20, 21, 22, 23].map((led) => run.amps(led) > LIT)
		// Eight bits, and all eight shown on the latch: the first in has
		// moved along to QH, the last is QA.
		const bits = [true, false, true, true, false, false, true, false]
		for (const one of bits) bit(one)
		run.press(2, false)
		expect(lit(), 'nothing before the latch').toEqual(Array(8).fill(false))
		tap(6)
		expect(lit(), 'shown once latched').toEqual([...bits].reverse())
	},
	and: (run) => expect(table(run, 2, 4, 7)).toEqual([false, false, false, true]),
	or: (run) => expect(table(run, 2, 4, 7)).toEqual([false, true, true, true]),
	xor: (run) => expect(table(run, 2, 4, 7)).toEqual([false, true, true, false]),
	nand: (run) => expect(table(run, 2, 4, 7)).toEqual([true, true, true, false]),
	not: (run) => {
		// The switch (2) and the LED (5): lit off, dark on.
		expect(run.for(0.1).amps(5), 'lit while off').toBeGreaterThan(LIT)
		expect(run.press(2, true).for(0.1).amps(5), 'out while on').toBeLessThan(DARK)
	},
	universal: (run) => {
		// Switches 1 and 3; red (8) NOT A, green (10) A AND B, blue (12) A OR B.
		const lit = (a: boolean, b: boolean) =>
			[8, 10, 12].map((led) => run.press(1, a).press(3, b).for(0.1).amps(led) > LIT)
		expect(lit(false, false), 'both off').toEqual([true, false, false])
		expect(lit(true, false), 'left on').toEqual([false, false, true])
		expect(lit(false, true), 'right on').toEqual([true, false, true])
		expect(lit(true, true), 'both on').toEqual([false, true, true])
	},
	latch: (run) => {
		// Buttons 3 (set, left) and 5 (reset, right); green (7) on 1Y, red
		// (9) on 2Y. Red to begin with and steady; each press swaps them, and
		// letting go keeps it.
		const shown = (seconds: number) => [7, 9].map((led) => run.for(seconds).amps(led) > LIT)
		expect(shown(0.1), 'red to begin with').toEqual([false, true])
		expect(shown(0.1), 'and steady').toEqual([false, true])
		run.press(3, true).for(0.1).press(3, false)
		expect(shown(0.1), 'set, and kept').toEqual([true, false])
		run.press(5, true).for(0.1).press(5, false)
		expect(shown(0.1), 'reset, and kept').toEqual([false, true])
	},
	blinker: (run) => {
		// The LED (5) blinks on its own: about once every 0.9 s, so five
		// times in five seconds, give or take one.
		let lit = true
		let blinks = 0
		for (let frame = 0; frame < 300; frame++) {
			const now = run.for(1 / 60).amps(5) > LIT
			if (now && !lit) blinks++
			lit = now
		}
		expect(blinks).toBeGreaterThanOrEqual(4)
		expect(blinks).toBeLessThanOrEqual(6)
	},
	bounces: (run) => {
		// The button (1) clocks the counter straight, and the digit (4) shows
		// three once pressed, five once let go.
		expect(run.for(0.1).lit(4), 'nought').toBe('abcdef')
		expect(run.press(1, true).for(0.2).lit(4), 'three on the press').toBe('abcdg')
		expect(run.press(1, false).for(0.2).lit(4), 'five once let go').toBe('acdfg')
	},
	debounced: (run) => {
		// Through the capacitor and the 74HC14, the digit (7) goes up one a
		// press.
		expect(run.for(0.1).lit(7), 'nought').toBe('abcdef')
		run.press(1, true).for(0.2)
		expect(run.lit(7), 'one').toBe('bc')
		run.press(1, false).for(0.5).press(1, true).for(0.2)
		expect(run.lit(7), 'two').toBe('abdeg')
	},
	chaser: (run) => {
		// The LEDs are elements 7, 9 … 25, Q0's first. For three seconds,
		// exactly one is lit and each change is to the next, round after the
		// tenth; at about seven a second, well over ten changes.
		const leds = Array.from({ length: 10 }, (_, index) => 7 + 2 * index)
		const seen: number[] = []
		for (let frame = 0; frame < 180; frame++) {
			run.for(1 / 60)
			const lit = leds.flatMap((led, index) => (run.amps(led) > LIT ? [index] : []))
			expect(lit.length, 'one at a time').toBe(1)
			if (seen[seen.length - 1] !== lit[0]) seen.push(lit[0])
		}
		expect(seen.length).toBeGreaterThan(10)
		for (let at = 1; at < seen.length; at++)
			expect(seen[at], 'the next one').toBe((seen[at - 1] + 1) % 10)
	},
	countfour: (run) => {
		// The LEDs on Q0 to Q3 are elements 7, 9, 11 and 13: each change is
		// to the next, round after the fourth, and every one of them lights.
		const leds = [7, 9, 11, 13]
		const seen: number[] = []
		for (let frame = 0; frame < 180; frame++) {
			run.for(1 / 60)
			const lit = leds.flatMap((led, index) => (run.amps(led) > LIT ? [index] : []))
			expect(lit.length, 'one at a time').toBe(1)
			if (seen[seen.length - 1] !== lit[0]) seen.push(lit[0])
		}
		expect(new Set(seen).size).toBe(4)
		for (let at = 1; at < seen.length; at++)
			expect(seen[at], 'the next one').toBe((seen[at - 1] + 1) % 4)
	},
	toggle: (run) => {
		// The bouncy button (1), debounced, toggles the LED (8): on, off, on.
		const press = () => run.press(1, true).for(0.2).press(1, false).for(0.5).amps(8) > LIT_BY_CMOS
		expect(run.for(0.1).amps(8), 'off to begin with').toBeLessThan(DARK)
		expect([press(), press(), press()]).toEqual([true, false, true])
	},
	halving: (run) => {
		// Red (7) on the 555, green (9) on Q1, blue (11) on Q2: over six
		// seconds green comes on half as often as red, and blue half as often
		// as green.
		const leds = [7, 9, 11]
		const was = [true, false, false]
		const rises = [0, 0, 0]
		for (let frame = 0; frame < 360; frame++) {
			run.for(1 / 60)
			leds.forEach((led, index) => {
				const lit = run.amps(led) > LIT
				if (lit && !was[index]) rises[index]++
				was[index] = lit
			})
		}
		expect(rises[0]).toBeGreaterThan(12)
		expect(Math.abs(rises[1] - rises[0] / 2), `${rises.join(', ')}`).toBeLessThanOrEqual(1)
		expect(Math.abs(rises[2] - rises[1] / 2), `${rises.join(', ')}`).toBeLessThanOrEqual(1)
	},
	toohot: (run) => {
		// The switch (2) on: both motors (4 and 7) run; the 2N7000 (9) is
		// heading past its limit, the IRLZ44N (6) barely warm.
		run.press(2, true).for(0.5)
		expect(run.active(4) && run.active(7), 'both turning').toBe(true)
		const small = run.heat(9)
		expect(small.settlesAt, 'the 2N7000 heading past its limit').toBeGreaterThan(small.maxCelsius)
		expect(run.heat(6).settlesAt, 'the IRLZ44N cool').toBeLessThan(30)
	},
	darlington: (run) => {
		// Held (1): the base meter (2) under a milliamp, the collector's (5)
		// over 0.4 A, and the motor (6) turning.
		expect(run.for(0.1).active(6), 'still until held').toBe(false)
		run.press(1, true).for(0.5)
		expect(run.amps(2), 'into the base').toBeLessThan(1e-3)
		expect(run.amps(5), 'through the motor').toBeGreaterThan(0.4)
		expect(run.active(6), 'turning').toBe(true)
	},
	knob317: (run) => {
		// The knob (3) from one end to the other, the meter (4) on OUT:
		// 1.25 V at nought, 1.25 × (1 + 1500 / 240) + 75 mV at the top.
		expect(run.dial(3, 0).for(0.1).volts(4)).toBeCloseTo(1.25, 1)
		expect(run.dial(3, 1).for(0.1).volts(4)).toBeCloseTo(9.14, 1)
		const middle = run.dial(3, 0.5).for(0.1).volts(4)
		expect(middle).toBeGreaterThan(4.5)
		expect(middle).toBeLessThan(5.5)
	},
	constant: (run) => {
		// The meter (3) at 1.25 / 62 + 50 µA on 9 V, and the same on 12.
		const nine = run.for(0.1).amps(3)
		expect(nine).toBeCloseTo(0.0202, 3)
		expect(run.set(0, '12').for(0.1).amps(3)).toBeCloseTo(nine, 4)
	},
	cheaper: (run) => {
		// The meter after the 1N4148 (3) and the one after the 1N5819 (6).
		const silicon = run.for(0.1).volts(3)
		const schottky = run.volts(6)
		expect(silicon).toBeGreaterThan(8.15)
		expect(silicon).toBeLessThan(8.3)
		expect(schottky).toBeGreaterThan(8.6)
		expect(schottky).toBeLessThan(8.75)
	},
	dooralarm: (run) => {
		// The reed (2) with its magnet 5 mm off: the buzzer (4) quiet; the
		// magnet away, it sounds; back, quiet again.
		expect(run.for(0.1).active(4), 'door shut').toBe(false)
		expect(run.dial(2, 0).for(0.1).active(4), 'door open').toBe(true)
		expect(
			run
				.dial(2, 1 - 5 / 30)
				.for(0.1)
				.active(4),
			'shut again',
		).toBe(false)
	},
	hallmagnet: (run) => {
		// The A3144 (1) and the LED (3): dark far off, lit at 8 mm, still lit
		// at 11.5 mm, dark at 14 mm and still dark back at 11.5.
		const at = (mm: number) =>
			run
				.dial(1, 1 - mm / 30)
				.for(0.1)
				.amps(3) > LIT
		expect([at(30), at(8), at(11.5), at(14), at(11.5)]).toEqual([false, true, true, false, false])
	},
	passing: (run) => {
		// The PIR (1) held from the start: the white LED (4) dark while it
		// settles, lit once it has; let go, lit for its 5 s and then dark.
		expect(run.press(1, true).for(30).amps(4), 'settling').toBeLessThan(DARK)
		expect(run.for(31).amps(4), 'seen').toBeGreaterThan(LIT)
		expect(run.press(1, false).for(4.5).amps(4), 'held 5 s').toBeGreaterThan(LIT)
		expect(run.for(1).amps(4), 'and off').toBeLessThan(DARK)
	},
	isolated: (run) => {
		// The red LED (element 6) is on the 9 V side, and the button (1) on
		// the 5 V one: dark until held, lit while held, dark again let go.
		expect(run.for(0.1).amps(6), 'dark until held').toBeLessThan(DARK)
		expect(run.press(1, true).for(0.1).amps(6), 'lit while held').toBeGreaterThan(LIT)
		expect(run.press(1, false).for(0.1).amps(6), 'dark let go').toBeLessThan(DARK)
	},
	twoway: (run) => {
		// 1A up and 2A down: forwards. Then 2A up too: both ends high, so it
		// stops. 1A down: backwards. 2A down: both low, and stopped again. The
		// motor is element 2, 1A's switch 3 and 2A's 5.
		const way = () => [run.for(0.1).active(2), run.reversed(2)]
		expect(way(), 'forwards').toEqual([true, false])
		run.press(5, true)
		expect(way(), 'both high').toEqual([false, false])
		run.press(3, false)
		expect(way(), 'backwards').toEqual([true, true])
		run.press(5, false)
		expect(way(), 'both low').toEqual([false, false])
	},
	organ: (run) => {
		// An A, read off the run: nothing in it is set to a frequency. The
		// knob is the 555's lower timing resistor, so more of it is lower.
		expect(Math.abs(run.for(0.25).note(8) - 440), 'an A').toBeLessThan(4.4)
		expect(run.active(8), 'drawn sounding').toBe(true)
		expect(run.dial(3, 0.6).for(0.25).note(8)).toBeLessThan(250)
	},
	siren: (run) => {
		// Two notes taking turns at the slow 555's pace, and never silent:
		// the nudge moves the fast one's pitch without stopping it.
		const notes: number[] = []
		run.for(0.15)
		for (let frame = 0; frame < 54; frame++) notes.push(run.for(1 / 60).note(8))
		expect(Math.min(...notes), 'never silent').toBeGreaterThan(0)
		expect(Math.max(...notes) / Math.min(...notes), 'two notes').toBeGreaterThan(1.2)
	},
	leaking: (run) => {
		// Filled through 100 Ω, τ 10 ms; then its own leak, 1 MΩ for 100 µF on
		// the sheet, empties it with τ = 100 s: a tenth in ten seconds (C-86).
		expect(run.for(0.1).volts(4), 'empty until held').toBeLessThan(0.01)
		const full = run
			.press(1, true)
			.for(0.2)
			.press(1, false)
			.for(1 / 60)
			.volts(4)
		expect(full).toBeGreaterThan(8.9)
		const later = run.for(10).volts(4)
		expect(later / full).toBeGreaterThan(0.88)
		expect(later / full).toBeLessThan(0.92)
	},
	oneshot: (run) => {
		expect(run.for(0.5).amps(7), 'dark until pressed').toBeLessThan(DARK)
		run.press(3, true).for(0.1).press(3, false)
		expect(run.for(0.5).amps(7), 'lit after a press').toBeGreaterThan(LIT)
		// 1.1·R·C = 1.1 s: out again well before three.
		expect(run.for(2.5).amps(7), 'out on its own').toBeLessThan(DARK)
	},
	twice: (run) => {
		run.for(0.1)
		expect(run.volts(6)).toBeCloseTo(2 * run.volts(5), 2)
		// Turned up to the top of its track (its wiper at 0 here), twice the
		// left would be more than the 9 V battery has: the right meter stops
		// short, below the battery.
		run.dial(1, 0).for(0.1)
		expect(2 * run.volts(5), 'asked for more than there is').toBeGreaterThan(9)
		expect(run.volts(6), 'out of battery').toBeLessThan(9)
		expect(run.volts(6), 'well short of twice').toBeLessThan(2 * run.volts(5) - 1)
	},
	buffer: (run) => {
		run.for(0.1)
		expect(run.volts(4), 'straight on, it sags').toBeLessThan(1)
		expect(run.volts(9), 'through an op-amp, it holds').toBeCloseTo(4.5, 2)
	},
}

/**
 * The examples whose point is a warning, and the one each opens onto, by the
 * part's place in the file: a resistor past its quarter watt, a 2N7000 past
 * its 200 mA (B-03), a 7805 with no heatsink (B-06). Any other warning in them
 * is still a mistake in them.
 */
const MEANT: Record<string, { rule: string; element: number }> = {
	hot: { rule: 'over-rated', element: 1 },
	toohot: { rule: 'over-rated', element: 9 },
	heatsink: { rule: 'regulator-hot', element: 1 },
}

describe('what every example says it shows', () => {
	it('has a claim for every example, and none for an example that is gone', () => {
		expect(Object.keys(CLAIMS).sort()).toEqual(EXAMPLES.map((example) => example.id).sort())
	})

	it.each(EXAMPLES.map((example) => [example.id, example] as const))(
		'%s does what its line says',
		async (id, example) => {
			const circuit = await createCircuit()
			circuit.load(example.document)
			CLAIMS[id](new Run(circuit))
			// And it says nothing is wrong with itself, whenever it is looked
			// at: an example that opens onto a warning teaches that the warning
			// can be ignored. Two LEDs taking turns on a square wave used to
			// be told one of them was backwards (ADR 0011).
			// A note is something worth knowing, not a mistake: "Not enough
			// volts" is there to show a regulator below its dropout, and is
			// told so (C-102).
			// The few that are about a warning say that one and no other.
			const warnings = circuit.state().findings.filter((finding) => finding.severity === 'warning')
			const meant = MEANT[id]
			if (meant) {
				const part = circuit.elements()[meant.element].id
				expect(
					warnings.map((finding) => [finding.rule, finding.elements]),
					`${id} raises its own warning`,
				).toEqual([[meant.rule, [part]]])
			} else {
				expect(warnings, `${id} raises no warning`).toEqual([])
			}
		},
		// Long enough for "Counting to 99" to go round on the unoptimised core
		// `just test-app` builds, which takes it at 30 ms a frame (C-83).
		60_000,
	)

	it('puts every example on a shelf that is offered', () => {
		for (const example of EXAMPLES) expect(EXAMPLE_GROUPS, example.id).toContain(example.group)
		// And no shelf is empty, which would be a tab that opens onto nothing.
		for (const group of EXAMPLE_GROUPS) {
			expect(
				EXAMPLES.some((example) => example.group === group),
				group,
			).toBe(true)
		}
	})
})

describe('an example’s card (C-131)', () => {
	it('shows the part that says what it is about, which the board can draw', async () => {
		const { heroOf } = await import('./examples')
		const { SYMBOLS } = await import('./symbols')
		for (const example of EXAMPLES) {
			const hero = heroOf(example)
			expect(Object.keys(SYMBOLS), `${example.id}: ${hero}`).toContain(hero)
		}
		const lit = EXAMPLES.find((example) => example.title === 'A lit LED')!
		expect(heroOf(lit)).toBe('Led')
	})
})
