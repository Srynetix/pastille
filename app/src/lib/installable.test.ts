// SPDX-License-Identifier: MIT OR Apache-2.0

// C-27: what makes the app installable, checked where it is served from.
//
// The offline half is `scripts/check-offline.mjs`, which needs a real build and
// a network to take away. This half is the manifest — a file that is never
// imported by anything, so nothing else would notice it going wrong.

import { describe, expect, it } from 'vitest'

async function manifest(): Promise<Record<string, unknown>> {
	const response = await fetch('/manifest.webmanifest')
	expect(response.ok, 'the manifest is served').toBe(true)
	return (await response.json()) as Record<string, unknown>
}

describe('the web app manifest', () => {
	it('says what the app is called and how to start it', async () => {
		const found = await manifest()

		expect(found.name).toBe('Pastille')
		expect(found.short_name).toBeTruthy()
		expect(found.description).toBeTruthy()
		// Relative, so the app installs from wherever it is served: the root of
		// a server, or GitHub Pages' /pastille/ (ADR 0021).
		expect(found.start_url).toBe('./')
		expect(found.scope).toBe('./')
	})

	it('asks to be its own window rather than a tab', async () => {
		// The point of installing it. `browser` would be an install that
		// changes nothing about what the user sees.
		expect((await manifest()).display).toBe('standalone')
	})

	it('opens on the colour the shell is drawn on', async () => {
		// Both, and the same: `background_color` is what fills the window
		// before the first paint, and a white flash into a dark app is the one
		// thing an installed app does that a tab does not.
		const found = await manifest()

		expect(found.background_color).toBe('#14161a')
		expect(found.theme_color).toBe(found.background_color)
	})

	it('carries the icons an install actually asks for', async () => {
		// A 192 and a 512 is what Chrome wants before it will offer to install,
		// and a maskable one is what stops Android cropping the logo's corners
		// off. Fetched rather than trusted: a manifest naming an icon that is
		// not there is a manifest that installs nothing.
		const icons = (await manifest()).icons as { src: string; sizes: string; purpose: string }[]

		for (const wanted of ['192x192', '512x512']) {
			const icon = icons.find((entry) => entry.sizes === wanted && entry.purpose === 'any')
			expect(icon, wanted).toBeDefined()
		}
		expect(icons.some((icon) => icon.purpose === 'maskable')).toBe(true)

		for (const icon of icons) {
			const response = await fetch(icon.src)
			expect(response.ok, icon.src).toBe(true)
			// And it is a picture rather than an empty file or a 404 page the
			// server answered with 200.
			expect((await response.blob()).size, icon.src).toBeGreaterThan(1000)
		}
	})
})

describe('the document', () => {
	it('links the manifest and dresses the window to match', async () => {
		// In `index.html`, which no test imports either. Fetched as text
		// because that is the file a browser is handed on a cold load.
		const html = await (await fetch('/index.html')).text()

		expect(html).toContain('rel="manifest"')
		expect(html).toContain('name="theme-color"')
		expect(html).toContain('rel="apple-touch-icon"')
	})
})
