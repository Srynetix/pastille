// SPDX-License-Identifier: MIT OR Apache-2.0

import { mount } from 'svelte'
import './fonts.css'
import './app.css'
import App from './App.svelte'
import { DESKTOP } from './lib/desktop'

const app = mount(App, {
	target: document.getElementById('app')!,
})

/**
 * Registers the offline shell (C-27).
 *
 * Production only: the worker is emitted by the build, and one in the dev
 * server would serve yesterday's app over the top of the one being edited.
 *
 * Failure is swallowed on purpose. Not being installable is not a reason to
 * refuse to run — the app works over the network exactly as it did before —
 * and a console error nobody can act on is the kind of noise M-21 cleared out.
 *
 * Not in the desktop shell, whose files are on disk already (ADR 0013).
 */
if (import.meta.env.PROD && !DESKTOP && 'serviceWorker' in navigator) {
	window.addEventListener('load', () => {
		navigator.serviceWorker.register(`${import.meta.env.BASE_URL}sw.js`).catch(() => {})
	})
}

export default app
