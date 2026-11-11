// SPDX-License-Identifier: MIT OR Apache-2.0

// Things the app should say once and never again.
//
// A first-time explanation that comes back every session is furniture, and
// furniture is what §14 of the design doc warns about: every sentence added to
// the interface is a sentence the next person has to read past.
//
// Local storage, because the fact is about this browser and nobody else's, and
// because there is nothing to lose if it is wiped — the worst case is being
// told something twice. It can throw outright (Safari in private browsing), so
// every access is guarded and a failure means "not seen yet" rather than a
// broken canvas. Autosaving the circuit is C-29 and wants more than this.

const PREFIX = 'pastille:seen:'

/** Has this been said before? */
export function hasSeen(key: string): boolean {
	try {
		return localStorage.getItem(PREFIX + key) !== null
	} catch {
		return false
	}
}

/** Remember that it has been said. */
export function remember(key: string): void {
	try {
		localStorage.setItem(PREFIX + key, '1')
	} catch {
		// Storage is full, blocked or disabled. Saying it again next time is a
		// smaller failure than refusing to say it now.
	}
}

/** Forget it, so it is said again. For tests, and for whatever resets the app. */
export function forget(key: string): void {
	try {
		localStorage.removeItem(PREFIX + key)
	} catch {
		// As above: nothing here is worth failing over.
	}
}

/** The moving dots on the wires, explained the first time any appear. */
export const CURRENT_OVERLAY = 'current-overlay'
