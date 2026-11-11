// SPDX-License-Identifier: MIT OR Apache-2.0

// Getting a circuit out of the app and back in.
//
// C-29. The core decides what a saved circuit *is* (C-04); this decides how it
// leaves the browser and how it comes back, which is browser plumbing and
// nothing else.
//
// Autosave lives here too, because it is the same text going to a different
// place. Local storage rather than a file: it is not a save the user asked
// for, so it must not put a file in their Downloads folder, and it must never
// be the only copy — which is why the export button exists beside it.

import { DESKTOP, openOnDesktop, openProgramOnDesktop, saveOnDesktop } from './desktop'

/** Where the last circuit is kept between visits. */
const AUTOSAVE = 'pastille:circuit'

/**
 * Where the last circuit goes when this build cannot open it (C-98) — saved by
 * a newer version, or written half way — so the autosave that follows does
 * not overwrite the only copy.
 */
const SET_ASIDE = 'pastille:circuit-unopened'

/** What a saved circuit is called when it lands in a Downloads folder. */
export const FILENAME = 'circuit.json'

/**
 * Hands the browser a file to save — or, in the desktop shell, asks where to
 * put it (ADR 0013).
 *
 * An object URL rather than a data URL: a circuit of any size would outgrow
 * what some browsers accept in an address, and the URL is revoked immediately
 * so nothing is left holding the text.
 */
export async function download(
	text: string,
	name: string = FILENAME,
	// What it is, when it is not a circuit (B-11): its media type for a
	// browser, and what the desktop's dialog calls it.
	as: { type: string; kind: string; extension: string } = {
		type: 'application/json',
		kind: 'Circuit',
		extension: 'json',
	},
): Promise<void> {
	if (DESKTOP) {
		await saveOnDesktop(text, name, as.type === 'application/json' ? undefined : as)
		return
	}
	const url = URL.createObjectURL(new Blob([text], { type: as.type }))
	const link = document.createElement('a')
	link.href = url
	link.download = name
	link.click()
	URL.revokeObjectURL(url)
}

/**
 * Remembers the circuit for next time.
 *
 * Guarded, like everything else that touches local storage: it throws outright
 * in private browsing, and losing an autosave is a smaller failure than a
 * canvas that will not draw.
 */
export function rememberCircuit(text: string): void {
	try {
		localStorage.setItem(AUTOSAVE, text)
	} catch {
		// Storage is full, blocked or disabled. The export button still works,
		// and it is the copy that was actually asked for.
	}
}

/** The circuit from last time, if there is one. */
export function recallCircuit(): string | undefined {
	try {
		return localStorage.getItem(AUTOSAVE) ?? undefined
	} catch {
		return undefined
	}
}

/**
 * Keeps a circuit this build could not open, apart from the autosave, so it
 * can still be downloaded and opened by a version that reads it.
 */
export function setCircuitAside(text: string): void {
	try {
		localStorage.setItem(SET_ASIDE, text)
	} catch {
		// Nowhere to keep it. The note still says what happened.
	}
}

/**
 * The circuit set aside, if there is one — read back at start, so the offer
 * of it outlasts the visit it was made in.
 */
export function recallSetAside(): string | undefined {
	try {
		return localStorage.getItem(SET_ASIDE) ?? undefined
	} catch {
		return undefined
	}
}

/** Lets go of it, once it has been downloaded or the user has said to. */
export function forgetSetAside(): void {
	try {
		localStorage.removeItem(SET_ASIDE)
	} catch {
		// Nothing here is worth failing over.
	}
}

/** Forgets it — after a load that could not be made sense of. */
export function forgetCircuit(): void {
	try {
		localStorage.removeItem(AUTOSAVE)
	} catch {
		// Nothing here is worth failing over.
	}
}

/**
 * Asks for a file and reads it, or resolves to nothing if the user changes
 * their mind.
 *
 * The input is never added to the document: it exists for the length of one
 * click, which keeps a hidden control out of the tab order and out of the way
 * of a screen reader.
 */
export function openFile(): Promise<string | undefined> {
	if (DESKTOP) return openOnDesktop()
	return new Promise((resolve) => {
		const input = document.createElement('input')
		input.type = 'file'
		input.accept = 'application/json,.json'

		input.onchange = async () => {
			const file = input.files?.[0]
			resolve(file ? await file.text() : undefined)
		}
		// A cancelled picker fires nothing in older browsers, so the promise
		// would hang forever. `cancel` is the modern answer and harmless where
		// it is not fired.
		input.oncancel = () => resolve(undefined)

		input.click()
	})
}

/**
 * Asks for a program to load into an Uno (A-03) — the `.hex` the Arduino IDE
 * exports with Sketch → Export Compiled Binary — and reads it: its file name
 * and its text, or nothing if the picker was dismissed.
 */
export function openProgram(): Promise<{ name: string; text: string } | undefined> {
	if (DESKTOP) return openProgramOnDesktop()
	return new Promise((resolve) => {
		const input = document.createElement('input')
		input.type = 'file'
		input.accept = '.hex,text/plain'
		input.onchange = async () => {
			const file = input.files?.[0]
			resolve(file ? { name: file.name, text: await file.text() } : undefined)
		}
		input.oncancel = () => resolve(undefined)
		input.click()
	})
}
