// SPDX-License-Identifier: MIT OR Apache-2.0

// The desktop shell's side of a link, with the shell stood in for. Whether this
// is the shell is decided when the module loads, so it is loaded here under a
// stand-in that says yes. The browser's side — the link left alone — is the
// Inspector's test in shell.test.ts, which loads the real one.

import { afterEach, describe, expect, it, vi } from 'vitest'

const invoke = vi.fn()

function link(): HTMLAnchorElement {
	const anchor = document.createElement('a')
	anchor.href = 'https://www.vishay.com/docs/81857/1n4148.pdf'
	anchor.target = '_blank'
	document.body.append(anchor)
	return anchor
}

async function load() {
	vi.resetModules()
	vi.doMock('@tauri-apps/api/core', () => ({ invoke, isTauri: () => true }))
	return import('./desktop')
}

afterEach(() => {
	invoke.mockReset()
	vi.doUnmock('@tauri-apps/api/core')
	document.body.replaceChildren()
})

describe('a link to a page of its own', () => {
	it('goes to the system browser in the shell, and is not followed', async () => {
		const { openOutside } = await load()
		const anchor = link()
		anchor.addEventListener('click', openOutside)
		const click = new MouseEvent('click', { cancelable: true })

		anchor.dispatchEvent(click)

		expect(click.defaultPrevented).toBe(true)
		expect(invoke).toHaveBeenCalledWith('open_link', { url: anchor.href })
	})
})
