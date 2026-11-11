// SPDX-License-Identifier: MIT OR Apache-2.0

// The only module in the app that touches the wasm-bindgen glue. Everything
// else talks to the core through here, so the boundary stays one file wide
// (design doc 7.1).

import init, {
	accuracy_note,
	Circuit,
	components,
	core_version,
	echo_probe,
	pin_names,
	provoke_panic,
	probe_with_validation,
	SolverFrame,
	type BillLine,
	type BoardPart,
	type BoardView,
	type BoundaryProbe,
	type ComponentKind,
	type ComponentView,
	type ElementHeat,
	type ElementSerial,
	type ElementView,
	type Failure,
	type FailureCode,
	type InitOutput,
	type NetCurrent,
	type NetView,
	type PinCurrent,
	type PinView,
	type Purchase,
	type Replacement,
	type SolutionView,
	type Steps,
} from './wasm/pastille_core.js'

export type {
	BillLine,
	BoardPart,
	BoardView,
	BoundaryProbe,
	ComponentKind,
	ComponentView,
	ElementHeat,
	ElementSerial,
	ElementView,
	Failure,
	FailureCode,
	NetCurrent,
	NetView,
	PinCurrent,
	PinView,
	Purchase,
	Replacement,
	SolutionView,
	Steps,
}
export { SolverFrame }

/**
 * Creates an empty circuit.
 *
 * The circuit lives in the core — netlist and layout both (ADR 0006). The app
 * holds a handle and asks it what to draw; it never keeps its own copy, because
 * a second copy is a second source of truth (design doc 7.1.1).
 */
export async function createCircuit(): Promise<Circuit> {
	await initCore()
	return new Circuit()
}

export type { Circuit }

let instantiation: Promise<InitOutput> | null = null
let wasm: InitOutput | null = null

/** Loads the wasm core. Concurrent and repeated callers share one instantiation. */
export function initCore(): Promise<InitOutput> {
	instantiation ??= init().then((output) => {
		wasm = output
		return output
	})
	return instantiation
}

/** Version of the Rust core. Proof, for now, that the bundle really loaded. */
export async function coreVersion(): Promise<string> {
	await initCore()
	return core_version()
}

/**
 * What the core says this simulation is (C-26).
 *
 * The app does not display this — it keeps its own copy in `lib/accuracy.ts`,
 * because the one sentence that must always be on screen cannot wait for a
 * bundle to load. This is here so a test can hold the two together, the same
 * way `pinNames` holds the symbols to the core's pins.
 */
export async function accuracyNote(): Promise<string> {
	await initCore()
	return accuracy_note()
}

// --- Control plane: structured values, low rate, ergonomics first ------------

export async function echoProbe(probe: BoundaryProbe, gain: number): Promise<BoundaryProbe> {
	await initCore()
	return echo_probe(probe, gain)
}

/** Throws if the probe is empty — core errors surface as exceptions, not sentinels. */
export async function validateProbe(probe: BoundaryProbe): Promise<BoundaryProbe> {
	await initCore()
	return probe_with_validation(probe)
}

/**
 * Every kind of component the core knows, in the order the palette should
 * offer them.
 *
 * The registry of C-16. The app keeps its own tables of what a component looks
 * like (`symbols.ts`) and what it is called (`palette.ts`), because a polyline
 * and a sentence of beginner English are not facts about a resistor — ADR 0008.
 * What stops those tables drifting from this one is the tests that compare
 * them, which is the same mechanism that has held the pin names together since
 * M-17.
 */
export async function componentRegistry(): Promise<ComponentView[]> {
	await initCore()
	return components()
}

/**
 * Panics inside the core on purpose, so C-31's hook can be shown to work.
 *
 * Here rather than reached for directly, because only this module may touch
 * the wasm glue (design doc 7.1.1) — a rule `just boundary` enforces, and which
 * caught the test that first went around it.
 */
export async function provokePanic(): Promise<void> {
	await initCore()
	provoke_panic()
}

/**
 * Pin names of an element kind, in pin order, as the core defines them.
 *
 * The symbols in `lib/symbols.ts` have to name their pins the same way or wiring
 * connects nothing. Asking the core rather than keeping a second table here is
 * what makes that a test failure instead of a silent one.
 */
export async function pinNames(kind: string): Promise<string[]> {
	await initCore()
	return pin_names(kind)
}

// --- Data plane: zero-copy views, frame rate, allocation-free ----------------

/**
 * Views a solver frame's voltages in place, without copying.
 *
 * Call this every time the values are read and never store the result: any
 * allocation on the Rust side can grow wasm memory, which detaches every
 * existing view over it. The view is cheap; a detached one is a silent bug.
 */
export function viewFrame(frame: SolverFrame): Float64Array {
	if (wasm === null) {
		throw new Error('viewFrame called before initCore resolved')
	}
	return new Float64Array(wasm.memory.buffer, frame.ptr, frame.len)
}

/**
 * Views the block `plotTrace` just filled, without copying it (C-22).
 *
 * Same rule as `viewFrame`, and it matters more here: this runs once per trace
 * per frame. Take the view immediately after the call that filled the block and
 * read it before calling anything else on the core — filling can grow wasm
 * memory, and a view taken over the old buffer is silently detached.
 */
export function viewPlot(circuit: Circuit, length: number): Float64Array {
	if (wasm === null) {
		throw new Error('viewPlot called before initCore resolved')
	}
	return new Float64Array(wasm.memory.buffer, circuit.plotPtr, length)
}
