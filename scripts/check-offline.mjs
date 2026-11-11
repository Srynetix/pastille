// SPDX-License-Identifier: MIT OR Apache-2.0

// C-27's acceptance, run rather than asserted: the app is visited once with a
// network, the network is taken away, and it has to open and simulate anyway.
//
// Not a vitest case, because the thing under test is the *build* — a service
// worker only exists in the production bundle, and it caches files by the
// hashed names only a real build produces. So this serves `app/dist` the way a
// web server would, drives a real browser at it, and then switches the network
// off underneath it.
//
// The circuit is planted in localStorage rather than drawn with the pointer:
// what is being tested is that the wasm core loads and solves with nothing to
// fetch, and C-29's "put back the last circuit" gets one in front of the solver
// in one line instead of twenty of synthetic drag events.

import { createReadStream } from 'node:fs'
import { stat } from 'node:fs/promises'
import { createServer } from 'node:http'
import { createRequire } from 'node:module'
import { extname, join, normalize } from 'node:path'

const { chromium } = createRequire(new URL('../app/package.json', import.meta.url))('playwright')

const DIST = new URL('../app/dist/', import.meta.url).pathname

/** Battery, 220 Ω, LED, wired into a loop — §10's circuit, as a saved file. */
const MILESTONE = JSON.stringify({
	schema: 1,
	elements: [
		{ kind: { DcSource: { volts: 9, series_ohms: 0 } }, x: 100, y: 100, turns: 0 },
		{ kind: { Resistor: { ohms: 220 } }, x: 200, y: 100, turns: 0 },
		{ kind: { Led: { colour: 'Red' } }, x: 300, y: 100, turns: 0 },
	],
	nets: [
		{ pins: [{ element: 0, pin: '+' }, { element: 1, pin: 'a' }] },
		{ pins: [{ element: 1, pin: 'b' }, { element: 2, pin: 'anode' }] },
		{ pins: [{ element: 2, pin: 'cathode' }, { element: 0, pin: '-' }] },
	],
})

const TYPES = {
	'.html': 'text/html; charset=utf-8',
	'.js': 'text/javascript; charset=utf-8',
	'.css': 'text/css; charset=utf-8',
	'.svg': 'image/svg+xml',
	'.png': 'image/png',
	// Wrong here and the browser refuses to compile it while streaming, which
	// is a failure that looks nothing like its cause.
	'.wasm': 'application/wasm',
	'.webmanifest': 'application/manifest+json',
	'.json': 'application/json',
	'.woff2': 'font/woff2',
	'.txt': 'text/plain; charset=utf-8',
}

/**
 * Where the build is served from: the root, unless it was built for GitHub
 * Pages, which serves it under /pastille/ (ADR 0021). Anything outside it is
 * not found, as on Pages, so a path the build forgot to make relative fails
 * here rather than in front of a visitor.
 */
const BASE = process.env.PASTILLE_BASE ?? '/'

const server = createServer(async (request, response) => {
	const asked = normalize(decodeURI(new URL(request.url, 'http://localhost').pathname))
	const path = asked.startsWith(BASE) ? `/${asked.slice(BASE.length)}` : undefined
	const file = path === undefined ? '' : join(DIST, path === '/' ? 'index.html' : path)

	try {
		if (!file || !file.startsWith(DIST) || !(await stat(file)).isFile()) throw new Error('not a file')
	} catch {
		response.writeHead(404).end('not found')
		return
	}

	response.writeHead(200, { 'content-type': TYPES[extname(file)] ?? 'application/octet-stream' })
	createReadStream(file).pipe(response)
})

await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve))
const origin = `http://127.0.0.1:${server.address().port}`

const browser = await chromium.launch()
const context = await browser.newContext()
const page = await context.newPage()
const problems = []
page.on('console', (message) => {
	if (message.type() === 'error') problems.push(message.text())
})
page.on('pageerror', (error) => problems.push(String(error)))

function fail(what) {
	console.error(`  ✗ ${what}`)
	process.exitCode = 1
}

try {
	// First visit, with a network. This is the one that installs the worker.
	await page.goto(`${origin}${BASE}`, { waitUntil: 'load' })
	await page.evaluate((circuit) => localStorage.setItem('pastille:circuit', circuit), MILESTONE)
	await page.waitForFunction(() => navigator.serviceWorker.controller !== null, null, {
		timeout: 15_000,
	})

	const cached = await page.evaluate(async () => {
		const names = await caches.keys()
		const cache = await caches.open(names[0])
		return {
			names,
			files: (await cache.keys()).map((request) => new URL(request.url).pathname).sort(),
		}
	})
	if (cached.names.length !== 1) fail(`expected one cache, found ${cached.names.join(', ')}`)
	if (!cached.files.some((file) => file.endsWith('.wasm'))) {
		fail('the wasm bundle is not in the cache')
	}
	console.log(`  cached ${cached.files.length} files as ${cached.names[0]}`)

	// The first visit is held to M-21's standard too: a console error on the
	// load that installs the worker is a worse problem than anything below.
	if (problems.length > 0) fail(`the first load logged: ${problems.join(' | ')}`)

	// And now there is no network at all.
	await context.setOffline(true)
	problems.length = 0
	await page.reload({ waitUntil: 'load' })

	// It opened: the shell is there, drawn from a document nothing fetched.
	await page.waitForSelector('[aria-label="Components"]', { timeout: 15_000 })

	// It is simulating: the clock only runs once the wasm core has loaded,
	// instantiated and solved the circuit that was put in front of it.
	await page.waitForFunction(
		() => /\d+\.\d s simulated/.test(document.querySelector('footer')?.textContent ?? ''),
		null,
		{ timeout: 15_000 },
	)
	const said = await page.evaluate(() => document.querySelector('footer')?.textContent?.trim())
	console.log(`  offline, the app says: ${said}`)

	// The typefaces came back too (C-126, ADR 0015): a font missing offline
	// is not an error anyone sees, only an app quietly set in the fallback.
	const fonts = await page.evaluate(async () => {
		const wanted = ['16px "Recursive Variable"', '16px "IBM Plex Sans Variable"']
		const loaded = await Promise.all(wanted.map((font) => document.fonts.load(font, '220 Ω')))
		return wanted.filter((_, index) => loaded[index].length === 0)
	})
	if (fonts.length > 0) fail(`offline, these typefaces did not load: ${fonts.join(', ')}`)
	else console.log('  offline, both typefaces load')

	if (problems.length > 0) fail(`the offline load logged: ${problems.join(' | ')}`)

	if (process.exitCode) console.error('\n  Offline check failed.')
	else console.log('\n  ✓ installable, and it opens and simulates with the network gone')
} catch (error) {
	// A timeout here is the interesting failure and deserves a sentence rather
	// than a stack: it means the app did not come back without a network, which
	// is the whole of what this checks.
	fail(`it did not come back offline: ${error.message.split('\n')[0]}`)
	console.error('\n  Offline check failed.')
} finally {
	await browser.close()
	server.close()
}
