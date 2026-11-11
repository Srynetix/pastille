// SPDX-License-Identifier: MIT OR Apache-2.0

// The programs an Uno can be given without a file (A-07): each a real Arduino
// sketch in programs/ at the root, built by its build.sh against Arduino's
// AVR core, bundled as the Intel HEX the core reads. Offered in the Uno's
// panel, and each has an example circuit it goes in (examples.ts).
//
// The HEX files carry Arduino's core, which is LGPL-2.1: programs/NOTICE.md
// says which, and where its source is.

import analogInput from '../../../programs/AnalogInput.hex?raw'
import analogReadSerial from '../../../programs/AnalogReadSerial.hex?raw'
import blink from '../../../programs/Blink.hex?raw'
import button from '../../../programs/Button.hex?raw'
import dimmer from '../../../programs/Dimmer.hex?raw'
import fade from '../../../programs/Fade.hex?raw'
import nightLight from '../../../programs/NightLight.hex?raw'
import serialLed from '../../../programs/SerialLed.hex?raw'
import serialThermometer from '../../../programs/SerialThermometer.hex?raw'
import smoothed from '../../../programs/Smoothed.hex?raw'
import sweep from '../../../programs/Sweep.hex?raw'
import thermometer from '../../../programs/Thermometer.hex?raw'
import trafficLight from '../../../programs/TrafficLight.hex?raw'
import twoPots from '../../../programs/TwoPots.hex?raw'

export interface Program {
	/** Its sketch's name, which is what the Uno says it is running. */
	readonly name: string
	/** What it does, and on which pins, in a line. */
	readonly does: string
	readonly hex: string
}

export const PROGRAMS: readonly Program[] = [
	{ name: 'Blink', does: 'D13 on for a second, off for a second.', hex: blink },
	{ name: 'Button', does: 'D13 lit while a button from D2 to GND is held.', hex: button },
	{ name: 'TrafficLight', does: 'Green on D10, then D11, then red on D12.', hex: trafficLight },
	{ name: 'Fade', does: 'D9 and D10 fade into each other with analogWrite.', hex: fade },
	{ name: 'Smoothed', does: 'D9 on a quarter of the time, for an RC to smooth.', hex: smoothed },
	{ name: 'Sweep', does: 'A servo on D9 swept from 0 to 180° and back.', hex: sweep },
	{ name: 'AnalogInput', does: 'D13 blinks as fast as a pot on A0 says.', hex: analogInput },
	{ name: 'Dimmer', does: 'A pot on A0 sets the brightness of D9.', hex: dimmer },
	{ name: 'NightLight', does: 'D13 lit while an LDR on A0 says it is dark.', hex: nightLight },
	{
		name: 'Thermometer',
		does: 'A TMP36 on A0 lights D2 to D6, one per 5 °C from 20 °C.',
		hex: thermometer,
	},
	{
		name: 'AnalogReadSerial',
		does: 'Prints A0’s reading to the serial monitor.',
		hex: analogReadSerial,
	},
	{
		name: 'SerialLed',
		does: 'Type on or off in the serial monitor to switch D13.',
		hex: serialLed,
	},
	{
		name: 'SerialThermometer',
		does: 'A TMP36 on A0 printed in °C, at 115200 baud.',
		hex: serialThermometer,
	},
	{ name: 'TwoPots', does: 'A0 and A1 printed side by side.', hex: twoPots },
]
