// SPDX-License-Identifier: MIT OR Apache-2.0

// What the app can say about a component, and which components it offers.
//
// Data, not markup: the palette and the inspector render whatever is here, and
// C-16's component registry will replace this file wholesale without either of
// them noticing.
//
// Ground is described but not offered. §5 of the design doc never listed it —
// it was added during M-15 on the way past — and ADR 0003 makes the reference
// implicit precisely so that a beginner never has to know what a ground is.
// Offering one anyway asks a question the app has gone to trouble to avoid
// asking, and the first M-22 tester asked it. A circuit that places one still
// has to be describable, which is why it is still in the catalogue.

/** Matches the core's ElementKind variant and the symbol table. */
export type ComponentKind =
	| 'DcSource'
	| 'SignalGenerator'
	| 'Cell'
	| 'SolarCell'
	| 'Resistor'
	| 'Capacitor'
	| 'Electrolytic'
	| 'Inductor'
	| 'Transformer'
	| 'Bridge'
	| 'Led'
	| 'RgbLed'
	| 'SevenSegment'
	| 'LcdDigit'
	| 'Bargraph'
	| 'Diode'
	| 'Zener'
	| 'Switch'
	| 'PushButton'
	| 'TiltSwitch'
	| 'Relay'
	| 'Fuse'
	| 'Buzzer'
	| 'PassiveBuzzer'
	| 'Motor'
	| 'Servo'
	| 'Potentiometer'
	| 'Ldr'
	| 'Phototransistor'
	| 'Thermistor'
	| 'Voltmeter'
	| 'Ammeter'
	| 'Bjt'
	| 'Mosfet'
	| 'Regulator'
	| 'OpAmp'
	| 'Comparator'
	| 'Tmp36'
	| 'Timer'
	| 'Decoder'
	| 'Counter'
	| 'MotorDriver'
	| 'Optocoupler'
	| 'Lm3914'
	| 'ShiftRegister'
	| 'Gate'
	| 'Inverter'
	| 'Schmitt'
	| 'Decade'
	| 'FlipFlop'
	| 'Adjustable'
	| 'Schottky'
	| 'Reed'
	| 'Hall'
	| 'Pir'
	| 'Uno'
	| 'Ground'

/** Everything the app knows how to say about one kind of component. */
export interface ComponentInfo {
	kind: ComponentKind
	label: string
	/**
	 * What it is, in one line. The palette has room for this and no more.
	 */
	hint: string
	/**
	 * What it does, for the inspector, where there is room for a sentence.
	 *
	 * Ambient help (design doc §14): always on screen, so it has to stay short
	 * enough that reading past it costs nothing.
	 */
	does: string
	/** The thing a beginner gets wrong about it first. */
	watch: string
	/**
	 * Values worth typing, for the field that takes them.
	 *
	 * Per kind, because "220R" is no help beside a capacitor: the prefixes a
	 * component is sold in are part of how it is described, not of what it is.
	 */
	examples?: string
	/**
	 * The one dial a part has, when it has one (C-20, C-65): what it is called
	 * beside its slider, how a position reads, and a line on what turning it
	 * does. The core holds the position from 0 to 1; saying what that means is
	 * a description, and descriptions are the app's (ADR 0008).
	 */
	dial?: {
		readonly name: string
		readonly reading: (position: number) => string
		readonly hint: string
	}
}

export const CATALOGUE: Readonly<Record<ComponentKind, ComponentInfo>> = {
	DcSource: {
		kind: 'DcSource',
		label: 'Battery',
		hint: 'Pushes current around the circuit',
		examples: 'Try 9, 3.3 or 12.',
		does: 'Pushes current out of its + pin and takes it back in at its −.',
		watch: 'Joining + straight back to − with nothing in between is a short circuit.',
	},
	SignalGenerator: {
		kind: 'SignalGenerator',
		label: 'Signal generator',
		hint: 'A voltage that keeps swinging up and down',
		does: 'Swings its voltage between + and − over and over: a square, a sine or a triangle.',
		watch: 'Half of every swing its + end is the negative one, so an LED on it blinks.',
	},
	Cell: {
		kind: 'Cell',
		label: 'Real battery',
		hint: 'A battery that sags under load and runs down',
		does: 'Its voltage falls as it empties, and sags further the more current you take from it.',
		watch:
			'A heavy load empties it sooner. Its label says how long it lasts at the current it gives.',
		dial: {
			name: 'Charge',
			reading: (position) => `${Math.round(position * 100)} % left`,
			hint: 'Slide it down to see what a nearly flat one does.',
		},
	},
	SolarCell: {
		kind: 'SolarCell',
		label: 'Solar cell',
		hint: 'Turns light into current',
		does: 'In full sun it gives up to 105 mA, or 4.6 V to nothing; less light, less current.',
		watch:
			'It is not a battery: the more you draw, the lower its voltage. At night it gives nothing.',
		dial: {
			name: 'Sun',
			// A share of the sun its sheet was measured in, 1000 W/m², on a
			// straight scale: its current follows the light (C-117).
			reading: (position) => `${Math.round(position * 100)} % sun`,
			hint: 'Slide it from night to the full midday sun.',
		},
	},
	Resistor: {
		kind: 'Resistor',
		label: 'Resistor',
		hint: 'Limits how much current flows',
		examples: 'Try 220, 220R, 4k7 or 1M.',
		does: 'Holds the current back. A bigger number means less current.',
		watch: 'It has no right way round — either way works.',
	},
	Capacitor: {
		kind: 'Capacitor',
		label: 'Capacitor',
		hint: 'Stores a little charge, and lets changes through',
		examples: 'Try 100n, 10u or 1m.',
		does: 'Fills up with charge, then stops the current until something changes.',
		watch: 'It blocks a steady voltage, so on its own nothing flows through it.',
	},
	Electrolytic: {
		kind: 'Electrolytic',
		label: 'Electrolytic capacitor',
		hint: 'A big capacitor with a + and a − leg',
		examples: 'Try 10u, 100u or 1000u.',
		does: 'Stores far more charge than a small one. Its + must face the higher voltage.',
		watch:
			'Backwards, or past the voltage printed on it, a real one can burst. The stripe marks −.',
	},
	Inductor: {
		kind: 'Inductor',
		label: 'Coil',
		hint: 'Resists a change in current',
		examples: 'Try 1m, 100u or 10m.',
		does: 'Keeps the current it already has running, and takes time to change it.',
		watch: 'It passes a steady current freely, like a plain piece of wire.',
	},
	Transformer: {
		kind: 'Transformer',
		label: 'Transformer',
		hint: 'Steps an alternating voltage up or down',
		examples: 'Try 2 for half the voltage, 10 for a tenth, 0.5 for double.',
		does: 'A sine on p1–p2 comes out of s1–s2 divided by its ratio. Nothing joins the two sides.',
		watch: 'It needs a voltage that keeps changing: on a battery the secondary gives nothing.',
	},
	Led: {
		kind: 'Led',
		label: 'LED',
		hint: 'Lights up — needs a resistor to survive',
		does: 'Lights when current flows in at the anode and out at the cathode.',
		watch: 'It only lights one way round, and only with a resistor to limit the current.',
	},
	RgbLed: {
		kind: 'RgbLed',
		label: 'RGB LED',
		hint: 'Red, green and blue in one — mixes any colour',
		does: 'Three LEDs sharing one pin. Common cathode: R, G and B each light when fed.',
		watch: 'Each colour needs its own resistor. Red lights from less voltage than the others.',
	},
	SevenSegment: {
		kind: 'SevenSegment',
		label: 'Digit',
		hint: 'Seven LEDs that draw a number',
		does: 'Each bar is an LED. Common cathode: a bar lights when its pin is fed.',
		watch: 'Every bar needs its own resistor, just like any LED.',
	},
	LcdDigit: {
		kind: 'LcdDigit',
		label: 'LCD digit',
		hint: 'A digit like a watch’s — draws almost nothing',
		does: 'A bar turns dark when enough AC is across it and COM, either way round.',
		watch: 'Needs AC: a bar swinging opposite to COM shows. DC shows too, and wears it out.',
	},
	Bargraph: {
		kind: 'Bargraph',
		label: 'LED bar',
		hint: 'Ten LEDs in a row, in one part',
		does: 'Ten red LEDs side by side: each lights from its own a to its own k.',
		watch: 'Each is an LED like any other: it needs its current limited, by a resistor or a chip.',
	},
	Diode: {
		kind: 'Diode',
		label: 'Diode',
		hint: 'Lets current through one way only',
		does: 'Passes current from the anode to the cathode, and blocks it the other way.',
		watch: 'It does not light. Same part as an LED, without the light.',
	},
	Schottky: {
		kind: 'Schottky',
		label: 'Schottky diode',
		hint: 'A diode that drops less',
		does: 'Lets current through one way only, like a diode, for about 0.3 V instead of 0.7.',
		watch: 'It leaks a little backwards. The stripe marks the cathode, as on any diode.',
	},
	Bridge: {
		kind: 'Bridge',
		label: 'Bridge rectifier',
		hint: 'Four diodes that turn AC into DC',
		does: 'Whichever way round the voltage on its two ~ pins, + comes out on top, less two diodes.',
		watch: 'It gives pulses, not a steady voltage: a big capacitor across + and − fills the gaps.',
	},
	Zener: {
		kind: 'Zener',
		label: 'Zener diode',
		hint: 'A diode that holds a fixed voltage when reversed',
		does: 'Forwards it is a diode. Reversed, it passes current at its own voltage and no lower.',
		watch: 'It needs a resistor in front of it, like an LED, or it burns out holding its voltage.',
	},
	Switch: {
		kind: 'Switch',
		label: 'Switch',
		hint: 'Breaks the circuit, or joins it again',
		does: 'Joins the two sides when it is on, and leaves a gap when it is off.',
		watch: 'Nothing flows anywhere in the loop while it is off — that is the point.',
	},
	PushButton: {
		kind: 'PushButton',
		label: 'Push-button',
		hint: 'Joins the circuit only while it is held',
		does: 'Joins the two sides while you hold it, and lets go when you do.',
		watch:
			'A real one chatters as it closes and opens. Choose Bouncy to see what a chip makes of it.',
	},
	Reed: {
		kind: 'Reed',
		label: 'Reed switch',
		hint: 'A switch a magnet closes',
		does: 'Two blades in glass, drawn together while a magnet is near: closer than about 18 mm here.',
		watch: 'Nothing on it moves that you can see — slide the magnet, not the switch.',
		dial: {
			name: 'Magnet',
			// 30 mm away at the bottom to touching at the top, which is the
			// core's reading of the same position (C-115).
			reading: (position) => `${Math.round(30 * (1 - position))} mm away`,
			hint: 'Slide it to bring a magnet closer or take it away.',
		},
	},
	TiltSwitch: {
		kind: 'TiltSwitch',
		label: 'Tilt switch',
		hint: 'Joins the circuit while it stands upright',
		does: 'A ball inside rests across its two legs while it is upright, and rolls off when it tips.',
		watch: 'It has no right way round, only a right way up. Click it to tip it over.',
	},
	Relay: {
		kind: 'Relay',
		label: 'Relay',
		hint: 'A switch thrown by current through its coil',
		does: 'Current in its coil pulls the contact from NC over to NO; it springs back when it stops.',
		watch: 'A coil takes far more current than an LED, so drive it through a transistor.',
	},
	Fuse: {
		kind: 'Fuse',
		label: 'Fuse',
		hint: 'Opens the circuit when too much flows',
		examples: 'Try 500m, 1 or 2.5.',
		does: 'Carries its rating for ever; well past it, it heats and opens before the wires do.',
		watch: 'A blown fuse stays blown until replaced. A polyfuse resets once cool, fault gone.',
	},

	Buzzer: {
		kind: 'Buzzer',
		label: 'Buzzer',
		hint: 'Beeps when current flows through it',
		does: 'Beeps once it has about 3 V across it, + to +. Reversed, it stays silent.',
		watch: 'It is made for 5 V: on a 9 V battery give it a resistor to share the voltage.',
	},
	PassiveBuzzer: {
		kind: 'PassiveBuzzer',
		label: 'Passive buzzer',
		hint: 'A tiny speaker — plays the note it is fed',
		does: 'Sounds at whatever frequency the voltage across it swings at. On steady DC it is silent.',
		watch: 'Its coil is only 16 Ω: drive it through a transistor, with a resistor in front.',
	},
	Motor: {
		kind: 'Motor',
		label: 'Motor',
		hint: 'Turns when current flows through it',
		does: 'Turns once it has about a volt across it, one way round or the other.',
		watch: 'It takes far more current than a transistor can give: switch it with a relay.',
	},
	Servo: {
		kind: 'Servo',
		label: 'Servo',
		hint: 'Turns to the angle its pulses ask for',
		does: 'A pulse every 20 ms: 1 ms high is one end, 2 ms the other, 1.5 ms the middle.',
		watch: 'A steady voltage on SIG does nothing: it needs pulses, from a 555 or an Arduino.',
	},
	Potentiometer: {
		kind: 'Potentiometer',
		label: 'Potentiometer',
		hint: 'A resistor you can turn up and down',
		examples: 'Try 10k, 1k or 100k.',
		does: 'Splits its resistance either side of a wiper you can move.',
		watch: 'Turned all the way down it holds nothing back at all.',
		dial: {
			name: 'Wiper',
			reading: (position) => `${Math.round(position * 100)}%`,
			hint: 'All the way down is a plain wire; all the way up is the whole track.',
		},
	},
	Ldr: {
		kind: 'Ldr',
		label: 'Light sensor',
		hint: 'A resistor that drops as light falls on it',
		does: 'Brighter light, less resistance: about 250 kΩ in the dark, under 100 Ω in sunlight.',
		watch: 'It is a resistor, not a switch: put it in a divider to turn light into a voltage.',
		dial: {
			name: 'Light',
			// 0.1 lux at the bottom to 10 000 at the top, on a log scale,
			// which is the core's reading of the same position (C-65).
			reading: (position) => {
				const lux = 10 ** (5 * position - 1)
				return `${lux >= 10 ? Math.round(lux) : lux.toFixed(1)} lux`
			},
			hint: 'Slide it to darken or brighten the room it is in.',
		},
	},
	Phototransistor: {
		kind: 'Phototransistor',
		label: 'Phototransistor',
		hint: 'A transistor switched on by light',
		does: 'Light lets current in at c and out at e: 2 µA for every lux, until it is fully on.',
		watch: 'It has a right way round: c towards +. Under a resistor it swings nearly rail to rail.',
		dial: {
			name: 'Light',
			// The LDR's scale, so the two read the same light (C-89).
			reading: (position) => {
				const lux = 10 ** (5 * position - 1)
				return `${lux >= 10 ? Math.round(lux) : lux.toFixed(1)} lux`
			},
			hint: 'Slide it to darken or brighten the room it is in.',
		},
	},
	Thermistor: {
		kind: 'Thermistor',
		label: 'Thermistor',
		hint: 'A resistor that drops as it warms up',
		does: 'Ten kilohms at room temperature, less when warm, more when cold.',
		watch: 'Like a light sensor, it wants a resistor beside it to make a voltage you can use.',
		dial: {
			name: 'Temperature',
			reading: (position) => `${Math.round(-20 + 120 * position)} °C`,
			hint: 'Slide it to cool it down or warm it up.',
		},
	},
	Bjt: {
		kind: 'Bjt',
		label: 'Transistor',
		hint: 'A small current at the base lets a big one through',
		does: 'A small current into the base lets a much bigger one run from collector to emitter.',
		watch: 'The base needs a resistor of its own, or it takes far too much current.',
	},
	Mosfet: {
		kind: 'Mosfet',
		label: 'MOSFET',
		hint: 'A transistor switched by a voltage, not a current',
		does: 'A voltage on the gate lets current run from drain to source. The gate itself takes none.',
		watch: 'A gate left unconnected holds whatever charge it last had, so give it a resistor.',
	},
	Regulator: {
		kind: 'Regulator',
		label: 'Regulator',
		hint: 'Turns a bigger voltage into a steady smaller one',
		does: 'Holds its output at a fixed voltage, whatever the battery feeding it does.',
		watch:
			'It needs a couple of volts more in than it gives out, and turns the difference into heat.',
	},
	Adjustable: {
		kind: 'Adjustable',
		label: 'Adjustable regulator',
		hint: 'Any steady voltage, set by two resistors',
		does: 'Holds out 1.25 V above adj, so a resistor from out to adj and one to − set the voltage.',
		watch: 'It needs 2 V more in than out. 240 Ω from out to adj is the usual first resistor.',
	},
	OpAmp: {
		kind: 'OpAmp',
		label: 'Op-amp',
		hint: 'Makes the difference between two voltages much bigger',
		does: 'Outputs the difference between its inputs, far bigger — up to what its supply allows.',
		watch: 'It needs power of its own: V+ to the battery’s +, V− to its −.',
	},
	Comparator: {
		kind: 'Comparator',
		label: 'Comparator',
		hint: 'Says which of two voltages is higher',
		does: 'Pulls its output down when − is higher than +, and lets go of it otherwise.',
		watch: 'It can only pull down: its output needs a resistor up to + to go high.',
	},
	Hall: {
		kind: 'Hall',
		label: 'Hall sensor',
		hint: 'Senses a magnet without touching it',
		does: 'Pulls OUT down to GND while a magnet is near: on at about 10 mm, off again past 12.',
		watch: 'OUT only pulls down: give it a resistor up to +, or an LED and its resistor.',
		dial: {
			name: 'Magnet',
			// 30 mm away at the bottom to touching at the top, which is the
			// core's reading of the same position (C-115).
			reading: (position) => `${Math.round(30 * (1 - position))} mm away`,
			hint: 'Slide it to bring a magnet closer or take it away.',
		},
	},
	Pir: {
		kind: 'Pir',
		label: 'Motion sensor',
		hint: 'Sees someone move, and says so for a while',
		examples: 'Its delay: try 5, 30 or 200.',
		does: 'OUT goes to 3.3 V when something warm moves in front of it, for the delay you set.',
		watch: 'It settles for a minute after power-up and sees nothing until then. Hold it to move.',
	},
	Uno: {
		kind: 'Uno',
		label: 'Arduino Uno',
		hint: 'A small computer whose pins drive the circuit',
		does: 'Each pin is 5 V, 0 V or listening, as its program says. 5V and 3V3 power the circuit.',
		watch: 'A pin gives 20 mA at most: an LED on it needs its resistor. Wire back to its GND.',
	},
	Tmp36: {
		kind: 'Tmp36',
		label: 'Temperature sensor',
		hint: 'Turns the temperature into a voltage',
		does: 'Puts out half a volt, plus 10 mV for every degree: 0.75 V at 25 °C.',
		watch: 'It runs on 2.7 V to 5.5 V — not straight off a 9 V battery.',
		dial: {
			name: 'Temperature',
			// −40 °C to 125 °C, its specified range, which is the core's
			// reading of the same position (C-87).
			reading: (position) => `${Math.round(-40 + 165 * position)} °C`,
			hint: 'Slide it to cool it down or warm it up.',
		},
	},
	Timer: {
		kind: 'Timer',
		label: '555 timer',
		hint: 'Makes an LED blink, or stay lit for a set time',
		does: 'Switches its output high and low by itself, at a pace set by the parts around it.',
		watch: 'Its timing comes from a resistor and a capacitor you add, not from the chip.',
	},
	Decoder: {
		kind: 'Decoder',
		label: 'Digit decoder',
		hint: 'Turns four switches into a number on a digit',
		does: 'Reads A to D as a number from 0 to 9 and lights the digit’s bars to draw it.',
		watch: 'Its inputs float: give each a pull-down resistor, and LT and BI a wire to +.',
	},
	Counter: {
		kind: 'Counter',
		label: 'Digit counter',
		hint: 'Counts pulses straight onto a digit',
		does: 'Adds one every time CLK goes from low to high, 0 to 9 and round again.',
		watch: 'Wire RST and INH to −, DEI to +, or it will not count or show anything.',
	},
	MotorDriver: {
		kind: 'MotorDriver',
		label: 'Motor driver',
		hint: 'Runs a motor forwards or backwards from two switches',
		does: 'Drives each Y pin high or low as its A pin says, from VCC2, while its EN is high.',
		watch: 'VCC1 is 5 V for its logic and VCC2 the motor’s supply: wire both, and GND.',
	},
	ShiftRegister: {
		kind: 'ShiftRegister',
		label: 'Shift register',
		hint: 'Drives eight LEDs from three wires',
		does: 'SRCLK rising takes SER in as a bit; RCLK rising shows the eight on QA to QH.',
		watch: 'SRCLR and OE work backwards: wire SRCLR to + and OE to −, or nothing shows.',
	},
	Optocoupler: {
		kind: 'Optocoupler',
		label: 'Optocoupler',
		hint: 'Lets one circuit switch another it shares no wire with',
		does: 'Current through A to K lights a hidden LED, and the light lets as much through C to E.',
		watch: 'Only light crosses it, so each side needs a battery of its own to push current.',
	},
	Lm3914: {
		kind: 'Lm3914',
		label: 'Bar driver',
		hint: 'Shows a voltage on ten LEDs',
		does: 'Lights one more LED for each tenth of the way SIG climbs from RLO to RHI.',
		watch: 'MODE on V+ for a bar, free for a dot. R1 on REFOUT sets how bright: about 12.5 V ÷ R1.',
	},
	Gate: {
		kind: 'Gate',
		label: 'Logic gate',
		hint: 'Four gates: NAND, AND, OR or XOR',
		does: 'Each Y follows its A and B: AND both high, OR either, XOR just one, NAND not both.',
		watch: 'An unwired input reads whatever is near it: wire both inputs of every gate you use.',
	},
	Inverter: {
		kind: 'Inverter',
		label: 'Inverter',
		hint: 'Six NOT gates: high in, low out',
		does: 'Each Y is the opposite of its A: high when A is low, low when A is high.',
		watch: 'It runs on 2 to 6 V. Give it 5 V, not the 9 V battery.',
	},
	Schmitt: {
		kind: 'Schmitt',
		label: 'Schmitt inverter',
		hint: 'Six NOT gates that ignore wobbles',
		does: 'Each Y is the opposite of its A. A switches it climbing past 2.8 V, falling below 1.8 V.',
		watch: 'Those are at 5 V. With a resistor and a capacitor, one gate makes a blinker.',
	},
	Decade: {
		kind: 'Decade',
		label: 'Decade counter',
		hint: 'Lights ten LEDs one after another',
		does: 'Each time CLK goes from low to high, the high output moves on one: Q0, Q1… Q9, Q0.',
		watch: 'Wire RST and INH to −, or it will not count. An output wired to RST counts that far.',
	},
	FlipFlop: {
		kind: 'FlipFlop',
		label: 'Flip-flop',
		hint: 'Two memories of one bit each',
		does: 'When CLK goes from low to high, Q takes whatever D is and keeps it until the next time.',
		watch: 'Wire SET and RST to − unless you are using them. /Q to D makes it toggle.',
	},
	Voltmeter: {
		kind: 'Voltmeter',
		label: 'Voltmeter',
		hint: 'Reads the voltage between two points',
		does: 'Shows the voltage between its two pins, without taking any current.',
		watch: 'It goes across a component, side by side with it — never in line.',
	},
	Ammeter: {
		kind: 'Ammeter',
		label: 'Ammeter',
		hint: 'Reads the current running through it',
		does: 'Shows the current that runs through it, without getting in its way.',
		watch: 'It goes in line with a component. Straight across a battery is a short circuit.',
	},
	Ground: {
		kind: 'Ground',
		label: 'Ground',
		hint: 'The point everything else is measured against',
		does: 'Fixes the reference every voltage is measured from.',
		watch: 'This app picks a reference on its own, so a circuit here never needs one.',
	},
}

/** A component a user can place on the canvas. */
export type PaletteEntry = ComponentInfo

/**
 * The palette's shelves, in the order they are shown (C-59).
 *
 * Grouped by what a part is for rather than by how it is modelled, and named
 * in words somebody who has never built a circuit can use: "power", not
 * "sources"; "lights and diodes", because an LED is the diode everybody has
 * already seen. How it is described is the app's business (ADR 0008), so the
 * categories live here; the core's registry is ordered to match them, and a
 * test holds the flat list below to the core's.
 */
export interface PaletteCategory {
	readonly name: string
	/** The name on its chip, where a whole heading does not fit (C-128). */
	readonly short: string
	/**
	 * Which colour it is drawn in: `--family-<this>` in `app.css` (C-126).
	 * A colour per family, so a part's tile and its heading say where it
	 * belongs before either is read.
	 */
	readonly family: Family
	readonly entries: readonly PaletteEntry[]
}

export type Family =
	'power' | 'lights' | 'passive' | 'switches' | 'motion' | 'chips' | 'logic' | 'meters'

export const PALETTE_CATEGORIES: readonly PaletteCategory[] = [
	// Lights before resistors, although a resistor is the simpler part: the
	// first circuit is a battery, a resistor and an LED, and on a laptop screen
	// the palette shows about six entries before it scrolls. In this order all
	// three are among them.
	{
		name: 'Power',
		short: 'Power',
		family: 'power',
		entries: [CATALOGUE.DcSource, CATALOGUE.SignalGenerator, CATALOGUE.Cell, CATALOGUE.SolarCell],
	},
	{
		name: 'Lights and diodes',
		short: 'Lights',
		family: 'lights',
		entries: [
			CATALOGUE.Led,
			CATALOGUE.RgbLed,
			CATALOGUE.SevenSegment,
			CATALOGUE.LcdDigit,
			CATALOGUE.Bargraph,
			CATALOGUE.Diode,
			CATALOGUE.Schottky,
			CATALOGUE.Bridge,
			CATALOGUE.Zener,
		],
	},
	{
		name: 'Resistors and capacitors',
		short: 'R & C',
		family: 'passive',
		entries: [
			CATALOGUE.Resistor,
			CATALOGUE.Potentiometer,
			CATALOGUE.Ldr,
			CATALOGUE.Phototransistor,
			CATALOGUE.Thermistor,
			CATALOGUE.Capacitor,
			CATALOGUE.Electrolytic,
			CATALOGUE.Inductor,
			CATALOGUE.Transformer,
		],
	},
	{
		name: 'Switches',
		short: 'Switches',
		family: 'switches',
		entries: [
			CATALOGUE.Switch,
			CATALOGUE.PushButton,
			CATALOGUE.TiltSwitch,
			CATALOGUE.Reed,
			CATALOGUE.Relay,
			CATALOGUE.Fuse,
		],
	},
	{
		name: 'Sound and motion',
		short: 'Motion',
		family: 'motion',
		entries: [CATALOGUE.Buzzer, CATALOGUE.PassiveBuzzer, CATALOGUE.Motor, CATALOGUE.Servo],
	},
	{
		name: 'Transistors and chips',
		short: 'Chips',
		family: 'chips',
		entries: [
			CATALOGUE.Bjt,
			CATALOGUE.Mosfet,
			CATALOGUE.Regulator,
			CATALOGUE.Adjustable,
			CATALOGUE.OpAmp,
			CATALOGUE.Comparator,
			CATALOGUE.Tmp36,
			CATALOGUE.Hall,
			CATALOGUE.Pir,
			CATALOGUE.Timer,
			CATALOGUE.Decoder,
			CATALOGUE.Counter,
			CATALOGUE.MotorDriver,
			CATALOGUE.ShiftRegister,
			CATALOGUE.Optocoupler,
			CATALOGUE.Lm3914,
		],
	},
	{
		name: 'Logic',
		short: 'Logic',
		family: 'logic',
		entries: [
			CATALOGUE.Gate,
			CATALOGUE.Inverter,
			CATALOGUE.Schmitt,
			CATALOGUE.Decade,
			CATALOGUE.FlipFlop,
		],
	},
	// After the chips it is made of, before the meters that watch it (A-02).
	{ name: 'Arduino', short: 'Arduino', family: 'chips', entries: [CATALOGUE.Uno] },
	{
		name: 'Meters',
		short: 'Meters',
		family: 'meters',
		entries: [CATALOGUE.Voltmeter, CATALOGUE.Ammeter],
	},
]

/**
 * The three parts a first circuit is made of, in the order it is built:
 * what the empty board offers first, numbered (C-128). The same three the
 * empty canvas's sentence has always named.
 */
export const STARTERS: readonly PaletteEntry[] = [
	CATALOGUE.DcSource,
	CATALOGUE.Resistor,
	CATALOGUE.Led,
]

/** Lower case, accents gone, so "diode" finds "Diode" and "resistance" finds nothing it should not. */
function folded(text: string): string {
	return text
		.normalize('NFD')
		.replace(/\p{Diacritic}/gu, '')
		.toLowerCase()
}

/**
 * Whether a part answers to what was typed in the palette's search (C-128):
 * every word, anywhere in its name or its hint, so "light" finds the LED by
 * its hint and "555" finds the timer by its name.
 */
export function answersTo(entry: PaletteEntry, query: string): boolean {
	const haystack = folded(`${entry.label} ${entry.hint}`)
	return folded(query)
		.split(/\s+/)
		.filter((word) => word.length > 0)
		.every((word) => haystack.includes(word))
}

/** The family a part is offered under, for its colour; nothing for one the palette does not offer. */
export function familyOf(kind: string): Family | undefined {
	return PALETTE_CATEGORIES.find((category) =>
		category.entries.some((entry) => entry.kind === kind),
	)?.family
}

/** Every offered component, category after category. */
export const PALETTE: readonly PaletteEntry[] = PALETTE_CATEGORIES.flatMap(
	(category) => category.entries,
)

/**
 * What the app can say about a kind, or nothing if it has never heard of it.
 *
 * Unknown kinds are possible — the core is free to grow an element the app has
 * not caught up with — and saying nothing about one is better than inventing.
 */
export function describe(kind: string): ComponentInfo | undefined {
	return CATALOGUE[kind as ComponentKind]
}
