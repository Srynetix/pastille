// SPDX-License-Identifier: MIT OR Apache-2.0

// What the simulation costs, measured where it runs.
//
// C-15. In the browser, against the real wasm bundle, through the same `advance`
// the frame loop calls — because that is the only number that means anything.
// A native benchmark would measure a different build of a different thing.
//
// Two tickets deferred their cost questions to this one. C-10 chose an error
// estimate that takes three solves per step, on the reasoning that "nothing is
// on this path at frame rate yet"; C-48 then put it there, and C-22 added a
// recording on every step. So this is the first measurement of the whole thing
// actually running.

import { describe, expect, it } from 'vitest'

import { createCircuit, type Circuit } from './core'

/**
 * Whether this run is measuring, rather than just checking the app works.
 *
 * The number below belongs to the *optimised* bundle. `just test-app` builds
 * the unoptimised one, because waiting for a release build is not what a test
 * run is for — and measured against that, the same ladder takes about a
 * hundred milliseconds a frame. So a run that has not said it is benchmarking
 * skips this file rather than failing it, and `just bench` and `just ci` say so.
 *
 * Found the first time another ticket ran `just wasm-dev` and the gate went off
 * for a regression nobody had written.
 */
const MEASURING = import.meta.env.VITE_BENCH === '1'

/**
 * Stages of series resistor and shunt capacitor.
 *
 * Forty-nine of them make fifty-one nets — forty-nine non-reference nodes and
 * the source's own branch — which is the fifty-node system §5 asks about. A
 * ladder rather than a heap of unrelated parts, because its nodes actually
 * depend on each other: the matrix is banded and the solve cannot be decomposed
 * into trivial pieces.
 */
const STAGES = 49

/** Nets the ladder should have. Asserted, so the benchmark cannot quietly shrink. */
const NETS = 51

/**
 * Milliseconds per frame this machine last measured, best of three.
 *
 * Recorded on 2026-09-21: **6.5 ms**, from five runs that landed between 6.35
 * and 6.93. That is 2.6 times faster than the simulated time it covers, and
 * 39% of a sixty-hertz frame — so a fifty-node circuit keeps up with the clock
 * and leaves most of the frame for drawing it.
 *
 * There is no hosted pipeline (ADR 0005), so this machine is the pipeline and
 * this number is its. Update it deliberately, in a commit that says why it
 * moved: the test fails both when it gets materially worse and when it gets
 * much better, because a recorded number nobody maintains has stopped being a
 * record of anything.
 */
const BASELINE_MS_PER_FRAME = 6.5

/** How much worse than the baseline is a regression rather than a busy laptop. */
const REGRESSION = 1.8

/** How much better than the baseline means the baseline is simply out of date. */
const STALE = 3

/** A sixtieth of a second, in milliseconds. */
const FRAME_MS = 1000 / 60

/** A fifty-node RC ladder, with a probe on the far end so the scope records too. */
async function ladder(): Promise<Circuit> {
	const circuit = await createCircuit()
	const battery = circuit.add('DcSource', 0, 0)
	let previous = { id: battery, pin: '+' }

	for (let stage = 0; stage < STAGES; stage++) {
		const resistor = circuit.add('Resistor', 40 + stage * 40, 0)
		const capacitor = circuit.add('Capacitor', 40 + stage * 40, 40)
		// A millisecond per stage: the timescale of something a person would
		// build, and the one the adaptive stepper is tuned around.
		circuit.setValue(resistor, '1k')
		circuit.setValue(capacitor, '1u')
		circuit.connect(previous.id, previous.pin, resistor, 'a')
		circuit.connect(resistor, 'b', capacitor, 'a')
		circuit.connect(capacitor, 'b', battery, '-')
		previous = { id: resistor, pin: 'b' }
	}

	const meter = circuit.add('Voltmeter', 0, 80)
	circuit.connect(meter, '+', previous.id, previous.pin)
	circuit.connect(meter, '-', battery, '-')

	return circuit
}

/**
 * Milliseconds to advance `frames` frames, including reading the state back.
 *
 * The read is in the measurement because the frame loop does it: a solve whose
 * answer nobody can get at is not the thing being paid for.
 */
function timeFrames(circuit: Circuit, frames: number): number {
	const started = performance.now()
	for (let frame = 0; frame < frames; frame++) {
		circuit.advance(1 / 60)
		circuit.state()
	}
	return performance.now() - started
}

describe.skipIf(!MEASURING)('what a fifty-node circuit costs', () => {
	it('sustains real time with headroom, and has not got slower', async () => {
		const circuit = await ladder()
		expect(circuit.netCount).toBe(NETS)
		expect(circuit.state().failure).toBeUndefined()

		// Warmed up first: the first frames pay for the compile, the first
		// allocation of every buffer, and whatever the JIT has not seen yet.
		timeFrames(circuit, 30)
		let best = Number.POSITIVE_INFINITY
		for (let attempt = 0; attempt < 3; attempt++) {
			best = Math.min(best, timeFrames(circuit, 60) / 60)
		}

		const measured = `${best.toFixed(2)} ms/frame against a ${BASELINE_MS_PER_FRAME} ms baseline`

		// The acceptance: it keeps up with the clock, with room left for
		// drawing. Half a frame is the floor whatever the baseline says, so
		// raising the baseline carelessly cannot raise this.
		expect(best, `${measured} — no longer real time with headroom`).toBeLessThan(FRAME_MS / 2)

		// And the regression gate proper, against the number in this file.
		expect(best, `${measured} — materially slower than recorded`).toBeLessThan(
			BASELINE_MS_PER_FRAME * REGRESSION,
		)

		// A baseline nobody updates records nothing, so getting much faster
		// fails too. Lower the number in this file and say why it moved.
		expect(
			best,
			`${measured} — much faster than recorded, so the baseline is stale`,
		).toBeGreaterThan(BASELINE_MS_PER_FRAME / STALE)
	}, 60_000)

	it('costs the same whether or not anything is being recorded', async () => {
		// C-22 writes a sample per probe on every solved step. It is worth
		// knowing that the buffer is not what the frame is spent on — if it ever
		// becomes so, this is where it shows up first.
		const watched = await ladder()
		const plain = await createCircuit()
		const battery = plain.add('DcSource', 0, 0)
		let previous = { id: battery, pin: '+' }
		for (let stage = 0; stage < STAGES; stage++) {
			const resistor = plain.add('Resistor', 40 + stage * 40, 0)
			const capacitor = plain.add('Capacitor', 40 + stage * 40, 40)
			plain.setValue(resistor, '1k')
			plain.setValue(capacitor, '1u')
			plain.connect(previous.id, previous.pin, resistor, 'a')
			plain.connect(resistor, 'b', capacitor, 'a')
			plain.connect(capacitor, 'b', battery, '-')
			previous = { id: resistor, pin: 'b' }
		}

		timeFrames(watched, 30)
		timeFrames(plain, 30)
		const withProbe = Math.min(timeFrames(watched, 60), timeFrames(watched, 60)) / 60
		const without = Math.min(timeFrames(plain, 60), timeFrames(plain, 60)) / 60

		// Generous, because this is one wall clock measuring two things that are
		// nearly the same: the claim is that recording is not a *fraction* of
		// the cost, not that it is free to the last microsecond.
		expect(
			withProbe,
			`${withProbe.toFixed(2)} ms/frame watched against ${without.toFixed(2)} ms/frame plain`,
		).toBeLessThan(without * 1.5)
	}, 60_000)

	it('spends a small fraction of the frame reading the circuit back', async () => {
		// C-24 reads the whole structure out of the core on every frame rather
		// than being told when it changed, which is what makes the canvas a
		// projection that cannot go stale. This is the price of that, and the
		// place it would show up if it ever stopped being small.
		//
		// A share of the solve rather than a number of milliseconds: both move
		// together with the machine, and it is the share that decides whether
		// this was a good trade.
		const circuit = await ladder()
		timeFrames(circuit, 30)
		for (let warm = 0; warm < 30; warm++) {
			circuit.elements()
			circuit.nets()
		}

		const solving = Math.min(timeFrames(circuit, 60), timeFrames(circuit, 60)) / 60
		let reading = Number.POSITIVE_INFINITY
		for (let attempt = 0; attempt < 3; attempt++) {
			const started = performance.now()
			for (let frame = 0; frame < 60; frame++) {
				JSON.stringify([circuit.elements(), circuit.nets()])
			}
			reading = Math.min(reading, (performance.now() - started) / 60)
		}

		// Measured at 4% of the solve for this ladder on 2026-09-21 — 0.40 ms
		// of reading against 9.2 ms of solving, ninety-nine elements and
		// fifty-one nets, which is far more of both than anyone will draw. A
		// fifth is the line: comfortably above the noise between runs, and low
		// enough that a change which made reading the structure expensive
		// could not slip past.
		expect(
			reading,
			`${reading.toFixed(2)} ms/frame reading against ${solving.toFixed(2)} ms/frame solving`,
		).toBeLessThan(solving / 5)
	}, 60_000)
})
