// SPDX-License-Identifier: MIT OR Apache-2.0

// What a buzzer sounds like.
//
// C-71. The core says which buzzers are sounding (C-66, `energised`), and the
// canvas draws them with waves beside them; this is the same fact, heard. An
// active buzzer carries its own oscillator and has one note, so the tone is
// fixed and the only thing the circuit decides is whether it is on. Several
// buzzers sounding at once are one note, as they are on a bench.
//
// A passive buzzer has no note of its own (C-85): it plays what the circuit
// feeds it, which the core reads off the run, so the voice's pitch is set
// rather than fixed and glides when the circuit changes it. Still one voice.
//
// The pitch is a typical 5 V active buzzer's, 2.3 kHz. The loudness is not:
// a real one is piercing on purpose, and a laptop speaker at that level would
// be the fastest way to have somebody close the tab. A sine, quietly, faded in
// and out over a few milliseconds so that switching it is not a click.
//
// Browsers will not make a sound before the page has been clicked or typed at,
// so the audio is started by the first of either — which is almost always the
// press that closed the buzzer's circuit anyway.

/** A 5 V active buzzer's note, in hertz. */
export const TONE_HZ = 2300

/** How loud, out of 1. Quiet: it is a laptop, not a doorbell. */
const LEVEL = 0.05

/** How long the tone takes to come in or go out, in seconds. */
const FADE = 0.01

/**
 * How long a change of note takes, in seconds: a frame or so, so a knob
 * turned sweeps rather than steps, and the run's reading from one frame to the
 * next, a hertz or two apart, is not heard as a warble.
 */
const GLIDE = 0.02

/** The part of the Web Audio API a tone needs, so a test can hand in its own. */
export type ToneContext = Pick<
	AudioContext,
	'createOscillator' | 'createGain' | 'currentTime' | 'destination' | 'resume' | 'state'
>

/** One buzzer's worth of sound, on or off. */
export class Buzz {
	private context: ToneContext | undefined
	private voice: { oscillator: OscillatorNode; gain: GainNode; hertz: number } | undefined

	/**
	 * `open` makes the audio context the first time a sound is wanted. Lazily,
	 * because a context made at load is one the browser suspends, and one made
	 * for a circuit with no buzzer in it is a context for nothing.
	 */
	constructor(private readonly open: () => ToneContext | undefined = () => new AudioContext()) {}

	/** Whether the tone is on. */
	get sounding(): boolean {
		return this.voice !== undefined
	}

	/** The note it is playing, in hertz, or nothing while it is silent. */
	get hertz(): number | undefined {
		return this.voice?.hertz
	}

	/**
	 * Turns the tone on or off, at `hertz` — an active buzzer's own note unless
	 * a passive one says otherwise (C-85). Asking for what is already so does
	 * nothing; a new note on a voice already playing glides to it.
	 */
	set(on: boolean, hertz = TONE_HZ): void {
		if (on && this.voice && this.context) {
			if (hertz !== this.voice.hertz) {
				this.voice.oscillator.frequency.setTargetAtTime(hertz, this.context.currentTime, GLIDE / 3)
				this.voice.hertz = hertz
			}
			return
		}
		if (on === this.sounding) return
		if (on) this.start(hertz)
		else this.stop()
	}

	/**
	 * Lets a suspended context make sound: called from a click or a key, which
	 * is the only place a browser allows it.
	 */
	wake(): void {
		if (this.context?.state === 'suspended') void this.context.resume().catch(() => {})
	}

	private start(hertz: number): void {
		try {
			this.context ??= this.open()
		} catch {
			// No audio at all — a browser without it, or one that refuses. The
			// buzzer is still drawn sounding, which is most of what it is for.
			return
		}
		const context = this.context
		if (!context) return

		const oscillator = context.createOscillator()
		oscillator.type = 'sine'
		oscillator.frequency.value = hertz
		const gain = context.createGain()
		const now = context.currentTime
		gain.gain.setValueAtTime(0, now)
		gain.gain.linearRampToValueAtTime(LEVEL, now + FADE)
		oscillator.connect(gain).connect(context.destination)
		oscillator.start(now)
		this.voice = { oscillator, gain, hertz }
	}

	private stop(): void {
		const voice = this.voice
		this.voice = undefined
		if (!voice || !this.context) return

		const now = this.context.currentTime
		voice.gain.gain.cancelScheduledValues(now)
		voice.gain.gain.setValueAtTime(voice.gain.gain.value, now)
		voice.gain.gain.linearRampToValueAtTime(0, now + FADE)
		voice.oscillator.stop(now + FADE)
	}
}
