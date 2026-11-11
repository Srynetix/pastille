// SPDX-License-Identifier: MIT OR Apache-2.0

// C-31. A Rust panic across the wasm boundary used to arrive as
// `unreachable executed` — no message, no location, nothing to tell a dropped
// invariant from a bug in the browser.
//
// Testing a panic hook needs a panic, which is why the core exports one in
// debug builds. A test that only asserted the hook was installed would never
// ask what it does.

import { afterEach, describe, expect, it, vi } from 'vitest'

import { provokePanic } from './core'

afterEach(() => vi.restoreAllMocks())

/** Calls the core's deliberate panic and returns what reached the console. */
async function provoke(): Promise<string> {
	const errors: string[] = []
	vi.spyOn(console, 'error').mockImplementation((...parts: unknown[]) => {
		errors.push(parts.map(String).join(' '))
	})

	await expect(provokePanic()).rejects.toThrow()

	return errors.join('\n')
}

describe('a panic in the core', () => {
	it('says what went wrong', async () => {
		const reported = await provoke()

		expect(reported).toContain('a deliberate panic, to prove the hook reports one')
	})

	it('says where it went wrong', async () => {
		// The file and line, which is the half that turns "something panicked"
		// into somewhere to look.
		const reported = await provoke()

		expect(reported).toContain('lib.rs')
	})

	it('does not merely say unreachable', async () => {
		const reported = await provoke()

		expect(reported).not.toMatch(/^unreachable/)
	})
})
