// SPDX-License-Identifier: MIT OR Apache-2.0

// The shell, mounted in a real browser. These check structure and wiring, not
// pixels: what the regions are, what the palette offers, and that selecting
// something reaches the inspector.

import { mount, tick, unmount, type Component } from 'svelte'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { page } from 'vitest/browser'

// The tokens live in the global stylesheet, which main.ts pulls in for the real
// app. Mounting a component directly does not, so the theme tests need it here.
import '../app.css'
import App from '../App.svelte'
import Canvas from './Canvas.svelte'
import Description from './Description.svelte'
import Inspector from './Inspector.svelte'
import Keys from './Keys.svelte'
import OverlayNote from './OverlayNote.svelte'
import Palette from './Palette.svelte'
import Readout from './Readout.svelte'
import Scope from './Scope.svelte'
import SerialMonitor from './SerialMonitor.svelte'
import StatusBar from './StatusBar.svelte'
import { ACCURACY } from '../lib/accuracy'
import { EXAMPLE_GROUPS, EXAMPLES } from '../lib/examples'
import { forgetCircuit } from '../lib/circuitfile'
import { createCircuit, type Circuit, type ComponentKind, type ElementView } from '../lib/core'
import { GESTURES, keyBindings } from '../lib/keys'
import { CATALOGUE, PALETTE, PALETTE_CATEGORIES } from '../lib/palette'
import { SerialLog } from '../lib/serial.svelte'
import { SYMBOLS } from '../lib/symbols'

let mounted: Record<string, unknown> | undefined
let host: HTMLElement | undefined

// Generic over the component's own props, so passing the wrong ones is a type
// error here rather than a surprise at runtime.
function render<Props extends Record<string, unknown>>(
	component: Component<Props>,
	props: Props = {} as Props,
): HTMLElement {
	host = document.createElement('div')
	document.body.append(host)
	mounted = mount(component, { target: host, props })
	return host
}

/** A placed resistor, as the core would describe it. */
function resistor(value = '220'): ElementView {
	return {
		id: 'r1',
		kind: 'Resistor',
		x: 0,
		y: 0,
		turns: 0,
		label: `${value} Ω`,
		value,
		unit: 'Ω',
		colour: undefined,
		seriesOhms: undefined,
		hertz: undefined,
		initialCondition: undefined,
		initialUnit: '',
		closed: undefined,
		blown: undefined,
		wiper: undefined,
		choice: undefined,
		choices: [],
		basedOn: undefined,
		program: undefined,
	}
}

afterEach(() => {
	if (mounted) void unmount(mounted)
	host?.remove()
	mounted = undefined
	host = undefined
})

describe('app shell', () => {
	it('lays out three regions plus a status line', () => {
		const shell = render(App)

		for (const label of ['Components', 'Circuit', 'Inspector']) {
			expect(shell.querySelector(`[aria-label="${label}"]`), label).not.toBeNull()
		}
		expect(shell.querySelector('footer')).not.toBeNull()
	})

	it('offers every palette entry as its own control', () => {
		const shell = render(Palette)
		const buttons = [...shell.querySelectorAll('button.entry')]

		expect(buttons).toHaveLength(PALETTE.length)
		expect(buttons.map((button) => button.querySelector('.label')?.textContent)).toEqual(
			PALETTE.map((entry) => entry.label),
		)
	})

	it('groups the components under headings, all open to begin with', () => {
		// C-59. Every component is under exactly one heading, and every one is
		// visible before anything is clicked: the three parts the empty canvas
		// sends a beginner for must not be behind a fold.
		const shell = render(Palette)
		const headings = [...shell.querySelectorAll('button.heading')].map((heading) =>
			heading.textContent?.replace(/[▸▾]/g, '').trim(),
		)
		expect(headings).toEqual(PALETTE_CATEGORIES.map((category) => category.name))
		expect(shell.querySelectorAll('button.entry')).toHaveLength(PALETTE.length)

		const counted = PALETTE_CATEGORIES.flatMap((category) => category.entries)
		expect(new Set(counted.map((entry) => entry.kind)).size).toBe(counted.length)
	})

	it('folds a category away and back', async () => {
		const shell = render(Palette)
		const heading = [...shell.querySelectorAll<HTMLButtonElement>('button.heading')].find(
			(candidate) => candidate.textContent?.includes('Meters'),
		)!
		expect(shell.textContent).toContain('Voltmeter')

		heading.click()
		await tick()
		expect(heading.getAttribute('aria-expanded')).toBe('false')
		expect(shell.textContent).not.toContain('Voltmeter')
		// The rest stay where they were.
		expect(shell.textContent).toContain('Battery')

		heading.click()
		await tick()
		expect(shell.textContent).toContain('Voltmeter')
	})

	it('describes each component in words rather than symbols', () => {
		// The audience has never built a circuit; "Limits how much current flows"
		// is the palette doing its job.
		const shell = render(Palette)

		for (const entry of PALETTE) {
			expect(shell.textContent).toContain(entry.hint)
		}
	})

	it('draws each part with the symbol the board draws (C-128)', () => {
		const shell = render(Palette)
		for (const entry of shell.querySelectorAll('button.entry')) {
			expect(entry.querySelector('.glyph svg path'), entry.textContent ?? '').not.toBeNull()
		}
	})

	it('narrows the list to what answers the search, by name or by hint (C-128)', async () => {
		const shell = render(Palette)
		const search = shell.querySelector<HTMLInputElement>('input[aria-label="Find a part"]')!
		const labels = () =>
			[...shell.querySelectorAll('button.entry .label')].map((label) => label.textContent)

		search.value = '555'
		search.dispatchEvent(new Event('input', { bubbles: true }))
		await tick()
		expect(labels()).toEqual([CATALOGUE.Timer.label])

		// By what it does, which is how somebody who does not know its name looks.
		search.value = 'needs a resistor'
		search.dispatchEvent(new Event('input', { bubbles: true }))
		await tick()
		expect(labels()).toContain(CATALOGUE.Led.label)

		search.value = 'flux capacitor'
		search.dispatchEvent(new Event('input', { bubbles: true }))
		await tick()
		expect(labels()).toEqual([])
		expect(shell.textContent).toContain('Nothing called that')
	})

	it('offers the first three parts, numbered, only on an empty board (C-128)', () => {
		const empty = render(Palette, { start: true })
		const starters = [...empty.querySelectorAll('button.starter')]
		expect(starters.map((button) => button.querySelector('.label')?.textContent)).toEqual([
			CATALOGUE.DcSource.label,
			CATALOGUE.Resistor.label,
			CATALOGUE.Led.label,
		])
		expect(starters.map((button) => button.querySelector('.step')?.textContent)).toEqual([
			'1',
			'2',
			'3',
		])
		void unmount(mounted!)
		host?.remove()

		expect(render(Palette).querySelector('button.starter')).toBeNull()
	})

	it('opens a folded family from its chip (C-128)', async () => {
		const shell = render(Palette)
		const heading = () =>
			[...shell.querySelectorAll<HTMLButtonElement>('button.heading')].find((candidate) =>
				candidate.textContent?.includes('Meters'),
			)!
		heading().click()
		await tick()
		expect(shell.textContent).not.toContain('Voltmeter')

		;[...shell.querySelectorAll<HTMLButtonElement>('.families .chip')]
			.find((chip) => chip.textContent?.trim() === 'Meters')!
			.click()
		await tick()
		await tick()
		expect(heading().getAttribute('aria-expanded')).toBe('true')
		expect(shell.textContent).toContain('Voltmeter')
	})

	it('hands a picked component to whoever is listening', () => {
		const picked: string[] = []
		const shell = render(Palette, {
			onpick: (entry: { label: string }) => picked.push(entry.label),
		})

		shell.querySelector<HTMLButtonElement>('button.entry')?.click()

		expect(picked).toEqual([PALETTE[0].label])
	})

	it('shows the selected component in the inspector', () => {
		const shell = render(Inspector, { selection: resistor() })

		expect(shell.textContent).toContain('Resistor')
		expect(shell.querySelector<HTMLInputElement>('input')?.value).toBe('220')
	})

	it('steps a value through the ones that are sold, and offers its neighbours (C-136)', () => {
		const edits: [string, string][] = []
		const shell = render(Inspector, {
			selection: resistor(),
			steps: {
				below: [
					{ value: '180', label: '180 Ω' },
					{ value: '150', label: '150 Ω' },
				],
				above: [
					{ value: '270', label: '270 Ω' },
					{ value: '330', label: '330 Ω' },
				],
			},
			onvalue: (id: string, text: string) => edits.push([id, text]),
		})
		shell.querySelector<HTMLButtonElement>('button[aria-label="Up to 270 Ω"]')!.click()
		shell.querySelector<HTMLButtonElement>('button[aria-label="Down to 180 Ω"]')!.click()
		const chips = shell.querySelector('[aria-label="Values that are sold"]')!
		// Smallest first, the part's own in the middle.
		expect([...chips.children].map((chip) => chip.textContent?.trim())).toEqual([
			'150 Ω',
			'180 Ω',
			'220 Ω',
			'270 Ω',
			'330 Ω',
		])
		;[...chips.querySelectorAll<HTMLButtonElement>('button')].at(-1)!.click()
		expect(edits).toEqual([
			['r1', '270'],
			['r1', '180'],
			['r1', '330'],
		])
		// Typing any value still works: the field is there as it was.
		expect(shell.querySelector<HTMLInputElement>('input')?.value).toBe('220')
	})

	it("offers every other part in the list to take the selected one's place (C-137)", () => {
		const replaced: string[] = []
		const shell = render(Palette, {
			replacing: { kind: 'Resistor', label: 'Resistor' },
			onreplace: (entry: { kind: string }) => replaced.push(entry.kind),
		})
		const buttons = [...shell.querySelectorAll<HTMLButtonElement>('button.replace')]
		// Every entry but the resistor's own.
		expect(buttons.length).toBe(PALETTE.length - 1)
		const pot = buttons.find((button) =>
			button.getAttribute('aria-label')?.endsWith('with this potentiometer'),
		)!
		pot.click()
		expect(replaced).toEqual(['Potentiometer'])
	})

	it('offers nothing to replace with nothing selected (C-137)', () => {
		const shell = render(Palette, {})
		expect(shell.querySelector('button.replace')).toBeNull()
	})

	it('says what an Uno runs, and loads or resets its program (A-03)', () => {
		const asked: string[] = []
		const uno: ElementView = {
			...resistor(),
			id: 'u1',
			kind: 'Uno',
			value: undefined,
			unit: '',
			label: 'Arduino Uno',
			program: 'Blink.hex',
		}
		const shell = render(Inspector, {
			selection: uno,
			onprogram: (id: string) => asked.push(`load ${id}`),
			onreset: (id: string) => asked.push(`reset ${id}`),
		})
		expect(shell.textContent).toContain('It is running Blink.hex.')
		const button = (text: string) =>
			[...shell.querySelectorAll<HTMLButtonElement>('button')].find(
				(b) => b.textContent?.trim() === text,
			)!
		button('Load a .hex…').click()
		button('Reset').click()
		expect(asked).toEqual(['load u1', 'reset u1'])
		expect(shell.textContent).not.toContain('no value to set')
	})

	it('offers an Uno the programs the app comes with (A-07)', () => {
		const chosen: [string, string][] = []
		const uno: ElementView = {
			...resistor(),
			id: 'u1',
			kind: 'Uno',
			value: undefined,
			unit: '',
			label: 'Arduino Uno',
		}
		const shell = render(Inspector, {
			selection: uno,
			onbuiltin: (id: string, name: string) => chosen.push([id, name]),
		})
		const select = shell.querySelector<HTMLSelectElement>('select[aria-label="Its programs"]')!
		expect([...select.options].map((option) => option.value)).toContain('Blink')
		select.value = 'Fade'
		select.dispatchEvent(new Event('change', { bubbles: true }))
		expect(chosen).toEqual([['u1', 'Fade']])
	})

	it('says an Uno with no program only listens, and offers no reset (A-03)', () => {
		const uno: ElementView = {
			...resistor(),
			id: 'u1',
			kind: 'Uno',
			value: undefined,
			unit: '',
			label: 'Arduino Uno',
			program: undefined,
		}
		const shell = render(Inspector, { selection: uno })
		expect(shell.textContent).toContain('It has no program')
		expect(
			[...shell.querySelectorAll('button')].some((b) => b.textContent?.trim() === 'Reset'),
		).toBe(false)
	})

	it('says what the selected part is doing right now, held (C-129)', () => {
		const shell = render(Inspector, {
			selection: resistor(),
			reading: { volts: 7, amps: 0.0318 },
			heat: {
				elementId: 'r1',
				watts: 0.2226,
				celsius: 61,
				settlesAt: 61,
				maxCelsius: 155,
				ratedWatts: 0.25,
			},
			room: 25,
		})
		const card = shell.querySelector('[aria-label="Right now"]')!
		expect(card.textContent).toContain('7.00 V')
		expect(card.textContent).toContain('31.8 mA')
		expect(card.textContent).toContain('61 °C')
		// Against its rating, which is the number a beginner can do something about.
		const meter = card.querySelector('[role="meter"]')!
		expect(Number(meter.getAttribute('aria-valuenow'))).toBeCloseTo(0.89, 2)
		expect(card.textContent).toContain('of its 0.25 W')
		// Drawn as the board draws it.
		expect(shell.querySelector('.who .glyph svg path')).not.toBeNull()
	})

	it('says nothing it does not know, rather than a zero (C-129)', () => {
		const shell = render(Inspector, { selection: resistor(), reading: {} })
		expect(shell.querySelector('[aria-label="Right now"]')).toBeNull()
	})

	it('heads the scope with its probes, its time bases on show, and a fold (C-130)', async () => {
		const probes: ElementView[] = [
			{ ...resistor(), id: 'v1', kind: 'Voltmeter', unit: 'V' },
			{ ...resistor(), id: 'a1', kind: 'Ammeter', unit: 'A' },
		]
		const shell = render(Scope, { elements: probes })
		expect(shell.textContent).toContain('2 probes')

		const bases = [
			...shell.querySelectorAll<HTMLButtonElement>('[aria-label="Time base"] [role="radio"]'),
		]
		expect(bases.length).toBeGreaterThan(1)
		bases[0].click()
		await tick()
		expect(bases[0].getAttribute('aria-checked')).toBe('true')

		// Each lane its own ink, so a trace can be matched to its name.
		const inks = [...shell.querySelectorAll<HTMLElement>('.lane')].map((lane) =>
			lane.style.getPropertyValue('--ink'),
		)
		expect(new Set(inks).size).toBe(2)

		shell.querySelector<HTMLButtonElement>('button[aria-label="Fold the scope"]')!.click()
		await tick()
		expect(shell.querySelector('.lane')).toBeNull()
		// Folded, it still says what it is watching, and over how long.
		expect(shell.textContent).toContain('2 probes')
		expect(shell.textContent).toContain(`last ${bases[0].textContent?.trim()}`)
		shell.querySelector<HTMLButtonElement>('button[aria-label="Show the scope"]')!.click()
		await tick()
		expect(shell.querySelectorAll('.lane')).toHaveLength(2)
	})

	it('says so when nothing is selected', () => {
		const shell = render(Inspector)

		expect(shell.textContent).toContain('Nothing selected')
	})

	it('states that the simulation is educational', () => {
		// C-26's first placement: ambient, permanent, beside the other things
		// that are true the whole time. The sentence is the core's; the app
		// keeps its own copy, and `accuracy.test.ts` holds the two together.
		const shell = render(StatusBar, { status: 'ready', detail: 'Core ready' })

		expect(shell.textContent).toContain(ACCURACY)
	})

	it('announces what the line says without its clock', () => {
		// The visible line ticks every tenth of a second; a live region that
		// held it read a clock aloud for as long as the circuit ran.
		const shell = render(StatusBar, {
			status: 'ready',
			detail: '12.3 s simulated',
			spoken: 'Simulating',
		})

		expect(shell.querySelector('footer')?.getAttribute('aria-live')).toBeNull()
		expect(shell.querySelector('[role="status"]')?.textContent).toBe('Simulating')
		expect(shell.textContent).toContain('12.3 s simulated')
	})

	it('states it even when the core never loaded', () => {
		// The case that decided where the sentence lives. A claim fetched from
		// the core would be missing exactly when the app has least right to be
		// believed, so the app holds its own copy and shows it regardless.
		const shell = render(StatusBar, { status: 'failed', detail: 'Core failed to load: boom' })

		expect(shell.textContent).toContain(ACCURACY)
	})

	it('puts a warning on the line before a note, and draws a note as one (C-102)', () => {
		const note = {
			severity: 'note',
			message: 'This 7812 needs 14 V in to give 12 V out.',
			elements: [],
		}
		const warning = {
			severity: 'warning',
			message: 'This LED is the wrong way round — or the battery is.',
			elements: [],
		}
		const both = render(StatusBar, { status: 'ready', detail: '', findings: [note, warning] })
		expect(both.textContent).toContain(warning.message)
		expect(both.textContent).not.toContain(note.message)
		void unmount(mounted!)
		host?.remove()
		const alone = render(StatusBar, { status: 'ready', detail: '', findings: [note] })
		expect(alone.textContent).toContain(note.message)
		expect(alone.querySelector('.dot.note')).not.toBeNull()
	})

	it('keeps saying it while a finding has the line', () => {
		// A finding takes the sentence beside it, not this one: the two are
		// different claims, and the permanent one is not a slot for the news.
		const shell = render(StatusBar, {
			status: 'ready',
			detail: '12.0 s simulated',
			findings: [{ severity: 'warning', message: 'This LED is the wrong way round' }],
		})

		expect(shell.textContent).toContain('wrong way round')
		expect(shell.textContent).toContain(ACCURACY)
	})

	it('reflects the core state without computing it', () => {
		const shell = render(StatusBar, { status: 'failed', detail: 'Core failed to load: boom' })

		expect(shell.textContent).toContain('boom')
		expect(shell.querySelector('.dot.failed')).not.toBeNull()
	})
})

describe('editing a value', () => {
	it('offers the current value, with its unit beside it', () => {
		const shell = render(Inspector, { selection: resistor('4.7k') })

		expect(shell.querySelector<HTMLInputElement>('input')?.value).toBe('4.7k')
		expect(shell.textContent).toContain('Ω')
	})

	it('reports what was typed when the field is committed', () => {
		const edits: [string, string][] = []
		const shell = render(Inspector, {
			selection: resistor(),
			onvalue: (id: string, text: string) => edits.push([id, text]),
		})

		const input = shell.querySelector<HTMLInputElement>('input')!
		input.value = '4k7'
		// `change` is what a browser fires on Enter and on blur, and it carries
		// the field's value whether or not the binding has caught up.
		input.dispatchEvent(new Event('change', { bubbles: true }))

		expect(edits).toEqual([['r1', '4k7']])
	})

	it('says what to buy, what the rounding does, and sets it on a press (B-09)', () => {
		const edits: [string, string][] = []
		const shell = render(Inspector, {
			selection: resistor('350'),
			purchase: {
				elementId: 'r1',
				value: '330',
				label: '330 Ω',
				series: 'E12',
				errorPercent: -5.71,
				amps: 0.0218,
				findings: [
					{
						rule: 'over-rated',
						severity: 'warning',
						message: 'This red LED carries 22 mA; it is rated for 20 mA and will burn out.',
						elements: ['l1'],
						fix: undefined,
					},
				],
			},
			onvalue: (id: string, text: string) => edits.push([id, text]),
		})

		const text = shell.textContent?.replace(/\s+/g, ' ')
		expect(text).toContain('Buy 330 Ω (E12): 5.7 % less, 21.8 mA through it.')
		expect(text).toContain('On 330 Ω: This red LED carries 22 mA')
		const use = [...shell.querySelectorAll('button')].find((button) =>
			button.textContent?.includes('Use 330 Ω'),
		)!
		use.click()
		expect(edits).toEqual([['r1', '330']])
	})

	it('keeps what was typed when the core rejects it', () => {
		// Snapping the field back to the old value would hide the mistake and
		// throw away the keystrokes that were nearly right.
		const shell = render(Inspector, {
			selection: resistor(),
			error: '"4..7k" is not a number',
		})

		expect(shell.textContent).toContain('is not a number')
		expect(shell.querySelector('input')?.getAttribute('aria-invalid')).toBe('true')
	})

	it('suggests the forms it accepts, for the component it is describing', () => {
		// "220R" is no help beside a capacitor: the prefixes a part is sold in
		// are part of how it is described, not of what it is.
		expect(render(Inspector, { selection: resistor() }).textContent).toContain('220R')

		const capacitor = render(Inspector, {
			selection: { ...resistor(), kind: 'Capacitor', value: '100n', unit: 'F' },
		})
		expect(capacitor.textContent).toContain('100n')
		expect(capacitor.textContent).not.toContain('220R')
	})

	it('names the real part a component is modelled on, and links its datasheet', () => {
		const sheet = 'https://www.vishay.com/docs/81857/1n4148.pdf'
		const shell = render(Inspector, {
			selection: {
				...resistor(),
				kind: 'Diode',
				basedOn: { part: '1N4148 (Vishay)', datasheet: sheet, approximate: false },
			},
		})

		expect(shell.textContent).toContain('Modelled on')
		expect(shell.textContent).toContain('1N4148 (Vishay)')
		const link = shell.querySelector<HTMLAnchorElement>('a[href]')
		expect(link?.href).toBe(sheet)
		// In a tab of its own: following it must not throw the circuit away.
		expect(link?.target).toBe('_blank')
		expect(link?.rel).toContain('noopener')
		// And in a browser it is followed as a link: only the desktop shell,
		// which opens no second window, takes it over (C-68).
		let followed: boolean | undefined
		link?.addEventListener('click', (event) => {
			followed = !event.defaultPrevented
			event.preventDefault()
		})
		link?.dispatchEvent(new MouseEvent('click', { cancelable: true }))
		expect(followed).toBe(true)
	})

	it('says when a model is only close to its part', () => {
		const close = render(Inspector, {
			selection: {
				...resistor(),
				kind: 'Bjt',
				basedOn: { part: '2N3904', datasheet: undefined, approximate: true },
			},
		})
		expect(close.textContent).toContain('2N3904, roughly')
		expect(close.querySelector('a')).toBeNull()
	})

	it('names no part for an ideal component', () => {
		const ideal = render(Inspector, { selection: { ...resistor(), kind: 'DcSource' } })
		expect(ideal.textContent).not.toContain('Modelled on')
	})

	it('shows how hot a part runs, and says so past its limits (C-74)', () => {
		const shell = render(Inspector, {
			selection: resistor('150'),
			heat: {
				elementId: 'r1',
				watts: 0.54,
				celsius: 165,
				settlesAt: 209,
				ratedWatts: 0.25,
				maxCelsius: 155,
			},
		})

		// The power and the temperature, each in its own cell of the "Right
		// now" card since C-129.
		const card = shell.querySelector('[aria-label="Right now"]')!
		expect(card.textContent).toContain('540 mW')
		expect(card.textContent).toContain('165 °C')
		expect(shell.textContent).toContain('Heading for about 209 °C.')
		expect(shell.querySelector('[role="alert"]')?.textContent).toContain('155 °C')
	})

	it('offers the room temperature when nothing is selected (C-74)', () => {
		const shell = render(Inspector, { room: 31 })
		const field = shell.querySelector<HTMLInputElement>('input[aria-label="Room temperature"]')

		expect(field?.value).toBe('31')
	})

	it('offers the description when nothing is selected, and hands on an edit', () => {
		const edits: string[] = []
		const shell = render(Inspector, {
			description: 'A first circuit.',
			ondescription: (text: string) => edits.push(text),
		})
		const field = shell.querySelector<HTMLTextAreaElement>('textarea[aria-label="Description"]')!
		expect(field.value).toBe('A first circuit.')

		field.value = 'A first circuit, lit.'
		field.dispatchEvent(new Event('change', { bubbles: true }))
		expect(edits).toEqual(['A first circuit, lit.'])
	})

	it('puts an error under the field it came from, and marks only that one (C-100)', () => {
		const capacitor: ElementView = {
			...resistor('100µ'),
			id: 'c1',
			kind: 'Capacitor',
			unit: 'F',
			initialCondition: 0,
			initialUnit: 'V',
		}
		const shell = render(Inspector, {
			selection: capacitor,
			error: 'That is not a number this can be set to.',
			errorField: 'initial',
		})
		const value = shell.querySelector<HTMLInputElement>('input[aria-label="Value"]')!
		const starts = shell.querySelector<HTMLInputElement>('input[aria-label="Starts at"]')!
		expect(starts.getAttribute('aria-invalid')).toBe('true')
		expect(value.getAttribute('aria-invalid')).toBeNull()
		const alerts = [...shell.querySelectorAll('[role="alert"]')]
		expect(alerts).toHaveLength(1)
		// Under "Starts at", which comes after the value's field.
		expect(
			starts.compareDocumentPosition(alerts[0]) & Node.DOCUMENT_POSITION_FOLLOWING,
		).toBeTruthy()
	})

	it('shows a refused choice for a part that has no value field', () => {
		const led: ElementView = {
			...resistor(),
			id: 'd1',
			kind: 'Led',
			value: undefined,
			unit: '',
			choices: ['Red', 'Green'],
			choice: 'Red',
		}
		const shell = render(Inspector, {
			selection: led,
			error: 'No such colour.',
			errorField: 'choice',
		})
		expect(shell.querySelector('[role="alert"]')?.textContent).toBe('No such colour.')
	})

	it('says plainly when a component has no value to set', () => {
		const shell = render(Inspector, {
			selection: { ...resistor(), id: 'g1', kind: 'Ground', label: '', value: undefined, unit: '' },
		})

		expect(shell.textContent).toContain('no value to set')
		expect(shell.querySelector('input')).toBeNull()
	})

	it('does not say that about a component with something to choose', () => {
		// An LED has a colour (C-46), and "no value to set" sitting under its
		// own chooser is the panel contradicting itself.
		const shell = render(Inspector, {
			selection: {
				...resistor(),
				kind: 'Led',
				label: 'Red',
				value: undefined,
				unit: '',
				choice: 'Red',
				choices: ['Red', 'Green', 'Blue', 'White'],
			},
		})

		expect(shell.textContent).not.toContain('no value to set')
		expect(shell.querySelector<HTMLSelectElement>('select')?.value).toBe('Red')
	})

	it('says which way a switch is, and offers to flip it', () => {
		// C-19. A contact has no value to type, and "no value to set" beside the
		// one component whose whole state is the interesting thing would be a
		// dead end. The button is also the only way to work one without a
		// pointer.
		const flipped: Array<[string, boolean]> = []
		const shell = render(Inspector, {
			selection: { ...resistor(), id: 's1', kind: 'Switch', value: undefined, closed: false },
			onflip: (id: string, closed: boolean) => flipped.push([id, closed]),
		})

		expect(shell.textContent).toContain('off')
		shell.querySelector('button')?.click()

		expect(flipped).toEqual([['s1', true]])
	})

	it('tells a push-button to be held rather than offering to latch it', () => {
		// A button that latched when clicked would be a lie about the part.
		const shell = render(Inspector, {
			selection: {
				...resistor(),
				id: 'b1',
				kind: 'PushButton',
				value: undefined,
				closed: false,
			},
		})

		expect(shell.textContent).toContain('Hold it down')
		expect(shell.querySelector('button')).toBeNull()
	})
})

describe('the canvas as a projection of the circuit', () => {
	// C-24. The circuit handed to the app is edited from outside it — no
	// handler in the shell is called, nothing is told to refresh — and what is
	// on screen follows within a frame. This is the acceptance, and it could
	// not be written before: the shell made its own circuit and nobody else
	// could reach it.

	/** The autosave is a side effect of mounting; it must not outlive the test. */
	afterEach(() => forgetCircuit())

	/** Battery, resistor, LED, wired into a loop. */
	async function milestone(): Promise<Circuit> {
		const circuit = await createCircuit()
		const battery = circuit.add('DcSource', 0, 0)
		const resistor = circuit.add('Resistor', 40, 0)
		const led = circuit.add('Led', 80, 0)
		circuit.connect(battery, '+', resistor, 'a')
		circuit.connect(resistor, 'b', led, 'anode')
		circuit.connect(led, 'cathode', battery, '-')
		return circuit
	}

	/** Waits for the shell to catch up, a frame at a time. */
	async function until(check: () => boolean, what: string): Promise<void> {
		for (let frame = 0; frame < 120; frame++) {
			await new Promise(requestAnimationFrame)
			await tick()
			if (check()) return
		}
		throw new Error(`the shell never ${what}`)
	}

	it('grows a scope when a probe is placed on the circuit from outside', async () => {
		const circuit = await milestone()
		const shell = render(App, { circuit })
		await until(() => shell.textContent?.includes('simulated') === true, 'started simulating')
		expect(shell.querySelector('[aria-label="Scope"]')).toBeNull()

		const meter = circuit.add('Voltmeter', 0, 80)
		circuit.connect(meter, '+', circuit.elements()[1].id, 'b')

		await until(
			() => shell.querySelector('[aria-label="Scope"]') !== null,
			'showed the scope for a probe placed behind its back',
		)
	})

	it('drops the scope again when the probe is removed from outside', async () => {
		const circuit = await milestone()
		const meter = circuit.add('Voltmeter', 0, 80)
		circuit.connect(meter, '+', circuit.elements()[1].id, 'b')
		const shell = render(App, { circuit })
		await until(
			() => shell.querySelector('[aria-label="Scope"]') !== null,
			'showed the scope it was handed',
		)

		circuit.remove(meter)

		await until(
			() => shell.querySelector('[aria-label="Scope"]') === null,
			'let go of the scope for a probe removed behind its back',
		)
	})

	it('says once that it is simulating, however long it runs', async () => {
		const circuit = await milestone()
		const shell = render(App, { circuit })
		const spoken = () => shell.querySelector('footer [role="status"]')?.textContent
		await until(() => spoken() === 'Simulating', 'announced that it was simulating')
		const shown = shell.querySelector('footer')?.textContent

		await until(() => shell.querySelector('footer')?.textContent !== shown, 'moved its clock on')

		expect(spoken()).toBe('Simulating')
	})

	it('lists the parts in words, and selects one from the keyboard', async () => {
		// The canvas is a picture to anything but a pointer. The list is how
		// a screen reader learns what is on the board, and how Tab reaches a
		// part to select it.
		const circuit = await milestone()
		const shell = render(App, { circuit })
		await until(
			() => shell.querySelector('[aria-label="Parts on the board"]') !== null,
			'listed the parts',
		)
		const buttons = [
			...shell.querySelectorAll<HTMLButtonElement>('[aria-label="Parts on the board"] button'),
		]
		expect(buttons.map((button) => button.textContent?.trim())).toEqual([
			expect.stringMatching(/^Battery, /),
			expect.stringMatching(/^Resistor, /),
			expect.stringMatching(/^LED/),
		])

		buttons[1].focus()
		buttons[1].click()
		await tick()

		expect(buttons[1].getAttribute('aria-pressed')).toBe('true')
		expect(shell.querySelector('[aria-label="Inspector"]')?.textContent).toContain('Resistor')
		// Shown while it holds the focus, so the focus is somewhere to see.
		expect(
			shell.querySelector<HTMLElement>('[aria-label="Parts on the board"]')!.offsetWidth,
		).toBeGreaterThan(1)
	})

	it('stops and offers a reload when the core fails mid-simulation', async () => {
		// What a Rust panic looks like from here: every call after it throws.
		// The loop used to die silently on the first, leaving a frozen board
		// that looked alive.
		const circuit = await milestone()
		const shell = render(App, { circuit })
		await until(() => shell.textContent?.includes('simulated') === true, 'started simulating')

		const quiet = console.error
		console.error = () => {}
		let calls = 0
		circuit.advance = () => {
			calls += 1
			throw new WebAssembly.RuntimeError('unreachable')
		}
		try {
			await until(() => shell.querySelector('.halted') !== null, 'said the simulation stopped')
			const after = calls
			for (let frame = 0; frame < 5; frame++) await new Promise(requestAnimationFrame)
			expect(calls).toBe(after)
			expect(shell.querySelector('.halted button')?.textContent).toBe('Reload')
		} finally {
			console.error = quiet
		}
	})

	it('reads the clock off the circuit it was given', async () => {
		// The simulation is the handed-over circuit's, not one the shell made
		// of its own: time only moves here because that circuit is being
		// advanced, and the footer is reading it back.
		const circuit = await milestone()
		const shell = render(App, { circuit })

		await until(
			() => /[1-9]\d*\.\d s simulated|0\.[1-9] s simulated/.test(shell.textContent ?? ''),
			'reported any simulated time',
		)
	})
})

describe('the circuits that ship with the app', () => {
	// C-47's placement decision, which is also its safety argument: the offer
	// exists on the empty canvas and nowhere else, so a curious click can never
	// throw away work — there is nothing there to throw away.

	afterEach(() => forgetCircuit())

	async function until(check: () => boolean, what: string): Promise<void> {
		for (let frame = 0; frame < 120; frame++) {
			await new Promise(requestAnimationFrame)
			await tick()
			if (check()) return
		}
		throw new Error(`the shell never ${what}`)
	}

	// The library follows the core rather than coming with it (M-21): until it
	// arrives there is no example to press, and none missing for a good reason.
	async function examplesOffered(shell: HTMLElement): Promise<void> {
		await until(() => shell.querySelector('.dot.ready') !== null, 'loaded the core')
		await until(
			() => shell.querySelector('.offer .examples button') !== null,
			'offered the examples',
		)
	}

	// Where nothing is to be offered, there is no button to wait for: the
	// shell asked for the library once the core was ready, so the same import
	// settles after its own, and a tick later the shell has drawn with it.
	async function libraryIn(): Promise<void> {
		await import('../lib/examples')
		await tick()
	}

	it('offers every one of them while the canvas is empty, a shelf at a time', async () => {
		const shell = render(App)
		await examplesOffered(shell)

		const buttons = () => [...shell.querySelectorAll('button')]
		for (const group of EXAMPLE_GROUPS) {
			// The shelf's own buttons: "Chips" and "Logic" are also families
			// in the palette (C-128).
			;[...shell.querySelectorAll<HTMLButtonElement>('.shelves button')]
				.find((button) => button.textContent?.trim() === group)!
				.click()
			await tick()

			const offered = buttons().map((button) => button.textContent?.trim())
			for (const example of EXAMPLES) {
				// What is on this shelf is offered with its line; what is on
				// another is not, which is what keeps the offer one size.
				const here = example.group === group
				expect(offered.includes(example.title), `${example.id} on ${group}`).toBe(here)
				expect(shell.textContent?.includes(example.look), `${example.id} on ${group}`).toBe(here)
			}
		}
	})

	it('takes the offer away once there is something to lose', async () => {
		// C-47's protection, kept by ADR 0016 where it does its work: nothing
		// is offered *unasked* once the canvas has something on it. Asked for
		// from the header, it is — one Undo away (the test below).
		const circuit = await createCircuit()
		circuit.add('Resistor', 40, 40)
		const shell = render(App, { circuit })

		await until(() => shell.querySelector('.dot.ready') !== null, 'loaded the core')
		await libraryIn()
		const offered = [...shell.querySelectorAll('button')].map((button) =>
			button.textContent?.trim(),
		)

		expect(offered).not.toContain(EXAMPLES[0].title)
	})

	it('says a finding beside the part it is about, with the count in the header (B-07)', async () => {
		// A battery, a resistor and an LED, the LED the wrong way round.
		const circuit = await createCircuit()
		const battery = circuit.add('DcSource', 0, 0)
		const resistor = circuit.add('Resistor', 80, 0)
		const led = circuit.add('Led', 160, 0)
		circuit.connect(battery, '+', resistor, 'a')
		circuit.connect(resistor, 'b', led, 'cathode')
		circuit.connect(led, 'anode', battery, '-')
		const shell = render(App, { circuit })
		// Pointing at the part: beside it, not pinned to the top. Pinned for as
		// long as the board has not measured itself, so it is waited for.
		await until(
			() => shell.querySelector('.callout:not(.pinned)') !== null,
			'said it beside the part',
		)

		const callout = shell.querySelector<HTMLElement>('.callout')!
		expect(callout.textContent).toContain('wrong way round')
		expect(shell.querySelector('header .checks')?.textContent?.trim()).toBe('1 check')
		// No longer in the status line, which says how the simulation is.
		expect(shell.querySelector('footer')?.textContent).not.toContain('wrong way round')

		shell.querySelector<HTMLButtonElement>('.callout button[aria-label="Put this away"]')!.click()
		await until(() => shell.querySelector('.callout') === null, 'put it away')
		shell.querySelector<HTMLButtonElement>('header .checks')!.click()
		await until(() => shell.querySelector('.callout') !== null, 'brought it back from the header')
	})

	it('offers a finding’s fix as a button, and one Undo takes it back (C-135)', async () => {
		// The commonest mistake: an LED straight across a battery.
		const circuit = await createCircuit()
		const battery = circuit.add('DcSource', 0, 0)
		const led = circuit.add('Led', 160, 0)
		circuit.connect(battery, '+', led, 'anode')
		circuit.connect(led, 'cathode', battery, '-')
		const shell = render(App, { circuit })
		const fix = () => shell.querySelector<HTMLButtonElement>('.callout button.fix')
		await until(() => fix() !== null, 'offered the fix')
		expect(fix()!.textContent?.trim()).toBe('Add a 470 Ω resistor')

		fix()!.click()
		expect(circuit.size, 'the resistor is in').toBe(3)
		await until(() => shell.querySelector('.callout') === null, 'had nothing left to say')

		;[...shell.querySelectorAll<HTMLButtonElement>('header button')]
			.find((button) => button.textContent?.trim() === 'Undo')!
			.click()
		expect(circuit.size, 'one undo takes it out').toBe(2)
	})

	it('opens one over work when asked, and says the work is one Undo away (ADR 0016)', async () => {
		const circuit = await createCircuit()
		circuit.add('Resistor', 40, 40)
		const shell = render(App, { circuit })
		await until(() => shell.querySelector('.dot.ready') !== null, 'loaded the core')

		;[...shell.querySelectorAll<HTMLButtonElement>('header button')]
			.find((button) => button.textContent?.trim() === 'Examples')!
			.click()
		await until(
			() => shell.querySelector('dialog[open] .examples button') !== null,
			'opened the examples',
		)
		;[...shell.querySelectorAll<HTMLButtonElement>('dialog .examples button')]
			.find((button) => button.textContent?.trim() === EXAMPLES[0].title)!
			.click()

		await until(() => shell.querySelector('.undo-notice') !== null, 'said how to get the work back')
		expect(shell.querySelector('dialog[open]'), 'the dialog closes').toBeNull()
		expect(shell.querySelector('.undo-notice')?.textContent).toContain(EXAMPLES[0].title)
		expect(circuit.size, 'the example replaced the lone resistor').toBeGreaterThan(1)

		;[...shell.querySelectorAll<HTMLButtonElement>('.undo-notice button')]
			.find((button) => button.textContent?.trim() === 'Undo')!
			.click()
		expect(circuit.size, 'the resistor is back').toBe(1)
		expect(circuit.elements()[0].kind).toBe('Resistor')
		await until(() => shell.querySelector('.undo-notice') === null, 'withdrew the notice')
	})

	it('frames what it opens over work, not the work it replaced', async () => {
		// Seen after a reload, which always leaves the last circuit on the
		// canvas: the fit ran before the projection had re-read the circuit,
		// so it framed the one being replaced and the opened one came up off
		// screen. The work here is far from what is opened, so framing the
		// wrong one puts all of it off the board — and its finding, which
		// points at its part when it can, pinned to the top instead.
		const opened = await createCircuit()
		const battery = opened.add('DcSource', 0, 0)
		const resistor = opened.add('Resistor', 80, 0)
		const led = opened.add('Led', 160, 0)
		opened.connect(battery, '+', resistor, 'a')
		opened.connect(resistor, 'b', led, 'cathode')
		opened.connect(led, 'anode', battery, '-')
		const text = opened.save()

		const circuit = await createCircuit()
		circuit.add('Resistor', 4000, 3000)
		const shell = render(App, { circuit })
		await until(() => shell.querySelector('.dot.ready') !== null, 'loaded the core')

		// The file picker, answered with the file as if it had been chosen.
		// Put back below, and only ever called with `call`, so `this` is kept.
		// eslint-disable-next-line @typescript-eslint/unbound-method
		const click = HTMLInputElement.prototype.click
		HTMLInputElement.prototype.click = function (this: HTMLInputElement) {
			if (this.type !== 'file') return click.call(this)
			const chosen = new DataTransfer()
			chosen.items.add(new File([text], 'reversed.json', { type: 'application/json' }))
			this.files = chosen.files
			this.dispatchEvent(new Event('change'))
		}
		try {
			;[...shell.querySelectorAll<HTMLButtonElement>('header button')]
				.find((button) => button.textContent?.trim() === 'Open…')!
				.click()
			await until(
				() => shell.querySelector('.callout') !== null,
				'said the LED is the wrong way round',
			)
		} finally {
			HTMLInputElement.prototype.click = click
		}

		expect(circuit.size, 'the file replaced the lone resistor').toBe(3)
		expect(shell.querySelector('.callout')?.textContent).toContain('wrong way round')
		// Beside the part once the board has measured itself, as above.
		await until(
			() => shell.querySelector('.callout:not(.pinned)') !== null,
			'said it beside the part',
		)
	})

	it('says nothing about an Undo when there was nothing to lose (ADR 0016)', async () => {
		const shell = render(App)
		await examplesOffered(shell)
		;[...shell.querySelectorAll<HTMLButtonElement>('.offer .examples button')]
			.find((button) => button.textContent?.trim() === EXAMPLES[0].title)!
			.click()
		await until(
			() => shell.querySelector('[aria-label="About this circuit"]') !== null,
			'opened it',
		)
		expect(shell.querySelector('.undo-notice')).toBeNull()
	})

	it('opens the one that was pressed', async () => {
		const shell = render(App)
		await examplesOffered(shell)

		const button = [...shell.querySelectorAll('button')].find(
			(candidate) => candidate.textContent?.trim() === EXAMPLES[0].title,
		)!
		button.click()

		// The canvas fills, which is also the offer withdrawing itself.
		await until(
			() => shell.textContent?.includes('simulated') === true,
			'started simulating what it opened',
		)
		const offered = [...shell.querySelectorAll('button')].map((candidate) =>
			candidate.textContent?.trim(),
		)
		expect(offered).not.toContain(EXAMPLES[1].title)
		// And what to look at stays, on the canvas, as its description.
		await until(
			() => shell.querySelector('[aria-label="About this circuit"]') !== null,
			'described what it opened',
		)
		const card = shell.querySelector('[aria-label="About this circuit"]')?.textContent
		expect(card).toContain(EXAMPLES[0].title)
		expect(card).toContain(EXAMPLES[0].look)
	})

	it('leaves the keys alone while the description is being typed', async () => {
		// Backspace and Delete remove the selection, and Cmd+A selects
		// everything: in the description field they are for the text.
		const circuit = await createCircuit()
		circuit.add('Resistor', 40, 40)
		const shell = render(App, { circuit })
		await until(() => shell.querySelector('.dot.ready') !== null, 'loaded the core')

		const field = shell.querySelector<HTMLTextAreaElement>('textarea[aria-label="Description"]')!
		field.focus()
		for (const key of ['a', 'Backspace', 'Delete']) {
			field.dispatchEvent(
				new KeyboardEvent('keydown', { key, metaKey: key === 'a', bubbles: true }),
			)
		}
		await tick()
		expect(circuit.size).toBe(1)
		expect(shell.textContent).toContain('Nothing selected.')
	})

	it('places a component clicked in the palette, and selects it (C-97)', async () => {
		// A click is how a keyboard (Enter on the entry) and a finger reach a
		// part the phone's canvas has scrolled away from.
		const circuit = await createCircuit()
		const shell = render(App, { circuit })
		await until(() => shell.querySelector('.dot.ready') !== null, 'loaded the core')

		const entry = (label: string) =>
			[...shell.querySelectorAll<HTMLButtonElement>('.palette .entry')].find(
				(button) => button.querySelector('.label')?.textContent === label,
			)!
		entry(CATALOGUE.Resistor.label).click()
		await tick()
		expect(circuit.size).toBe(1)
		// Selected, so the Inspector says what it is once the frame has read it.
		await until(
			() => shell.textContent?.includes(CATALOGUE.Resistor.does) === true,
			'selected what it placed',
		)

		// A second lands beside the first, not on it.
		entry(CATALOGUE.Resistor.label).click()
		await tick()
		const [first, second] = circuit.elements()
		expect([first.x, first.y]).not.toEqual([second.x, second.y])
	})

	it('opens an example as one step that an undo takes back (C-98)', async () => {
		const circuit = await createCircuit()
		const shell = render(App, { circuit })
		await examplesOffered(shell)

		const button = [...shell.querySelectorAll('button')].find(
			(candidate) => candidate.textContent?.trim() === EXAMPLES[0].title,
		)!
		button.click()
		await tick()
		expect(circuit.size).toBeGreaterThan(0)
		expect(circuit.undo()).toBe(true)
		expect(circuit.size).toBe(0)
	})

	it('keeps a circuit from last time it cannot open, and offers it as a file (C-98)', async () => {
		// Saved by a newer version: before, it was deleted without a word.
		const newer = '{"schema":99,"elements":[],"nets":[]}'
		localStorage.setItem('pastille:circuit', newer)
		const shell = render(App)
		await until(() => shell.querySelector('.unopened') !== null, 'said it could not open it')

		expect(shell.querySelector('.unopened')?.textContent).toContain('could not be opened')
		expect(localStorage.getItem('pastille:circuit-unopened')).toBe(newer)
		const buttons = [...shell.querySelectorAll('.unopened button')].map((b) => b.textContent)
		expect(buttons).toEqual(['Download it', 'Discard it'])
		localStorage.removeItem('pastille:circuit-unopened')
	})

	it('offers the circuit it kept again on the next visit, until it is let go of (C-98)', async () => {
		// Set aside on an earlier visit: the autosave it came from is gone.
		const newer = '{"schema":99,"elements":[],"nets":[]}'
		localStorage.setItem('pastille:circuit-unopened', newer)
		const shell = render(App)
		await until(() => shell.querySelector('.unopened') !== null, 'offered it again')
		expect(shell.querySelector('.unopened p')?.textContent).toContain(
			'could not be opened here. It has been kept',
		)

		const discard = [...shell.querySelectorAll<HTMLButtonElement>('.unopened button')].find(
			(b) => b.textContent === 'Discard it',
		)!
		discard.click()
		await tick()
		expect(shell.querySelector('.unopened')).toBeNull()
		expect(localStorage.getItem('pastille:circuit-unopened')).toBeNull()
	})

	it('deletes and turns by button, and undoes by button, with no keyboard (C-99)', async () => {
		const circuit = await createCircuit()
		const shell = render(App, { circuit })
		await until(() => shell.querySelector('.dot.ready') !== null, 'loaded the core')
		const button = (text: string) =>
			[...shell.querySelectorAll<HTMLButtonElement>('button')].find(
				(candidate) => candidate.textContent?.trim() === text,
			)

		;[...shell.querySelectorAll<HTMLButtonElement>('.palette .entry')]
			.find((entry) => entry.querySelector('.label')?.textContent === CATALOGUE.Resistor.label)!
			.click()
		await until(() => button('Turn 90°') !== undefined, 'offered to turn it')
		button('Turn 90°')!.click()
		expect(circuit.elements()[0].turns).toBe(1)
		button('Delete')!.click()
		expect(circuit.size).toBe(0)

		await until(() => button('Undo')?.getAttribute('aria-disabled') === 'false', 'offered undo')
		button('Undo')!.click()
		expect(circuit.size).toBe(1)
		await until(() => button('Redo')?.getAttribute('aria-disabled') === 'false', 'offered redo')
		button('Redo')!.click()
		expect(circuit.size).toBe(0)
	})

	it('lets go of a part being carried when the browser takes the gesture back (C-99)', async () => {
		const circuit = await createCircuit()
		const shell = render(App, { circuit })
		await until(() => shell.querySelector('.dot.ready') !== null, 'loaded the core')
		const entry = shell.querySelector<HTMLButtonElement>('.palette .entry')!
		const box = entry.getBoundingClientRect()
		entry.dispatchEvent(
			new PointerEvent('pointerdown', { clientX: box.x + 5, clientY: box.y + 5, bubbles: true }),
		)
		// A finger drag that became a scroll: the browser cancels the pointer.
		window.dispatchEvent(new PointerEvent('pointercancel'))
		const canvas = shell.querySelector('main')!.getBoundingClientRect()
		const middle = { clientX: canvas.x + canvas.width / 2, clientY: canvas.y + canvas.height / 2 }
		window.dispatchEvent(new PointerEvent('pointermove', middle))
		window.dispatchEvent(new PointerEvent('pointerup', middle))
		await tick()
		expect(circuit.size).toBe(0)
	})

	it('clears the canvas in one step that an undo takes back', async () => {
		const circuit = await createCircuit()
		circuit.add('Resistor', 40, 40)
		circuit.add('Led', 120, 40)
		circuit.setDescription('Two parts, not yet wired.')
		const shell = render(App, { circuit })
		await until(() => shell.querySelector('.dot.ready') !== null, 'loaded the core')

		const clear = [...shell.querySelectorAll('button')].find(
			(candidate) => candidate.textContent?.trim() === 'Clear',
		)!
		expect(clear.getAttribute('aria-disabled')).toBe('false')
		clear.click()
		// The examples come back, because the canvas is empty again.
		await until(
			() => shell.textContent?.includes(EXAMPLES[0].look) === true,
			'offered the examples again',
		)
		expect(circuit.size).toBe(0)
		expect(circuit.description()).toBe('')
		// Asleep, not disabled, so the focus is not thrown to the page (C-103).
		expect(clear.getAttribute('aria-disabled'), 'nothing left to clear').toBe('true')
		clear.click()
		expect(circuit.canUndo, 'a second press is no step').toBe(true)
		expect(circuit.undo()).toBe(true)
		expect(circuit.size).toBe(2)
		circuit.redo()

		expect(circuit.undo()).toBe(true)
		expect(circuit.size).toBe(2)
		expect(circuit.description()).toBe('Two parts, not yet wired.')
	})

	it('still answers Cmd+Z with a slider holding the focus (C-103)', async () => {
		// After a knob is turned, its slider keeps the focus, and it is not
		// typing.
		const circuit = await createCircuit()
		circuit.add('Resistor', 40, 40)
		const shell = render(App, { circuit })
		await until(() => shell.querySelector('.dot.ready') !== null, 'loaded the core')

		const slider = document.createElement('input')
		slider.type = 'range'
		shell.append(slider)
		slider.focus()
		slider.dispatchEvent(
			new KeyboardEvent('keydown', { key: 'z', metaKey: true, ctrlKey: true, bubbles: true }),
		)
		expect(circuit.size).toBe(0)
	})

	it('lets go of the selection on Escape (C-103)', async () => {
		const circuit = await createCircuit()
		const shell = render(App, { circuit })
		await until(() => shell.querySelector('.dot.ready') !== null, 'loaded the core')

		const entry = [...shell.querySelectorAll<HTMLButtonElement>('.palette .entry')].find(
			(button) => button.querySelector('.label')?.textContent === CATALOGUE.Resistor.label,
		)!
		entry.click()
		await until(
			() => shell.textContent?.includes(CATALOGUE.Resistor.does) === true,
			'selected what it placed',
		)
		window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
		await until(() => shell.textContent?.includes('Nothing selected.') === true, 'deselected')
		expect(circuit.size, 'and kept the part').toBe(1)
	})
})

describe('the description card', () => {
	it('keeps the focus on itself when it is folded and unfolded (C-103)', async () => {
		const shell = render(Description, { text: 'A first circuit.' })
		// Settled first, as it is long before anybody clicks it: its effect
		// that unfolds on new text runs once on mount.
		await tick()
		shell.querySelector<HTMLButtonElement>('button[aria-label="Fold away"]')!.click()
		await tick()
		await tick()
		const tab = shell.querySelector('.tab')
		expect(tab, 'folded').not.toBeNull()
		expect(document.activeElement, `${document.activeElement?.outerHTML.slice(0, 80)}`).toBe(tab)
		;(document.activeElement as HTMLButtonElement).click()
		await tick()
		await tick()
		expect(document.activeElement?.getAttribute('aria-label')).toBe('Fold away')
	})
})

describe('pointing the camera at a circuit that arrived', () => {
	// C-47's other half. `fitTo` is checked arithmetically in canvas.test.ts;
	// this is the plumbing — that the canvas can be *told* to show something,
	// which is what an opened file and an opened example both do. Observed
	// through `onviewport`, which is how the shell learns where the camera is.

	it('moves the camera to what it was told to show', async () => {
		const reported: { pan: { x: number; y: number }; zoom: number }[] = []
		const far = { x: 4000, y: 3000 }
		const shell = render(Canvas, {
			items: [{ id: 'r1', position: far, symbol: SYMBOLS.Resistor }],
			onviewport: (viewport: { pan: { x: number; y: number }; zoom: number }) =>
				reported.push(viewport),
		})
		// Mounted on its own it has no size: in the app it is a row of the
		// shell's grid, and here it is a block element in an empty div. The
		// camera cannot be fitted to a canvas of no size, and declines to try.
		const region = shell.querySelector('main')!
		region.style.width = '800px'
		region.style.height = '600px'
		const canvas = mounted as unknown as {
			show: (bounds: unknown) => void
			bounds: () => DOMRect | undefined
		}

		// The size arrives from a resize observer a frame or two after mount,
		// and `show` declines to guess before it has one.
		for (let frame = 0; frame < 120 && !(canvas.bounds()?.width ?? 0); frame++) {
			await new Promise(requestAnimationFrame)
			await tick()
		}
		const before = reported.at(-1)!
		canvas.show({ min: { x: far.x - 20, y: far.y - 20 }, max: { x: far.x + 20, y: far.y + 20 } })
		await tick()

		const after = reported.at(-1)!
		expect(after).not.toEqual(before)
		// The circuit is where the camera is now looking: a component four
		// thousand units out is on screen rather than off it.
		const box = canvas.bounds()!
		const onScreen = {
			x: (far.x - after.pan.x) * after.zoom,
			y: (far.y - after.pan.y) * after.zoom,
		}
		expect(onScreen.x).toBeGreaterThan(0)
		expect(onScreen.x).toBeLessThan(box.width)
		expect(onScreen.y).toBeGreaterThan(0)
		expect(onScreen.y).toBeLessThan(box.height)
	})

	it('fits to the size it has now, not the one it was last told', async () => {
		// An example with a probe brings the scope's row in, and the canvas
		// shrinks in the same update that asks for the fit: the resize
		// observer only reports it afterwards, and a fit to the old height
		// left the bottom of the circuit under the scope.
		const reported: { pan: { x: number; y: number }; zoom: number }[] = []
		const shell = render(Canvas, {
			items: [],
			onviewport: (viewport: { pan: { x: number; y: number }; zoom: number }) =>
				reported.push(viewport),
		})
		const region = shell.querySelector('main')!
		region.style.width = '800px'
		region.style.height = '600px'
		const canvas = mounted as unknown as {
			show: (bounds: unknown) => void
			bounds: () => DOMRect | undefined
		}
		for (let frame = 0; frame < 120 && !(canvas.bounds()?.width ?? 0); frame++) {
			await new Promise(requestAnimationFrame)
			await tick()
		}

		region.style.height = '200px'
		const bottom = { x: 400, y: 600 }
		canvas.show({ min: { x: 0, y: 0 }, max: bottom })
		await tick()

		const after = reported.at(-1)!
		expect((bottom.y - after.pan.y) * after.zoom).toBeLessThanOrEqual(200)
	})
})

describe('fingers on the board (C-138)', () => {
	type Camera = { pan: { x: number; y: number }; zoom: number }

	/**
	 * A canvas 800 × 600 with a resistor at the origin, once it has its size:
	 * the canvas, where the camera is, and what it was told.
	 */
	async function board(extra: Record<string, unknown> = {}) {
		const cameras: Camera[] = []
		const said: string[] = []
		const shell = render(Canvas, {
			items: [{ id: 'r1', position: { x: 0, y: 0 }, symbol: SYMBOLS.Resistor }],
			onviewport: (viewport: Camera) => cameras.push(viewport),
			onmove: (id: string, x: number, y: number) => said.push(`move ${id} ${x},${y}`),
			onrelease: (id: string, moved: boolean) => said.push(`release ${id} ${moved}`),
			onarea: (ids: string[]) => said.push(`area ${ids.join(',')}`),
			...extra,
		})
		const region = shell.querySelector('main')!
		region.style.width = '800px'
		region.style.height = '600px'
		const surface = shell.querySelector('canvas')!
		for (let frame = 0; frame < 120 && surface.getBoundingClientRect().width === 0; frame++) {
			await new Promise(requestAnimationFrame)
			await tick()
		}
		const box = surface.getBoundingClientRect()
		/** A finger's event at a point on the canvas. */
		const finger = (type: string, pointerId: number, x: number, y: number) =>
			surface.dispatchEvent(
				new PointerEvent(type, {
					pointerId,
					pointerType: 'touch',
					isPrimary: pointerId === 1,
					clientX: box.x + x,
					clientY: box.y + y,
					bubbles: true,
				}),
			)
		/** Where on the canvas a point of the circuit is drawn now. */
		const onScreen = (world: { x: number; y: number }) => {
			const camera = cameras.at(-1)!
			return {
				x: (world.x - camera.pan.x) * camera.zoom,
				y: (world.y - camera.pan.y) * camera.zoom,
			}
		}
		return { cameras, said, finger, onScreen }
	}

	it('zooms by how far two fingers spread, about the point between them', async () => {
		const { cameras, finger, onScreen } = await board()
		const before = cameras.at(-1)!
		finger('pointerdown', 1, 350, 300)
		finger('pointerdown', 2, 450, 300)
		// Twice as far apart, about the same middle.
		finger('pointermove', 1, 300, 300)
		finger('pointermove', 2, 500, 300)
		await tick()
		const after = cameras.at(-1)!
		expect(after.zoom / before.zoom).toBeCloseTo(2, 5)
		// The point under the middle of the fingers stays under it.
		const middle = { x: 400 / before.zoom + before.pan.x, y: 300 / before.zoom + before.pan.y }
		expect(onScreen(middle).x).toBeCloseTo(400, 5)
		expect(onScreen(middle).y).toBeCloseTo(300, 5)
	})

	it('pans with two fingers moved together', async () => {
		const { cameras, finger } = await board()
		const before = cameras.at(-1)!
		finger('pointerdown', 1, 350, 300)
		finger('pointerdown', 2, 450, 300)
		finger('pointermove', 1, 400, 340)
		finger('pointermove', 2, 500, 340)
		await tick()
		const after = cameras.at(-1)!
		expect(after.zoom).toBeCloseTo(before.zoom, 9)
		expect((before.pan.x - after.pan.x) * after.zoom).toBeCloseTo(50, 5)
		expect((before.pan.y - after.pan.y) * after.zoom).toBeCloseTo(40, 5)
	})

	it('puts back a part the first finger had begun to carry, and flips nothing', async () => {
		const { said, finger, onScreen } = await board()
		const part = onScreen({ x: 0, y: 0 })
		finger('pointerdown', 1, part.x, part.y)
		finger('pointermove', 1, part.x + 60, part.y)
		finger('pointerdown', 2, part.x + 200, part.y)
		finger('pointermove', 2, part.x + 300, part.y)
		finger('pointerup', 1, part.x + 60, part.y)
		// The finger left behind is spent: it moves nothing until it lifts.
		const count = said.length
		finger('pointermove', 2, part.x + 400, part.y)
		finger('pointerup', 2, part.x + 400, part.y)
		expect(said.length).toBe(count)
		expect(said.filter((line) => line.startsWith('move')).at(-1)).toBe('move r1 0,0')
		expect(said).toContain('release r1 true')
	})

	it('picks an area with a finger held still on the grid, then dragged', async () => {
		const { said, finger, onScreen } = await board()
		const part = onScreen({ x: 0, y: 0 })
		const from = { x: part.x - 120, y: part.y - 120 }
		finger('pointerdown', 1, from.x, from.y)
		await new Promise((resolve) => setTimeout(resolve, 550))
		finger('pointermove', 1, part.x + 120, part.y + 120)
		finger('pointerup', 1, part.x + 120, part.y + 120)
		expect(said).toContain('area r1')
	})

	it('pans rather than picks when the finger sets off at once', async () => {
		const { cameras, said, finger } = await board()
		const before = cameras.at(-1)!
		finger('pointerdown', 1, 100, 100)
		finger('pointermove', 1, 160, 100)
		await new Promise((resolve) => setTimeout(resolve, 550))
		finger('pointermove', 1, 200, 100)
		finger('pointerup', 1, 200, 100)
		expect(said.some((line) => line.startsWith('area'))).toBe(false)
		expect(cameras.at(-1)!.pan.x).not.toBe(before.pan.x)
	})
})

describe('the shell on a phone', () => {
	// C-51. The regions stack below 60rem, and stacked they are taller than a
	// phone — so a shell pinned to the viewport gave the canvas, the only row
	// that has to have a size, none of it. A drawing surface 0 px tall, on the
	// device C-27 had just made this installable on.

	let viewport: { width: number; height: number } | undefined
	beforeEach(async () => {
		viewport = { width: window.innerWidth, height: window.innerHeight }
		await page.viewport(375, 812)
	})
	afterEach(async () => {
		if (viewport) await page.viewport(viewport.width, viewport.height)
		forgetCircuit()
	})

	/** Waits for the shell to catch up, a frame at a time. */
	async function settle(check: () => boolean, what: string): Promise<void> {
		for (let frame = 0; frame < 120; frame++) {
			await new Promise(requestAnimationFrame)
			await tick()
			if (check()) return
		}
		throw new Error(`the shell never ${what}`)
	}

	it('still has a canvas to draw on', async () => {
		const shell = render(App)
		await settle(
			() => (shell.querySelector('canvas')?.getBoundingClientRect().height ?? 0) > 0,
			'gave the canvas a size',
		)

		const box = shell.querySelector('canvas')!.getBoundingClientRect()
		// Most of the screen, which is what the app is: anything much smaller
		// and a circuit of three components does not fit on it.
		expect(box.width).toBeGreaterThan(300)
		expect(box.height).toBeGreaterThan(400)
	})

	it('is one screen, with the status line at its foot and nothing to scroll (C-132)', async () => {
		// The findings and the accuracy claim are only true if they are visible
		// (C-26). Since C-132 the shell is exactly one screen and nothing in it
		// scrolls, so a drag on the board is never the page moving instead.
		const shell = render(App)
		await settle(() => shell.querySelector('footer') !== null, 'drew a status line')

		// Measured from the top of the shell rather than of the window: the
		// test runner mounts this inside a page of its own.
		const box = shell.querySelector('.shell')!.getBoundingClientRect()
		const footer = shell.querySelector('footer')!.getBoundingClientRect()
		expect(Math.abs(box.height - window.innerHeight)).toBeLessThanOrEqual(1)
		expect(footer.bottom - box.top).toBeLessThanOrEqual(window.innerHeight + 1)
		expect(getComputedStyle(shell.querySelector('.shell')!).overflow).toBe('hidden')
	})

	it('gives the scope’s time bases a row of their own, each a finger’s width (C-138)', async () => {
		// Beside the title they wrapped onto two lines, 24 px tall.
		const probes: ElementView[] = [{ ...resistor(), id: 'v1', kind: 'Voltmeter', unit: 'V' }]
		const shell = render(Scope, { elements: probes })
		await tick()
		const bases = [
			...shell.querySelectorAll<HTMLElement>('[aria-label="Time base"] [role="radio"]'),
		]
		const tops = new Set(bases.map((base) => Math.round(base.getBoundingClientRect().top)))
		expect(tops.size, 'on one row').toBe(1)
		for (const base of bases) {
			expect(base.getBoundingClientRect().height).toBeGreaterThanOrEqual(44)
			// One line: as tall as one line of its text would make it.
			expect(base.scrollHeight).toBeLessThanOrEqual(base.clientHeight)
		}
		const title = shell.querySelector('h2')!.getBoundingClientRect()
		expect(bases[0].getBoundingClientRect().top).toBeGreaterThan(title.bottom)
	})

	it('leaves the examples on an empty board clear of the buttons over it (C-138)', async () => {
		// The thumb bar and the view switch sat over the first examples, and
		// the kinds of example took three rows of the little room left.
		const circuit = await createCircuit()
		const shell = render(App, { circuit })
		await settle(() => shell.querySelector('.offer .examples li') !== null, 'offered the examples')

		expect(shell.querySelector('.view-switch'), 'no breadboard of nothing to switch to').toBeNull()
		const offer = shell.querySelector('.offer')!.getBoundingClientRect()
		const bar = shell.querySelector('.thumbbar')!.getBoundingClientRect()
		expect(offer.bottom).toBeLessThanOrEqual(bar.top)
		const kinds = [...shell.querySelectorAll<HTMLElement>('.offer .shelves button')]
		const tops = new Set(kinds.map((kind) => Math.round(kind.getBoundingClientRect().top)))
		expect(tops.size, 'the kinds on one row').toBe(1)
		const first = shell.querySelector('.offer .examples li')!.getBoundingClientRect()
		expect(first.bottom, 'a whole example on screen').toBeLessThanOrEqual(bar.top)
	})

	it('puts the parts one tap away, in a sheet over the board (C-132)', async () => {
		// The board has the screen, so the parts are not beside it: a button
		// where a thumb is opens them, and a tap on one places it.
		const circuit = await createCircuit()
		const shell = render(App, { circuit })
		await settle(() => shell.querySelector('.dot.ready') !== null, 'loaded the core')
		// The sheet is fixed to the window, so it is measured against the window.
		const rail = () => shell.querySelector('.rail')!.getBoundingClientRect()
		expect(rail().top, 'out of the way to begin with').toBeGreaterThanOrEqual(
			window.innerHeight - 1,
		)

		const add = [...shell.querySelectorAll<HTMLButtonElement>('.thumbbar button')].find(
			(button) => button.textContent?.trim() === 'Add a part',
		)!
		expect(add.getBoundingClientRect().height).toBeGreaterThanOrEqual(44)
		add.click()
		await new Promise((resolve) => setTimeout(resolve, 300))
		await settle(() => rail().top < window.innerHeight / 2, 'raised the parts')
		// Then the first part is on screen to be tapped.
		const first = shell.querySelector<HTMLButtonElement>(
			'[aria-label="Components"] button.starter, [aria-label="Components"] button.entry',
		)!
		expect(first.getBoundingClientRect().bottom).toBeLessThan(window.innerHeight)

		// A tap is a press and a release where it started, then a click. The
		// sheet lies over the board's rectangle, and the release must not be
		// taken for a drop on the board under it as well.
		const box = first.getBoundingClientRect()
		const at = { clientX: box.x + 10, clientY: box.y + 10, bubbles: true }
		first.dispatchEvent(new PointerEvent('pointerdown', at))
		window.dispatchEvent(new PointerEvent('pointerup', at))
		first.click()
		expect(circuit.size, 'a tap placed it, once').toBe(1)
		// The sheet steps aside for the part, and the panel rises for it.
		await settle(() => shell.querySelector('.shell.panel-open') !== null, 'opened the panel on it')
		expect(shell.querySelector('.shell.parts-open')).toBeNull()
	})
})

describe('theme', () => {
	it('defines every colour token the components reference', () => {
		render(App)
		const tokens = getComputedStyle(document.documentElement)

		// `--grid` among them: the canvas reads its colours out of CSS rather
		// than holding its own, so a token that goes missing is a canvas drawn
		// in whatever the fallback happened to be.
		for (const token of ['--bg', '--sunken', '--raised', '--edge', '--grid', '--fg', '--muted']) {
			expect(tokens.getPropertyValue(token).trim(), token).not.toBe('')
		}
	})

	it('writes every family colour, and selected text, legibly in both themes', () => {
		// WCAG's 4.5:1 for text. The family colours are the palette's headings
		// as well as its tints, and three of them were too light on white.
		const luminance = (hex: string) => {
			const [r, g, b] = [1, 3, 5].map((at) => {
				const channel = parseInt(hex.slice(at, at + 2), 16) / 255
				return channel <= 0.03928 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4
			})
			return 0.2126 * r + 0.7152 * g + 0.0722 * b
		}
		const contrast = (a: string, b: string) => {
			const [light, dark] = [luminance(a), luminance(b)].sort((x, y) => y - x)
			return (light + 0.05) / (dark + 0.05)
		}
		render(App)
		const root = document.documentElement
		const families = [
			'power',
			'lights',
			'passive',
			'switches',
			'motion',
			'chips',
			'logic',
			'meters',
		]
		try {
			for (const theme of ['light', 'dark']) {
				root.setAttribute('data-theme', theme)
				const token = (name: string) => getComputedStyle(root).getPropertyValue(name).trim()
				for (const family of families) {
					const ratio = contrast(token(`--family-${family}`), token('--bg'))
					expect(ratio, `${family} on ${theme}`).toBeGreaterThanOrEqual(4.5)
				}
				const selected = contrast(token('--on-selection'), token('--selection'))
				expect(selected, `selected text on ${theme}`).toBeGreaterThanOrEqual(4.5)
			}
		} finally {
			root.removeAttribute('data-theme')
		}
	})

	it('answers to an explicit theme as well as to the system one', () => {
		// Nothing sets data-theme yet; the override exists so a toggle is a
		// one-liner rather than a restructuring.
		render(App)
		const root = document.documentElement
		const automatic = getComputedStyle(root).getPropertyValue('--bg').trim()

		root.setAttribute('data-theme', 'dark')
		const dark = getComputedStyle(root).getPropertyValue('--bg').trim()
		root.setAttribute('data-theme', 'light')
		const light = getComputedStyle(root).getPropertyValue('--bg').trim()
		root.removeAttribute('data-theme')

		expect(dark).not.toBe(light)
		expect([dark, light]).toContain(automatic)
	})
})

describe('the canvas follows the theme', () => {
	it('redraws in the new colours when the theme changes under it', async () => {
		// The canvas reads its colours once and keeps them, rather than asking
		// CSS every frame; this is what says it still notices a change.
		const shell = render(Canvas, {
			items: [{ id: 'r1', position: { x: 0, y: 0 }, symbol: SYMBOLS.Resistor }],
		})
		const region = shell.querySelector('main')!
		region.style.width = '300px'
		region.style.height = '200px'
		const surface = shell.querySelector('canvas')!
		const pixels = () =>
			surface.getContext('2d')!.getImageData(0, 0, surface.width, surface.height).data.join(',')
		const frames = async (count: number) => {
			for (let frame = 0; frame < count; frame++) {
				await new Promise(requestAnimationFrame)
				await tick()
			}
		}
		const root = document.documentElement
		try {
			root.setAttribute('data-theme', 'dark')
			await frames(10)
			const dark = pixels()

			root.setAttribute('data-theme', 'light')
			await frames(3)

			expect(pixels()).not.toBe(dark)
		} finally {
			root.removeAttribute('data-theme')
		}
	})
})

describe('the serial monitor', () => {
	it('can be scrolled back from the keyboard', () => {
		// It scrolls, so it has to be somewhere Tab can reach: axe's
		// scrollable-region-focusable, which it failed.
		const log = new SerialLog()
		log.text = { u1: 'hello\n'.repeat(200) }
		const shell = render(SerialMonitor, {
			ports: [{ elementId: 'u1', monitorBaud: 9600, programBaud: 9600 }],
			log,
		})

		const transcript = shell.querySelector<HTMLElement>('[aria-label="What the program printed"]')!
		transcript.focus()
		expect(document.activeElement).toBe(transcript)
	})
})

describe('selection', () => {
	it('says how many are selected rather than none', () => {
		// Two components highlighted on the canvas and "Nothing selected." in the
		// panel is the app contradicting itself.
		const shell = render(Inspector, { count: 3 })

		expect(shell.textContent).toContain('3 components selected')
	})

	it('still says nothing when nothing is selected', () => {
		expect(render(Inspector, { count: 0 }).textContent).toContain('Nothing selected')
	})
})

describe('the empty canvas', () => {
	it('says how to start when there is nothing yet', () => {
		const shell = render(Canvas)

		expect(shell.textContent).toContain('Drag a component onto the grid')
	})

	it('says what the components are for', () => {
		// The audience has never built a circuit: naming the three that make a
		// light is the difference between a palette and a puzzle.
		const shell = render(Canvas)
		expect(shell.textContent).toContain('will light up')
		// C-131: the three, in the order a first circuit is built, each drawn.
		const steps = [...shell.querySelectorAll('.steps li')]
		expect(steps.map((step) => step.querySelector('.part')?.textContent)).toEqual([
			CATALOGUE.DcSource.label,
			CATALOGUE.Resistor.label,
			CATALOGUE.Led.label,
		])
		expect(steps.every((step) => step.querySelector('svg path') !== null)).toBe(true)
	})

	it('gets out of the way once something is placed', () => {
		const shell = render(Canvas, {
			items: [{ id: 'r1', position: { x: 0, y: 0 }, symbol: SYMBOLS.Resistor }],
		})

		expect(shell.textContent).not.toContain('Drag a component')
	})

	it('never intercepts the drag it is describing', () => {
		// A hint covering the canvas that swallowed the drop would be a cruel
		// joke, and it is one CSS property away at all times.
		const shell = render(Canvas)
		const hint = shell.querySelector('.empty')!

		expect(getComputedStyle(hint).pointerEvents).toBe('none')
	})
})

describe('what the app noticed', () => {
	it('says the finding instead of the clock', () => {
		// A finding is the most useful thing the app can say, so it takes the
		// line. Reporting how long the simulation has run while an LED is
		// backwards would be the app admiring itself.
		const shell = render(StatusBar, {
			status: 'ready',
			detail: '12.0 s simulated',
			findings: [{ severity: 'warning', message: 'This LED is the wrong way round' }],
		})

		expect(shell.textContent).toContain('wrong way round')
		expect(shell.textContent).not.toContain('12.0 s')
	})

	it('goes back to the clock once there is nothing to say', () => {
		const shell = render(StatusBar, {
			status: 'ready',
			detail: '12.0 s simulated',
			findings: [],
		})

		expect(shell.textContent).toContain('12.0 s')
	})

	it('says it quietly rather than in a dialog', () => {
		// B-07 is where "interrupt or annotate" is decided properly. Until then
		// the reversible choice is the quiet one: nothing to dismiss, nothing
		// blocking the canvas.
		const shell = render(StatusBar, {
			status: 'ready',
			detail: '',
			findings: [{ severity: 'warning', message: 'Backwards' }],
		})

		expect(shell.querySelector('dialog')).toBeNull()
		expect(shell.querySelector('footer')).not.toBeNull()
	})
})

describe('what the palette offers', () => {
	it('offers nothing it cannot describe', () => {
		// The list itself lives in the core's registry now, and that the palette
		// matches it exactly — same kinds, same order — is checked against the
		// registry in `canvas.test.ts` (ADR 0008). What is checked here is that
		// every entry the palette renders is a real catalogue entry rather than
		// a stray object, which is the part this file can see.
		expect(PALETTE.length).toBeGreaterThan(0)
		for (const entry of PALETTE) {
			expect(CATALOGUE[entry.kind], entry.kind).toBe(entry)
		}
	})

	it('still knows how to draw a ground that exists', () => {
		// The element did not go away, only the offer of one: a circuit that
		// places a ground still has a reference and still draws.
		expect(SYMBOLS.Ground).toBeDefined()
		expect(SYMBOLS.Ground.pins.map((pin) => pin.name)).toEqual(['gnd'])
	})
})

describe('a selected component explains itself', () => {
	it('names it the way a beginner would, not the way the core does', () => {
		// "DcSource" is the core's spelling of a battery. Printing it to someone
		// who has never built a circuit is the app talking to itself.
		const shell = render(Inspector, {
			selection: { ...resistor(), id: 'b1', kind: 'DcSource', label: '9 V', value: '9', unit: 'V' },
		})

		expect(shell.textContent).toContain('Battery')
		expect(shell.textContent).not.toContain('DcSource')
	})

	it('says what the component does', () => {
		expect(render(Inspector, { selection: resistor() }).textContent).toContain(
			CATALOGUE.Resistor.does,
		)
	})

	it('says what to watch out for', () => {
		// The LED is the one that gets destroyed by being got wrong, so the
		// warning belongs beside it rather than in a manual.
		const shell = render(Inspector, {
			selection: { ...resistor(), id: 'd1', kind: 'Led', label: 'red', value: undefined, unit: '' },
		})

		expect(shell.textContent).toContain(CATALOGUE.Led.watch)
	})

	it("falls back to the core's spelling for a kind it has never heard of", () => {
		// The core is free to grow an element before the app catches up. Saying
		// nothing about it beats inventing something. The types now say this
		// cannot happen — `ComponentKind` is the core's own list — so this is
		// the runtime half: a mismatched build still shows something.
		const shell = render(Inspector, {
			selection: { ...resistor(), kind: 'Thyristor' as unknown as ComponentKind },
		})

		expect(shell.textContent).toContain('Thyristor')
	})

	it('keeps the explanation short enough to always be there', () => {
		// Ambient help earns its place by costing nothing to read past (design
		// doc §14). Two sentences is the budget; a paragraph is a different
		// mechanism and belongs on demand.
		for (const info of Object.values(CATALOGUE)) {
			expect(info.does.length, info.label).toBeLessThan(90)
			expect(info.watch.length, info.label).toBeLessThan(90)
		}
	})
})

describe('hovering reads a component out loud', () => {
	const at = { x: 100, y: 100 }

	it('says what it is and what it is doing', () => {
		const shell = render(Readout, {
			info: CATALOGUE.Resistor,
			reading: { volts: 1.5, amps: 0.0068 },
			at,
		})

		expect(shell.textContent).toContain('Resistor')
		expect(shell.textContent).toContain('Limits how much current flows')
		expect(shell.textContent).toContain('1.50 V across')
		expect(shell.textContent).toContain('6.80 mA through')
	})

	it('says only what it knows', () => {
		// A component with one pin dangling has no voltage across it. Printing
		// "0 V" would be the app answering a question it cannot answer.
		const shell = render(Readout, { info: CATALOGUE.Led, reading: {}, at })

		expect(shell.textContent).toContain('LED')
		expect(shell.textContent).not.toContain('across')
		expect(shell.textContent).not.toContain('through')
	})

	it('never intercepts the pointer it is following', () => {
		const shell = render(Readout, { info: CATALOGUE.Resistor, reading: {}, at })

		expect(getComputedStyle(shell.querySelector('.readout')!).pointerEvents).toBe('none')
	})

	it('stays on screen near the right edge', () => {
		// Beside the pointer normally, on the other side of it when there is no
		// room: a readout half off the screen answers nothing.
		const shell = render(Readout, {
			info: CATALOGUE.Resistor,
			reading: {},
			at: { x: window.innerWidth - 4, y: 100 },
		})

		expect(shell.querySelector<HTMLElement>('.readout')!.style.transform).toContain('-100%')
	})
})

describe('the pointer leaving the canvas', () => {
	it('reports that nothing is under it any more', async () => {
		// No further move event arrives once the pointer is gone, so without
		// this the last thing hovered stays hovered for good.
		let left = 0
		const shell = render(Canvas, { onleave: () => left++ })
		// Svelte attaches pointer listeners in a microtask, to work around a
		// Chrome bug with cloned nodes. Dispatching before that lands tests
		// nothing at all.
		await tick()

		shell.querySelector('canvas')!.dispatchEvent(new PointerEvent('pointerleave'))

		expect(left).toBe(1)
	})
})

describe('pressing something on the canvas', () => {
	it('reports the release before it closes the drag', async () => {
		// C-19. Both halves of a press become edits, and both have to land
		// inside the gesture the press opened: a release that landed outside
		// would leave a step of history with the button still down, and undoing
		// a press would put it back stuck.
		const order: string[] = []
		const shell = render(Canvas, {
			items: [{ id: 's1', position: { x: 60, y: 40 }, symbol: SYMBOLS.Switch }],
			onbeginmove: () => order.push('begin'),
			onpress: (id: string) => order.push(`press ${id}`),
			onrelease: (id: string, moved: boolean) => order.push(`release ${id} ${moved}`),
			onendmove: () => order.push('end'),
		})
		await tick()

		const canvas = shell.querySelector('canvas')!
		// A synthetic event has no active pointer behind it, and capturing one
		// that does not exist throws.
		canvas.setPointerCapture = () => {}
		canvas.releasePointerCapture = () => {}
		const box = canvas.getBoundingClientRect()
		for (const type of ['pointerdown', 'pointerup']) {
			canvas.dispatchEvent(
				new PointerEvent(type, {
					clientX: box.left + 60,
					clientY: box.top + 40,
					pointerId: 1,
					bubbles: true,
				}),
			)
		}

		// And `moved` is false, because nothing was dragged anywhere: that is
		// what tells a click on a switch from carrying one across the canvas.
		expect(order).toEqual(['begin', 'press s1', 'release s1 false', 'end'])
	})
})

describe('the overlay explains itself', () => {
	it('says what the moving dots are', () => {
		// The animation is the app's main channel for "current is flowing, this
		// way round", and it arrives unannounced.
		const shell = render(OverlayNote)

		expect(shell.textContent).toContain('moving dots are the current')
	})

	it('can be sent away', () => {
		let dismissed = 0
		const shell = render(OverlayNote, { ondismiss: () => dismissed++ })

		shell.querySelector('button')!.click()

		expect(dismissed).toBe(1)
	})

	it('does not block the canvas it is explaining', () => {
		// Anything that has to be clicked through before the circuit can be
		// touched again is a dialog, and §14 is explicit that this is not one.
		const shell = render(OverlayNote)

		expect(shell.querySelector('dialog')).toBeNull()
		expect(getComputedStyle(shell.querySelector('.note')!).position).toBe('absolute')
	})
})

describe('the app says which keys it answers to', () => {
	async function until(check: () => boolean, what: string): Promise<void> {
		for (let frame = 0; frame < 120; frame++) {
			await new Promise(requestAnimationFrame)
			await tick()
			if (check()) return
		}
		throw new Error(`the shell never ${what}`)
	}

	it('lists every binding, not a useful subset', () => {
		// The finding was not a missing shortcut. It was that a user could not
		// tell "there is no shortcut for this" from "I have not guessed it yet",
		// and a partial list answers neither (the author's test, M-22).
		const shell = render(Keys)

		for (const binding of keyBindings()) {
			expect(shell.textContent, binding.does).toContain(binding.does)
			expect(shell.textContent, binding.label).toContain(binding.label)
		}
	})

	it('lists what is done with a pointer too', () => {
		// Shift-click and drag-to-pan have no key to press and are no easier to
		// guess for it.
		const shell = render(Keys)

		for (const gesture of GESTURES) {
			expect(shell.textContent, gesture.does).toContain(gesture.does)
		}
	})

	it('says how to delete, which is what nobody could guess', () => {
		expect(render(Keys).textContent).toContain('Remove the selection')
	})

	it('is one press away, of the header’s ? or of the key ? (C-127)', async () => {
		// It used to sit in the rail whether or not anyone was reading it
		// (C-42); the board needed the room. What C-42 was for is kept: the
		// whole list, from the one table, and nothing to hunt for.
		const shell = render(App)
		const open = () => shell.querySelector('[role="dialog"][aria-label="Shortcuts"]')
		expect(open(), 'closed to begin with').toBeNull()

		shell.querySelector<HTMLButtonElement>('header button[aria-label="Shortcuts"]')!.click()
		await tick()
		expect(open()?.textContent).toContain('Remove the selection')

		window.dispatchEvent(new KeyboardEvent('keydown', { key: '?', shiftKey: true }))
		await tick()
		expect(open(), 'the key closes it again').toBeNull()
		window.dispatchEvent(new KeyboardEvent('keydown', { key: '?', shiftKey: true }))
		await tick()
		expect(open(), 'and opens it').not.toBeNull()

		open()!.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
		await tick()
		expect(open(), 'Escape puts it away').toBeNull()
	})

	it('writes the key on the canvas’s Turn button, beside what it does (C-127)', async () => {
		// The half of C-42's finding that a closed list cannot answer: how do
		// I turn this? Said where the question is asked.
		const circuit = await createCircuit()
		const shell = render(App, { circuit })
		await until(() => shell.querySelector('.dot.ready') !== null, 'loaded the core')
		;[...shell.querySelectorAll<HTMLButtonElement>('.palette .entry')]
			.find((entry) => entry.querySelector('.label')?.textContent === CATALOGUE.Resistor.label)!
			.click()
		await until(() => shell.querySelector('.selection-actions') !== null, 'offered to turn it')

		const turn = [...shell.querySelectorAll<HTMLButtonElement>('.selection-actions button')].find(
			(button) => button.textContent?.trim() === 'Turn 90°',
		)!
		const rotate = keyBindings().find((binding) => binding.action === 'rotate')!
		expect(turn.dataset.key).toBe(rotate.label)
		expect(getComputedStyle(turn, '::after').content).toBe(`"${rotate.label}"`)
	})
})

describe('deleting a wire', () => {
	it('describes the selected wire instead of claiming nothing is selected', () => {
		// The canvas lights the wire up. An inspector saying "Nothing selected"
		// beside it is the contradiction C-23 fixed for components.
		const shell = render(Inspector, {
			wire: {
				id: 'n1',
				waypoints: [],
				pins: [
					{ elementId: 'b1', pin: '+' },
					{ elementId: 'r1', pin: 'a' },
				],
			},
		})

		expect(shell.textContent).toContain('Wire')
		expect(shell.textContent).not.toContain('Nothing selected')
	})

	it('says what deleting it will and will not do', () => {
		// The whole finding: until now the only way to be rid of a wire was to
		// delete a component that was perfectly fine.
		const shell = render(Inspector, { wire: { id: 'n1', pins: [], waypoints: [] } })

		expect(shell.textContent).toContain('leaves the components where they are')
	})

	it('offers no value to edit, because a wire has none', () => {
		const shell = render(Inspector, { wire: { id: 'n1', pins: [], waypoints: [] } })

		expect(shell.querySelector('input')).toBeNull()
	})

	it('still says nothing when nothing at all is selected', () => {
		expect(render(Inspector).textContent).toContain('Nothing selected')
	})
})

describe("a source's internal resistance", () => {
	/** A battery, as the core describes one. */
	function battery(seriesOhms: number | undefined = 0): ElementView {
		return {
			...resistor(),
			id: 'b1',
			kind: 'DcSource',
			label: '9 V',
			value: '9',
			unit: 'V',
			seriesOhms,
			hertz: undefined,
		}
	}

	it('is offered on a source and nothing else', () => {
		// C-17. A resistor has no inside to have a resistance in.
		expect(
			render(Inspector, { selection: battery() }).querySelector(
				'[aria-label="Internal resistance"]',
			),
		).not.toBeNull()
		expect(
			render(Inspector, { selection: resistor() }).querySelector(
				'[aria-label="Internal resistance"]',
			),
		).toBeNull()
	})

	it('shows an ideal source as empty rather than as a zero', () => {
		// Nought ohms is the absence of a thing, and a field reading "0" invites
		// a beginner to wonder what they got wrong.
		const shell = render(Inspector, { selection: battery(0) })

		expect(shell.querySelector<HTMLInputElement>('[aria-label="Internal resistance"]')?.value).toBe(
			'',
		)
	})

	it('shows a resistance it has been given', () => {
		const shell = render(Inspector, { selection: battery(0.5) })

		expect(shell.querySelector<HTMLInputElement>('[aria-label="Internal resistance"]')?.value).toBe(
			'0.5',
		)
	})

	it('reports what was typed, separately from the value', () => {
		const edits: [string, string][] = []
		const shell = render(Inspector, {
			selection: battery(),
			onseries: (id: string, text: string) => edits.push([id, text]),
		})

		const field = shell.querySelector<HTMLInputElement>('[aria-label="Internal resistance"]')!
		field.value = '0.1'
		field.dispatchEvent(new Event('change', { bubbles: true }))

		expect(edits).toEqual([['b1', '0.1']])
	})

	it('says what leaving it empty means', () => {
		expect(render(Inspector, { selection: battery() }).textContent).toContain('perfect battery')
	})
})

describe('getting a circuit out and back in', () => {
	it('opens on the schematic, the breadboard one press away (B-19)', () => {
		// Denis's call, 2026-10-03: schematic first for a first-time user.
		const shell = render(App)
		const pressed = [...shell.querySelectorAll('header .views button')].map((button) => [
			button.textContent?.trim(),
			button.getAttribute('aria-pressed'),
		])
		expect(pressed).toEqual([
			['Schematic', 'true'],
			['Breadboard', 'false'],
		])
		expect(shell.querySelector('.breadboard')).toBeNull()
	})

	it('offers both without hunting for them', () => {
		// C-29. A save nobody can find is a save nobody makes, and the autosave
		// beside it must never be the only copy.
		const shell = render(App)
		const buttons = [...shell.querySelectorAll('header button')].map(
			(b) => b.textContent?.trim() || b.getAttribute('aria-label'),
		)

		// And Clear beside them, which an undo takes back (so it needs no
		// "are you sure").
		// Undo and Redo first, for a phone with no keyboard (C-99), and the
		// shortcuts last, behind their ? (C-127). Not in a File menu: that
		// would be hunting for them.
		// The two views of the circuit come before all of them (B-14).
		expect(buttons).toEqual([
			'Schematic',
			'Breadboard',
			'Undo',
			'Redo',
			'Examples',
			'Open…',
			'Save',
			'Clear',
			'Shortcuts',
		])
	})
})

describe('an edit still in the field when the selection moves on', () => {
	// The whole sequence, in the app rather than in a component: type a value,
	// then click another component instead of pressing Enter. `change` fires on
	// blur, and blur happens *after* the click has already moved the selection
	// on — so an inspector that read the selection at that moment applied the
	// resistor's text to the battery, and left the resistor at 470.

	/** Mounting the shell autosaves; the next test must not open it again. */
	afterEach(() => forgetCircuit())

	// The canvas has to be big enough to drop a component onto and click it
	// again. Tests run at a phone's width by default, where the shell stacks its
	// regions and the canvas row is squeezed to nothing — so this one asks for
	// the width the app is drawn for, and puts it back afterwards.
	let viewport: { width: number; height: number } | undefined
	beforeEach(async () => {
		viewport = { width: window.innerWidth, height: window.innerHeight }
		await page.viewport(1280, 800)
	})
	afterEach(async () => {
		if (viewport) await page.viewport(viewport.width, viewport.height)
	})

	/** Waits for the shell to catch up, a frame at a time. */
	async function until(check: () => boolean, what: string): Promise<void> {
		for (let frame = 0; frame < 120; frame++) {
			await new Promise(requestAnimationFrame)
			await tick()
			if (check()) return
		}
		throw new Error(`the shell never ${what}`)
	}

	/** The drawing surface, with pointer capture stubbed: a synthetic pointer has none. */
	function surfaceOf(shell: HTMLElement): HTMLCanvasElement {
		const canvas = shell.querySelector('canvas')!
		canvas.setPointerCapture = () => {}
		canvas.releasePointerCapture = () => {}
		return canvas
	}

	/** Client coordinates of a world point, at the untouched viewport. */
	function at(shell: HTMLElement, x: number, y: number) {
		const box = surfaceOf(shell).getBoundingClientRect()
		return { clientX: box.left + x, clientY: box.top + y, pointerId: 1, bubbles: true }
	}

	/** Carries a palette entry onto the canvas, which is how a component is placed. */
	async function place(shell: HTMLElement, label: string, x: number, y: number) {
		const entry = [...shell.querySelectorAll('[aria-label="Components"] button')].find(
			(button) => button.querySelector('.label')?.textContent === label,
		)!
		entry.dispatchEvent(new PointerEvent('pointerdown', at(shell, 0, 0)))
		window.dispatchEvent(new PointerEvent('pointermove', at(shell, x, y)))
		window.dispatchEvent(new PointerEvent('pointerup', { bubbles: true }))

		// Placed once the canvas can find it there: hovering a component is what
		// produces the readout naming what is under the pointer, so that is the
		// shell saying the element exists and is where it was dropped.
		await until(() => {
			surfaceOf(shell).dispatchEvent(new PointerEvent('pointermove', at(shell, x, y)))
			return shell.querySelector('.readout')?.textContent?.includes(label) === true
		}, `placed a ${label}`)
	}

	/** Reads the inspector: what it says is selected, and what it offers to edit. */
	function inspector(shell: HTMLElement) {
		const panel = shell.querySelector('[aria-label="Inspector"]')!
		return {
			name: panel.querySelector('.name')?.textContent,
			value: panel.querySelector<HTMLInputElement>('[aria-label="Value"]')?.value,
		}
	}

	it('says what a part is doing when a finger rests on it, where there is no hover (C-133)', async () => {
		const shell = render(App)
		await until(() => shell.querySelector('.dot.ready') !== null, 'loaded the core')
		await until(() => surfaceOf(shell).getBoundingClientRect().width > 0, 'measured its canvas')
		await place(shell, 'Resistor', 140, 40)
		const canvas = surfaceOf(shell)
		canvas.dispatchEvent(new PointerEvent('pointerleave', at(shell, 140, 40)))
		await until(() => shell.querySelector('.readout') === null, 'forgot the hover')

		const finger = { ...at(shell, 140, 40), pointerType: 'touch' }
		canvas.dispatchEvent(new PointerEvent('pointerdown', finger))
		// A tap is not a question: nothing yet.
		expect(shell.querySelector('.readout')).toBeNull()
		await until(
			() => shell.querySelector('.readout')?.textContent?.includes('Resistor') === true,
			'answered the hold',
		)

		canvas.dispatchEvent(new PointerEvent('pointerup', finger))
		await until(
			() => shell.querySelector('.readout') === null,
			'withdrew it when the finger lifted',
		)
	})

	it('withdraws it when the resting finger sets off to drag (C-133)', async () => {
		const shell = render(App)
		await until(() => shell.querySelector('.dot.ready') !== null, 'loaded the core')
		await until(() => surfaceOf(shell).getBoundingClientRect().width > 0, 'measured its canvas')
		await place(shell, 'Resistor', 140, 40)
		const canvas = surfaceOf(shell)
		canvas.dispatchEvent(new PointerEvent('pointerleave', at(shell, 140, 40)))

		canvas.dispatchEvent(
			new PointerEvent('pointerdown', { ...at(shell, 140, 40), pointerType: 'touch' }),
		)
		await until(() => shell.querySelector('.readout') !== null, 'answered the hold')
		canvas.dispatchEvent(
			new PointerEvent('pointermove', { ...at(shell, 200, 40), pointerType: 'touch' }),
		)
		await until(() => shell.querySelector('.readout') === null, 'got out of the way of the drag')
		canvas.dispatchEvent(
			new PointerEvent('pointerup', { ...at(shell, 200, 40), pointerType: 'touch' }),
		)
	})

	it('applies it to the component it was typed for, not the one clicked next', async () => {
		const shell = render(App)
		await until(() => shell.querySelector('.dot.ready') !== null, 'loaded the core')
		// A drop is only on the canvas if the canvas has a size, and it gets one
		// from a resize observer a frame or two after it is mounted. Dropping
		// before that lands outside it, which is a cancelled drag and places
		// nothing at all.
		await until(() => surfaceOf(shell).getBoundingClientRect().width > 0, 'measured its canvas')

		// A battery and a resistor, far enough apart that a click on one lands
		// nowhere near the other's pins.
		await place(shell, 'Battery', 40, 40)
		await place(shell, 'Resistor', 140, 40)

		const canvas = surfaceOf(shell)
		canvas.dispatchEvent(new PointerEvent('pointerdown', at(shell, 140, 40)))
		canvas.dispatchEvent(new PointerEvent('pointerup', at(shell, 140, 40)))
		await until(() => inspector(shell).value === '470', 'selected the resistor')

		// Typed, not committed: no Enter, and the field is left dirty.
		const field = shell.querySelector<HTMLInputElement>('[aria-label="Value"]')!
		field.value = '1k'

		// The browser's order, and the whole of the bug: the click moves the
		// selection first, the field commits afterwards. Nothing is awaited in
		// between, because nothing is in the real thing either — the shell has
		// not redrawn yet, so this is still the resistor's input.
		canvas.dispatchEvent(new PointerEvent('pointerdown', at(shell, 40, 40)))
		field.dispatchEvent(new Event('change', { bubbles: true }))
		canvas.dispatchEvent(new PointerEvent('pointerup', at(shell, 40, 40)))

		await until(() => inspector(shell).name === 'Battery', 'selected the battery')
		expect(inspector(shell).value).toBe('9')

		canvas.dispatchEvent(new PointerEvent('pointerdown', at(shell, 140, 40)))
		canvas.dispatchEvent(new PointerEvent('pointerup', at(shell, 140, 40)))
		await until(() => inspector(shell).name === 'Resistor', 'came back to the resistor')
		expect(inspector(shell).value).toBe('1k')
	})
})
