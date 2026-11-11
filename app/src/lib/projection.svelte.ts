// SPDX-License-Identifier: MIT OR Apache-2.0

// What the app draws, read back from the circuit rather than kept beside it.
//
// C-24. The netlist is the stable data model and every other subsystem is a
// consumer of it (design doc 7.1.1); the canvas is one of those consumers, and
// this is the seam. Nothing here decides anything about a circuit — it asks.

import type { Circuit, ElementView, NetView, SolutionView } from './core'
import { SerialLog } from './serial.svelte'

/**
 * The circuit as the app currently sees it.
 *
 * It does hold arrays of elements and nets, because a DOM cannot be drawn from
 * a promise of one. What makes them a projection rather than a second source of
 * truth is that **nothing writes to them**: they are re-read from the circuit on
 * every frame, so anything that edits the netlist — a handler in the shell, a
 * test, a feature written next year that forgets this file exists — is followed
 * without being asked to say so.
 *
 * That is the whole of the change C-24 made. What was here before was the same
 * two arrays kept current by a `refresh()` call after every mutation, at
 * sixteen call sites: correct exactly as long as nobody forgot one, which is
 * the definition of a second source of truth that happens to agree.
 *
 * ### What it costs
 *
 * Reading the structure back is 0.40 ms a frame for the fifty-node ladder
 * C-15 measures — 4% of that circuit's 9.2 ms solve, and under 3% of a
 * sixty-hertz frame. A beginner's ten-element circuit is a twentieth of that.
 * The bench pins the share, so if it ever stops being small it says so rather
 * than being discovered.
 *
 * The *publishing* is what is guarded, not the reading: a fresh structure is
 * handed on only when it differs from the last one (0.04 ms to find out), so
 * the inspector and everything else downstream still wake only for real edits
 * rather than sixty times a second.
 */
export class Projection {
	readonly circuit: Circuit

	/** Everything placed, as the core describes it. Never written to. */
	elements = $state<ElementView[]>([])
	/** Every net, likewise. */
	nets = $state<NetView[]>([])
	/** The last solved state, which is new every frame by nature. */
	solution = $state<SolutionView | undefined>()
	/**
	 * The temperature of the room the circuit is in, in °C (C-74). Part of
	 * the structure rather than the solution: a circuit that fails to solve
	 * is still in a room.
	 */
	room = $state(25)
	/** What the circuit is, in its author's words, or empty. Structure too. */
	description = $state('')
	/**
	 * What each Uno's serial monitor has shown (A-06): read from the circuit
	 * every frame like the solution, but kept, since the core hands over only
	 * what is new.
	 */
	readonly serial = new SerialLog()

	/**
	 * How many times the structure has actually changed.
	 *
	 * The signal for anything that should happen per *edit* rather than per
	 * frame — the autosave is the one that exists. A counter rather than a
	 * callback, so a consumer subscribes by reading it like anything else.
	 */
	edits = $state(0)

	/** The last structure published, as text, for telling a change from a frame. */
	#published = ''

	constructor(circuit: Circuit) {
		this.circuit = circuit
		// Once up front: the first paint happens before the first frame, and a
		// canvas that is blank until then is a canvas that flickers on load.
		this.project()
	}

	/**
	 * Moves the simulation on by `seconds` and re-reads everything.
	 *
	 * One call per frame from the shell's loop. The order matters: the circuit
	 * is advanced first, so the solution and the structure handed on afterwards
	 * describe the same moment.
	 */
	advance(seconds: number): void {
		this.circuit.advance(seconds)
		this.solution = this.circuit.state()
		this.serial.drain(this.circuit, this.solution)
		this.project()
	}

	/**
	 * Re-reads the structure, and publishes it if it is not what was published
	 * last.
	 *
	 * Returns whether anything changed, for a caller that wants to know without
	 * watching [`edits`](Projection#edits).
	 */
	project(): boolean {
		const elements = this.circuit.elements()
		const nets = this.circuit.nets()
		const room = this.circuit.roomCelsius()
		const description = this.circuit.description()
		// A move on the breadboard changes none of the above (B-14), and the
		// board view has to know of one, and of its undo.
		const board = this.circuit.boardRevision()

		// One string for all of it, because half a structure is not a thing
		// that can change on its own: a wire arrives when the element it joins
		// does.
		const published = JSON.stringify([elements, nets, room, description, board])
		if (published === this.#published) return false

		this.#published = published
		this.elements = elements
		this.nets = nets
		this.room = room
		this.description = description
		this.edits += 1
		return true
	}
}
