// SPDX-License-Identifier: MIT OR Apache-2.0

// Circuits somebody else built, for the person who has not built one yet.
//
// C-47. Each example is an ordinary saved document — the same format C-04
// writes and C-29 opens, kept as a file rather than built in code, so `just run
// app/src/examples/milestone.json` works on it and a diff of one is readable.
// They are bundled rather than fetched: the app has to work offline (C-27), and
// an example that needs the network is one that is missing when it is wanted.
//
// **Where they are offered is the decision this ticket carried.** §10 says the
// first thirty seconds are somebody *building* a circuit, and an example is a
// circuit somebody else built — the easier experience that could quietly
// replace the thing the product is about. So they are offered on the empty
// canvas and nowhere else: to the person who has nothing yet, beside the
// sentence already telling them how to start, and gone the moment they place a
// component of their own.
//
// That placement settles the other question for free. Opening a circuit throws
// away what is on the canvas, history and all, and for a curious click that
// would be a beginner losing their first circuit — but an offer that only
// exists while the canvas is empty cannot destroy anything. No dialog, no
// "are you sure", no undo past an open: the offer is simply not there once
// there is something to lose.
//
// §14 says M-22 settles placement rather than taste, and M-22 has not run. This
// is the reasoning available now, written down so there is something specific
// for five beginners to disagree with.
//
// **Since ADR 0016 they can also be opened over work**, from the header's
// Examples. C-98 made an open one step of undo, so "no undo past an open" no
// longer holds, and with 97 examples the library is too much of the teaching
// to put out of reach the moment somebody places a part. What survives of the
// reasoning above is the part still doing work: nothing is offered *unasked*
// over somebody's circuit, and an example opened over one says, at once, that
// the circuit is one Undo away.
//
// **There are many, in groups.** Each example is how somebody finds out what a
// part does without having to know to ask, so the library is meant to grow with
// every part and every behaviour the simulator gets (C-58). The offer stays one
// size however long the list becomes: it shows the groups, and one group's
// examples at a time, each still with the line that says what to look at —
// which is the only reason to press it, and has to be read before, not after.
//
// Every line is a claim, and `examples.test.ts` holds each one to the circuit:
// an example whose sentence is false teaches the thing it says.

import type { SymbolKind } from './symbols'
import buffer from '../examples/buffer.json?raw'
import charging from '../examples/charging.json?raw'
import coil from '../examples/coil.json?raw'
import dimmer from '../examples/dimmer.json?raw'
import divider from '../examples/divider.json?raw'
import dropout from '../examples/dropout.json?raw'
import fading from '../examples/fading.json?raw'
import leaking from '../examples/leaking.json?raw'
import heatsink from '../examples/heatsink.json?raw'
import hotfan from '../examples/hotfan.json?raw'
import held from '../examples/held.json?raw'
import hot from '../examples/hot.json?raw'
import loop from '../examples/loop.json?raw'
import milestone from '../examples/milestone.json?raw'
import mixing from '../examples/mixing.json?raw'
import oneway from '../examples/oneway.json?raw'
import parallel from '../examples/parallel.json?raw'
import astable from '../examples/astable.json?raw'
import oneshot from '../examples/oneshot.json?raw'
import organ from '../examples/organ.json?raw'
import siren from '../examples/siren.json?raw'
import clipped from '../examples/clipped.json?raw'
import zener from '../examples/zener.json?raw'
import gate from '../examples/gate.json?raw'
import remembers from '../examples/remembers.json?raw'
import clicking from '../examples/clicking.json?raw'
import glass from '../examples/glass.json?raw'
import hundred from '../examples/hundred.json?raw'
import antiphase from '../examples/antiphase.json?raw'
import rms from '../examples/rms.json?raw'
import kick from '../examples/kick.json?raw'
import caught from '../examples/caught.json?raw'
import relay from '../examples/relay.json?raw'
import seven from '../examples/seven.json?raw'
import decoded from '../examples/decoded.json?raw'
import counting from '../examples/counting.json?raw'
import one from '../examples/one.json?raw'
import nightlight from '../examples/nightlight.json?raw'
import nightphoto from '../examples/nightphoto.json?raw'
import luxmeter from '../examples/luxmeter.json?raw'
import ninemotor from '../examples/ninemotor.json?raw'
import lasting from '../examples/lasting.json?raw'
import fused from '../examples/fused.json?raw'
import resetting from '../examples/resetting.json?raw'
import lightbar from '../examples/lightbar.json?raw'
import gauge from '../examples/gauge.json?raw'
import solarmeter from '../examples/solarmeter.json?raw'
import sunlight from '../examples/sunlight.json?raw'
import thermometer from '../examples/thermometer.json?raw'
import thermometer36 from '../examples/thermometer36.json?raw'
import buzzer from '../examples/buzzer.json?raw'
import fan from '../examples/fan.json?raw'
import blink from '../examples/blink.json?raw'
import lifted from '../examples/lifted.json?raw'
import rectified from '../examples/rectified.json?raw'
import rounded from '../examples/rounded.json?raw'
import turns from '../examples/turns.json?raw'
import regulated from '../examples/regulated.json?raw'
import ringing from '../examples/ringing.json?raw'
import series from '../examples/series.json?raw'
import servo from '../examples/servo.json?raw'
import twoway from '../examples/twoway.json?raw'
import isolated from '../examples/isolated.json?raw'
import byhand from '../examples/byhand.json?raw'
import nand from '../examples/nand.json?raw'
import andGate from '../examples/and.json?raw'
import orGate from '../examples/or.json?raw'
import xorGate from '../examples/xor.json?raw'
import notGate from '../examples/not.json?raw'
import latch from '../examples/latch.json?raw'
import universal from '../examples/universal.json?raw'
import blinker from '../examples/blinker.json?raw'
import bounces from '../examples/bounces.json?raw'
import debounced from '../examples/debounced.json?raw'
import chaser from '../examples/chaser.json?raw'
import countfour from '../examples/countfour.json?raw'
import toggle from '../examples/toggle.json?raw'
import halving from '../examples/halving.json?raw'
import toohot from '../examples/toohot.json?raw'
import darlington from '../examples/darlington.json?raw'
import knob317 from '../examples/knob317.json?raw'
import constant from '../examples/constant.json?raw'
import cheaper from '../examples/cheaper.json?raw'
import dooralarm from '../examples/dooralarm.json?raw'
import hallmagnet from '../examples/hallmagnet.json?raw'
import passing from '../examples/passing.json?raw'
import smoothed from '../examples/smoothed.json?raw'
import oldsupply from '../examples/oldsupply.json?raw'
import sum from '../examples/sum.json?raw'
import switched from '../examples/switched.json?raw'
import threevolts from '../examples/threevolts.json?raw'
import threshold from '../examples/threshold.json?raw'
import tilted from '../examples/tilted.json?raw'
import upright from '../examples/upright.json?raw'
import transistor from '../examples/transistor.json?raw'
import twice from '../examples/twice.json?raw'
import white from '../examples/white.json?raw'
import unoAnalogInput from '../examples/uno-analog-input.json?raw'
import unoAnalogReadSerial from '../examples/uno-analog-read-serial.json?raw'
import unoBlink from '../examples/uno-blink.json?raw'
import unoButton from '../examples/uno-button.json?raw'
import unoDimmer from '../examples/uno-dimmer.json?raw'
import unoFade from '../examples/uno-fade.json?raw'
import unoNightLight from '../examples/uno-night-light.json?raw'
import unoSerialLed from '../examples/uno-serial-led.json?raw'
import unoSerialThermometer from '../examples/uno-serial-thermometer.json?raw'
import unoSmoothed from '../examples/uno-smoothed.json?raw'
import unoTraffic from '../examples/uno-traffic.json?raw'
import unoSweep from '../examples/uno-sweep.json?raw'
import unoThermometer from '../examples/uno-thermometer.json?raw'
import unoTwoPots from '../examples/uno-two-pots.json?raw'

/** A shelf of examples, named as the offer names it. */
export type ExampleGroup =
	| 'First circuits'
	| 'Measuring'
	| 'Charge and time'
	| 'Signals'
	| 'Diodes and transistors'
	| 'Chips'
	| 'Logic'
	| 'Arduino'

/** The groups, in the order a beginner meets what is in them. */
export const EXAMPLE_GROUPS: readonly ExampleGroup[] = [
	'First circuits',
	'Measuring',
	'Charge and time',
	'Signals',
	'Diodes and transistors',
	'Chips',
	'Logic',
	// Last: a program is the step after the parts it drives (A-07).
	'Arduino',
]

export interface Example {
	/** Stable id, for a test to name one by. */
	readonly id: string
	/** Which shelf it is on. */
	readonly group: ExampleGroup
	/** What it is, in the two or three words a button has room for. */
	readonly title: string
	/**
	 * What to look at once it opens.
	 *
	 * A circuit with no reason to open it teaches nothing, and the reason has
	 * to be in front of the person before they click rather than after.
	 */
	readonly look: string
	/** The document, exactly as a saved file. */
	readonly document: string
}

/** Parts nearly every circuit has, which say nothing about what this one is for. */
const EVERYWHERE = new Set(['DcSource', 'Cell', 'Resistor', 'Ground', 'Voltmeter', 'Ammeter'])

/**
 * The part that says what an example is about, for its card's thumbnail
 * (C-131): the first one that is not in every circuit — the LED of "A lit
 * LED", the 555 of a blinker — or, in a circuit made only of those, its
 * first part that is not a source.
 */
export function heroOf(example: Example): SymbolKind {
	// Read out of the file's text, so typed here: a saved kind is spelled as
	// the core spells it, and every example is opened by the core in a test.
	// A kind with no fields is saved as its name, one with fields as an object
	// keyed by it.
	type Saved = { elements?: { kind: SymbolKind | Partial<Record<SymbolKind, unknown>> }[] }
	const { elements = [] } = JSON.parse(example.document) as Saved
	const kinds = elements.map((element) =>
		typeof element.kind === 'string' ? element.kind : (Object.keys(element.kind)[0] as SymbolKind),
	)
	return (
		kinds.find((kind) => !EVERYWHERE.has(kind)) ??
		kinds.find((kind) => kind !== 'DcSource' && kind !== 'Cell' && kind !== 'Ground') ??
		kinds[0] ??
		'Resistor'
	)
}

export const EXAMPLES: readonly Example[] = [
	{
		id: 'milestone',
		group: 'First circuits',
		title: 'A lit LED',
		look: 'The smallest circuit that does something. Change the resistor and watch it dim.',
		document: milestone,
	},
	{
		id: 'switched',
		group: 'First circuits',
		title: 'A switch',
		look: 'Click the switch to break the circuit. Everything stops, everywhere at once.',
		document: switched,
	},
	{
		id: 'held',
		group: 'First circuits',
		title: 'Hold to light',
		look: 'The LED is lit only while you hold the button down. Let go and it goes out at once.',
		document: held,
	},
	{
		id: 'series',
		group: 'First circuits',
		title: 'Two in a row',
		look: 'Two LEDs in one line share one current, so both glow exactly as brightly as each other.',
		document: series,
	},
	{
		id: 'parallel',
		group: 'First circuits',
		title: 'Side by side',
		look: 'Each LED has a path of its own. Delete one and the other stays lit.',
		document: parallel,
	},
	{
		id: 'dimmer',
		group: 'First circuits',
		title: 'A dimmer',
		look: 'Turn the knob and the LED dims and brightens: the knob adds resistance to the loop.',
		document: dimmer,
	},
	{
		id: 'white',
		group: 'First circuits',
		title: 'White from three',
		look: 'Red, green and blue lit together add up to white. The red gets a larger resistor: it needs less voltage.',
		document: white,
	},
	{
		id: 'mixing',
		group: 'First circuits',
		title: 'Mixing a colour',
		look: 'Three knobs, one lamp: turn each to add more or less red, green or blue, and the colour mixes.',
		document: mixing,
	},
	{
		id: 'relay',
		group: 'First circuits',
		title: 'A switch that throws',
		look: 'Hold the button: the relay’s coil pulls its contact over, and the green LED hands over to the red.',
		document: relay,
	},
	{
		id: 'seven',
		group: 'First circuits',
		title: 'A seven',
		look: 'A digit is seven LEDs: feed bars a, b and c, each through its own resistor, and it reads 7.',
		document: seven,
	},
	{
		id: 'one',
		group: 'First circuits',
		title: 'A one, held',
		look: 'Hold the button: bars b and c light, and the digit reads 1 for as long as you hold it.',
		document: one,
	},
	{
		id: 'buzzer',
		group: 'First circuits',
		title: 'A doorbell',
		look: 'Hold the button and the buzzer sounds; the resistor keeps a 5 V buzzer safe on 9 V.',
		document: buzzer,
	},
	{
		id: 'upright',
		group: 'First circuits',
		title: 'Which way up',
		look: 'A tilt switch is shut while it stands up. Click it to tip it over and the LED goes out.',
		document: upright,
	},
	{
		id: 'tilted',
		group: 'First circuits',
		title: 'A tilt alarm',
		look: 'Click the tilt switch to tip it over: the ball rolls off its legs and the buzzer sounds.',
		document: tilted,
	},
	{
		id: 'hot',
		group: 'First circuits',
		title: 'A resistor running hot',
		look: 'Nine volts across 150 Ω is half a watt in a quarter-watt part. Select it and watch it heat.',
		document: hot,
	},
	{
		id: 'fused',
		group: 'First circuits',
		title: 'The fuse that saves it',
		look: 'Shut the switch to short the LED: the fuse blows. Open it again and the LED stays dark until a new fuse.',
		document: fused,
	},
	{
		id: 'resetting',
		group: 'First circuits',
		title: 'A fuse that resets',
		look: 'A polyfuse in its place: the short trips it hot. Open the switch and it cools, and conducts again by itself.',
		document: resetting,
	},
	{
		id: 'thermometer36',
		group: 'Measuring',
		title: 'A thermometer chip',
		look: 'Turn the sensor’s dial: the meter moves 10 mV a degree, 0.75 V at 25 °C and 1 V at 50 °C.',
		document: thermometer36,
	},
	{
		id: 'divider',
		group: 'Measuring',
		title: 'Two meters',
		look: 'One circuit read two ways: volts across the lower resistor, amps through the loop.',
		document: divider,
	},
	{
		id: 'loop',
		group: 'Measuring',
		title: 'None used up',
		look: 'Three meters around one loop all read the same current: none of it is used up on the way.',
		document: loop,
	},
	{
		id: 'sum',
		group: 'Measuring',
		title: 'It adds up',
		look: 'Three meters across three resistors. Add their readings and you get the battery’s 9 V.',
		document: sum,
	},
	{
		id: 'thermometer',
		group: 'Measuring',
		title: 'A thermometer',
		look: 'Warm the thermistor with its slider and watch the meter fall: 4.5 V at 25 °C, less when hot.',
		document: thermometer,
	},
	{
		id: 'nightlight',
		group: 'Measuring',
		title: 'A night light',
		look: 'Slide the light down on the sensor: in the dark the transistor turns the LED on by itself.',
		document: nightlight,
	},
	{
		id: 'dooralarm',
		group: 'Measuring',
		title: 'A door alarm',
		look: 'The magnet is on the door, 5 mm from the reed. Slide it away — open the door — and the buzzer sounds.',
		document: dooralarm,
	},
	{
		id: 'hallmagnet',
		group: 'Measuring',
		title: 'A magnet, felt',
		look: 'Bring the magnet within 10 mm: the LED lights. It stays lit until the magnet is back past 12 mm.',
		document: hallmagnet,
	},
	{
		id: 'passing',
		group: 'Measuring',
		title: 'A light for passers-by',
		look: 'It settles for a minute first. Then hold the sensor — someone walking by — and the light stays on 5 s after.',
		document: passing,
	},
	{
		id: 'nightphoto',
		group: 'Measuring',
		title: 'A night light, faster',
		look: 'The night light with a phototransistor for the LDR: dim the room past about 40 lux and the LED comes on.',
		document: nightphoto,
	},
	{
		id: 'luxmeter',
		group: 'Measuring',
		title: 'Light on a meter',
		look: 'A phototransistor passes 2 µA a lux: over 1 kΩ the meter reads 2 mV a lux, in a straight line.',
		document: luxmeter,
	},
	{
		id: 'solarmeter',
		group: 'Measuring',
		title: 'A solar cell on a meter',
		look: 'In full sun 40 Ω takes 4 V and 100 mA, the panel’s best. Halve the sun: half the current.',
		document: solarmeter,
	},
	{
		id: 'ninemotor',
		group: 'Measuring',
		title: 'A motor on a 9 V',
		look: 'The 9 V sags to 5.6 V under the motor and has under an hour in it; four AAs give 5.7 V for hours.',
		document: ninemotor,
	},
	{
		id: 'lasting',
		group: 'Measuring',
		title: 'How long will it last?',
		look: 'A coin cell lights an LED for a day and a half — its label says so. Turn its charge down to see less.',
		document: lasting,
	},
	{
		id: 'gauge',
		group: 'Measuring',
		title: 'A battery gauge',
		look: 'The knob stands for a battery from flat to 5 V. Each LED is half a volt more: full lights all ten.',
		document: gauge,
	},
	{
		id: 'lightbar',
		group: 'Measuring',
		title: 'A light meter',
		look: 'An LDR’s divider into an LM3914: slide the light up and the bar climbs, one LED for each step.',
		document: lightbar,
	},
	{
		id: 'charging',
		group: 'Charge and time',
		title: 'A capacitor filling',
		look: 'The scope draws it filling up — fast at first, then slower, and never quite full.',
		document: charging,
	},
	{
		id: 'sunlight',
		group: 'Charge and time',
		title: 'Sunlight in a capacitor',
		look: 'A dull day fills 4 700 µF through a diode, as a garden lamp does. At night the diode keeps it full.',
		document: sunlight,
	},
	{
		id: 'fading',
		group: 'Charge and time',
		title: 'A fading glow',
		look: 'No battery: a charged capacitor lights the LED on its own, and the glow fades as it empties.',
		document: fading,
	},
	{
		id: 'leaking',
		group: 'Charge and time',
		title: 'Holding a charge',
		look: 'Hold the button to fill it, then let go: the meter slowly drops as the charge leaks away.',
		document: leaking,
	},
	{
		id: 'coil',
		group: 'Charge and time',
		title: 'A slow coil',
		look: 'Current through a coil cannot jump. The ammeter’s trace climbs, quickly and then slowly.',
		document: coil,
	},
	{
		id: 'ringing',
		group: 'Charge and time',
		title: 'Back and forth',
		look: 'Charge sloshes between a capacitor and a coil: the trace swings up and down, dying away.',
		document: ringing,
	},
	// A source that changes (ADR 0011): what the scope and the LEDs were
	// waiting for. After charge and time, because a signal is what makes a
	// capacitor's slowness something you can see happen over and over.
	{
		id: 'blink',
		group: 'Signals',
		title: 'A blinking LED',
		look: 'A square wave pushes one way, then the other. The LED lights only for the forward half.',
		document: blink,
	},
	{
		id: 'turns',
		group: 'Signals',
		title: 'Taking turns',
		look: 'Two LEDs facing opposite ways on one signal: one lights on each half of the swing.',
		document: turns,
	},
	{
		id: 'lifted',
		group: 'Signals',
		title: 'Lifted by a battery',
		look: 'A battery in series lifts the signal: it swings from 0 to 10 V instead of −5 to +5.',
		document: lifted,
	},
	{
		id: 'rounded',
		group: 'Signals',
		title: 'Rounded edges',
		look: 'A capacitor cannot follow a sudden jump: its trace curves where the signal is square.',
		document: rounded,
	},
	{
		id: 'rectified',
		group: 'Signals',
		title: 'Only the top half',
		look: 'A diode lets through only the top of each wave: the meter never shows the bottom half.',
		document: rectified,
	},
	{
		id: 'smoothed',
		group: 'Signals',
		title: 'A smoothed wave',
		look: 'A big capacitor after the diode fills on each peak and feeds the gaps: the meter barely moves.',
		document: smoothed,
	},
	{
		id: 'oldsupply',
		group: 'Signals',
		title: 'The old power supply',
		look: 'A sine halved by a transformer, a bridge, 1 000 µF with its ripple on the scope, a 7805: a steady 5 V.',
		document: oldsupply,
	},
	{
		id: 'clipped',
		group: 'Signals',
		title: 'Clipped at the top',
		look: 'A ±9 V wave through a Zener: the top is cut off flat at 5.1 V, the bottom at a diode drop.',
		document: clipped,
	},
	// Glass, which only a signal can drive properly (C-76): what an LCD
	// answers is the RMS across a bar, so it is here rather than beside the
	// LED digit it is lettered like.
	{
		id: 'glass',
		group: 'Signals',
		title: 'A watch’s digit',
		look: 'An LCD needs AC: a 32 Hz swing between bars b and c and COM turns them dark — a 1.',
		document: glass,
	},
	{
		id: 'antiphase',
		group: 'Signals',
		title: 'In step, out of step',
		look: 'Bars fed COM’s own signal stay clear; bars fed its opposite turn dark. It reads 7.',
		document: antiphase,
	},
	{
		id: 'rms',
		group: 'Signals',
		title: 'Same height, paler',
		look: 'A sine and a square of the same height: the glass answers the RMS, so the sine’s 1 is paler.',
		document: rms,
	},
	{
		id: 'oneway',
		group: 'Diodes and transistors',
		title: 'One way only',
		look: 'Two diodes, one each way round. Only the LED behind the one facing forwards lights.',
		document: oneway,
	},
	{
		id: 'cheaper',
		group: 'Diodes and transistors',
		title: 'A diode that costs less',
		look: 'Either diode blocks a battery put in backwards. The Schottky, right, leaves the load 8.7 V; the 1N4148 8.2.',
		document: cheaper,
	},
	{
		id: 'zener',
		group: 'Diodes and transistors',
		title: 'A chip-free 5 V',
		look: 'A resistor and a reversed Zener hold the meter just under 5.1 V, with no regulator at all.',
		document: zener,
	},
	{
		id: 'transistor',
		group: 'Diodes and transistors',
		title: 'A small current',
		look: 'A tiny current into the transistor’s base lets a far bigger one through the LED. Flip the switch.',
		document: transistor,
	},
	{
		id: 'gate',
		group: 'Diodes and transistors',
		title: 'Switched by a voltage',
		look: 'A square wave on a MOSFET’s gate blinks the LED, and the meter on the gate reads a few picoamps.',
		document: gate,
	},
	{
		id: 'remembers',
		group: 'Diodes and transistors',
		title: 'A gate that remembers',
		look: 'Press once and let go: the gate keeps its charge, so the LED stays lit for seconds.',
		document: remembers,
	},
	{
		id: 'toohot',
		group: 'Diodes and transistors',
		title: 'Too much for a 2N7000',
		look: 'Switch on: both motors run, but the 2N7000 on the right heads past 150 °C. The IRLZ44N stays cool.',
		document: toohot,
	},
	{
		id: 'darlington',
		group: 'Diodes and transistors',
		title: 'A Darlington on a motor',
		look: 'Hold the button: under a milliamp into the TIP120’s base runs the motor’s half an amp.',
		document: darlington,
	},
	{
		id: 'fan',
		group: 'Diodes and transistors',
		title: 'A fan for the heat',
		look: 'Warm the thermistor past 50 °C: a MOSFET closes a relay, and the relay starts the motor.',
		document: fan,
	},
	{
		id: 'clicking',
		group: 'Diodes and transistors',
		title: 'A relay that clicks',
		look: 'A transistor lets a weak signal drive a relay coil it could never drive on its own.',
		document: clicking,
	},
	{
		id: 'kick',
		group: 'Diodes and transistors',
		title: 'A kick from the coil',
		look: 'Each time the transistor lets go, the trace spikes far past 9 V: the coil’s current has nowhere to go.',
		document: kick,
	},
	{
		id: 'caught',
		group: 'Diodes and transistors',
		title: 'A diode that catches it',
		look: 'The same relay with a diode across its coil: the trace stops at 9 V, where the kick used to be.',
		document: caught,
	},
	// The integrated circuits of ADR 0010, each doing the one thing it is
	// bought for. Last, because everything above is made of parts a beginner
	// meets first — a regulator is where the second circuit starts.
	{
		id: 'regulated',
		group: 'Chips',
		title: 'A steady 5 V',
		look: 'Change the battery and the meter does not move — until the battery is too low to feed it.',
		document: regulated,
	},
	{
		id: 'dropout',
		group: 'Chips',
		title: 'Not enough volts',
		look: 'A 12 V regulator on a 9 V battery gives 7 V: it always needs two volts more than it makes.',
		document: dropout,
	},
	{
		id: 'knob317',
		group: 'Chips',
		title: 'Any voltage from a knob',
		look: 'Turn the knob: an LM317 holds its output anywhere from 1.25 V to about 9 V, whatever the load.',
		document: knob317,
	},
	{
		id: 'constant',
		group: 'Chips',
		title: 'An LED held at 20 mA',
		look: 'Make the battery 12 V: the meter stays at 20 mA. One 62 Ω resistor sets it: 1.25 V ÷ 62 Ω.',
		document: constant,
	},
	{
		id: 'threevolts',
		group: 'Chips',
		title: 'A 3.3 V supply',
		look: 'The same job at 3.3 V, the voltage most modern chips run on — from the same 9 V battery.',
		document: threevolts,
	},
	{
		id: 'heatsink',
		group: 'Chips',
		title: 'A regulator too hot',
		look: 'Dropping 7 V to run a motor, a 7805 with no heatsink heads past 125 °C. Select it and watch.',
		document: heatsink,
	},
	{
		id: 'threshold',
		group: 'Chips',
		title: 'A knob that decides',
		look: 'Turn the knob slowly. The LED does not dim: it snaps on at one exact point.',
		document: threshold,
	},
	{
		id: 'hotfan',
		group: 'Chips',
		title: 'A fan when it’s hot',
		look: 'Warm the sensor past 30 °C: the comparator lets go, the MOSFET pulls the relay in and the fan starts.',
		document: hotfan,
	},
	{
		id: 'decoded',
		group: 'Chips',
		title: 'Switches to a number',
		look: 'Four switches make a number in binary, and a decoder draws it on the digit. Past nine, it goes blank.',
		document: decoded,
	},
	{
		id: 'counting',
		group: 'Chips',
		title: 'A digit that counts',
		look: 'A 555 ticks about one and a half times a second, and a counter adds one to the digit on every tick.',
		document: counting,
	},
	{
		id: 'hundred',
		group: 'Chips',
		title: 'Counting to 99',
		look: 'A second counter, clocked by the first one’s carry, counts the tens: 00 to 99, then round again.',
		document: hundred,
	},
	{
		id: 'astable',
		group: 'Chips',
		title: 'A 555 that blinks',
		look: 'Two resistors and a capacitor set the pace: the LED blinks about one and a half times a second.',
		document: astable,
	},
	{
		id: 'servo',
		group: 'Chips',
		title: 'A servo on a knob',
		look: 'A 555 sends a pulse every 20 ms, 1 to 2 ms long: turn the knob and the horn follows.',
		document: servo,
	},
	{
		id: 'twoway',
		group: 'Chips',
		title: 'A motor either way',
		look: 'Two switches set the motor’s two ends: one up, it turns; the other, it turns back; both alike, it stops.',
		document: twoway,
	},
	{
		id: 'isolated',
		group: 'Chips',
		title: 'Across a gap',
		look: 'Hold the button: the 5 V side lights a hidden LED, and the 9 V side’s LED comes on. No wire joins them.',
		document: isolated,
	},
	{
		id: 'byhand',
		group: 'Chips',
		title: 'Eight LEDs by hand',
		look: 'Left to right: data, shift, latch. Hold data, tap shift for a 1; latch shows all eight. These never bounce.',
		document: byhand,
	},
	{
		id: 'organ',
		group: 'Chips',
		title: 'A one-note organ',
		look: 'A 555 plays an A, 440 Hz, on a passive buzzer. Turn the knob and the note slides down.',
		document: organ,
	},
	{
		id: 'siren',
		group: 'Chips',
		title: 'A 555 siren',
		look: 'A slow 555 nudges a fast one, so the buzzer swaps between two notes, high and low, like a siren.',
		document: siren,
	},
	{
		id: 'oneshot',
		group: 'Chips',
		title: 'A timed light',
		look: 'Press the button once: the LED stays lit for about a second, then goes out on its own.',
		document: oneshot,
	},
	{
		id: 'twice',
		group: 'Chips',
		title: 'Twice as much',
		look: 'Turn the knob: the right meter reads twice the left, until the op-amp runs out of battery.',
		document: twice,
	},
	{
		id: 'buffer',
		group: 'Chips',
		title: 'A divider that holds',
		look: 'Two dividers, the same load on each. Loaded directly it sags; through an op-amp it holds.',
		document: buffer,
	},
	{
		id: 'and',
		group: 'Logic',
		title: 'AND: both at once',
		look: 'The LED lights only while both switches are on — one alone does nothing.',
		document: andGate,
	},
	{
		id: 'or',
		group: 'Logic',
		title: 'OR: either will do',
		look: 'Either switch lights the LED, and so do both. Only both off puts it out.',
		document: orGate,
	},
	{
		id: 'xor',
		group: 'Logic',
		title: 'XOR: one or the other',
		look: 'One switch on lights the LED; turn the second on as well and it goes out.',
		document: xorGate,
	},
	{
		id: 'not',
		group: 'Logic',
		title: 'NOT: the opposite',
		look: 'The LED is lit while the switch is off, and goes out when it is on.',
		document: notGate,
	},
	{
		id: 'nand',
		group: 'Logic',
		title: 'NAND: anything but both',
		look: 'Lit for every pair of switches except both on — an AND with its answer turned round.',
		document: nand,
	},
	{
		id: 'universal',
		group: 'Logic',
		title: 'A NAND makes everything',
		look: 'Only NANDs: red is NOT left, green is AND, blue is OR — on less resistance, as blue needs 3 V.',
		document: universal,
	},
	{
		id: 'latch',
		group: 'Logic',
		title: 'Two gates that remember',
		look: 'Red is lit. Press the left button: green comes on and stays when you let go. The right one swaps back.',
		document: latch,
	},
	{
		id: 'blinker',
		group: 'Logic',
		title: 'A gate that blinks',
		look: 'One Schmitt inverter, a resistor and a capacitor: the LED blinks by itself, about once a second.',
		document: blinker,
	},
	{
		id: 'bounces',
		group: 'Logic',
		title: 'A button that bounces',
		look: 'Press once: the count jumps by three, and by two more as you let go. That is the button bouncing.',
		document: bounces,
	},
	{
		id: 'debounced',
		group: 'Logic',
		title: 'A button that does not',
		look: 'The same bouncy button, through a capacitor and a Schmitt inverter: each press counts one.',
		document: debounced,
	},
	{
		id: 'chaser',
		group: 'Logic',
		title: 'A running light',
		look: 'A 555 clocks a decade counter: ten LEDs light one after another, top to bottom, and round.',
		document: chaser,
	},
	{
		id: 'countfour',
		group: 'Logic',
		title: 'Counting to four',
		look: 'Q4 is wired to reset, so the moment it would light the count starts again: four LEDs, never five.',
		document: countfour,
	},
	{
		id: 'toggle',
		group: 'Logic',
		title: 'One button, on and off',
		look: 'Press once and the LED stays on; press again and it goes off. A flip-flop remembers which.',
		document: toggle,
	},
	{
		id: 'halving',
		group: 'Logic',
		title: 'Halve a frequency',
		look: 'Red blinks with the 555; green, on one flip-flop, half as often; blue, on the next, half again.',
		document: halving,
	},
	// A-07. Each with its program in it, built from programs/ at the root.
	{
		id: 'uno-blink',
		group: 'Arduino',
		title: 'Blink',
		look: 'A real Arduino program: D13 high for a second, low for a second, and the LED with it.',
		document: unoBlink,
	},
	{
		id: 'uno-button',
		group: 'Arduino',
		title: 'A button on D2',
		look: 'Hold the button: D2 reads low through its own pull-up, and the program lights D13.',
		document: unoButton,
	},
	{
		id: 'uno-traffic',
		group: 'Arduino',
		title: 'Traffic light',
		look: 'Green for two seconds, the middle light for half a second, red for two: three pins and delay().',
		document: unoTraffic,
	},
	{
		id: 'uno-fade',
		group: 'Arduino',
		title: 'Fade',
		look: 'analogWrite switches D9 and D10 on and off 490 times a second; one LED fades into the other.',
		document: unoFade,
	},
	{
		id: 'uno-smoothed',
		group: 'Arduino',
		title: 'PWM, smoothed',
		look: 'D9 is on a quarter of the time; 10 kΩ and 1 µF smooth it to about 1.25 V. The scope shows the ripple.',
		document: unoSmoothed,
	},
	{
		id: 'uno-sweep',
		group: 'Arduino',
		title: 'A servo sweeping',
		look: 'The Servo library sends D9 a pulse every 20 ms; its length is the angle, and the horn sweeps to 180°.',
		document: unoSweep,
	},
	// A-04: analogRead.
	{
		id: 'uno-analog-input',
		group: 'Arduino',
		title: 'A pot sets the blink',
		look: 'analogRead turns the wiper’s 0–5 V into 0–1023; D13 waits that many milliseconds. Turn it.',
		document: unoAnalogInput,
	},
	{
		id: 'uno-dimmer',
		group: 'Arduino',
		title: 'Dimmer',
		look: 'The pot on A0 read as 0–1023, divided by four, is the LED’s analogWrite on D9.',
		document: unoDimmer,
	},
	{
		id: 'uno-night-light',
		group: 'Arduino',
		title: 'Night light',
		look: 'Turn the light down: the LDR on A0 reads past 600 and D13 lights; back up past 500 and it goes out.',
		document: unoNightLight,
	},
	{
		id: 'uno-thermometer',
		group: 'Arduino',
		title: 'Thermometer',
		look: 'A TMP36 read against the chip’s 1.1 V, four times finer than 5 V: an LED for every 5 °C from 20.',
		document: unoThermometer,
	},
	// A-06: Serial, and the monitor under the board.
	{
		id: 'uno-analog-read-serial',
		group: 'Arduino',
		title: 'AnalogReadSerial',
		look: 'A0’s reading in the serial monitor under the board, ten times a second. Turn the pot.',
		document: unoAnalogReadSerial,
	},
	{
		id: 'uno-serial-led',
		group: 'Arduino',
		title: 'D13 by serial',
		look: 'Type on or off in the serial monitor and press Send: the program switches D13 and answers.',
		document: unoSerialLed,
	},
	{
		id: 'uno-serial-thermometer',
		group: 'Arduino',
		title: 'Serial at 115200',
		look: 'The temperature at 115200 baud; the monitor starts at 9600, so it reads garbage until set to match.',
		document: unoSerialThermometer,
	},
	{
		id: 'uno-two-pots',
		group: 'Arduino',
		title: 'Two pots',
		look: 'A0 and A1 printed side by side: one converter, switched from one pin to the other between readings.',
		document: unoTwoPots,
	},
]
