// SPDX-License-Identifier: MIT OR Apache-2.0

import { mount, tick, unmount } from 'svelte'
import { afterEach, describe, expect, it } from 'vitest'

import '../app.css'
import Breadboard from './Breadboard.svelte'
import { createCircuit, type BoardView } from '../lib/core'
import milestone from '../examples/milestone.json?raw'
import astable from '../examples/astable.json?raw'
import blink from '../examples/blink.json?raw'
import chaser from '../examples/chaser.json?raw'
import traffic from '../examples/uno-traffic.json?raw'

let mounted: Record<string, unknown> | undefined
let host: HTMLElement | undefined

function render(props: Record<string, unknown>): HTMLElement {
	host = document.createElement('div')
	host.style.width = '800px'
	document.body.append(host)
	mounted = mount(Breadboard, { target: host, props: { selected: new Set<string>(), ...props } })
	return host
}

afterEach(() => {
	if (mounted) void unmount(mounted)
	host?.remove()
	mounted = undefined
})

async function boardOf(text: string): Promise<BoardView> {
	const circuit = await createCircuit()
	circuit.load(text)
	return circuit.breadboard()
}

describe('the breadboard view (B-14)', () => {
	it('draws every part, the battery above the board with its leads in the rails', async () => {
		const shell = render({ board: await boardOf(milestone) })
		expect(shell.querySelectorAll('.part')).toHaveLength(3)
		expect(shell.querySelectorAll('.part.fixed')).toHaveLength(1)
		expect(shell.textContent).toContain('Half breadboard, 400 holes')
		expect(shell.querySelector('[role="alert"]')).toBeNull()
	})

	it('never draws the tile of one part over another, however many share strips', async () => {
		// The placer puts parts that share nets in neighbouring holes, so their
		// tiles start out in one spot; the view moves them apart.
		for (const [name, text] of [
			['blink', blink],
			['astable', astable],
			['traffic light', traffic],
			['chaser', chaser],
		] as const) {
			const shell = render({ board: await boardOf(text) })
			const tiles = [...shell.querySelectorAll('.tile')].map((tile) => ({
				x: Number(tile.getAttribute('x')),
				y: Number(tile.getAttribute('y')),
			}))
			expect(tiles.length, name).toBeGreaterThan(2)
			for (const [index, a] of tiles.entries()) {
				for (const b of tiles.slice(index + 1)) {
					const apart = Math.abs(a.x - b.x) >= 16 || Math.abs(a.y - b.y) >= 16
					expect(apart, `${name}: tiles at ${a.x},${a.y} and ${b.x},${b.y}`).toBe(true)
				}
			}
			if (mounted) void unmount(mounted)
			host?.remove()
			mounted = undefined
		}
	})

	it('draws a chip across the channel, named and with pin 1 marked', async () => {
		const shell = render({ board: await boardOf(astable) })
		expect(shell.querySelectorAll('.chip')).toHaveLength(1)
		expect(shell.querySelector('.chip-label')?.textContent).toBe('NE555')
		expect(shell.querySelector('.pin-one')).not.toBeNull()
	})

	it('says what the board builds that the schematic does not', async () => {
		const board = await boardOf(milestone)
		const shell = render({
			board: {
				...board,
				problems: [
					{ message: 'Row 4 (a–e) joins wires the schematic keeps apart.', holes: ['4a'] },
				],
			},
		})
		expect(shell.querySelector('[role="alert"]')?.textContent).toContain('Row 4 (a–e) joins wires')
		expect(shell.querySelectorAll('.hole.trouble')).toHaveLength(1)
	})

	it('picks a part pressed and let go where it was, and moves one dragged', async () => {
		const board = await boardOf(milestone)
		const picks: (string | undefined)[] = []
		const moves: [string, number, number][] = []
		const shell = render({
			board,
			onpick: (id: string | undefined) => picks.push(id),
			onmove: (id: string, dx: number, dy: number) => moves.push([id, dx, dy]),
		})
		await tick()
		const part = shell.querySelectorAll<SVGGElement>('.part:not(.fixed)')[0]
		const svg = shell.querySelector('svg')!
		const box = part.getBoundingClientRect()
		const at = {
			clientX: box.x + box.width / 2,
			clientY: box.y + box.height / 2,
			pointerId: 1,
			bubbles: true,
		}
		part.dispatchEvent(new PointerEvent('pointerdown', at))
		svg.dispatchEvent(new PointerEvent('pointerup', at))
		const id = board.parts.find((candidate) => candidate.movable)!.elementId
		expect(picks).toEqual([id])

		// A pitch is ten units of the drawing; drag five along.
		const scale = svg.getBoundingClientRect().width / svg.viewBox.baseVal.width
		part.dispatchEvent(new PointerEvent('pointerdown', at))
		const along = { ...at, clientX: at.clientX + 50 * scale }
		svg.dispatchEvent(new PointerEvent('pointermove', along))
		svg.dispatchEvent(new PointerEvent('pointerup', along))
		expect(moves).toEqual([[id, 5, 0]])
	})

	it('builds it a step at a time, showing what is in so far (B-15)', async () => {
		const board = await boardOf(astable)
		const shell = render({ board })
		const button = (text: string) =>
			[...shell.querySelectorAll('button')].find((each) => each.textContent?.trim() === text)!
		button('Build it step by step').click()
		await tick()
		expect(shell.querySelector('.count')?.textContent).toBe(`Step 1 of ${board.steps.length}`)
		expect(shell.querySelector('.say')?.textContent).toContain('Push the NE555 across the channel')
		// The chip alone, its legs' holes ringed — six, the pins the model
		// has; no jumper yet.
		const chip = board.parts.find((part) => part.dip)!
		expect(shell.querySelectorAll('.part')).toHaveLength(1)
		expect(shell.querySelectorAll('.ring')).toHaveLength(chip.legs.length)
		expect(shell.querySelectorAll('.jumper')).toHaveLength(0)

		button('Next').click()
		await tick()
		expect(shell.querySelector('.say')?.textContent).toContain('RESET (pin 4) to VCC')
		expect(shell.querySelectorAll('.jumper')).toHaveLength(1)

		// To the last step, which is the battery, and Done shows it all.
		for (let index = 2; index < board.steps.length; index++) {
			button('Next').click()
			await tick()
		}
		expect(shell.querySelector('.say')?.textContent).toContain('Last, connect the 9 V battery')
		button('Done').click()
		await tick()
		expect(shell.querySelector('.steps')).toBeNull()
		expect(shell.querySelectorAll('.part')).toHaveLength(board.parts.length)
	})
})
