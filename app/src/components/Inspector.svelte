<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

<script lang="ts">
	import type { ElementHeat, ElementView, NetView, Purchase, Steps } from '../lib/core'
	import { openOutside } from '../lib/desktop'
	import { glyphOf } from '../lib/glyph'
	import { heatHeading, heatWarning, warmth } from '../lib/heat'
	import { isApple } from '../lib/keys'
	import { describe, familyOf } from '../lib/palette'
	import { PROGRAMS } from '../lib/programs'
	import { symbolFor, type SymbolDefinition } from '../lib/symbols'
	import { formatQuantity } from '../lib/units'

	// Shows the selected component and takes edits. It validates nothing itself:
	// the core owns what a value means, and a second opinion here would be a
	// second definition to keep in step.
	let {
		selection,
		wire,
		count,
		error,
		errorField,
		onvalue,
		onseries,
		onfrequency,
		oninitial,
		onflip,
		onreplace,
		onprogram,
		onbuiltin,
		onreset,
		onwiper,
		onchoice,
		room,
		onroom,
		description,
		ondescription,
		onbill,
		onkicad,
		heat,
		purchase,
		steps,
		reading = {},
		onclose,
		onturn,
		ondelete,
	}: {
		selection?: ElementView
		/** The selected wire, when a wire is what is selected (C-41). */
		wire?: NetView
		/** How many components are selected, when it is not exactly one. */
		count?: number
		/** What the core said about the last rejected edit, if anything. */
		error?: string
		/**
		 * Which field it was about (C-100): it is shown under that field and
		 * marks that one. Left out, it is the value's with a component
		 * selected and the room's with none, which is where it used to go.
		 */
		errorField?: 'value' | 'series' | 'frequency' | 'initial' | 'room' | 'choice'
		onvalue?: (id: string, text: string) => void
		/** A source's internal resistance was edited (C-17). */
		onseries?: (id: string, text: string) => void
		/** A signal generator's frequency was edited (ADR 0011). */
		onfrequency?: (id: string, text: string) => void
		/** A reactive component's initial condition was edited (C-18). */
		oninitial?: (id: string, text: string) => void
		/** A switch was turned on or off from here rather than on the canvas (C-19). */
		onflip?: (id: string, closed: boolean) => void
		/** A blown fuse swapped for a new one (C-120). */
		onreplace?: (id: string) => void
		/** An Uno's program asked to be loaded from a file (A-03). */
		onprogram?: (id: string) => void
		/** One of the programs the app comes with, put on an Uno (A-07). */
		onbuiltin?: (id: string, name: string) => void
		/** An Uno's reset button pressed: its program starts again (A-03). */
		onreset?: (id: string) => void
		/**
		 * A wiper was moved (C-20).
		 *
		 * `settled` is false while the knob is being dragged and true when it is
		 * let go, so a whole turn is one thing that happened rather than a
		 * hundred steps of undo. A keypress on the slider is both at once,
		 * which is right: that really is one turn.
		 */
		onwiper?: (id: string, wiper: number, settled: boolean) => void
		/** One of a component's alternatives was picked (C-46). */
		onchoice?: (id: string, choice: string) => void
		/** The temperature of the room the circuit is in, in °C (C-74). */
		room?: number
		/** The room's temperature was edited. */
		onroom?: (text: string) => void
		/** What the circuit is, in its author's words. */
		description?: string
		/** The description was edited. */
		ondescription?: (text: string) => void
		/** The parts list was asked for, as a spreadsheet or as a page (B-11). */
		onbill?: (format: 'csv' | 'md') => void
		/** The circuit was asked for in KiCad: a netlist (B-16), or a schematic (B-21). */
		onkicad?: (what: 'netlist' | 'schematic') => void
		/** How hot the selected component is running, if it is a part that warms. */
		heat?: ElementHeat
		/**
		 * The value the selected part can be bought in nearest its own, and what
		 * the circuit does on it (B-09); absent for one already at such a value.
		 */
		purchase?: Purchase
		/**
		 * The values either side of the selected part’s own, in the series it is
		 * bought in (C-136): what
		 * its − and + step to, and the chips under its field.
		 */
		steps?: Steps
		/** The selected part's voltage and current, held as a meter holds them (C-129). */
		reading?: { volts?: number; amps?: number }
		/** Put the phone's panel away, which lets go of the selection (C-132). */
		onclose?: () => void
		/**
		 * Turn and Delete, for the phone's sheet (C-132). Beside columns they
		 * are on the canvas, beside what is selected (C-99), and these are not
		 * drawn.
		 */
		onturn?: () => void
		ondelete?: () => void
	} = $props()

	function commitRoom(event: Event & { currentTarget: HTMLInputElement }) {
		onroom?.(event.currentTarget.value)
	}

	function commitRoomOnEnter(event: KeyboardEvent & { currentTarget: HTMLInputElement }) {
		if (event.key === 'Enter') commitRoom(event)
	}

	// Several lines, so Enter is a new line; a change is committed when the
	// field is left, or on Ctrl/Cmd+Enter, and is one step of undo either way.
	function commitDescription(event: Event & { currentTarget: HTMLTextAreaElement }) {
		ondescription?.(event.currentTarget.value)
	}

	function commitDescriptionOnEnter(event: KeyboardEvent & { currentTarget: HTMLTextAreaElement }) {
		if (event.key === 'Enter' && (event.metaKey || event.ctrlKey)) commitDescription(event)
	}

	const warning = $derived(heat ? heatWarning(heat) : undefined)

	/** The part's symbol, as the palette and the board draw it (C-128). */
	const glyph = $derived.by(() => {
		// A kind the app has not heard of has no symbol yet (the core may be
		// ahead of it), and is named without one.
		const symbol = selection
			? (symbolFor(selection.kind) as SymbolDefinition | undefined)
			: undefined
		return symbol ? glyphOf(symbol) : undefined
	})
	const family = $derived(selection ? familyOf(selection.kind) : undefined)

	/**
	 * What it is doing right now (C-129), as many of the four as it has: a
	 * part with three pins has no single "across", a part that does not warm
	 * has no temperature, and saying nothing is better than a zero that is
	 * a claim (`readings.ts`).
	 */
	const now = $derived.by(() => {
		const cells: { value: string; label: string }[] = []
		if (reading.volts !== undefined)
			cells.push({ value: formatQuantity(reading.volts, 'V'), label: 'across it' })
		if (reading.amps !== undefined)
			cells.push({ value: formatQuantity(reading.amps, 'A'), label: 'through it' })
		const watts =
			heat?.watts ??
			(reading.volts !== undefined && reading.amps !== undefined
				? reading.volts * reading.amps
				: undefined)
		if (watts !== undefined)
			cells.push({ value: formatQuantity(watts, 'W'), label: 'turned into heat' })
		if (heat) cells.push({ value: `${Math.round(heat.celsius)} °C`, label: 'its temperature' })
		return cells
	})

	/**
	 * How much of what it can take it is taking: of its power rating where
	 * it has one, or of the way from the room to the hottest it survives.
	 */
	const load = $derived.by(() => {
		if (!heat) return undefined
		if (heat.ratedWatts != null && heat.ratedWatts > 0) {
			return {
				share: heat.watts / heat.ratedWatts,
				// In the rating's own unit, so the two numbers can be compared
				// at a glance: "0.54 of its 0.25 W", not "540 mW of its 0.25 W".
				says: `${heat.watts.toFixed(heat.watts < 1 ? 2 : 1)} of its ${heat.ratedWatts} W`,
			}
		}
		return {
			share: warmth(heat, room ?? 25),
			says: `${Math.round(heat.celsius)} of its ${Math.round(heat.maxCelsius)} °C`,
		}
	})
	const apple = isApple()

	/** The error, if it belongs to `field`. */
	function errorFor(field: NonNullable<typeof errorField>): string | undefined {
		if (!error) return undefined
		return (errorField ?? (selection ? 'value' : 'room')) === field ? error : undefined
	}

	/**
	 * Which component a field is editing, read off the field itself rather than
	 * off `selection`.
	 *
	 * `change` fires on blur, and clicking another component on the canvas has
	 * already moved the selection on by the time it does — so a field that
	 * committed against `selection` wrote the text typed for the resistor to the
	 * battery clicked next, and left the resistor alone. Every field below sits
	 * inside `{#key selection.id}`, which is what makes the attribute the one
	 * thing that cannot have moved underneath it: another component is another
	 * input, never the same one relabelled.
	 */
	function edited(input: HTMLInputElement): string | undefined {
		return input.dataset.component
	}

	// The field holds no state of its own. It starts at whatever the core says
	// the value is, and `change` reads it straight back off the element — so a
	// rejected edit leaves the typed text alone, and an accepted one comes back
	// in the core's own spelling ("4k7" becomes "4.7k").
	function commit(event: Event & { currentTarget: HTMLInputElement }) {
		const id = edited(event.currentTarget)
		if (id) onvalue?.(id, event.currentTarget.value)
	}

	// `change` covers blur and, in a real browser, Enter. Enter is handled
	// explicitly as well because it is the keystroke people expect to mean
	// "apply this", and it should not depend on the browser's notion of when a
	// field became dirty.
	function commitOnEnter(event: KeyboardEvent & { currentTarget: HTMLInputElement }) {
		if (event.key === 'Enter') commit(event)
	}

	// Ambient help (design doc §14). The inspector used to print the core's own
	// spelling of the kind — "DcSource" — to an audience that has never built a
	// circuit, and then a number. What it is and what it is for are the two
	// things a beginner needs before the value means anything.
	const about = $derived(selection ? describe(selection.kind) : undefined)

	function commitInitial(event: Event & { currentTarget: HTMLInputElement }) {
		const id = edited(event.currentTarget)
		if (id) oninitial?.(id, event.currentTarget.value)
	}

	function commitInitialOnEnter(event: KeyboardEvent & { currentTarget: HTMLInputElement }) {
		if (event.key === 'Enter') commitInitial(event)
	}

	function commitSeries(event: Event & { currentTarget: HTMLInputElement }) {
		const id = edited(event.currentTarget)
		if (id) onseries?.(id, event.currentTarget.value)
	}

	function commitSeriesOnEnter(event: KeyboardEvent & { currentTarget: HTMLInputElement }) {
		if (event.key === 'Enter') commitSeries(event)
	}

	function commitFrequency(event: Event & { currentTarget: HTMLInputElement }) {
		const id = edited(event.currentTarget)
		if (id) onfrequency?.(id, event.currentTarget.value)
	}

	function commitFrequencyOnEnter(event: KeyboardEvent & { currentTarget: HTMLInputElement }) {
		if (event.key === 'Enter') commitFrequency(event)
	}
</script>

<aside class="inspector" aria-label="Inspector">
	<h2>Inspector</h2>

	{#if !selection && wire}
		<!-- A wire has no value to edit and one useful thing to say: what it
		     joins. Saying "nothing selected" while it is lit up on the canvas
		     would be the app contradicting itself. -->
		<p class="name">Wire</p>
		<p class="does">Joins {wire.pins.length} pins. It carries no value of its own.</p>
		<p class="hint">Delete removes it and leaves the components where they are.</p>
	{:else if !selection}
		<!-- Several selected is not the same as none, and saying "nothing" when
		     two components are highlighted is the app contradicting itself. -->
		<p class="hint">
			{count && count > 1 ? `${count} components selected.` : 'Nothing selected.'}
		</p>

		{#if !count || count < 2}
			<!-- C-129. With nothing selected the panel is about the whole
			     circuit, and says so: the room it is in and what it is. -->
			<p class="name circuit">This circuit</p>
			<!-- C-74. With nothing selected, what is left to set is the
			     circuit's own: the room it is in, which every part that warms
			     starts from and settles back to. Saved with the circuit. -->
			<hr />
			<label>
				<span class="field-name">Room temperature</span>
				<span class="field">
					<!-- Not keyed on the room: a field rebuilt by its own commit
					     loses the focus and the cursor (C-103). The value still
					     follows the circuit — an accepted edit comes back in the
					     core's spelling, an undo puts the old one back — and a
					     refused one leaves the typed text where it was. -->
					<input
						type="text"
						value={room}
						aria-label="Room temperature"
						aria-invalid={errorFor('room') ? 'true' : undefined}
						onchange={commitRoom}
						onkeydown={commitRoomOnEnter}
					/>
					<span class="unit">°C</span>
				</span>
			</label>
			{#if errorFor('room')}
				<p class="error" role="alert">{errorFor('room')}</p>
			{:else}
				<p class="hint">The air around the circuit. Parts warm up from it and cool back to it.</p>
			{/if}
			<!-- What the circuit is, shown on the canvas and saved with it:
			     for whoever opens it next, or for its author a week on. -->
			<label class="stacked">
				<span class="field-name">Description</span>
				<textarea
					rows="4"
					maxlength="1000"
					value={description ?? ''}
					placeholder="What this circuit is, and what to try on it."
					aria-label="Description"
					onchange={commitDescription}
					onkeydown={commitDescriptionOnEnter}></textarea>
			</label>
			<!-- Says when it applies, which is not as it is typed (C-103). -->
			<p class="hint">
				Its first line is the circuit's name, in the header. Kept when you leave the field, or on
				{apple ? '⌘' : 'Ctrl'}+Enter; shown on the canvas once something is on it, and saved with
				the circuit.
			</p>

			{#if onbill}
				<!-- B-11. What to buy to build it: how many, the value each is sold
				     in, the part number, and what the checks say about them. -->
				<span class="field-name bill-name">Parts list</span>
				<p class="bill">
					<button type="button" onclick={() => onbill?.('csv')}>Download CSV</button>
					<button type="button" onclick={() => onbill?.('md')}>Download Markdown</button>
				</p>
				<p class="hint">
					What to buy to build it, at values that are sold, with what the checks say.
				</p>
			{/if}

			{#if onkicad}
				<!-- B-16. Where it goes next: a PCB, laid out in KiCad. -->
				<span class="field-name bill-name">KiCad</span>
				<p class="bill">
					<button type="button" onclick={() => onkicad?.('schematic')}
						>Download KiCad schematic</button
					>
					<button type="button" onclick={() => onkicad?.('netlist')}>Download KiCad netlist</button>
				</p>
				<p class="hint">
					The schematic opens in KiCad's schematic editor, each part a box joined by labels. The
					netlist goes straight to its PCB editor: File → Import → Netlist. Each part comes with its
					footprint.
				</p>
			{/if}
		{/if}
	{:else}
		<!-- C-129. What it is, drawn as it is on the board and named, with
		     what it is for under the name. -->
		<div class="who" style:--tint={family ? `var(--family-${family})` : 'var(--muted)'}>
			{#if glyph}
				<span class="glyph" aria-hidden="true">
					<svg viewBox={glyph.viewBox}>
						{#each glyph.shapes as shape, index (index)}
							<path d={shape.d} class:filled={shape.filled} />
						{/each}
						{#each glyph.pins as pin, index (index)}
							<circle cx={pin.x} cy={pin.y} r="1.6" />
						{/each}
					</svg>
				</span>
			{/if}
			<div>
				<p class="name">{about?.label ?? selection.kind}</p>
				{#if about}<p class="does">{about.does}</p>{/if}
			</div>
		</div>
		{#if about}
			<p class="watch">{about.watch}</p>
		{/if}

		{#if now.length > 0}
			<!-- C-129. The readout's numbers, for the part that is selected
			     rather than only the one under the pointer, held as a meter
			     holds them so they can be read while they move. -->
			<section class="now" aria-label="Right now">
				<p class="now-title"><span class="live" aria-hidden="true"></span>Right now</p>
				<dl>
					{#each now as cell (cell.label)}
						<div>
							<dt>{cell.label}</dt>
							<dd>{cell.value}</dd>
						</div>
					{/each}
				</dl>
				{#if load}
					<div class="load">
						<p class="load-says">
							<span>Heat it can take</span><span class="figure">{load.says}</span>
						</p>
						<div
							class="bar"
							role="meter"
							aria-label="Heat it can take"
							aria-valuemin="0"
							aria-valuemax="1"
							aria-valuenow={Math.min(1, load.share)}
						>
							<span
								class:warm={load.share >= 0.5}
								class:hot={load.share >= 1}
								style:width="{Math.min(100, Math.max(0, load.share * 100))}%"
							></span>
						</div>
						{#if heatHeading(heat!)}
							<p class="hint">{heatHeading(heat!)}</p>
						{/if}
						{#if warning}
							<p class="error" role="alert">{warning}</p>
						{/if}
					</div>
				{/if}
			</section>
		{/if}

		{#if selection.basedOn}
			<!-- The real part its numbers come from, and the sheet to check
			     them against: the answer to "which one do I buy?" and to "is
			     this right?". The part is the core's (the registry's basis). -->
			<p class="basis">
				<span class="field-name">Modelled on</span>
				{selection.basedOn.part}{selection.basedOn.approximate ? ', roughly' : ''}
				{#if selection.basedOn.datasheet}
					· <a
						href={selection.basedOn.datasheet}
						target="_blank"
						rel="noopener noreferrer"
						onclick={openOutside}>datasheet</a
					>
				{/if}
			</p>
		{/if}

		{#if selection.choices.length > 0}
			<!-- C-46. The second shape of property: a choice from a closed set
			     rather than a number to type. One control for any of them —
			     an LED's colour today, a transistor's polarity when one can be
			     placed — because they are the same question. The options are
			     the core's, so nothing here can offer one it would refuse. -->
			<label>
				<span class="field-name">Kind</span>
				<select
					aria-label="Kind"
					value={selection.choice}
					onchange={(event) => selection && onchoice?.(selection.id, event.currentTarget.value)}
				>
					{#each selection.choices as choice (choice)}
						<option value={choice}>{choice}</option>
					{/each}
				</select>
			</label>
			{#if errorFor('choice')}
				<p class="error" role="alert">{errorFor('choice')}</p>
			{/if}
		{/if}

		{#if selection.kind === 'Pir'}
			<!-- C-116. Held on the canvas is someone walking past; its delay is
			     still a value to type, below. -->
			<p class="state">
				{selection.closed
					? 'Someone is moving in front of it.'
					: 'Nothing is moving in front of it.'}
			</p>
			<p class="hint">Hold it down on the canvas to walk past it.</p>
		{/if}

		{#if selection.kind === 'Uno'}
			<!-- A-03. What it runs, and the two things done to a real one: a
			     new program put on it, and its reset button. -->
			<span class="field-name">Program</span>
			{#if selection.program}
				<p class="state">It is running <strong>{selection.program}</strong>.</p>
			{:else}
				<p class="state">It has no program, so its pins only listen.</p>
			{/if}
			<label class="stacked">
				<span class="field-name">Its programs</span>
				<select
					aria-label="Its programs"
					value=""
					onchange={(event) => {
						const name = event.currentTarget.value
						event.currentTarget.value = ''
						if (name && selection) onbuiltin?.(selection.id, name)
					}}
				>
					<option value="" disabled>Put one on it…</option>
					{#each PROGRAMS as program (program.name)}
						<option value={program.name}>{program.name} — {program.does}</option>
					{/each}
				</select>
			</label>
			<p class="bill">
				<button type="button" onclick={() => selection && onprogram?.(selection.id)}
					>Load a .hex…</button
				>
				{#if selection.program}
					<button type="button" onclick={() => selection && onreset?.(selection.id)}>Reset</button>
				{/if}
			</p>
			<p class="hint">
				Or your own: in the Arduino IDE, Sketch → Export Compiled Binary, then the .hex without the
				bootloader.
			</p>
		{/if}

		{#if selection.kind === 'Fuse' && selection.blown}
			<!-- C-120. A blown fuse stays blown, as a real one does: what mends
			     it is a new one, which is a thing a person does. -->
			<p class="state">It has blown: nothing gets through it.</p>
			<button type="button" onclick={() => selection && onreplace?.(selection.id)}
				>Replace it</button
			>
			<p class="hint">Find what drew too much first, or the new one blows too.</p>
		{/if}

		{#if selection.closed !== undefined && selection.kind !== 'Pir'}
			<!-- C-19. A contact has no value to type, and "no value to set" is a
			     dead end beside the one component whose whole state is the thing
			     you want to know. What it says instead is what it is doing. -->
			{#if selection.kind === 'Reed'}
				<!-- C-115. Shut or open by its magnet, which is its dial. -->
				<p class="state">
					It is {selection.closed
						? 'shut: the magnet is close enough'
						: 'open: the magnet is too far away'}.
				</p>
			{:else if selection.kind === 'TiltSwitch'}
				<!-- C-88. Its state is which way up it is, so that is the word:
				     "on" would be true and tell a beginner nothing about why. -->
				<p class="state">
					It is {selection.closed ? 'upright, so it is on' : 'tipped over, so it is off'}.
				</p>
				<button
					type="button"
					onclick={() => selection && onflip?.(selection.id, !selection.closed)}
				>
					{selection.closed ? 'Tip it over' : 'Stand it up'}
				</button>
				<p class="hint">Clicking it on the canvas does the same thing.</p>
			{:else}
				<p class="state">It is {selection.closed ? 'on' : 'off'} at the moment.</p>
				{#if selection.kind === 'Switch'}
					<button
						type="button"
						onclick={() => selection && onflip?.(selection.id, !selection.closed)}
					>
						Turn it {selection.closed ? 'off' : 'on'}
					</button>
					<p class="hint">Clicking it on the canvas does the same thing.</p>
				{:else}
					<!-- No button for a button: one that latched when clicked would be
				     a lie about the part, and a real press is a press and a hold. -->
					<p class="hint">Hold it down on the canvas to close it.</p>
				{/if}
			{/if}
		{:else if selection.value !== undefined}
			<label>
				<span class="field-name">Value</span>
				<span class="field">
					<!-- Keyed on the selection so picking another component starts the
					     field afresh instead of carrying the last one's text over, and
					     so the id written on it is the one it was created for. -->
					{#key selection.id}
						<input
							type="text"
							value={selection.value}
							data-component={selection.id}
							aria-label="Value"
							aria-invalid={errorFor('value') ? 'true' : undefined}
							onchange={commit}
							onkeydown={commitOnEnter}
						/>
					{/key}
					<span class="unit">{selection.unit}</span>
					{#if steps && (steps.below.length > 0 || steps.above.length > 0)}
						<!-- C-136. Through the values that are sold, a step at a
						     time; typing any value still works. -->
						<span class="steps">
							<button
								type="button"
								aria-label={steps.below[0] ? `Down to ${steps.below[0].label}` : 'Down'}
								disabled={!steps.below[0]}
								onclick={() =>
									selection && steps?.below[0] && onvalue?.(selection.id, steps.below[0].value)}
								>−</button
							>
							<button
								type="button"
								aria-label={steps.above[0] ? `Up to ${steps.above[0].label}` : 'Up'}
								disabled={!steps.above[0]}
								onclick={() =>
									selection && steps?.above[0] && onvalue?.(selection.id, steps.above[0].value)}
								>+</button
							>
						</span>
					{/if}
				</span>
			</label>
			{#if steps && (steps.below.length > 0 || steps.above.length > 0)}
				<!-- The values either side, smallest first, to jump straight to. -->
				<p class="chips" aria-label="Values that are sold">
					{#each [...steps.below].reverse() as step (step.value)}
						<button type="button" onclick={() => selection && onvalue?.(selection.id, step.value)}
							>{step.label}</button
						>
					{/each}
					<span class="here" aria-current="true">{selection.label}</span>
					{#each steps.above as step (step.value)}
						<button type="button" onclick={() => selection && onvalue?.(selection.id, step.value)}
							>{step.label}</button
						>
					{/each}
				</p>
			{/if}

			{#if errorFor('value')}
				<p class="error" role="alert">{errorFor('value')}</p>
			{:else}
				<p class="hint">{about?.examples ?? ''}</p>
			{/if}

			{#if purchase && purchase.elementId === selection.id}
				<!-- B-09. Nobody sells 217 Ω: what to buy instead, what flows on
				     it, and anything the rounding itself would make wrong. -->
				<div class="purchase">
					<p class="state">
						Buy {purchase.label} ({purchase.series}):
						{Math.abs(purchase.errorPercent).toFixed(1)} % {purchase.errorPercent > 0
							? 'more'
							: 'less'}{#if purchase.amps !== undefined && purchase.amps !== null},
							{formatQuantity(purchase.amps, 'A')} through it{/if}.
					</p>
					{#each purchase.findings as finding (finding.rule + finding.elements.join())}
						<p class="error">On {purchase.label}: {finding.message}</p>
					{/each}
					<button
						type="button"
						onclick={() => selection && onvalue?.(selection.id, purchase.value)}
					>
						Use {purchase.label}
					</button>
				</div>
			{/if}
		{:else if selection.choices.length === 0 && selection.kind !== 'Uno'}
			<!-- Last, and only when there is genuinely nothing: an LED has a
			     colour to choose now (C-46), and "no value to set" under its own
			     chooser is the panel contradicting itself. -->
			<p class="hint">This component has no value to set.</p>
		{/if}

		{#if selection.wiper !== undefined}
			<!-- C-20. The knob itself: dragging it changes the circuit while it
			     moves, which is the whole of "updates live" — and a slider is
			     also the one control a keyboard can turn. -->
			<hr />
			<label>
				<span class="field-name"
					>{about?.dial?.name ?? 'Wiper'} — {about?.dial?.reading(selection.wiper) ??
						`${Math.round(selection.wiper * 100)}%`}</span
				>
				<input
					type="range"
					min="0"
					max="1"
					step="0.01"
					value={selection.wiper}
					aria-label={about?.dial?.name ?? 'Wiper'}
					oninput={(event) =>
						selection && onwiper?.(selection.id, Number(event.currentTarget.value), false)}
					onchange={(event) =>
						selection && onwiper?.(selection.id, Number(event.currentTarget.value), true)}
				/>
			</label>
			<p class="hint">{about?.dial?.hint}</p>
		{/if}

		{#if selection.seriesOhms !== undefined || selection.initialCondition !== undefined || selection.hertz !== undefined}
			<!-- A second property needs air between it and the first, or the two
			     read as one field with a stray label. -->
			<hr />
		{/if}

		{#if selection.seriesOhms !== undefined}
			<!-- C-17. A real cell has a little resistance of its own, and it is
			     what stops a short circuit being an unbounded current. Left at
			     zero the source is ideal, which is what every textbook circuit
			     assumes and what a beginner should not have to think about. -->
			<label>
				<span class="field-name">Internal resistance</span>
				<span class="field">
					{#key selection.id}
						<input
							type="text"
							value={selection.seriesOhms || ''}
							data-component={selection.id}
							aria-label="Internal resistance"
							aria-invalid={errorFor('series') ? 'true' : undefined}
							placeholder="0"
							onchange={commitSeries}
							onkeydown={commitSeriesOnEnter}
						/>
					{/key}
					<span class="unit">Ω</span>
				</span>
			</label>
			{#if errorFor('series')}
				<p class="error" role="alert">{errorFor('series')}</p>
			{:else}
				<p class="hint">Leave it empty for a perfect battery.</p>
			{/if}
		{/if}

		{#if selection.hertz !== undefined}
			<!-- ADR 0011. The second number a signal has, beside its size: how
			     often it swings. The live canvas runs in real time, so slow is
			     what can be watched — an LED blinking, a trace a person can
			     follow. -->
			<label>
				<span class="field-name">Frequency</span>
				<span class="field">
					{#key selection.id}
						<input
							type="text"
							value={selection.hertz}
							data-component={selection.id}
							aria-label="Frequency"
							aria-invalid={errorFor('frequency') ? 'true' : undefined}
							onchange={commitFrequency}
							onkeydown={commitFrequencyOnEnter}
						/>
					{/key}
					<span class="unit">Hz</span>
				</span>
			</label>
			{#if errorFor('frequency')}
				<p class="error" role="alert">{errorFor('frequency')}</p>
			{:else}
				<p class="hint">Swings a second. Keep it slow enough to watch.</p>
			{/if}
		{/if}

		{#if selection.initialCondition !== undefined}
			<!-- C-18. What it is holding when the simulation starts: volts for a
			     capacitor, amps for a coil. Empty means it starts with nothing,
			     which is what a component just out of the palette has. -->
			<label>
				<span class="field-name">Starts at</span>
				<span class="field">
					{#key selection.id}
						<input
							type="text"
							value={selection.initialCondition || ''}
							data-component={selection.id}
							aria-label="Starts at"
							aria-invalid={errorFor('initial') ? 'true' : undefined}
							placeholder="0"
							onchange={commitInitial}
							onkeydown={commitInitialOnEnter}
						/>
					{/key}
					<span class="unit">{selection.initialUnit}</span>
				</span>
			</label>
			{#if errorFor('initial')}
				<p class="error" role="alert">{errorFor('initial')}</p>
			{/if}
		{/if}
	{/if}
	<!-- Last, so it is not the panel's first control; drawn in its corner. -->
	{#if onturn || ondelete}
		<div class="sheet-actions">
			{#if onturn}<button type="button" onclick={onturn}>Turn 90°</button>{/if}
			{#if ondelete}<button type="button" class="delete" onclick={ondelete}>Delete</button>{/if}
		</div>
	{/if}
	{#if onclose}
		<button type="button" class="sheet-close" aria-label="Close" onclick={() => onclose()}>
			<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M18 6L6 18M6 6l12 12" /></svg>
		</button>
	{/if}
</aside>

<style>
	.inspector {
		grid-area: inspector;
		border-left: 1px solid var(--edge);
		padding: 1rem;
		overflow-y: auto;
	}

	h2 {
		font-size: 0.75rem;
		text-transform: uppercase;
		letter-spacing: 0.08em;
		color: var(--muted);
		margin: 0 0 0.75rem;
	}

	.name {
		font-family: var(--font-display);
		font-variation-settings: var(--display-axes);
		font-size: 1.1875rem;
		font-weight: 800;
		margin: 0 0 0.125rem;
	}

	.name.circuit {
		margin-top: 0.75rem;
	}

	.who {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		margin-bottom: 0.5rem;
	}

	.who .does {
		margin: 0;
	}

	.glyph {
		width: 3rem;
		height: 3rem;
		flex: none;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 0.5rem;
		border-radius: 10px;
		background: color-mix(in srgb, var(--tint) 12%, transparent);
		border: 1px solid color-mix(in srgb, var(--tint) 40%, transparent);
		color: var(--tint);
	}

	.glyph svg {
		width: 100%;
		height: 100%;
		overflow: visible;
	}

	.glyph path {
		fill: none;
		stroke: currentColor;
		stroke-width: 1.6;
		stroke-linecap: round;
		stroke-linejoin: round;
		vector-effect: non-scaling-stroke;
	}

	.glyph path.filled,
	.glyph circle {
		fill: currentColor;
	}

	/* The card of live numbers: large, in the mono face, so they can be read
	   while they move without shifting sideways. */
	.now {
		margin: 0 0 0.875rem;
		padding: 0.75rem;
		background: var(--raised);
		border: 1px solid var(--edge);
		border-radius: 12px;
	}

	.now-title {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		margin: 0 0 0.625rem;
		font-size: 0.6875rem;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--muted);
	}

	.live {
		width: 0.4375rem;
		height: 0.4375rem;
		border-radius: 50%;
		background: var(--ok);
	}

	dl {
		margin: 0;
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 0.75rem 0.5rem;
	}

	dl div {
		display: flex;
		flex-direction: column-reverse;
		gap: 0.0625rem;
	}

	dt {
		font-size: 0.75rem;
		color: var(--muted);
	}

	dd {
		margin: 0;
		font-family: var(--font-mono);
		font-variation-settings: var(--mono-axes);
		font-size: 1.125rem;
		font-weight: 500;
		letter-spacing: -0.02em;
	}

	.load {
		margin-top: 0.75rem;
		padding-top: 0.625rem;
		border-top: 1px solid var(--edge);
	}

	.load-says {
		display: flex;
		flex-wrap: wrap;
		justify-content: space-between;
		gap: 0.5rem;
		margin: 0 0 0.375rem;
		font-size: 0.75rem;
	}

	.figure {
		font-family: var(--font-mono);
		font-variation-settings: var(--mono-axes);
	}

	.bar {
		height: 0.375rem;
		border-radius: 3px;
		background: var(--sunken);
		overflow: hidden;
	}

	.bar span {
		display: block;
		height: 100%;
		border-radius: 3px;
		background: var(--ok);
	}

	.bar span.warm {
		background: var(--warning);
	}

	.bar span.hot {
		background: var(--failed);
	}

	/* Short enough to always be there: two lines, muted, above the value the
	   panel exists to edit rather than in front of it. */
	.does,
	.watch {
		font-size: 0.8125rem;
		color: var(--muted);
		margin: 0 0 0.375rem;
	}

	.watch {
		margin-bottom: 0.75rem;
	}

	.basis {
		font-size: 0.8125rem;
		margin: 0 0 0.75rem;
	}

	.basis a {
		color: var(--accent);
	}

	.field-name {
		display: block;
		font-size: 0.75rem;
		color: var(--muted);
		margin-bottom: 0.25rem;
	}

	.field {
		display: flex;
		align-items: center;
		gap: 0.375rem;
	}

	input {
		flex: 1;
		min-width: 0;
		background: var(--sunken);
		border: 1px solid var(--edge);
		border-radius: 4px;
		padding: 0.375rem 0.5rem;
		color: inherit;
		font: inherit;
		font-variant-numeric: tabular-nums;
	}

	input[aria-invalid='true'] {
		border-color: var(--failed);
	}

	.stacked {
		display: block;
		margin-top: 0.75rem;
	}

	textarea {
		box-sizing: border-box;
		width: 100%;
		resize: vertical;
		background: var(--sunken);
		border: 1px solid var(--edge);
		border-radius: 4px;
		padding: 0.375rem 0.5rem;
		color: inherit;
		font: inherit;
		font-size: 0.8125rem;
		line-height: 1.4;
	}

	select {
		width: 100%;
		background: var(--sunken);
		border: 1px solid var(--edge);
		border-radius: 4px;
		padding: 0.375rem 0.5rem;
		color: inherit;
		font: inherit;
	}

	input[type='range'] {
		width: 100%;
		padding: 0;
		background: none;
		border: none;
		accent-color: var(--accent);
	}

	.state {
		font-size: 0.8125rem;
		margin: 0 0 0.5rem;
	}

	button {
		background: var(--sunken);
		border: 1px solid var(--edge);
		border-radius: 4px;
		padding: 0.375rem 0.625rem;
		color: inherit;
		font: inherit;
		cursor: pointer;
	}

	button:focus-visible {
		border-color: var(--muted);
	}

	@media (hover: hover) {
		button:hover {
			border-color: var(--muted);
		}
	}

	.unit {
		color: var(--muted);
	}

	hr {
		border: none;
		border-top: 1px solid var(--edge);
		margin: 1rem 0 0.75rem;
	}

	.hint,
	.error {
		font-size: 0.8125rem;
		margin: 0.5rem 0 0;
	}

	.hint {
		color: var(--muted);
	}

	.purchase {
		margin-top: 0.75rem;
	}

	.steps {
		display: flex;
		gap: 0.25rem;
	}

	.steps button {
		min-width: 2rem;
		padding: 0.375rem 0;
	}

	.steps button:disabled {
		opacity: 0.4;
		cursor: default;
	}

	.chips {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 0.375rem;
		margin: 0.5rem 0 0;
	}

	.chips button {
		padding: 0.125rem 0.5rem;
		border-radius: 999px;
		font-size: 0.75rem;
	}

	.chips .here {
		font-size: 0.75rem;
		font-weight: 600;
		padding: 0 0.25rem;
	}

	.bill-name {
		display: block;
		margin-top: 1.25rem;
	}

	.bill {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
		margin: 0.5rem 0 0;
	}

	.purchase button {
		margin-top: 0.5rem;
	}

	.error {
		color: var(--failed);
	}

	/* A phone's sheet closes from here (C-132); beside columns there is
	   nothing to close. */
	.sheet-close,
	.sheet-actions {
		display: none;
	}

	@media (width < 60rem) {
		.sheet-actions {
			display: flex;
			gap: 0.5rem;
			margin-top: 1rem;
		}

		.sheet-actions button {
			flex: 1;
			min-height: 3rem;
			border-radius: 12px;
			font-size: 0.9375rem;
		}

		.sheet-actions .delete {
			color: var(--failed);
		}

		.sheet-close {
			position: absolute;
			right: 0.75rem;
			top: 0.625rem;
			width: 2.75rem;
			height: 2.75rem;
			display: flex;
			align-items: center;
			justify-content: center;
			padding: 0;
			background: var(--raised);
			border: none;
			border-radius: 50%;
			color: var(--muted);
			cursor: pointer;
			z-index: 1;
		}

		.sheet-close svg {
			width: 1.125rem;
			height: 1.125rem;
			fill: none;
			stroke: currentColor;
			stroke-width: 2;
			stroke-linecap: round;
		}
	}
</style>
