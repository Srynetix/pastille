// SPDX-License-Identifier: MIT OR Apache-2.0

// Seam #1 of the design doc says the app talks to the core through one module.
// `app/src/lib/core.ts` is that module; anything else reaching into the
// generated wasm glue dissolves the seam quietly, and by the time C-24 notices,
// the imports are everywhere.
//
// A single script rather than an ESLint setup: one rule does not pay for a
// plugin chain, and this runs in milliseconds. If the project grows a real lint
// config, move the rule there and delete this.
//
// Import paths are resolved rather than matched, because the tempting import is
// the relative one — `./wasm/pastille_core.js` from inside lib/ — which no
// pattern containing "lib/wasm" would catch.

import { readdir, readFile } from 'node:fs/promises'
import { dirname, join, relative, resolve } from 'node:path'

const ROOT = new URL('../app/src', import.meta.url).pathname
const WASM_DIRECTORY = join(ROOT, 'lib/wasm')
const GATEKEEPER = 'lib/core.ts'
const IMPORT = /(?:from|import)\s*\(?\s*['"]([^'"]+)['"]/g

async function* sourceFiles(directory) {
	for (const entry of await readdir(directory, { withFileTypes: true })) {
		const path = join(directory, entry.name)
		if (entry.isDirectory()) {
			// The generated package imports itself; it is not ours to police.
			if (path === WASM_DIRECTORY) continue
			yield* sourceFiles(path)
		} else if (/\.(ts|svelte)$/.test(entry.name)) {
			yield path
		}
	}
}

function reachesIntoWasm(specifier, fromFile) {
	if (!specifier.startsWith('.')) return false
	return resolve(dirname(fromFile), specifier).startsWith(WASM_DIRECTORY)
}

const offenders = []
for await (const path of sourceFiles(ROOT)) {
	const relativePath = relative(ROOT, path)
	if (relativePath === GATEKEEPER) continue

	const source = await readFile(path, 'utf8')
	for (const [, specifier] of source.matchAll(IMPORT)) {
		if (reachesIntoWasm(specifier, path)) {
			offenders.push(`app/src/${relativePath} → ${specifier}`)
		}
	}
}

if (offenders.length > 0) {
	console.error(`Only ${GATEKEEPER} may import the wasm glue. These do too:`)
	for (const offender of offenders) console.error(`  ${offender}`)
	console.error('\nGo through the core module instead (design doc 7.1.1).')
	process.exit(1)
}

console.log(`Import boundary intact: only ${GATEKEEPER} reaches into lib/wasm.`)
