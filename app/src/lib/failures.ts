// SPDX-License-Identifier: MIT OR Apache-2.0

// Why a circuit did not solve, in words its author can act on.
//
// C-25. The solver's own sentence for the commonest beginner mistake — a
// battery wired straight to an LED — is "matrix is singular at column 1". True,
// and useless: it names a column of a matrix the user has never seen, in a
// circuit they can see perfectly well.
//
// Reactive help, in the sense of §14: the app noticed something and speaks. The
// rule for that kind is that it waits until it has something specific to say,
// which is why these name the thing to go and look at rather than describing
// the failure.
//
// The table is keyed by the core's `FailureCode`, so TypeScript refuses to
// compile a new failure mode that nobody has written words for. The core keeps
// no prose of its own (ADR 0008); it says which kind of failure it was and this
// says what that means.

import type { Failure, FailureCode } from './core'

const EXPLANATIONS: Record<FailureCode, string> = {
	'nothing-wired':
		'Nothing is joined up yet — drag from one pin to another to wire two parts together.',
	'no-limit':
		'Something here has nothing to limit the current. Look for a wire straight from + to −, or an LED with no resistor in front of it.',
	unsettled:
		'This circuit will not settle. It is probably asking for far more current than anything real could carry.',
	internal: 'The simulator ran into a problem of its own. Nothing you did caused it.',
}

/** What to tell the user about a failure, or nothing if there is none. */
export function explain(failure: Failure | undefined): string | undefined {
	return failure && EXPLANATIONS[failure.code]
}

/**
 * The solver's own words, for whoever is debugging the solver.
 *
 * Kept, and kept out of the status line: "matrix is singular at column 1" is
 * the right sentence for exactly one reader, and they are not the audience of
 * §4. It rides along as a tooltip.
 */
export function detailOf(failure: Failure | undefined): string | undefined {
	return failure?.detail
}

/**
 * A refusal from the core — a file that will not open, an edit it would not
 * take — split into the sentence for a person and the detail for whoever
 * asks (C-100). The core writes the sentence on the first line and anything
 * technical after it, so the app can show the one and keep the other for a
 * tooltip.
 */
export function splitRefusal(error: unknown): { sentence: string; detail?: string } {
	const message = error instanceof Error ? error.message : String(error)
	const [sentence, ...rest] = message.split('\n')
	return { sentence, detail: rest.join('\n') || undefined }
}
