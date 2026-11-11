// SPDX-License-Identifier: MIT OR Apache-2.0

// How long a first visit takes before the app works (M-21), on a phone.
//
// M-21 budgets three seconds for the cold load, and until this script nothing
// measured it: ADRs 0004 and 0009 turned things down for its sake on reasoning
// alone. This serves the production build the way a host would — compressed,
// nothing cached, no service worker yet — through a throttled line and a
// throttled processor, and times two moments: the shell drawn, and the core
// ready to simulate. The second is the budget's, because until then the app
// can only say it is loading.
//
// Two lines. "Slow 4G with a 4x slower CPU" is what Lighthouse calls a mobile
// device, and it is the one held to the budget; fast 4G is reported beside it
// so a change that only helps the slow case is visible as one.

import { createReadStream } from 'node:fs'
import { stat } from 'node:fs/promises'
import { createServer } from 'node:http'
import { createRequire } from 'node:module'
import { extname, join, normalize } from 'node:path'
import { createGzip } from 'node:zlib'

const { chromium } = createRequire(new URL('../app/package.json', import.meta.url))('playwright')

const DIST = new URL('../app/dist/', import.meta.url).pathname

/** M-21: milliseconds from the first byte asked for to a core that simulates. */
const BUDGET_MS = 3000

/** Runs per line; the median is reported, so one hiccup does not decide it. */
const RUNS = 3

const LINES = [
	{ name: 'fast 4G', down: 9e6, up: 1.5e6, latency: 85, cpu: 1, held: false },
	{ name: 'slow 4G, 4x slower CPU', down: 1.6e6, up: 0.75e6, latency: 150, cpu: 4, held: true },
]

const TYPES = {
	'.html': 'text/html; charset=utf-8',
	'.js': 'text/javascript; charset=utf-8',
	'.css': 'text/css; charset=utf-8',
	'.svg': 'image/svg+xml',
	'.png': 'image/png',
	'.wasm': 'application/wasm',
	'.webmanifest': 'application/manifest+json',
	'.json': 'application/json',
	'.woff2': 'font/woff2',
	'.txt': 'text/plain; charset=utf-8',
}

/** Already compressed: gzipping them again is work a host would not do. */
const PACKED = new Set(['.woff2', '.png'])

const server = createServer(async (request, response) => {
	const path = normalize(decodeURI(new URL(request.url, 'http://localhost').pathname))
	const file = join(DIST, path === '/' ? 'index.html' : path)

	try {
		if (!file.startsWith(DIST) || !(await stat(file)).isFile()) throw new Error('not a file')
	} catch {
		response.writeHead(404).end('not found')
		return
	}

	const type = extname(file)
	const packed = PACKED.has(type)
	response.writeHead(200, {
		'content-type': TYPES[type] ?? 'application/octet-stream',
		...(packed ? {} : { 'content-encoding': 'gzip' }),
	})
	const body = createReadStream(file)
	;(packed ? body : body.pipe(createGzip())).pipe(response)
})

await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve))
const origin = `http://127.0.0.1:${server.address().port}`
const browser = await chromium.launch()

/** One cold visit over `line`: when the shell was drawn, and when the core was ready. */
async function visit(line) {
	// A fresh context is a first visit: nothing cached, and no worker to
	// answer from a cache it has not filled yet.
	const context = await browser.newContext({ serviceWorkers: 'block' })
	const page = await context.newPage()
	const devtools = await context.newCDPSession(page)
	await devtools.send('Network.enable')
	await devtools.send('Network.setCacheDisabled', { cacheDisabled: true })
	await devtools.send('Network.emulateNetworkConditions', {
		offline: false,
		latency: line.latency,
		downloadThroughput: line.down / 8,
		uploadThroughput: line.up / 8,
	})
	await devtools.send('Emulation.setCPUThrottlingRate', { rate: line.cpu })

	let bytes = 0
	devtools.on('Network.loadingFinished', (event) => (bytes += event.encodedDataLength))

	const start = Date.now()
	try {
		await page.goto(`${origin}/`, { waitUntil: 'commit' })
		await page.waitForSelector('header', { timeout: 30_000 })
		const shell = Date.now() - start
		// The status line's two ready states: the core loaded, or, with a
		// circuit already on the board, already simulating it.
		await page.waitForFunction(
			() => /Core ready|simulated/.test(document.body.textContent ?? ''),
			null,
			{ timeout: 30_000, polling: 50 },
		)
		return { shell, ready: Date.now() - start, bytes }
	} finally {
		await context.close()
	}
}

const seconds = (ms) => `${(ms / 1000).toFixed(2)} s`

try {
	for (const line of LINES) {
		const visits = []
		for (let run = 0; run < RUNS; run++) visits.push(await visit(line))
		visits.sort((a, b) => a.ready - b.ready)
		const { shell, ready, bytes } = visits[Math.floor(RUNS / 2)]

		const verdict = !line.held ? ' ' : ready <= BUDGET_MS ? '✓' : '✗'
		console.log(
			`  ${verdict} ${line.name.padEnd(24)} shell ${seconds(shell)}, core ready ${seconds(ready)}` +
				`, ${Math.round(bytes / 1024)} KiB`,
		)
		if (line.held && ready > BUDGET_MS) {
			console.error(`    over M-21's ${seconds(BUDGET_MS)} by ${seconds(ready - BUDGET_MS)}`)
			process.exitCode = 1
		}
	}
} finally {
	await browser.close()
	server.close()
}
