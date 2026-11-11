// SPDX-License-Identifier: MIT OR Apache-2.0

// The app, when it is running in the desktop shell rather than a browser.
//
// C-68, ADR 0013. The shell shows this same build, so almost nothing needs to
// know which one it is in. Three things do. A browser saves a file by
// following a download link and the system web view on macOS ignores those, so
// saving and opening go through the shell's own dialogs. A link that asks for a
// tab of its own opens nothing either, so it goes to the system's browser. And
// the offline service worker has nothing to do in a window whose files are
// already on disk.
//
// Everything else — the canvas, the core, autosave — is the browser's code,
// unchanged, and the browser build takes none of these paths.

import { invoke, isTauri } from '@tauri-apps/api/core'

/** Whether this is the desktop shell. Fixed for the life of the page. */
export const DESKTOP: boolean = isTauri()

/** Saves `text` wherever the user says; false if they dismissed the dialog. */
export function saveOnDesktop(
	text: string,
	name: string,
	as?: { kind: string; extension: string },
): Promise<boolean> {
	return invoke<boolean>('save_circuit', { text, name, kind: as?.kind, extension: as?.extension })
}

/** Reads a circuit the user picks, or nothing if they dismissed the dialog. */
export async function openOnDesktop(): Promise<string | undefined> {
	return (await invoke<string | null>('open_circuit')) ?? undefined
}

/** An Uno's program, chosen in the shell's own dialog (A-03): its name and text. */
export async function openProgramOnDesktop(): Promise<{ name: string; text: string } | undefined> {
	const chosen = await invoke<[string, string] | null>('open_program')
	return chosen ? { name: chosen[0], text: chosen[1] } : undefined
}

/**
 * For a link that opens in a tab of its own (`target="_blank"`), as its click
 * handler. In a browser it does nothing and the link is followed as usual; in
 * the shell, which opens no second window, the page goes to the system's
 * browser instead.
 */
export function openOutside(event: MouseEvent): void {
	if (!DESKTOP) return
	const link = event.currentTarget
	if (!(link instanceof HTMLAnchorElement)) return
	event.preventDefault()
	void invoke('open_link', { url: link.href })
}
