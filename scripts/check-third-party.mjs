// SPDX-License-Identifier: MIT OR Apache-2.0

// Every MPL-2.0 crate the desktop build links is named in its notice, at the
// version it links, and nothing else is (ADR 0013).
//
// The MPL's one obligation on a build that links an unmodified file is to say
// the file is there and where its source is. That is `desktop/THIRD-PARTY.md`,
// and a notice is only true until the next `cargo update` — so it is checked
// against the lockfile's own answer rather than trusted.

import { execFileSync } from 'node:child_process'
import { readFileSync } from 'node:fs'

const desktop = new URL('../desktop/', import.meta.url)

const listed = JSON.parse(
	execFileSync('cargo', ['deny', 'list', '--format', 'json', '--layout', 'crate'], {
		cwd: desktop,
		encoding: 'utf8',
		stdio: ['ignore', 'pipe', 'inherit'],
	}),
)

// A crate that offers MPL-2.0 *or* something permissive is taken under the
// permissive one; only a crate with no other choice has to be named.
const linked = Object.entries(listed)
	.filter(([, { licenses }]) => licenses.includes('MPL-2.0'))
	.filter(([, { licenses }]) => licenses.every((licence) => licence === 'MPL-2.0'))
	.map(([key]) => key.split(' ').slice(0, 2).join(' '))
	.sort()

const notice = readFileSync(new URL('THIRD-PARTY.md', desktop), 'utf8')
const named = [...notice.matchAll(/^\| ([a-z0-9_-]+) \| ([0-9][^ |]*) \|/gm)]
	.map(([, name, version]) => `${name} ${version}`)
	.sort()

const missing = linked.filter((crate) => !named.includes(crate))
const stale = named.filter((crate) => !linked.includes(crate))

if (missing.length || stale.length) {
	for (const crate of missing) console.error(`  linked but not in THIRD-PARTY.md: ${crate}`)
	for (const crate of stale) console.error(`  in THIRD-PARTY.md but not linked: ${crate}`)
	process.exit(1)
}
console.log(`  ✓ THIRD-PARTY.md names the ${linked.length} MPL-2.0 crates the desktop build links`)
