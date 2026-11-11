<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

<script lang="ts">
	import { untrack } from 'svelte'
	import Callout from './components/Callout.svelte'
	import Canvas from './components/Canvas.svelte'
	import Breadboard from './components/Breadboard.svelte'
	import Description from './components/Description.svelte'
	import Inspector from './components/Inspector.svelte'
	import OverlayNote from './components/OverlayNote.svelte'
	import Palette from './components/Palette.svelte'
	import Readout from './components/Readout.svelte'
	import Scope from './components/Scope.svelte'
	import SerialMonitor from './components/SerialMonitor.svelte'
	import Shortcuts from './components/Shortcuts.svelte'
	import StatusBar from './components/StatusBar.svelte'
	import { coreVersion, createCircuit, type BoardView, type Circuit } from './lib/core'
	import {
		download,
		forgetCircuit,
		forgetSetAside,
		recallSetAside,
		setCircuitAside,
		openFile,
		openProgram,
		recallCircuit,
		rememberCircuit,
	} from './lib/circuitfile'
	import { afterPress, afterRelease } from './lib/contacts'
	import { detailOf, explain, splitRefusal } from './lib/failures'
	import { actionFor, keyBindings, touchOnly } from './lib/keys'
	import { CURRENT_OVERLAY, hasSeen, remember } from './lib/once'
	import { describe } from './lib/palette'
	import type { Example, ExampleGroup } from './lib/examples'
	import { glyphOf } from './lib/glyph'
	import { paintVoltages, scaleOf } from './lib/voltage'
	import { formatQuantity, spokenValue } from './lib/units'
	import { billToCsv, billToMarkdown } from './lib/bom'
	import { familyOf } from './lib/palette'
	import { clickPosition, dropPosition, moveDrag, startDrag, type Drag } from './lib/placement'
	import { readingFor } from './lib/readings'
	import { Steady } from './lib/steady'
	import { Projection } from './lib/projection.svelte'
	import { lanesFor } from './lib/scope'
	import { Buzz, TONE_HZ } from './lib/sound'
	import { Pace, slowedWords } from './lib/pace'
	import { PROGRAMS } from './lib/programs'
	import { replacedSentence } from './lib/replace'
	import { boundsOf, hitTest, RouteCache, toScene, toWires, type Hit } from './lib/scene'
	import { withoutRedundant } from './lib/routing'
	import { halfSizeOf, symbolFor, type SymbolDefinition, type SymbolKind } from './lib/symbols'
	import { INITIAL_VIEWPORT, toScreen, toWorld, type Point, type Viewport } from './lib/viewport'

	/**
	 * A circuit to show, for a caller that has one.
	 *
	 * The app makes its own when nothing is passed, which is what `main.ts`
	 * does and what every screenshot of it is. Being able to hand it one is
	 * what lets a test edit the netlist from outside the app entirely and
	 * watch the canvas follow, which is C-24's acceptance and was not
	 * expressible while the shell alone could reach the core.
	 */
	let { circuit: given }: { circuit?: Circuit } = $props()

	// The shell owns the projection and the transient interaction state.
	//
	// Every component below is given what to show; none of them reaches for the
	// core, and none of them holds a second copy of the circuit (design doc
	// 7.1.1). What is drawn is re-read from the circuit every frame by the
	// projection (C-24), so an edit is followed by having happened — no handler
	// here tells the canvas anything, and none can forget to.
	let projection = $state<Projection | undefined>()
	const circuit = $derived(projection?.circuit)
	const elements = $derived(projection?.elements ?? [])
	const nets = $derived(projection?.nets ?? [])
	const solution = $derived(projection?.solution)
	let viewport = $state<Viewport>(INITIAL_VIEWPORT)
	let drag = $state<Drag | undefined>()
	let selectedIds = $state<Set<string>>(new Set())
	/**
	 * Handles of the selected nets.
	 *
	 * Apart from the elements rather than mixed in with them: element and net
	 * handles come from different slotmaps in the core, so the same string can
	 * name one of each and a single set could not tell them apart.
	 */
	let selectedNetIds = $state<Set<string>>(new Set())
	/** Which Inspector field an error belongs to (C-100). */
	type Field = 'value' | 'series' | 'frequency' | 'initial' | 'room' | 'choice'
	/**
	 * What the core said about the last rejected edit, and which field it
	 * came from: shown under that field and nowhere else, and gone with the
	 * next change to the circuit or the selection.
	 */
	let fieldError = $state<{ field: Field; message: string } | undefined>()
	/**
	 * Something that went wrong that is not about a field — a file that will
	 * not open or save, a part that could not be placed — said on the canvas,
	 * where it can be seen whatever is selected (C-100). The sentence is for a
	 * person; the detail, when there is one, is a tooltip.
	 */
	let notice = $state<{ sentence: string; detail?: string } | undefined>()

	function messageOf(error: unknown): string {
		return error instanceof Error ? error.message : String(error)
	}

	const noticeOf = splitRefusal

	function refused(field: Field, error: unknown) {
		fieldError = { field, message: messageOf(error) }
	}
	let cursor = $state<Point | undefined>()
	/** The component under the pointer, and where the pointer is on screen. */
	let hoveredId = $state<string | undefined>()
	let pointer = $state<Point | undefined>()
	/** Whether the note explaining the moving dots is on screen (C-39). */
	let explainingOverlay = $state(false)
	let status = $state<'loading' | 'ready' | 'failed'>('loading')
	/** The share of real time the simulation runs at, when it cannot keep up (C-83). */
	let slowedTo = $state<number | undefined>(undefined)
	let detail = $state('Loading the simulation core…')
	/**
	 * Whether the core stopped on an error of its own mid-simulation. A Rust
	 * panic leaves the wasm instance unusable, so the honest offer is a
	 * reload: the autosave has the circuit from half a second ago at most.
	 */
	let halted = $state(false)
	let canvas = $state<ReturnType<typeof Canvas> | undefined>()

	// The inspector shows one component; the canvas can have several selected.
	const selection = $derived(
		selectedIds.size === 1 ? elements.find((element) => selectedIds.has(element.id)) : undefined,
	)
	/**
	 * What buying the selected part would mean (B-09): asked again on an edit,
	 * not every frame, since the core solves the circuit twice to answer.
	 */
	const purchase = $derived.by(() => {
		void projection?.edits
		if (!selection || !circuit) return undefined
		try {
			return circuit.purchase(selection.id) ?? undefined
		} catch {
			return undefined
		}
	})
	/**
	 * The values either side of the selected resistor’s or capacitor’s
	 * own, for its − and + (C-136): asked again on an edit, as the purchase is.
	 */
	const steps = $derived.by(() => {
		void projection?.edits
		if (!selection || !circuit) return undefined
		try {
			return circuit.steps(selection.id)
		} catch {
			return undefined
		}
	})
	const simulationNote = $derived.by(() => {
		// An empty canvas is not a broken circuit. Greeting someone with "this
		// circuit has no reference point" before they have placed anything
		// teaches nothing and reads as the app already being wrong.
		if (halted || !solution || elements.length === 0) return undefined

		// Every other failure now arrives as a code with words of its own
		// (C-25), including the unwired one — which used to need a special
		// case here precisely because the solver's sentence for it was true
		// and useless.
		if (solution.failure) return explain(solution.failure)
		// A circuit too big for real time runs slower than it, rather than
		// freezing the page, and says so (C-83).
		if (slowedTo !== undefined) {
			return `${solution.time.toFixed(1)} s simulated · slowed to ${slowedWords(slowedTo)} of real time`
		}
		return `${solution.time.toFixed(1)} s simulated`
	})
	/**
	 * The same line, as it is read aloud: what it says without the clock,
	 * so it is announced when the circuit starts, stops or fails rather than
	 * every tenth of a second it runs.
	 */
	const spokenNote = $derived.by(() => {
		if (halted || !solution || elements.length === 0) return undefined
		if (solution.failure) return explain(solution.failure)
		if (slowedTo !== undefined) return 'Simulating, slower than real time'
		return 'Simulating'
	})
	/** The solver's own words, for whoever is debugging the solver. */
	const simulationDetail = $derived(detailOf(solution?.failure))
	// On-demand help (design doc §14): what is under the pointer, and what it is
	// doing. Recomputed from the live solution every frame, so the numbers move
	// as the circuit does rather than freezing at whatever they were when the
	// pointer arrived. Hidden while something is being carried, because the
	// carried chip is already following the pointer and two labels racing each
	// other is neither.
	// One wire selected and nothing else. The inspector saying "nothing" while a
	// wire is lit up on the canvas is the app contradicting itself, which is the
	// bug C-23 fixed for elements and this one would reintroduce for wires.
	const selectedWire = $derived(
		selectedIds.size === 0 && selectedNetIds.size === 1
			? nets.find((net) => selectedNetIds.has(net.id))
			: undefined,
	)
	const hoveredElement = $derived(
		hoveredId && !drag ? elements.find((element) => element.id === hoveredId) : undefined,
	)
	const hoveredInfo = $derived(hoveredElement ? describe(hoveredElement.kind) : undefined)
	/**
	 * Every number written out as a reading — the hover readout, a meter's
	 * label, a buzzer's note, a part's heat — held for a third of a second and then moved to
	 * the mean of what it saw, as a multimeter's display is: solved sixty
	 * times a second and written out as often, a fast circuit's digits were a
	 * blur. Keyed by element and quantity, so each holds its own.
	 */
	const steady = new Steady()
	const settleAt = (now: number) => (key: string, measured: number | undefined) =>
		steady.value(key, now, measured)
	const hoveredReading = $derived.by(() => {
		if (!hoveredElement) return {}
		const reading = readingFor(hoveredElement, nets, solution)
		const settle = settleAt(performance.now())
		return {
			volts: settle(`${hoveredElement.id}:hover:V`, reading.volts),
			amps: settle(`${hoveredElement.id}:hover:A`, reading.amps),
		}
	})
	/**
	 * The selected part's voltage and current, held as the hover readout's are,
	 * for the Inspector's "Right now" card (C-129): the numbers a beginner
	 * wants are about the part they have just clicked, not only the one under
	 * the pointer.
	 */
	const selectedReading = $derived.by(() => {
		if (!selection) return {}
		const reading = readingFor(selection, nets, solution)
		const settle = settleAt(performance.now())
		return {
			volts: settle(`${selection.id}:selected:V`, reading.volts),
			amps: settle(`${selection.id}:selected:A`, reading.amps),
		}
	})
	/**
	 * The selected part's temperature and power, held the same way (C-74): a
	 * resistor under a 555 at audio frequency dissipates a different power
	 * every frame, and the Inspector's line was as unreadable as the readout
	 * was. Its warning reads the held numbers too, so a sentence can never
	 * contradict the figure beside it.
	 */
	const selectedHeat = $derived.by(() => {
		const heat = selection
			? solution?.heat.find((entry) => entry.elementId === selection.id)
			: undefined
		if (!heat) return undefined
		const settle = settleAt(performance.now())
		const key = `${heat.elementId}:heat`
		return {
			...heat,
			watts: settle(`${key}:W`, heat.watts) ?? heat.watts,
			celsius: settle(`${key}:C`, heat.celsius) ?? heat.celsius,
			settlesAt: settle(`${key}:settles`, heat.settlesAt) ?? heat.settlesAt,
		}
	})
	// A rejected edit changes nothing, so anything that does change the
	// circuit — another edit, a delete, an undo — leaves its error stale.
	$effect(() => {
		void projection?.edits
		untrack(() => (fieldError = undefined))
	})

	const room = $derived(projection?.room ?? 25)
	const description = $derived(projection?.description ?? '')
	/**
	 * What the header calls the circuit: the first line of its description,
	 * which is where an example puts its title and where anyone describing
	 * their own would put one (C-127).
	 */
	const circuitName = $derived(description.split('\n')[0].trim())
	/** Whether the shortcut list is open over the header (C-127). */
	let shortcutsOpen = $state(false)
	/**
	 * On a phone, the parts and the panel are sheets over the board rather
	 * than columns beside it (C-132). The parts sheet is asked for; the panel
	 * opens with a selection, or when the circuit's own settings are asked
	 * for with nothing selected.
	 */
	let partsOpen = $state(false)
	let circuitOpen = $state(false)
	const panelOpen = $derived(selectedIds.size > 0 || selectedNetIds.size > 0 || circuitOpen)
	/**
	 * The keys written on the canvas's Turn and Delete buttons, from the one
	 * table (C-42) — and only where the key is not already the button's own
	 * word, or a finger is all there is.
	 */
	const keyOn = (() => {
		if (touchOnly()) return { rotate: undefined, delete: undefined }
		const label = (action: string) =>
			keyBindings().find((binding) => binding.action === action)?.label
		const unless = (key: string | undefined, word: string) => (key === word ? undefined : key)
		return {
			rotate: unless(label('rotate'), 'Turn 90°'),
			delete: unless(label('delete'), 'Delete'),
		}
	})()
	const scene = $derived(toScene(elements, solution, nets, room, settleAt(performance.now())))
	// The scope is only worth its row when something is being probed: a panel
	// with no trace in it is furniture (design doc §14).
	const probed = $derived(lanesFor(elements).length > 0)
	// The serial monitor likewise (A-06): there while a program has begun
	// Serial, or has printed something still worth reading.
	const serialPorts = $derived(solution?.serial ?? [])
	const serialShown = $derived(
		serialPorts.some(
			(port) => port.programBaud != null || (projection?.serial.text[port.elementId] ?? '') !== '',
		),
	)
	/**
	 * Whether wires are drawn by voltage (C-134). On unless turned off, since
	 * M-22 (Denis, 2026-10-02); the choice is remembered for whoever made it.
	 */
	const VOLTAGE_KEY = 'pastille:voltage-colours'
	let showVoltage = $state(
		(() => {
			try {
				return localStorage.getItem(VOLTAGE_KEY) !== 'off'
			} catch {
				return true
			}
		})(),
	)
	function toggleVoltage() {
		showVoltage = !showVoltage
		try {
			localStorage.setItem(VOLTAGE_KEY, showVoltage ? 'on' : 'off')
		} catch {
			// A preference that cannot be kept is still in force until reload.
		}
	}
	/** The scale's three stops, read from the theme so dark and light each have theirs. */
	function voltageStops(): [string, string, string] {
		const styles = getComputedStyle(document.documentElement)
		const token = (name: string, fallback: string) =>
			styles.getPropertyValue(name).trim() || fallback
		return [
			token('--volt-low', '#3a5bd0'),
			token('--volt-mid', '#9a6ae0'),
			token('--volt-high', '#ff8aa6'),
		]
	}
	const voltageScale = $derived(showVoltage ? scaleOf(solution) : undefined)
	/**
	 * The findings, said beside their parts (B-07). One callout at a time, the
	 * warnings before the notes; the header's count steps through the rest
	 * and brings back one that was put away. A finding put away stays away
	 * until it changes — the same rule, part and words — so a beginner who
	 * has read it is not shown it again every frame.
	 */
	const findings = $derived(
		[...(solution?.findings ?? [])].sort(
			(a, b) => Number(a.severity !== 'warning') - Number(b.severity !== 'warning'),
		),
	)
	const keyOf = (finding: { rule: string; elements: string[]; message: string }) =>
		`${finding.rule}|${finding.elements.join(',')}|${finding.message}`
	let putAway = $state(new Set<string>())
	let checkIndex = $state(0)
	const showing = $derived(findings.filter((finding) => !putAway.has(keyOf(finding))))
	const shownCheck = $derived(showing.length > 0 ? showing[checkIndex % showing.length] : undefined)
	const warningCount = $derived(findings.filter((finding) => finding.severity === 'warning').length)
	function stepCheck(by: number) {
		if (showing.length === 0) return
		checkIndex = (checkIndex + by + showing.length) % showing.length
	}
	function putCheckAway() {
		if (!shownCheck) return
		putAway = new Set([...putAway, keyOf(shownCheck)])
	}
	/** The header's count: everything back, and on to the next. */
	function bringChecks() {
		if (showing.length < findings.length) putAway = new Set()
		else stepCheck(1)
	}
	/**
	 * What the shown finding's fix button says and does (C-135), when it
	 * offers one: the core makes the edit, as one step of undo.
	 */
	const shownFix = $derived.by(() => {
		const fix = shownCheck?.fix
		if (!fix || !circuit) return undefined
		// A value set on another part than the one the finding is about — an
		// LED's resistor, for an LED past its rating (B-03) — says which.
		const target = elements.find((element) => element.id === fix.element)
		const value = spokenValue(fix.value, fix.kind === 'addResistor' ? 'Ω' : (target?.unit ?? ''))
		const label =
			fix.kind === 'addResistor'
				? `Add a ${value} resistor`
				: target && !shownCheck.elements.includes(fix.element)
					? `Use a ${value} ${target.kind.toLowerCase()}`
					: `Set it to ${value}`
		return {
			label,
			apply: () => {
				try {
					circuit.applyFix(fix)
					fieldError = undefined
				} catch (error) {
					notice = noticeOf(error)
				}
			},
		}
	})
	/** Where the callout points from, and which side of the part it sits on. */
	const calloutAt = $derived.by(() => {
		const id = shownCheck?.elements[0]
		const item = id ? scene.find((candidate) => candidate.id === id) : undefined
		const bounds = canvas?.bounds()
		if (!item || !bounds) return undefined
		const centre = toScreen(viewport, item.position)
		const half = halfSizeOf(item.symbol, item.turns)
		const reach = (half.x + 6) * viewport.zoom
		// Off the board: pinned to its top instead of pointing at nothing.
		if (centre.x < 0 || centre.y < 0 || centre.x > bounds.width || centre.y > bounds.height)
			return undefined
		const room = bounds.width - (centre.x + reach)
		const side: 'left' | 'right' = room < 18 * 16 && centre.x - reach > room ? 'left' : 'right'
		const y = Math.min(Math.max(centre.y, 64), bounds.height - 64)
		return { at: { x: side === 'right' ? centre.x + reach + 10 : centre.x - reach - 10, y }, side }
	})
	// The scene is new every frame, its readings being part of it; the routes
	// are not, so they are kept between frames and only the currents redone.
	const routes = new RouteCache()
	const plainWires = $derived(toWires(nets, scene, solution?.pinCurrents, routes))
	const wires = $derived(
		showVoltage ? paintVoltages(plainWires, solution, voltageStops()) : plainWires,
	)
	const currentFlows = $derived(wires.some((wire) => (wire.flows?.length ?? 0) > 0))

	/**
	 * The dots explain themselves, once (C-39).
	 *
	 * Held outside the reactive graph on purpose: the effect must fire exactly
	 * once per session, and a `$state` flag it both reads and writes would tear
	 * down its own timer on the re-run.
	 */
	let overlayNoteDone = hasSeen(CURRENT_OVERLAY)
	/**
	 * How long the note stays before withdrawing itself.
	 *
	 * Dismissing it is the point, but a note nobody dismisses has to go anyway
	 * or it becomes the furniture §14 warns about. Long enough to read twice.
	 */
	const OVERLAY_NOTE_SECONDS = 12

	$effect(() => {
		// Not before there is a current to point at: explaining an animation
		// that is not running is worse than saying nothing.
		if (overlayNoteDone || !currentFlows) return

		overlayNoteDone = true
		remember(CURRENT_OVERLAY)
		explainingOverlay = true

		const timer = setTimeout(() => (explainingOverlay = false), OVERLAY_NOTE_SECONDS * 1000)
		return () => clearTimeout(timer)
	})

	/**
	 * Keeps the last circuit for next time (C-29).
	 *
	 * Runs on the projection's edit count rather than on every frame, and
	 * debounced on top of that: a drag across the canvas is one edit per
	 * pointer move, and serialising the whole circuit that often to write it
	 * somewhere nobody is reading would be work for its own sake. Each edit
	 * cancels the timer the one before it set, which is what makes the pause
	 * at the end of a gesture the thing that saves.
	 */
	$effect(() => {
		const saving = projection
		if (!saving) return
		// Read so the effect re-runs per edit rather than once.
		void saving.edits

		const timer = setTimeout(() => {
			try {
				rememberCircuit(saving.circuit.save())
			} catch {
				// A circuit that cannot be written is not a reason to stop
				// editing it. The export button will say so if it is asked.
			}
		}, 500)
		return () => clearTimeout(timer)
	})

	/**
	 * A buzzer, heard as well as drawn (C-71): one note while any buzzer is
	 * sounding in a circuit that solves. Not while the tab is hidden, because
	 * the frame loop stops there and the last thing it heard would ring on.
	 *
	 * A passive buzzer's note is the one the run hears (C-85), and wins over
	 * an active one's fixed beep: one voice, and the note that moves is the
	 * one somebody is listening for.
	 */
	const buzz = new Buzz()
	let visible = $state(true)
	const pitch = $derived.by(() => {
		if (!solution || solution.failure) return undefined
		const fed = solution.tones[0]
		if (fed) return fed.hertz
		const beeping = elements.some(
			(element) => element.kind === 'Buzzer' && solution.energised.includes(element.id),
		)
		return beeping ? TONE_HZ : undefined
	})
	$effect(() => buzz.set(pitch !== undefined && visible, pitch))
	$effect(() => {
		const seen = () => (visible = document.visibilityState === 'visible')
		// A browser lets a page make sound only from a click or a key.
		const wake = () => buzz.wake()
		document.addEventListener('visibilitychange', seen)
		window.addEventListener('pointerdown', wake, true)
		window.addEventListener('keydown', wake, true)
		return () => {
			document.removeEventListener('visibilitychange', seen)
			window.removeEventListener('pointerdown', wake, true)
			window.removeEventListener('keydown', wake, true)
			buzz.set(false)
		}
	})

	/**
	 * The live loop. There is no run button and nothing to start: the frame
	 * callback reports how much wall time passed and the core's scheduler
	 * decides what that is worth (C-13). An edit is picked up on the next frame
	 * simply by having happened.
	 */
	$effect(() => {
		const running = projection
		if (!running) return

		let live = true
		let last = performance.now()
		const pace = new Pace()

		const frame = (now: number) => {
			if (!live) return
			const elapsed = (now - last) / 1000
			last = now

			// One call: the simulation moves, and what is drawn is re-read from
			// the circuit it moved. Anything edited since — by a handler below,
			// or by whoever else holds this circuit — is picked up here. Only as
			// much of the time that passed as the frame can afford, measured on
			// the frames before it (C-83).
			const granted = pace.grant(elapsed)
			const started = performance.now()
			try {
				running.advance(granted)
			} catch (error) {
				// Once is enough: a trapped core traps on every call after,
				// and a loop that kept asking would fill the console instead
				// of saying anything (the panic hook has already said where).
				console.error(error)
				live = false
				halted = true
				status = 'failed'
				detail = 'The simulation stopped on an error of its own'
				return
			}
			pace.record(elapsed, granted, performance.now() - started)
			const slowed = pace.slowedTo()
			// In tenths, so the status line is not re-rendered for every
			// hundredth the estimate moves.
			const rounded = slowed === undefined ? undefined : Math.round(slowed * 10) / 10
			if (rounded !== slowedTo) slowedTo = rounded

			requestAnimationFrame(frame)
		}
		requestAnimationFrame(frame)

		return () => {
			live = false
		}
	})

	async function start() {
		try {
			// A circuit that was handed to us is the caller's: it is already
			// the one they want shown, and putting the autosave over the top of
			// it would be the app overwriting what it was given.
			const opened = given ?? (await createCircuit())
			if (!given) restore(opened)

			projection = new Projection(opened)
			// Not awaited: the app is ready without them, and a library that
			// failed to arrive is an offer missing, not an app that failed.
			// Asked for before the app says it is ready, so that anything
			// waiting on "ready" knows the library is already on its way.
			import('./lib/examples').then((loaded) => (library = loaded)).catch(() => {})
			status = 'ready'
			detail = `Core ready — pastille-core ${await coreVersion()}`
		} catch (error) {
			status = 'failed'
			detail = `Core failed to load: ${(error as Error).message}`
		}
	}
	void start()

	/**
	 * Puts back the circuit from last time, if it can be made sense of.
	 *
	 * A remembered circuit that will not load is forgotten rather than
	 * reported: it is not something the user did, they never asked for it to
	 * be saved, and an error about it would be the first thing a beginner saw.
	 * Starting empty is what would have happened anyway.
	 */
	/**
	 * The circuit from last time that this build could not open, and why
	 * (C-98). Set aside rather than deleted, and offered as a file, because
	 * the reason is usually that a newer version saved it.
	 *
	 * Kept until it is downloaded or let go of, and offered again on every
	 * visit until then: the note says it has been kept, and a copy nobody can
	 * reach after a reload would not be. The reason is only known on the
	 * visit that found it.
	 */
	let unopened = $state<{ text: string; reason?: string } | undefined>()

	function restore(created: Circuit) {
		const saved = recallCircuit()
		if (!saved) {
			const aside = recallSetAside()
			if (aside) unopened = { text: aside }
			return
		}

		try {
			created.load(saved)
			// Framed, as an opened file is: built away from the middle, it
			// came back off screen onto what looked like an empty grid.
			fitWhenDrawn = true
			// Set aside on an earlier visit, and still waiting.
			const aside = recallSetAside()
			if (aside) unopened = { text: aside }
		} catch (error) {
			setCircuitAside(saved)
			forgetCircuit()
			const reason = noticeOf(error).sentence
			unopened = {
				text: saved,
				// The core's sentences do not all end in a full stop, and this
				// one has another after it.
				reason: /[.!?]$/.test(reason) ? reason : `${reason}.`,
			}
		}
	}

	async function handleDownloadUnopened() {
		if (!unopened) return
		await download(unopened.text, 'unopened-circuit.json')
		forgetSetAside()
		unopened = undefined
	}

	function handleDiscardUnopened() {
		forgetSetAside()
		unopened = undefined
	}

	/**
	 * Set when a circuit has arrived from somewhere other than the pointer, so
	 * the view can be pointed at it once there is something to point at.
	 *
	 * Deferred rather than done on the spot, because the canvas may not be
	 * there yet: a circuit restored at start-up arrives before it is.
	 */
	let fitWhenDrawn = $state(false)

	/**
	 * Frames a circuit that has just been opened over whatever was there.
	 *
	 * The projection re-reads the circuit on the next frame (C-24), but the
	 * effect below runs before that one: over a canvas that was not empty —
	 * always the case after a reload, which restores the last circuit — it
	 * framed the circuit that had just been replaced, and the new one came
	 * up wherever that left it. Read now, so the scene is the new circuit's
	 * by the time anything looks at it.
	 */
	function fitOpened() {
		projection?.project()
		fitWhenDrawn = true
	}

	$effect(() => {
		if (!fitWhenDrawn || scene.length === 0) return
		canvas?.show(boundsOf(scene))
		fitWhenDrawn = false
	})

	/** Opens one of the circuits that ship with the app (C-47). */
	/**
	 * The examples, fetched once the core is ready rather than with the app.
	 *
	 * They were two fifths of the script — and a library that grows by one
	 * with every part — on the cold-load path M-21 budgets three seconds for,
	 * ahead of the core they cannot be opened without. Fetched afterwards,
	 * they arrive with the app that can open them.
	 */
	let library = $state<typeof import('./lib/examples') | undefined>()

	// Which shelf of examples the empty canvas is showing (C-58). The first
	// one to begin with, because it is made of the parts somebody meets first.
	let chosenShelf = $state<ExampleGroup | undefined>()
	const shelf = $derived(chosenShelf ?? library?.EXAMPLE_GROUPS[0])

	/**
	 * Each example's thumbnail (C-131): the part that says what it is about,
	 * drawn as the board draws it, in its family's colour.
	 */
	const thumbnails = $derived.by(() => {
		const loaded = library
		if (!loaded) return new Map()
		return new Map(
			loaded.EXAMPLES.map((example) => {
				const kind = loaded.heroOf(example)
				const symbol = symbolFor(kind) as SymbolDefinition | undefined
				return [example.id, { glyph: symbol ? glyphOf(symbol) : undefined, family: familyOf(kind) }]
			}),
		)
	})

	/** Whether the Examples dialog is open (ADR 0016). */
	let examplesOpen = $state(false)
	let examplesDialog = $state<HTMLDialogElement | undefined>()
	$effect(() => {
		if (!examplesDialog) return
		if (examplesOpen && !examplesDialog.open) examplesDialog.showModal()
		if (!examplesOpen && examplesDialog.open) examplesDialog.close()
	})

	/**
	 * The example just opened over somebody's work, and how long ago (ADR
	 * 0016): said at once, with the Undo that brings the work back, because
	 * a beginner may not know there is one.
	 */
	let openedOver = $state<{ title: string } | undefined>()
	$effect(() => {
		if (!openedOver) return
		const timer = setTimeout(() => (openedOver = undefined), 15_000)
		return () => clearTimeout(timer)
	})

	function handleExample(example: Example) {
		if (!circuit) return
		// Anything that opening would replace, description included: the
		// open is one step of undo (C-98) either way, and over work it is said.
		const over = scene.length > 0 || nets.length > 0 || description !== ''
		try {
			// Opened with its own line as its description, so what to look at
			// stays on the canvas once the offer has gone — and goes with it
			// into a file if it is saved.
			const document = JSON.parse(example.document)
			document.description = `${example.title}\n${example.look}`
			circuit.open(JSON.stringify(document))
			projection?.serial.forget()
			selectedIds = new Set()
			selectedNetIds = new Set()
			notice = undefined
			fitOpened()
			examplesOpen = false
			openedOver = over ? { title: example.title } : undefined
		} catch (error) {
			// An example that will not load is the app's fault rather than the
			// user's, and there is a test that stops one shipping. Said out
			// loud anyway: silence would look like the click did nothing.
			notice = noticeOf(error)
		}
	}

	/**
	 * Empties the canvas, description and all, as one step of undo — which is
	 * what makes a button for it safe to leave lying in the header, and brings
	 * the examples back.
	 */
	function handleClear() {
		// Asleep rather than disabled when there is nothing to clear, so the
		// focus stays on it after a clear instead of falling to the page.
		if (!circuit || (scene.length === 0 && nets.length === 0 && !description)) return
		circuit.clear()
		projection?.serial.forget()
		selectedIds = new Set()
		selectedNetIds = new Set()
		notice = undefined
	}

	/** The circuit was described, or its description edited. */
	function handleDescription(text: string) {
		circuit?.setDescription(text)
	}

	/** Hands the circuit to the browser as a file. */
	async function handleSave() {
		if (!circuit) return
		try {
			await download(circuit.save())
			notice = undefined
		} catch (error) {
			notice = noticeOf(error)
		}
	}

	/** Opens a circuit from a file, replacing what is on the canvas. */
	async function handleOpen() {
		const text = await openFile()
		if (!text || !circuit) return

		try {
			// One step of undo, as Clear is: opening a file over an hour of
			// work must not lose it (C-98).
			circuit.open(text)
			projection?.serial.forget()
			selectedIds = new Set()
			selectedNetIds = new Set()
			notice = undefined
			// C-29 left the camera where it was, so a circuit saved while
			// panned away opened off screen onto what looked like an empty
			// canvas. Same fix as an example's, and the same one line.
			fitOpened()
		} catch (error) {
			// Said out loud, and where it can be seen whatever is selected:
			// this file is one the user chose, and silence would look like the
			// app ignoring them.
			notice = noticeOf(error)
		}
	}

	/**
	 * Whether a point on screen is the board itself, rather than something
	 * lying over its rectangle — on a phone, the parts sheet does (C-132), and
	 * a tap on a part in it would otherwise be a drop on the board under it.
	 */
	function overBoard(at: Point): boolean {
		return document.elementFromPoint(at.x, at.y)?.closest('main.canvas') != null
	}

	function handleDragMove(event: PointerEvent) {
		// Tracked whether or not anything is being carried: the readout is
		// positioned in client coordinates, and the canvas reports what is under
		// the pointer but not where the pointer is on the screen.
		// Not for a finger: it has no hover, and its readout is placed by the
		// hold that asked for it (C-133), above the finger rather than under.
		if (event.pointerType !== 'touch') pointer = { x: event.clientX, y: event.clientY }
		if (!drag) return
		drag = moveDrag(drag, { x: event.clientX, y: event.clientY }, canvas?.bounds())
		// A part dragged out of the phone's parts sheet needs the board the
		// sheet is covering (C-132).
		if (partsOpen && drag.overCanvas && overBoard(drag.at)) partsOpen = false
	}

	function handleDrop() {
		const carried = drag
		drag = undefined
		if (!carried || !circuit) return

		// Outside the canvas there is no position, so nothing is created — that is
		// the whole of "cancel leaves the netlist untouched".
		const bounds = canvas?.bounds()
		const at = dropPosition(carried, bounds, viewport)
		if (!at || !bounds || !overBoard(carried.at)) return

		// Dropped onto a part of another kind: that part becomes this one, in
		// place, keeping the wires that still make sense (C-137).
		const onto = hitTest(
			scene,
			toWorld(viewport, { x: carried.at.x - bounds.left, y: carried.at.y - bounds.top }),
		)
		const ontoKind = onto && elements.find((element) => element.id === onto.item.id)?.kind
		if (onto && ontoKind && ontoKind !== carried.kind) {
			handleReplace(onto.item.id, carried.kind)
			return
		}

		circuit.add(carried.kind, at.x, at.y)
	}

	/** A part made another kind in place (C-137), and what became of its wires said. */
	function handleReplace(id: string, kind: string) {
		if (!circuit) return
		try {
			const replacement = circuit.replace(id, kind)
			selectedIds = new Set([id])
			selectedNetIds = new Set()
			notice = { sentence: replacedSentence(describe(kind)?.label ?? kind, replacement) }
		} catch (error) {
			notice = noticeOf(error)
		}
	}

	/**
	 * A component clicked in the palette, or chosen there with Enter: placed
	 * in the middle of the view, beside anything already there, and selected
	 * (C-97). A drag that started on the entry and ended over the canvas has
	 * already been dropped; one that ended where it started is this click.
	 */
	function handlePalettePick(kind: SymbolKind) {
		const bounds = canvas?.bounds()
		if (!circuit || !bounds) return
		const occupants = scene.map((item) => ({
			position: item.position,
			half: halfSizeOf(item.symbol, item.turns),
		}))
		const at = clickPosition(kind, bounds, viewport, occupants)
		try {
			const id = circuit.add(kind, at.x, at.y)
			selectedIds = new Set([id])
			selectedNetIds = new Set()
			// On a phone: the part is placed, so the sheet steps aside for it
			// and the panel opens on it (C-132).
			partsOpen = false
		} catch (error) {
			notice = noticeOf(error)
		}
	}

	/** Selection after a click: additive toggles, plain replaces, empty clears. */
	function pick(current: Set<string>, id: string | undefined, additive: boolean): Set<string> {
		if (!id) return additive ? current : new Set()
		if (!additive) return current.has(id) ? current : new Set([id])

		// A copy, handed back whole: the selection is replaced, never mutated.
		// eslint-disable-next-line svelte/prefer-svelte-reactivity
		const next = new Set(current)
		if (!next.delete(id)) next.add(id)
		return next
	}

	/**
	 * A click on the canvas, wherever it landed.
	 *
	 * Elements and wires are two selections kept side by side, and a plain click
	 * on one clears the other — shift-clicking a wire while a resistor is picked
	 * would otherwise build a selection that Delete treats as one thing and the
	 * inspector cannot describe.
	 */
	/**
	 * Which view of the circuit is on screen (B-14): the schematic, or the
	 * same circuit on a breadboard. The schematic opens first (B-19, Denis,
	 * 2026-10-03).
	 */
	let view = $state<'schematic' | 'breadboard'>('schematic')

	/**
	 * The circuit on the board, asked again on an edit or a move on it — or
	 * why it cannot be put on one: more than a full board holds.
	 */
	const onBoard = $derived.by((): { board?: BoardView; refused?: string } => {
		void projection?.edits
		if (view !== 'breadboard' || !circuit) return {}
		try {
			return { board: circuit.breadboard() }
		} catch (error) {
			return { refused: noticeOf(error).sentence }
		}
	})

	function handleBoardMove(id: string, dx: number, dy: number) {
		try {
			circuit?.moveOnBoard(id, dx, dy)
			selectedIds = new Set([id])
		} catch (error) {
			notice = noticeOf(error)
		}
	}

	function handlePick(hit: Hit | undefined, additive: boolean) {
		fieldError = undefined

		if (hit?.kind === 'wire') {
			selectedNetIds = pick(selectedNetIds, hit.wire.id, additive)
			if (!additive) selectedIds = new Set()
			return
		}

		selectedIds = pick(selectedIds, hit?.item.id, additive)
		if (!additive) selectedNetIds = new Set()
	}

	/**
	 * A rectangle dragged out with shift held: what it touched joins the
	 * selection, the way a shift-click adds one thing to it.
	 */
	function handleArea(ids: string[]) {
		if (ids.length === 0) return
		selectedIds = new Set([...selectedIds, ...ids])
	}

	function handleKey(event: KeyboardEvent) {
		// Never while typing: Backspace in the value field means backspace, and
		// in the description, R is a letter and Cmd+A selects its text. Only
		// a field that takes text, though: a knob's slider or a button that
		// has kept the focus is not typing, and Cmd+Z right after turning a
		// knob is exactly when undo is wanted (C-103).
		const target = event.target as HTMLElement | null
		const textual =
			target instanceof HTMLInputElement &&
			!['range', 'checkbox', 'radio', 'button', 'submit', 'reset', 'color', 'file'].includes(
				target.type,
			)
		const typing = textual || ['TEXTAREA', 'SELECT'].includes(target?.tagName ?? '')
		if (typing || target?.isContentEditable) return

		// One table decides both what the app answers to and what the Keys panel
		// says it answers to (C-42), so the list on screen cannot drift from the
		// behaviour it describes.
		const action = actionFor(event)
		if (!action) return

		// Before the circuit is asked for: the list is worth reading while
		// the core is still loading.
		if (action === 'help') {
			event.preventDefault()
			shortcutsOpen = !shortcutsOpen
			return
		}
		if (!circuit) return

		if (action === 'cancel') {
			const carried = drag !== undefined
			drag = undefined
			if (!canvas?.cancel() && !carried) {
				selectedIds = new Set()
				selectedNetIds = new Set()
			}
			fieldError = undefined
			return
		}

		if (action === 'undo' || action === 'redo') {
			event.preventDefault()
			if (action === 'redo') circuit.redo()
			else circuit.undo()
			selectedIds = new Set()
			selectedNetIds = new Set()
			return
		}

		if (action === 'zoomIn' || action === 'zoomOut' || action === 'showAll') {
			event.preventDefault()
			if (action === 'showAll') canvas?.showAll()
			else canvas?.zoomBy(action === 'zoomIn' ? 1.25 : 1 / 1.25)
			return
		}

		if (action === 'selectAll') {
			// Taken from the browser, which would otherwise select the page's
			// text. The components, not the wires: every wire runs between them,
			// so deleting or moving the components takes the wires along.
			event.preventDefault()
			selectedIds = new Set(elements.map((element) => element.id))
			selectedNetIds = new Set()
			return
		}

		if (action === 'rotate') {
			if (selectedIds.size === 0) return
			event.preventDefault()
			rotateSelection()
			return
		}

		if (selectedIds.size === 0 && selectedNetIds.size === 0) return
		event.preventDefault()
		deleteSelection()
	}

	/**
	 * Turns what is selected a quarter turn — from R, or the Inspector's
	 * button where there is no keyboard (C-99).
	 */
	function rotateSelection() {
		// A wire has no orientation of its own: it is drawn from where the
		// pins ended up, so turning the components turns it too.
		if (!circuit || selectedIds.size === 0) return
		// However many are selected, one press is one press of undo.
		circuit.beginGesture()
		try {
			for (const id of selectedIds) circuit.rotate(id)
		} finally {
			circuit.endGesture()
		}
	}

	/** Removes what is selected — from Delete, or the Inspector's button (C-99). */
	function deleteSelection() {
		if (!circuit || (selectedIds.size === 0 && selectedNetIds.size === 0)) return
		// One press is one thing the user did, however many components and
		// wires it removed, so it is one press of undo.
		circuit.beginGesture()
		try {
			// Wires first: removing an element takes its wires with it, and
			// asking for a net that has already gone would fail the whole
			// gesture.
			for (const id of selectedNetIds) circuit.removeNet(id)
			for (const id of selectedIds) circuit.remove(id)
		} finally {
			// Closed whatever happened, or every edit after a failed delete
			// would fold into one step of undo (C-103).
			circuit.endGesture()
		}
		selectedIds = new Set()
		selectedNetIds = new Set()
	}

	/** Undo and Redo from the header, for a hand with no keyboard (C-99). */
	function handleHistory(forwards: boolean) {
		if (!circuit) return
		openedOver = undefined
		if (forwards) circuit.redo()
		else circuit.undo()
		selectedIds = new Set()
		selectedNetIds = new Set()
	}

	// Whether there is anything to step to, re-read whenever the circuit
	// changes: the core's flags are not reactive themselves.
	const canUndo = $derived((projection?.edits ?? 0) >= 0 && (circuit?.canUndo ?? false))
	const canRedo = $derived((projection?.edits ?? 0) >= 0 && (circuit?.canRedo ?? false))

	/**
	 * Saves the circuit as a KiCad netlist (B-16), or says which parts it
	 * cannot write yet, by the names the palette gives them (B-17).
	 */
	async function exportKicad(what: 'netlist' | 'schematic') {
		if (!circuit) return
		const view = circuit.kicadNetlist()
		if (view.netlist == null || view.schematic == null) {
			const names = view.unmapped.map((kind) => describe(kind)?.label ?? kind)
			const list =
				names.length > 1 ? `${names.slice(0, -1).join(', ')} and ${names.at(-1)}` : names[0]
			notice = {
				sentence: `The KiCad export cannot write ${names.length > 1 ? 'these parts' : 'this part'} yet: ${list}.`,
			}
			return
		}
		if (what === 'schematic') {
			await download(view.schematic, 'circuit.kicad_sch', {
				type: 'text/plain',
				kind: 'KiCad schematic',
				extension: 'kicad_sch',
			})
		} else {
			await download(view.netlist, 'circuit.net', {
				type: 'text/plain',
				kind: 'KiCad netlist',
				extension: 'net',
			})
		}
	}

	/** Saves the parts list (B-11), named after the circuit when it has a name. */
	async function exportBill(format: 'csv' | 'md') {
		if (!circuit) return
		const lines = circuit.bill()
		const title = description.split('\n')[0]?.trim() || 'Parts list'
		if (format === 'csv') {
			await download(billToCsv(lines), 'parts-list.csv', {
				type: 'text/csv',
				kind: 'Parts list',
				extension: 'csv',
			})
		} else {
			await download(billToMarkdown(lines, title), 'parts-list.md', {
				type: 'text/markdown',
				kind: 'Parts list',
				extension: 'md',
			})
		}
	}

	/**
	 * One field's edit, handed to the core.
	 *
	 * If the core refuses it, nothing changed and the simulation is still
	 * running on the last good value: say what was wrong, under the field it
	 * was typed in, and leave the field alone.
	 */
	function edit(field: Field, change: () => void) {
		try {
			change()
			fieldError = undefined
		} catch (error) {
			refused(field, error)
		}
	}

	function handleValue(id: string, text: string) {
		edit('value', () => circuit?.setValue(id, text))
	}

	/**
	 * A wire dragged into a new shape (C-43, ADR 0007).
	 *
	 * The bends the user placed are stored; the runs from each pin to the
	 * nearest bend stay derived, which is what lets a bend survive its
	 * components moving. A bend that ends up in line with its neighbours is
	 * dropped rather than stored invisibly.
	 */
	function handleBend(netId: string, waypoints: readonly Point[]) {
		const wire = wires.find((candidate) => candidate.id === netId)
		const [from, to] = wire?.pins ?? []
		const kept = from && to ? withoutRedundant(from, waypoints, to) : [...waypoints]

		circuit?.setWaypoints(netId, kept)
	}

	function handleSeries(id: string, text: string) {
		edit('series', () => circuit?.setSeriesResistance(id, text))
	}

	function handleFrequency(id: string, text: string) {
		edit('frequency', () => circuit?.setFrequency(id, text))
	}

	/** The circuit was put in a warmer or cooler room (C-74). */
	function handleRoom(text: string) {
		edit('room', () => circuit?.setRoom(text))
	}

	function handleInitial(id: string, text: string) {
		edit('initial', () => circuit?.setInitialCondition(id, text))
	}

	/**
	 * A pointer went down on something, or came up off it (C-19).
	 *
	 * Only a contact has anything to do with either, and what it does is
	 * `contacts.ts`'s to say: a button is shut while it is held, a switch is
	 * flipped by a click and merely moved by a drag.
	 */
	function handlePress(id: string) {
		flip(id, afterPress(elements.find((element) => element.id === id)))
	}

	function handleRelease(id: string, moved: boolean) {
		flip(
			id,
			afterRelease(
				elements.find((element) => element.id === id),
				moved,
			),
		)
	}

	function flip(id: string, closed: boolean | undefined) {
		if (closed === undefined) return
		circuit?.setClosed(id, closed)
	}

	/**
	 * A wiper turned (C-20).
	 *
	 * The whole turn is one gesture, so it is one press of undo however many
	 * values the slider reported on the way — the same rule a drag across the
	 * canvas follows, and the reason C-45 made a gesture record lazily.
	 */
	let turning = false
	/** A program from a file onto an Uno (A-03): it starts from reset. */
	async function handleProgram(id: string) {
		const chosen = await openProgram()
		if (!chosen || !circuit) return
		try {
			circuit.setProgram(id, chosen.name, chosen.text)
			notice = undefined
		} catch (error) {
			notice = noticeOf(error)
		}
	}

	function handleChoice(id: string, choice: string) {
		edit('choice', () => circuit?.setChoice(id, choice))
	}

	function handleWiper(id: string, wiper: number, settled: boolean) {
		if (!circuit) return
		if (!turning) {
			circuit.beginGesture()
			turning = true
		}
		circuit.setWiper(id, wiper)
		if (settled) {
			circuit.endGesture()
			turning = false
		}
	}

	function handleWire(
		from: { item: { id: string }; pin: { name: string } },
		to: { item: { id: string }; pin: { name: string } },
	) {
		circuit?.connect(from.item.id, from.pin.name, to.item.id, to.pin.name)
	}
</script>

<!-- A gesture the browser takes back — a finger drag that became a scroll —
     lets go of whatever was being carried, rather than leaving it to land
     wherever the next touch comes up (C-99). -->
{#snippet offer()}
	<!-- C-131. A card of its own, which takes its events back from the
	     note around it and scrolls inside itself: twenty examples on a
	     shelf would otherwise run off the board. -->
	{#if library}
		<section class="offer" aria-label="Examples">
			<p class="examples-lead">Or open one that works, and change it</p>
			<!-- One shelf at a time, so the offer is the same size however
		     many examples there are (C-58): the groups say what else is
		     there, and each example still carries its line. -->
			<div class="shelves" role="group" aria-label="Kinds of example">
				{#each library.EXAMPLE_GROUPS as group (group)}
					<button
						type="button"
						class:current={group === shelf}
						aria-pressed={group === shelf}
						onclick={() => (chosenShelf = group)}
					>
						{group}
					</button>
				{/each}
			</div>
			<ul class="examples">
				{#each library.EXAMPLES.filter((example) => example.group === shelf) as example (example.id)}
					{@const thumb = thumbnails.get(example.id)}
					<li style:--tint={thumb?.family ? `var(--family-${thumb.family})` : 'var(--muted)'}>
						<span class="thumb" aria-hidden="true">
							{#if thumb?.glyph}
								<svg viewBox={thumb.glyph.viewBox}>
									{#each thumb.glyph.shapes as shape, index (index)}
										<path d={shape.d} class:filled={shape.filled} />
									{/each}
								</svg>
							{/if}
						</span>
						<!-- The title is the button, and the button covers the card:
					     its name stays the example's name alone. -->
						<button type="button" onclick={() => handleExample(example)}>
							{example.title}
						</button>
						<span>{example.look}</span>
					</li>
				{/each}
			</ul>
		</section>
	{/if}
{/snippet}

<svelte:window
	onpointermove={handleDragMove}
	onpointerup={handleDrop}
	onpointercancel={() => (drag = undefined)}
	onkeydown={handleKey}
/>

<div class="shell" class:parts-open={partsOpen} class:panel-open={panelOpen}>
	<header>
		<h1 aria-label="Pastille">
			<span class="mark" aria-hidden="true">
				<svg viewBox="0 0 24 24"><path d="M2 12h4l2-5 4 10 4-10 2 5h4" /></svg>
			</span>
			<span aria-hidden="true">pas<span class="dot">·</span>tille</span>
		</h1>
		<!-- The circuit's name, so a screen of wires says what it is meant to
		     be. Read from the description, which is where it is edited. -->
		<p class="name" class:untitled={!circuitName}>{circuitName || 'Untitled circuit'}</p>
		{#if findings.length > 0}
			<!-- B-07. How many things the app has noticed, and the way back to
			     each: a press brings back one put away, or steps on. -->
			<button
				type="button"
				class="checks"
				class:warn={warningCount > 0}
				onclick={bringChecks}
				title="Show the checks on the board"
			>
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<path
						d="M10.3 3.9L1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0zM12 9v4M12 17h.01"
					/>
				</svg>
				{findings.length === 1 ? '1 check' : `${findings.length} checks`}
			</button>
		{/if}
		<!-- B-14. The same circuit, drawn as a schematic or on the board it
		     is built on. -->
		<div class="views" role="group" aria-label="View">
			<button
				type="button"
				aria-pressed={view === 'schematic'}
				onclick={() => (view = 'schematic')}
			>
				Schematic
			</button>
			<button
				type="button"
				aria-pressed={view === 'breadboard'}
				onclick={() => (view = 'breadboard')}
			>
				Breadboard
			</button>
		</div>
		<div class="files">
			<!-- Icons, with the word kept in the button for whoever cannot see
			     the icon and for whoever is looking for the word (C-99). -->
			<button
				type="button"
				class="icon"
				onclick={() => canUndo && handleHistory(false)}
				aria-disabled={!canUndo}
				title="Undo"
			>
				<svg viewBox="0 0 24 24" aria-hidden="true"
					><path d="M9 14L4 9l5-5M4 9h11a5 5 0 0 1 0 10h-3" /></svg
				>
				<span class="word">Undo</span>
			</button>
			<button
				type="button"
				class="icon"
				onclick={() => canRedo && handleHistory(true)}
				aria-disabled={!canRedo}
				title="Redo"
			>
				<svg viewBox="0 0 24 24" aria-hidden="true"
					><path d="M15 14l5-5-5-5M20 9H9a5 5 0 0 0 0 10h3" /></svg
				>
				<span class="word">Redo</span>
			</button>
			<span class="divider" aria-hidden="true"></span>
			<!-- Open and Save stay in plain sight rather than in a File menu:
			     a save nobody can find is a save nobody makes (C-29). -->
			<!-- ADR 0016: the library, whenever it is wanted. Over work, an
			     example is one Undo away and says so. -->
			<!-- On a phone each of these is its icon, with its word kept for a
			     screen reader and for the eye that hovers it (C-132). -->
			<button
				type="button"
				class="labelled always"
				onclick={() => (examplesOpen = true)}
				title="Examples"
			>
				<svg viewBox="0 0 24 24" aria-hidden="true"
					><path d="M4 5a2 2 0 0 1 2-2h13v16H6a2 2 0 0 0-2 2zM4 21V5" /></svg
				>
				<span class="label">Examples</span>
			</button>
			<button type="button" class="labelled" onclick={handleOpen} title="Open a circuit file">
				<svg viewBox="0 0 24 24" aria-hidden="true"
					><path
						d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"
					/></svg
				>
				<span class="label">Open…</span>
			</button>
			<button
				type="button"
				class="labelled"
				onclick={handleSave}
				title="Save the circuit as a file"
			>
				<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3v12M7 10l5 5 5-5M5 21h14" /></svg>
				<span class="label">Save</span>
			</button>
			<button
				type="button"
				class="labelled"
				onclick={handleClear}
				aria-disabled={scene.length === 0 && nets.length === 0 && !description}
				title="Empty the canvas. Undo brings it back."
			>
				<svg viewBox="0 0 24 24" aria-hidden="true"
					><path d="M3 6h18M8 6V4h8v2M19 6l-1 14H6L5 6" /></svg
				>
				<span class="label">Clear</span>
			</button>
			<Shortcuts bind:open={shortcutsOpen} />
		</div>
	</header>
	<div class="rail">
		<Palette
			start={scene.length === 0}
			onclose={() => (partsOpen = false)}
			ondragstart={(entry, at) => (drag = startDrag(entry.kind, at))}
			onpick={(entry) => handlePalettePick(entry.kind)}
			replacing={selection
				? { kind: selection.kind, label: describe(selection.kind)?.label ?? selection.kind }
				: undefined}
			onreplace={(entry) => selection && handleReplace(selection.id, entry.kind)}
		/>
	</div>
	<Canvas
		bind:this={canvas}
		items={scene}
		{wires}
		time={solution?.time ?? 0}
		onhover={(grid, hit) => {
			cursor = grid
			hoveredId = hit?.item.id
		}}
		onhold={(hit, at) => {
			// C-133. The hover readout, for a finger: raised above where it
			// rests, which is where the finger itself would cover it.
			hoveredId = hit?.item.id
			pointer = hit ? { x: at.x, y: at.y - 56 } : pointer
		}}
		onleave={() => {
			cursor = undefined
			hoveredId = undefined
		}}
		selected={selectedIds}
		selectedNets={selectedNetIds}
		onpick={handlePick}
		onarea={handleArea}
		onbeginmove={() => circuit?.beginGesture()}
		onendmove={() => circuit?.endGesture()}
		onmove={(id, x, y) => {
			circuit?.moveElement(id, x, y)
		}}
		onwire={handleWire}
		onpress={handlePress}
		onrelease={handleRelease}
		onbend={handleBend}
		onviewport={(next) => (viewport = next)}
	>
		{#snippet whenEmpty()}
			<!-- Offered here unasked (C-47) to somebody who has not started,
			     beside the sentence telling them how to — and gone the moment
			     they place anything. Over work, only when asked for, from the
			     header (ADR 0016). -->
			{#if unopened}
				<!-- C-98. Said here, where the circuit would have been, and
				     never silently: the copy is kept, and offered as a file. -->
				<div class="unopened" role="alert">
					<p>
						Your last circuit could not be opened here{unopened.reason
							? `: ${unopened.reason}`
							: '.'} It has been kept, so you can open it in a version that reads it.
					</p>
					<button type="button" onclick={handleDownloadUnopened}>Download it</button>
					<button type="button" onclick={handleDiscardUnopened}>Discard it</button>
				</div>
			{/if}
			{@render offer()}
		{/snippet}
		<!-- C-134. Wires by voltage: the switch, and once it is on, the key
		     that says what the colours mean on this circuit. -->
		{#if scene.length > 0}
			<div class="volt-key" class:on={showVoltage}>
				<button type="button" aria-pressed={showVoltage} onclick={toggleVoltage}>
					Voltage colours
				</button>
				{#if showVoltage && voltageScale}
					<span class="end">{formatQuantity(voltageScale.low, 'V')}</span>
					<span class="ramp" aria-hidden="true"></span>
					<span class="end">{formatQuantity(voltageScale.high, 'V')}</span>
				{/if}
			</div>
		{/if}
		<!-- C-132. The phone's two ways in, where a thumb is: hidden beside
		     columns, and while a sheet is up. -->
		{#if !partsOpen && !panelOpen}
			<div class="thumbbar">
				<button type="button" class="add" onclick={() => (partsOpen = true)}>
					<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 5v14M5 12h14" /></svg>
					Add a part
				</button>
				<button
					type="button"
					class="circuit"
					aria-label="Circuit settings"
					onclick={() => (circuitOpen = true)}
				>
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<path d="M4 21v-7M4 10V3M12 21v-9M12 8V3M20 21v-5M20 12V3M1 14h6M9 8h6M17 16h6" />
					</svg>
				</button>
			</div>
		{/if}
		{#if selectedIds.size > 0 || selectedNetIds.size > 0}
			<!-- C-99. What Delete and R do, for a hand with no keyboard: on
			     the canvas beside what is selected, since on a phone the
			     Inspector is a long way down the page. -->
			<div class="selection-actions" role="toolbar" aria-label="Selection">
				<!-- Each with its key written on it, beside the thing it does:
				     the half of C-42's finding that the list no longer answers
				     from the rail. In CSS, so the button's name is still only
				     what it does. -->
				{#if selectedIds.size > 0}
					<button type="button" onclick={rotateSelection} data-key={keyOn.rotate}>Turn 90°</button>
				{/if}
				<button type="button" onclick={deleteSelection} data-key={keyOn.delete}>Delete</button>
			</div>
		{/if}
		<!-- It points at the part on the schematic; on the breadboard the
		     header's count stays, and the schematic is one press away. -->
		{#if shownCheck && view === 'schematic'}
			<Callout
				severity={shownCheck.severity}
				message={shownCheck.message}
				at={calloutAt?.at}
				side={calloutAt?.side}
				index={showing.indexOf(shownCheck)}
				count={showing.length}
				onprevious={() => stepCheck(-1)}
				onnext={() => stepCheck(1)}
				ondismiss={putCheckAway}
				fix={shownFix}
			/>
		{/if}
		{#if findings.length > 0 && showing.length === 0}
			<!-- On a phone, where the header has no room for the count: the
			     way back to the checks that were put away (B-07). -->
			<button type="button" class="checks-chip" class:warn={warningCount > 0} onclick={bringChecks}>
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<path
						d="M10.3 3.9L1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0zM12 9v4M12 17h.01"
					/>
				</svg>
				{findings.length === 1 ? '1 check' : `${findings.length} checks`}
			</button>
		{/if}
		{#if openedOver}
			<!-- ADR 0016: the work an example replaced, one press away. -->
			<div class="notice undo-notice" role="status">
				<p>Opened “{openedOver.title}”. Your circuit is one Undo away.</p>
				<button type="button" onclick={() => handleHistory(false)}>Undo</button>
				<button type="button" onclick={() => (openedOver = undefined)}>OK</button>
			</div>
		{/if}
		{#if notice}
			<div class="notice" role="alert" title={notice.detail}>
				<p>{notice.sentence}</p>
				<button type="button" onclick={() => (notice = undefined)}>OK</button>
			</div>
		{/if}
		{#if description && scene.length > 0}
			<Description text={description} />
		{/if}
		{#if explainingOverlay}
			<OverlayNote ondismiss={() => (explainingOverlay = false)} />
		{/if}
	</Canvas>
	<!-- On a phone the header has no room left (C-132), so the switch is one
	     button over the board, saying where it goes. -->
	<!-- Not on an empty schematic, where the breadboard has nothing to show
	     and the button sat over the examples (C-138); kept on the breadboard,
	     so emptying it there still leaves the way back. -->
	{#if scene.length > 0 || view !== 'schematic'}
		<button
			type="button"
			class="view-switch"
			onclick={() => (view = view === 'schematic' ? 'breadboard' : 'schematic')}
		>
			{view === 'schematic' ? 'On a breadboard' : 'As a schematic'}
		</button>
	{/if}
	{#if view === 'breadboard'}
		<Breadboard
			board={onBoard.board}
			refused={onBoard.refused}
			selected={selectedIds}
			onpick={(id) => {
				fieldError = undefined
				selectedIds = id ? new Set([id]) : new Set()
				selectedNetIds = new Set()
			}}
			onmove={handleBoardMove}
		/>
	{/if}
	{#if partsOpen}
		<!-- A tap beside the phone's parts sheet puts it away (C-132). -->
		<button
			type="button"
			class="sheet-backdrop"
			aria-label="Close the parts"
			onclick={() => (partsOpen = false)}
		></button>
	{/if}
	<!-- ADR 0016. The same offer, over the board, from the header's
	     Examples. Escape and a press outside close it; neither reaches the
	     board, so the selection underneath is not dropped. -->
	<dialog
		class="examples-dialog"
		aria-label="Examples"
		bind:this={examplesDialog}
		onclose={() => (examplesOpen = false)}
		onkeydown={(event) => event.key === 'Escape' && event.stopPropagation()}
		onclick={(event) => event.target === examplesDialog && (examplesOpen = false)}
	>
		{#if examplesOpen}
			<div class="dialog-head">
				<h2>Examples</h2>
				<button
					type="button"
					class="close"
					aria-label="Close"
					onclick={() => (examplesOpen = false)}
				>
					<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M18 6L6 18M6 6l12 12" /></svg>
				</button>
			</div>
			{#if scene.length > 0 || nets.length > 0}
				<p class="dialog-note">Opening one replaces what is on the board. Undo brings it back.</p>
			{/if}
			{@render offer()}
		{/if}
	</dialog>
	{#if probed}
		<Scope {circuit} {elements} time={solution?.time ?? 0} />
	{/if}
	{#if projection && serialShown}
		<SerialMonitor {circuit} ports={serialPorts} log={projection.serial} />
	{/if}
	<Inspector
		onturn={selectedIds.size > 0 ? rotateSelection : undefined}
		ondelete={selectedIds.size > 0 || selectedNetIds.size > 0 ? deleteSelection : undefined}
		onclose={() => {
			selectedIds = new Set()
			selectedNetIds = new Set()
			circuitOpen = false
		}}
		{selection}
		wire={selectedWire}
		count={selectedIds.size}
		error={fieldError?.message}
		errorField={fieldError?.field}
		onvalue={handleValue}
		onseries={handleSeries}
		onfrequency={handleFrequency}
		oninitial={handleInitial}
		onflip={flip}
		onreplace={(id) => circuit?.replaceFuse(id)}
		onwiper={handleWiper}
		onchoice={handleChoice}
		onprogram={(id) => void handleProgram(id)}
		onbuiltin={(id, name) => {
			const program = PROGRAMS.find((entry) => entry.name === name)
			if (program) circuit?.setProgram(id, program.name, program.hex)
		}}
		onreset={(id) => circuit?.resetProgram(id)}
		{room}
		onroom={handleRoom}
		{description}
		ondescription={handleDescription}
		onbill={(format) => exportBill(format).catch((error) => (notice = noticeOf(error)))}
		onkicad={(what) => exportKicad(what).catch((error) => (notice = noticeOf(error)))}
		heat={selectedHeat}
		{purchase}
		{steps}
		reading={selectedReading}
	/>
	{#if halted}
		<div class="halted" role="alert">
			<p>
				The simulation stopped on an error of its own, not of your circuit's. Your work is kept:
				reload to carry on.
			</p>
			<button type="button" onclick={() => location.reload()}>Reload</button>
		</div>
	{/if}
	<StatusBar
		{status}
		detail={simulationNote ?? detail}
		spoken={spokenNote ?? detail}
		title={simulationDetail}
		position={cursor}
		findings={[]}
	/>
</div>

{#if hoveredInfo && pointer}
	<Readout info={hoveredInfo} reading={hoveredReading} at={pointer} />
{/if}

{#if drag}
	<!-- Follows the pointer so it is obvious something is being carried, and
	     obvious when it will not land. -->
	<div class="carried" class:over={drag.overCanvas} style="left: {drag.at.x}px; top: {drag.at.y}px">
		{drag.kind}
	</div>
{/if}

<style>
	/* The scope's row is `auto`, so it takes nothing at all when nothing is
	   being probed and the panel is not rendered; the serial monitor's row
	   below it likewise, while no program has begun Serial (A-06). The rail and the inspector
	   span both rows, because they belong beside the canvas and the scope
	   alike. */
	.shell {
		height: 100vh;
		display: grid;
		/* The rail is 16rem since C-128: wide enough for a symbol beside a
		   part's name and hint, and for the family chips on two rows. */
		grid-template-columns: 16rem 1fr 16rem;
		grid-template-rows: auto 1fr auto auto auto;
		grid-template-areas:
			'header header header'
			'palette canvas inspector'
			'palette scope inspector'
			'palette monitor inspector'
			'status status status';
	}

	/* The left rail holds what can be placed. What the app answers to used to
	   share it, and is behind the header's ? since C-127. */
	.rail {
		grid-area: palette;
		display: flex;
		flex-direction: column;
		min-height: 0;
	}

	.rail > :global(.palette) {
		flex: 1;
		min-height: 0;
	}

	.views {
		display: inline-flex;
		border: 1px solid var(--edge);
		border-radius: 0.5rem;
		overflow: hidden;
		margin-left: auto;
	}

	.views button {
		border: 0;
		border-radius: 0;
		background: transparent;
		padding: 0.3rem 0.7rem;
		font-size: 0.8125rem;
	}

	/* The phone's switch; beside columns the header has the pair. */
	.view-switch {
		display: none;
	}

	.views button[aria-pressed='true'] {
		background: var(--selection);
		color: var(--on-selection);
	}

	header {
		grid-area: header;
		border-bottom: 1px solid var(--edge);
		padding: 0.5rem 0.75rem 0.5rem 1rem;
		display: flex;
		align-items: center;
		gap: 1rem;
	}

	h1 {
		display: flex;
		align-items: center;
		gap: 0.625rem;
		margin: 0;
		font-size: 1.1875rem;
		font-weight: 850;
		white-space: nowrap;
	}

	h1 .dot {
		color: var(--brand);
	}

	/* The mark: a pulse on a tilted square, its shadow the top of the voltage
	   scale. The one place colour is decoration rather than a meaning. */
	.mark {
		width: 1.875rem;
		height: 1.875rem;
		flex: none;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 9px;
		background: var(--brand);
		color: var(--on-brand);
		transform: rotate(-8deg);
		box-shadow: 3px 3px 0 var(--volt-high);
	}

	.mark svg {
		width: 1.125rem;
		height: 1.125rem;
		fill: none;
		stroke: currentColor;
		stroke-width: 2.4;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.name {
		flex: 1;
		min-width: 0;
		margin: 0;
		font-family: var(--font-display);
		font-variation-settings: var(--display-axes);
		font-size: 0.9375rem;
		font-weight: 700;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.name.untitled {
		color: var(--muted);
		font-weight: 500;
	}

	.checks {
		flex: none;
		display: inline-flex;
		align-items: center;
		gap: 0.375rem;
		height: 1.875rem;
		padding: 0 0.75rem;
		border-radius: 0.9375rem;
		background: color-mix(in srgb, var(--family-passive) 14%, transparent);
		border: 1px solid color-mix(in srgb, var(--family-passive) 50%, transparent);
		color: var(--fg);
		font: inherit;
		font-size: 0.8125rem;
		cursor: pointer;
	}

	.shell :global(.checks-chip) {
		display: none;
	}

	.checks.warn {
		background: color-mix(in srgb, var(--warning) 14%, transparent);
		border-color: color-mix(in srgb, var(--warning) 55%, transparent);
	}

	.checks svg {
		width: 0.9375rem;
		height: 0.9375rem;
		fill: none;
		stroke: currentColor;
		stroke-width: 1.8;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	/* On a phone they wrap, rather than the name (C-99). */
	.files {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: flex-end;
		gap: 0.25rem;
	}

	.files button {
		height: 2.125rem;
		background: none;
		border: 1px solid var(--edge);
		border-radius: 8px;
		padding: 0 0.75rem;
		color: var(--fg);
		font: inherit;
		font-size: 0.8125rem;
		cursor: pointer;
	}

	.files button.icon {
		position: relative;
		width: 2.25rem;
		height: 2.25rem;
		padding: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		border: none;
		color: var(--muted);
	}

	.files button.icon svg {
		width: 1.125rem;
		height: 1.125rem;
		fill: none;
		stroke: currentColor;
		stroke-width: 1.8;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	/* The word is for a screen reader and for a search of the page, not for
	   the eye, which has the icon and the tooltip. */
	.files .word {
		position: absolute;
		width: 1px;
		height: 1px;
		overflow: hidden;
		clip-path: inset(50%);
		white-space: nowrap;
	}

	.files button:focus-visible {
		color: var(--fg);
		border-color: var(--muted);
	}

	@media (hover: hover) {
		.files button:hover:not([aria-disabled='true']) {
			color: var(--fg);
			border-color: var(--muted);
		}
	}

	.files button.icon:focus-visible {
		background: var(--raised);
	}

	@media (hover: hover) {
		.files button.icon:hover:not([aria-disabled='true']) {
			background: var(--raised);
		}
	}

	/* Nothing to undo, redo or clear: still there, so nothing jumps about,
	   and plainly asleep. */
	.files button[aria-disabled='true'] {
		opacity: 0.4;
		cursor: default;
	}

	.files button.labelled {
		display: inline-flex;
		align-items: center;
		gap: 0.375rem;
	}

	/* Beside columns the words are enough, and Examples alone has an icon. */
	.files button.labelled:not(.always) svg {
		display: none;
	}

	/* Bottom left, under Turn and Delete when they are up: the top is the
	   circuit's own card, the right the zoom. */
	.shell :global(.volt-key) {
		position: absolute;
		bottom: 0.75rem;
		left: 0.75rem;
		z-index: 1;
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.25rem;
		background: var(--raised);
		border: 1px solid var(--edge);
		border-radius: 10px;
		font-size: 0.75rem;
	}

	.shell :global(.volt-key button) {
		height: 1.75rem;
		padding: 0 0.625rem;
		background: none;
		border: none;
		border-radius: 7px;
		color: var(--muted);
		font: inherit;
		cursor: pointer;
	}

	.shell :global(.volt-key button[aria-pressed='true']) {
		background: var(--sunken);
		color: var(--fg);
	}

	.shell :global(.volt-key .end) {
		font-family: var(--font-mono);
		font-variation-settings: var(--mono-axes);
		color: var(--fg);
	}

	.shell :global(.volt-key .end:last-child) {
		margin-right: 0.375rem;
	}

	.shell :global(.volt-key .ramp) {
		width: 5rem;
		height: 0.375rem;
		border-radius: 3px;
		background: linear-gradient(90deg, var(--volt-low), var(--volt-mid), var(--volt-high));
	}

	/* The phone's thumb bar and the parts sheet's backdrop: phone only. */
	.shell :global(.thumbbar),
	.sheet-backdrop {
		display: none;
	}

	.files button.labelled svg {
		width: 1rem;
		height: 1rem;
		fill: none;
		stroke: currentColor;
		stroke-width: 1.8;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.examples-dialog {
		width: min(48rem, calc(100vw - 2rem));
		height: min(40rem, calc(100dvh - 4rem));
		padding: 1rem 1.25rem 0;
		background: var(--bg);
		color: var(--fg);
		border: 1px solid var(--edge);
		border-radius: 16px;
		box-shadow: 0 24px 64px rgb(0 0 0 / 0.45);
	}

	.examples-dialog[open] {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.examples-dialog::backdrop {
		background: rgb(5 6 10 / 0.6);
	}

	.dialog-head {
		display: flex;
		align-items: center;
	}

	.dialog-head h2 {
		flex: 1;
		margin: 0;
		font-size: 1.375rem;
		font-weight: 850;
	}

	.close {
		width: 2.25rem;
		height: 2.25rem;
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--raised);
		border: none;
		border-radius: 50%;
		color: var(--muted);
		cursor: pointer;
	}

	.close svg {
		width: 1rem;
		height: 1rem;
		fill: none;
		stroke: currentColor;
		stroke-width: 2;
		stroke-linecap: round;
	}

	.dialog-note {
		margin: 0;
		font-size: 0.8125rem;
		color: var(--muted);
	}

	/* In the dialog the offer is the whole of it, with no card around it. */
	.examples-dialog[open] :global(.offer) {
		width: 100%;
		margin-top: 0.25rem;
		padding: 0;
		background: none;
		border: none;
	}

	.examples-dialog[open] :global(.offer .examples-lead) {
		display: none;
	}

	.divider {
		width: 1px;
		height: 1.25rem;
		margin: 0 0.375rem;
		background: var(--edge);
	}

	/* The examples sit under the note on the empty canvas: a title to press and
	   a line saying what to look at, which is the only reason to press it.

	   `:global` because the snippet is rendered inside the canvas but written
	   here, so Svelte scopes it to this component and the canvas's own rules
	   for that note do not reach it. Hence also the sizes, which are the
	   canvas's repeated rather than inherited. */
	.shell :global(.selection-actions) {
		position: absolute;
		left: 0.75rem;
		/* Above the voltage key, which has the corner (C-134). */
		bottom: 3.25rem;
		display: flex;
		gap: 0.375rem;
		z-index: 1;
	}

	.shell :global(.selection-actions button) {
		background: var(--raised);
		border: 1px solid var(--edge);
		border-radius: 6px;
		padding: 0.375rem 0.625rem;
		color: inherit;
		font: inherit;
		font-size: 0.8125rem;
		cursor: pointer;
		box-shadow: 0 2px 8px rgb(0 0 0 / 0.25);
	}

	/* The key, from `data-key`, after the word: drawn rather than written,
	   so the button is still called only what it does. */
	.shell :global(.selection-actions button[data-key]::after) {
		content: attr(data-key);
		margin-left: 0.5rem;
		padding: 0.0625rem 0.3125rem;
		border: 1px solid var(--edge);
		border-radius: 4px;
		font-family: var(--font-mono);
		font-variation-settings: var(--mono-axes);
		font-size: 0.6875rem;
		color: var(--muted);
	}

	.shell :global(.selection-actions button:focus-visible) {
		border-color: var(--muted);
	}

	@media (hover: hover) {
		.shell :global(.selection-actions button:hover) {
			border-color: var(--muted);
		}
	}

	.shell :global(.notice) {
		position: absolute;
		left: 50%;
		top: 0.75rem;
		transform: translateX(-50%);
		max-width: min(28rem, calc(100% - 2rem));
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.5rem 0.5rem 0.5rem 0.75rem;
		border: 1px solid var(--failed);
		border-radius: 8px;
		background: var(--raised);
		box-shadow: 0 2px 12px rgb(0 0 0 / 0.3);
		z-index: 1;
	}

	.shell :global(.notice p) {
		margin: 0;
		font-size: 0.8125rem;
	}

	.shell :global(.notice button) {
		flex: none;
		background: none;
		border: 1px solid var(--edge);
		border-radius: 5px;
		padding: 0.25rem 0.5rem;
		color: inherit;
		font: inherit;
		font-size: 0.75rem;
		cursor: pointer;
	}

	/* At the foot of the board: the top is where the example's own card
	   has just appeared, and nothing is selected after an open. */
	.shell :global(.undo-notice) {
		top: auto;
		bottom: 0.75rem;
		border-color: var(--brand);
	}

	.halted {
		position: fixed;
		left: 50%;
		bottom: 3rem;
		transform: translateX(-50%);
		z-index: 20;
		max-width: min(32rem, calc(100vw - 2rem));
		padding: 0.5rem 0.75rem;
		background: var(--bg);
		border: 1px solid var(--failed);
		border-radius: 6px;
		font-size: 0.8125rem;
		display: flex;
		align-items: center;
		gap: 0.75rem;
	}

	.halted p {
		margin: 0;
	}

	.halted button {
		flex: none;
		background: none;
		border: 1px solid var(--edge);
		border-radius: 5px;
		padding: 0.25rem 0.5rem;
		color: inherit;
		font: inherit;
		font-size: 0.75rem;
		cursor: pointer;
	}

	.shell :global(.unopened) {
		margin: 0.75rem 0;
		padding: 0.5rem 0.75rem;
		border: 1px solid var(--failed);
		border-radius: 6px;
		font-size: 0.8125rem;
		display: flex;
		align-items: center;
		gap: 0.75rem;
		pointer-events: auto;
	}

	.shell :global(.unopened p) {
		margin: 0;
	}

	.shell :global(.unopened button) {
		flex: none;
		background: none;
		border: 1px solid var(--edge);
		border-radius: 5px;
		padding: 0.25rem 0.5rem;
		color: inherit;
		font: inherit;
		font-size: 0.75rem;
		cursor: pointer;
	}

	.shell :global(.offer) {
		pointer-events: auto;
		flex: 1;
		/* Never squeezed to nothing on a short board: two cards at least. */
		min-height: 13rem;
		width: min(46rem, 100%);
		margin-top: 0.75rem;
		display: flex;
		flex-direction: column;
		gap: 0.625rem;
		padding: 1rem 1rem 0;
		background: color-mix(in srgb, var(--bg) 94%, transparent);
		border: 1px solid var(--edge);
		border-radius: 14px;
		text-align: left;
	}

	.shell :global(.examples-lead) {
		margin: 0;
		font-family: var(--font-display);
		font-variation-settings: var(--display-axes);
		font-size: 1rem;
		font-weight: 750;
		color: var(--fg);
	}

	.shell :global(.shelves) {
		display: flex;
		flex-wrap: wrap;
		gap: 0.125rem 0.25rem;
		border-bottom: 1px solid var(--edge);
	}

	.shell :global(.shelves button) {
		background: none;
		border: none;
		border-bottom: 2px solid transparent;
		margin-bottom: -1px;
		padding: 0.375rem 0.5rem;
		color: var(--muted);
		font: inherit;
		font-size: 0.8125rem;
		cursor: pointer;
	}

	.shell :global(.shelves button:focus-visible) {
		color: var(--fg);
	}

	@media (hover: hover) {
		.shell :global(.shelves button:hover) {
			color: var(--fg);
		}
	}

	.shell :global(.shelves button.current) {
		color: var(--fg);
		font-weight: 600;
		border-bottom-color: var(--brand);
	}

	.shell :global(.examples) {
		list-style: none;
		margin: 0;
		padding: 0 0 1rem;
		min-height: 0;
		overflow-y: auto;
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(17rem, 1fr));
		gap: 0.5rem;
		align-content: start;
	}

	.shell :global(.examples li) {
		position: relative;
		display: grid;
		grid-template-columns: 2.75rem 1fr;
		grid-template-rows: auto auto;
		column-gap: 0.75rem;
		row-gap: 0.125rem;
		align-items: start;
		padding: 0.625rem 0.75rem;
		background: var(--sunken);
		border: 1px solid var(--edge);
		border-radius: 10px;
	}

	.shell :global(.examples li:focus-within) {
		border-color: var(--tint);
		background: var(--raised);
	}

	@media (hover: hover) {
		.shell :global(.examples li:hover) {
			border-color: var(--tint);
			background: var(--raised);
		}
	}

	.shell :global(.examples .thumb) {
		grid-row: span 2;
		width: 2.75rem;
		height: 2.75rem;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 0.5rem;
		border-radius: 9px;
		background: color-mix(in srgb, var(--tint) 12%, transparent);
		color: var(--tint);
	}

	.shell :global(.examples .thumb svg) {
		width: 100%;
		height: 100%;
		overflow: visible;
	}

	.shell :global(.examples .thumb path) {
		fill: none;
		stroke: currentColor;
		stroke-width: 1.6;
		stroke-linecap: round;
		stroke-linejoin: round;
		vector-effect: non-scaling-stroke;
	}

	.shell :global(.examples .thumb path.filled) {
		fill: currentColor;
	}

	.shell :global(.examples button) {
		background: none;
		border: none;
		padding: 0;
		color: var(--fg);
		font: inherit;
		font-size: 0.875rem;
		font-weight: 600;
		text-align: left;
		cursor: pointer;
	}

	/* The whole card is the button's: its box stretched over the card. */
	.shell :global(.examples button::after) {
		content: '';
		position: absolute;
		inset: 0;
		border-radius: 10px;
	}

	.shell :global(.examples button:focus-visible) {
		outline: none;
	}

	.shell :global(.examples li:has(button:focus-visible)) {
		outline: 2px solid var(--selection);
		outline-offset: 2px;
	}

	.shell :global(.examples span:not(.thumb)) {
		color: var(--muted);
		font-size: 0.8125rem;
		line-height: 1.4;
	}

	.carried {
		position: fixed;
		transform: translate(-50%, -50%);
		pointer-events: none;
		padding: 0.25rem 0.5rem;
		border-radius: 4px;
		border: 1px dashed var(--muted);
		background: var(--raised);
		color: var(--muted);
		font-size: 0.75rem;
		opacity: 0.6;
	}

	.carried.over {
		border-style: solid;
		border-color: var(--ok);
		color: var(--fg);
		opacity: 1;
	}

	/* Below this the three columns stop being columns. The canvas keeps the
	   width; the panels stack under it.

	   And the shell stops being exactly one screen. Stacked, the palette, the
	   keys and the inspector are taller than a phone on their own, so pinning
	   the grid to the viewport left the canvas — the row that was `1fr`, the
	   only one that has to have a size — with nothing at all: a drawing surface
	   0 px tall, on the device C-27 just made this installable on. The page
	   scrolls instead, and the canvas is given a height of its own rather than
	   whatever is left over. */
	/* Below this the three columns stop being columns (C-132).

	   The board is the screen: one row of header, the board, the scope when
	   something is probed, and the status line — and nothing scrolls, so a
	   drag on the board is never the page moving instead (C-51 stacked the
	   regions and let the page scroll, and a scroll over the board zoomed it).
	   The parts and the panel are sheets that rise over the board's foot:
	   the parts when asked for, the panel with a selection. */
	@media (width < 60rem) {
		header .views {
			display: none;
		}

		.view-switch {
			display: block;
			grid-area: canvas;
			align-self: end;
			justify-self: end;
			z-index: 2;
			margin: 0 0.75rem 4.75rem 0;
			min-height: 2.75rem;
			padding: 0 1rem;
			border-radius: 0.75rem;
			background: var(--raised);
			border: 1px solid var(--edge);
		}

		.shell {
			height: 100dvh;
			overflow: hidden;
			/* Clear of a notch, rounded corners and the home bar (C-138): zero
			   wherever there are none. */
			padding: env(safe-area-inset-top) env(safe-area-inset-right) env(safe-area-inset-bottom)
				env(safe-area-inset-left);
			grid-template-columns: minmax(0, 1fr);
			grid-template-rows: auto minmax(0, 1fr) auto auto auto;
			grid-template-areas:
				'header'
				'canvas'
				'scope'
				'monitor'
				'status';
		}

		/* The status line carries the findings and the accuracy claim (C-26),
		   both of which are only true if they are on screen. */
		.shell > :global(footer) {
			position: sticky;
			bottom: 0;
			background: var(--bg);
			/* Over the sheets too: the claim stays true while one is up. */
			z-index: 35;
		}

		header {
			padding: 0.25rem 0.25rem 0.25rem 0.5rem;
			gap: 0.25rem;
		}

		/* The buttons are what fits; the name is in the panel's description,
		   and the count of checks is on the board (B-07). */
		h1 > span:not(.mark),
		.name,
		.checks {
			display: none;
		}

		.shell :global(.checks-chip) {
			display: inline-flex;
			align-items: center;
			gap: 0.375rem;
			position: absolute;
			top: 0.75rem;
			right: 0.75rem;
			z-index: 3;
			height: 2.75rem;
			padding: 0 0.875rem;
			border-radius: 1.375rem;
			background: color-mix(in srgb, var(--family-passive) 18%, var(--bg));
			border: 1px solid color-mix(in srgb, var(--family-passive) 55%, transparent);
			color: var(--fg);
			font: inherit;
			font-size: 0.875rem;
		}

		.shell :global(.checks-chip.warn) {
			background: color-mix(in srgb, var(--warning) 18%, var(--bg));
			border-color: color-mix(in srgb, var(--warning) 55%, transparent);
		}

		.shell :global(.checks-chip svg) {
			width: 1rem;
			height: 1rem;
			fill: none;
			stroke: currentColor;
			stroke-width: 1.8;
		}

		h1 {
			flex: 1;
		}

		.files {
			flex-wrap: nowrap;
			gap: 0;
		}

		.divider {
			margin: 0 0.125rem;
		}

		/* Every one a 44 px target, its icon on show and its word kept for a
		   screen reader. */
		.files button,
		.files button.icon {
			width: 2.75rem;
			height: 2.75rem;
			padding: 0;
			justify-content: center;
			border: none;
		}

		.files button.labelled:not(.always) svg {
			display: block;
		}

		.files .label {
			position: absolute;
			width: 1px;
			height: 1px;
			overflow: hidden;
			clip-path: inset(50%);
			white-space: nowrap;
		}

		.files button.labelled {
			position: relative;
		}

		.files button.labelled svg {
			width: 1.25rem;
			height: 1.25rem;
		}

		.files :global(.shortcuts .icon) {
			width: 2.75rem;
			height: 2.75rem;
		}

		/* The parts: a sheet from the foot of the screen, most of its height,
		   when asked for. Off screen otherwise, so the board is all there is. */
		.rail {
			position: fixed;
			left: 0;
			right: 0;
			bottom: 0;
			height: 80dvh;
			z-index: 30;
			background: var(--bg);
			border-top: 1px solid var(--edge);
			border-radius: 20px 20px 0 0;
			box-shadow: 0 -12px 32px rgb(0 0 0 / 0.4);
			transform: translateY(105%);
			transition: transform 0.2s ease-out;
			/* Room at its foot for the status line drawn over it, and the
			   home bar under that. */
			padding-bottom: calc(4.75rem + env(safe-area-inset-bottom));
		}

		.parts-open .rail {
			transform: none;
		}

		.rail > :global(.palette) {
			border-right: none;
		}

		.sheet-backdrop {
			display: block;
			position: fixed;
			inset: 0;
			z-index: 25;
			border: none;
			background: rgb(5 6 10 / 0.55);
		}

		/* The panel: a lower sheet, so the part it is about stays in view
		   above it, and the board stays usable around it. */
		.shell > :global(.inspector) {
			position: fixed;
			left: 0;
			right: 0;
			bottom: 0;
			max-height: 55dvh;
			z-index: 20;
			background: var(--bg);
			border-left: none;
			border-top: 1px solid var(--edge);
			border-radius: 20px 20px 0 0;
			box-shadow: 0 -12px 32px rgb(0 0 0 / 0.4);
			transform: translateY(105%);
			transition: transform 0.2s ease-out;
			/* The status line is up to three lines on a phone, and the home
			   bar is under it. */
			padding-bottom: calc(4.75rem + env(safe-area-inset-bottom));
		}

		.panel-open > :global(.inspector) {
			transform: none;
		}

		/* Turn and Delete are in the panel's sheet on a phone, where the
		   thumb already is (C-132): on the board they would sit under it. */
		.shell :global(.selection-actions) {
			display: none;
		}

		/* Above the thumb bar, which has the foot of a phone's board, and short
		   of the view switch on the same row: its ramp gives way, rather than
		   running under the switch (C-138). */
		.shell :global(.volt-key) {
			bottom: 4.75rem;
			max-width: calc(100% - 10.5rem);
			gap: 0.25rem;
		}

		.shell :global(.volt-key .end) {
			white-space: nowrap;
		}

		.shell :global(.volt-key button) {
			height: 2.75rem;
			padding: 0 0.5rem;
			flex: none;
		}

		.shell :global(.volt-key .ramp) {
			width: auto;
			flex: 1;
			min-width: 0.5rem;
		}

		/* Across the board, and an opened example's above the thumb bar and
		   the row over it (C-138): at the foot, as on a wide screen, it lay
		   under both, squeezed to half the board by its own centring. */
		.shell :global(.notice) {
			left: 0.75rem;
			right: 0.75rem;
			transform: none;
			max-width: none;
		}

		.shell :global(.undo-notice) {
			bottom: 8.25rem;
		}

		.shell :global(.notice button),
		.shell :global(.shelves button) {
			min-height: 2.75rem;
		}

		/* One row that scrolls sideways, rather than three that took the
		   examples' room (C-138). */
		.shell :global(.shelves) {
			flex-wrap: nowrap;
			overflow-x: auto;
			scrollbar-width: none;
			flex: none;
		}

		.shell :global(.shelves button) {
			flex: none;
			white-space: nowrap;
		}

		.shell :global(.thumbbar) {
			display: flex;
			position: absolute;
			left: 0.75rem;
			right: 0.75rem;
			bottom: 0.75rem;
			gap: 0.5rem;
			z-index: 2;
			/* Clear of the zoom buttons in the corner. */
			padding-right: 7.5rem;
			pointer-events: none;
		}

		.shell :global(.thumbbar button) {
			pointer-events: auto;
			height: 3.25rem;
			display: inline-flex;
			align-items: center;
			justify-content: center;
			gap: 0.5rem;
			border-radius: 16px;
			font: inherit;
			cursor: pointer;
		}

		.shell :global(.thumbbar .add) {
			flex: 1;
			background: var(--brand);
			color: var(--on-brand);
			border: none;
			font-family: var(--font-display);
			font-variation-settings: var(--display-axes);
			font-size: 1rem;
			font-weight: 850;
			box-shadow: 3px 3px 0 var(--volt-high);
		}

		.shell :global(.thumbbar .circuit) {
			width: 3.25rem;
			flex: none;
			background: var(--raised);
			border: 1px solid var(--edge);
			color: var(--fg);
		}

		.shell :global(.thumbbar svg) {
			width: 1.25rem;
			height: 1.25rem;
			fill: none;
			stroke: currentColor;
			stroke-width: 2;
			stroke-linecap: round;
			stroke-linejoin: round;
		}
	}
</style>
