// SPDX-License-Identifier: MIT OR Apache-2.0

// Renders the app's installed icons from `favicon.svg` (C-27).
//
// A manifest icon has to be a raster: Safari needs a PNG for the home screen,
// and Chrome's install prompt has never committed to SVG. Rather than keep two
// drawings that have to be edited together, the PNGs are generated from the one
// SVG and committed beside it — run this again when the logo changes.
//
// Rendered in the same Chromium the tests run in, because the logo is built out
// of gaussian-blur filters that ImageMagick's own SVG renderer gets wrong. The
// maskable one is inset into a filled square: Android crops a maskable icon to
// whatever shape the launcher likes, and a logo drawn edge to edge loses its
// corners.

import { readFile, writeFile } from 'node:fs/promises'
import { createRequire } from 'node:module'

// Resolved against the app's package rather than this script's: playwright is
// the app's test dependency, and the workspace root has no node_modules of its
// own to find it in.
const { chromium } = createRequire(new URL('../app/package.json', import.meta.url))(
	'playwright',
)

const PUBLIC = new URL('../app/public/', import.meta.url)

/** Background for the maskable icon: the app's dark surface, not transparent. */
const BACKGROUND = '#14161a'

/** How much of a maskable icon must survive the crop (the safe zone is 80%). */
const INSET = 0.72

const ICONS = [
	{ file: 'icon-192.png', size: 192, maskable: false },
	{ file: 'icon-512.png', size: 512, maskable: false },
	{ file: 'icon-maskable-512.png', size: 512, maskable: true },
]

const logo = await readFile(new URL('favicon.svg', PUBLIC), 'utf8')
const browser = await chromium.launch()

try {
	for (const { file, size, maskable } of ICONS) {
		const page = await browser.newPage({
			viewport: { width: size, height: size },
			deviceScaleFactor: 1,
		})
		const scale = maskable ? INSET : 1
		await page.setContent(
			`<!doctype html><meta charset="utf-8">
			<style>
				html, body { margin: 0; width: ${size}px; height: ${size}px; }
				body {
					background: ${maskable ? BACKGROUND : 'transparent'};
					display: grid;
					place-items: center;
				}
				svg { width: ${Math.round(size * scale)}px; height: auto; }
			</style>
			${logo}`,
			{ waitUntil: 'load' },
		)
		const png = await page.screenshot({ omitBackground: !maskable })
		await writeFile(new URL(file, PUBLIC), png)
		await page.close()
		console.log(`  ${file}  ${size}×${size}${maskable ? ' maskable' : ''}`)
	}
} finally {
	await browser.close()
}
