// SPDX-License-Identifier: MIT OR Apache-2.0

import { beforeEach, describe, expect, it } from 'vitest'

import { CURRENT_OVERLAY, forget, hasSeen, remember } from './once'

beforeEach(() => forget(CURRENT_OVERLAY))

describe('saying something once', () => {
	it('has not been said until it has', () => {
		expect(hasSeen(CURRENT_OVERLAY)).toBe(false)
	})

	it('stays said', () => {
		remember(CURRENT_OVERLAY)

		expect(hasSeen(CURRENT_OVERLAY)).toBe(true)
	})

	it('can be forgotten', () => {
		remember(CURRENT_OVERLAY)
		forget(CURRENT_OVERLAY)

		expect(hasSeen(CURRENT_OVERLAY)).toBe(false)
	})

	it('keeps its keys to itself', () => {
		// The app shares an origin with whatever else is served from it, and a
		// bare "seen" is a key anything could have written.
		remember(CURRENT_OVERLAY)

		expect(Object.keys(localStorage).some((key) => key.startsWith('pastille:'))).toBe(true)
	})

	it('treats storage it cannot reach as not having said it', () => {
		// Safari in private browsing throws outright. Being told something
		// twice is a smaller failure than a canvas that will not draw.
		const original = Object.getOwnPropertyDescriptor(window, 'localStorage')
		Object.defineProperty(window, 'localStorage', {
			configurable: true,
			get() {
				throw new Error('denied')
			},
		})

		expect(() => remember(CURRENT_OVERLAY)).not.toThrow()
		expect(hasSeen(CURRENT_OVERLAY)).toBe(false)

		if (original) Object.defineProperty(window, 'localStorage', original)
	})
})
