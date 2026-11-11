// SPDX-License-Identifier: MIT OR Apache-2.0

// ADR 0001's fourth consequence: every source file carries the SPDX line. It
// was a rule and not a check, and fourteen files had gone without it by the
// time C-106 looked. Tracked files only, so nothing generated or vendored is
// asked for one, and in the first three lines, so a shebang or a Svelte file's
// opening comment can come first.

import { execFileSync } from 'node:child_process'
import { readFileSync } from 'node:fs'

const LINE = 'SPDX-License-Identifier: MIT OR Apache-2.0'
const SOURCE = /\.(rs|ts|js|mjs|svelte)$/

const tracked = execFileSync('git', ['ls-files'], { encoding: 'utf8' })
	.split('\n')
	.filter((path) => SOURCE.test(path))

const missing = tracked.filter((path) => {
	const head = readFileSync(path, 'utf8').split('\n', 3).join('\n')
	return !head.includes(LINE)
})

if (missing.length > 0) {
	console.error(`Source files without "${LINE}" in their first lines:`)
	console.error(missing.map((path) => `  ${path}`).join('\n'))
	process.exit(1)
}
console.log(`SPDX: ${tracked.length} source files, all with the header.`)
