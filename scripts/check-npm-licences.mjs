// SPDX-License-Identifier: MIT OR Apache-2.0

// ADR 0001's third consequence, on the npm side: every package in the tree is
// under a licence from its allow-list, or the gate fails. `cargo deny` has held
// the Rust side since C-02; this one was promised beside it and never written
// (C-106). A GPL package arriving three levels down is the failure it is for.
//
// The whole tree, development tools included, because the line between what
// ships and what only builds moves without anybody deciding it should: the
// Svelte runtime is a development dependency and is in every bundle. A tool
// whose licence is not on the list but which ships nothing can be named below,
// with the reason, which is the decision written where it is enforced.

import { execFileSync } from 'node:child_process'

/** ADR 0001's list, and the two it implies by being permissive. */
const ALLOWED = new Set([
	'MIT',
	'Apache-2.0',
	'BSD-2-Clause',
	'BSD-3-Clause',
	'ISC',
	'0BSD',
	'Unicode-3.0',
	'Zlib',
])

/** Packages under another licence, by name, and why each is acceptable. */
const EXCEPTIONS = new Map([
	[
		'lightningcss',
		'MPL-2.0: the CSS minifier Vite runs at build time. None of it is in the bundle, and MPL is file-level, so using it changes nothing about our licence.',
	],
	['lightningcss-darwin-arm64', 'MPL-2.0: the native half of lightningcss, above.'],
	[
		'@fontsource-variable/recursive',
		'OFL-1.1: a typeface, shipped unmodified in the bundle with its licence beside it. The OFL lets a font be bundled with software under any licence (ADR 0015).',
	],
	['@fontsource-variable/ibm-plex-sans', 'OFL-1.1: the text typeface, on the same terms as Recursive above (ADR 0015).'],
	[
		'minimatch',
		'BlueOak-1.0.0: a permissive, OSI-approved licence, pulled in by ESLint (ADR 0019). A development tool: none of it is in the bundle.',
	],
])

/** Whether an SPDX expression is satisfied by the allow-list: "A OR B" needs one. */
function allowed(expression) {
	const bare = expression.replace(/[()]/g, '').trim()
	return bare.split(/\s+OR\s+/).some((option) => ALLOWED.has(option.trim()))
}

const report = JSON.parse(
	execFileSync('pnpm', ['licenses', 'list', '--json'], {
		encoding: 'utf8',
		stdio: ['ignore', 'pipe', 'ignore'],
	}),
)

const refused = []
let checked = 0
for (const [licence, packages] of Object.entries(report)) {
	for (const { name } of packages) {
		checked += 1
		if (allowed(licence) || EXCEPTIONS.has(name)) continue
		refused.push(`  ${name}: ${licence}`)
	}
}

if (refused.length > 0) {
	console.error('Packages under a licence ADR 0001 does not allow:')
	console.error(refused.join('\n'))
	console.error('Replace them, or name them in scripts/check-npm-licences.mjs with the reason.')
	process.exit(1)
}
console.log(`npm licences: ${checked} packages, all on ADR 0001's list or named exceptions.`)
