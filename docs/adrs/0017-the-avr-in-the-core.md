# 17. The AVR runs in the core, ported from avr8js

- **Status:** accepted
- **Date:** 2026-10-04
- **Ticket:** A-01
- **Deciders:** Denis Bourge

## Context and problem statement

Phase 3 puts an Arduino Uno — an ATmega328P — in the circuit (design doc §8).
The design doc names avr8js (Wokwi's, MIT) to emulate it, and fixes the shape
of the coupling: *analog follows, never leads* — the program's pins change, the
circuit is solved up to that moment, and what the circuit does is read back by
the program. What it leaves open is the mechanics, and they decide everything
after: where the emulator runs, who advances time, and how a pin that switches
every millisecond reaches a solver that steps every millisecond.

What the core already is, and what the choice has to fit:

- **Time is the core's** (seam 4, `scheduler.rs`): the app hands over the wall
  time that passed, the core decides what it is worth. A transient run picks
  its own steps inside a frame, and cuts them at a signal generator's edges
  (ADR 0011, `next_breakpoint`) and at a bouncing button's flips (C-109).
- **Every part that has a state carries it the same way**: the 555's latch, a
  counter's count, a relay's coil, a fuse's heat — held by the engine between
  steps, kept through an edit (`adopt`), started afresh when the part changes
  kind (C-137). Undo, saving, the scope, the review (C-104) and the desktop
  build all read the circuit from the core and nothing else.
- **avr8js is JavaScript.** 0.21.1, no dependencies, ~4,600 lines; ~65–82 MHz
  simulated in V8 on this Mac, ~5× real time (the author's measurement, 2026-10-04). It has no HEX
  loader, applies no pull-ups, and runs flat out: pacing it against another
  clock is the embedder's job.
- **Following PWM edge by edge is affordable.** Measured 2026-10-04 (native
  release build, an RC and an LED on a square wave): 490 Hz at 79× real time,
  976 Hz at 58×, 31.25 kHz at 2.8×. The Uno's `analogWrite` is 490 or 976 Hz.

## Decision

**avr8js is ported to Rust and runs inside the core, as a device of the
transient run.** The CPU and its instruction set, the clock-event queue, GPIO,
the three timers, the ADC and the USART first; SPI, TWI, EEPROM and the
watchdog when a ticket needs them. The port keeps avr8js's structure and names
so its fixes can be followed, and its MIT notice (Uri Shaked's, and the
assembler's if it is ported) is carried in each ported file and in the
repository's third-party notice. Its spec tests are ported with it: they are
the evidence that the port is the emulator.

**How the AVR and the solver take turns.** Inside each step the run is about
to take, from *t* to *t + h*:

1. The AVR runs, instruction by instruction, from the cycle it is at towards
   the cycle *t + h* is (16 MHz).
2. If an output pin changes on the way — a `digitalWrite`, a timer's compare
   match — the AVR stops on that instruction, at *t′*. The step is cut there:
   the circuit is solved to *t′* with the pins as they were, then the pins
   take their new levels, and the run goes on from *t′*. Nothing is rolled
   back on either side: the AVR never runs past a moment the circuit has not
   reached.
3. What the AVR reads — a digital input's level, an ADC channel's voltage — is
   the circuit's at the last solved point. Its error is at most one step of
   the run, and an input's hysteresis keeps it from flickering.

**PWM reaches the solver as its edges**, each one a breakpoint as a signal
generator's are, so an RC on a PWM pin shows its ripple and an LED's current
is the real pulses. Where edges come faster than the run can follow in real
time — a prescaler of 1, 31 kHz — averaging the pin is a later ticket's
fallback, not the default.

**A pin is electrical**, as the 74HC and CD4000 parts already are (C-108,
C-124): an output drives high or low behind the ATmega328P's output
resistance, an input is a leak read through the sheet's thresholds with
hysteresis, a pull-up is a resistor to VCC. The emulator never sees a
voltage, only the levels and ADC readings the glue gives it.

**The program is the part's; the run is the engine's.** The flash image is a
property of the Uno in the circuit — saved with it, undone with it, swapped
when another program is loaded. Its registers, SRAM, timers and cycle count
are the run's, carried through an edit as a 555's latch is, and started from
reset when the program changes or the part is placed.

## Options considered

### avr8js in JavaScript, the app driving the loop — rejected

What the design doc named. avr8js would be used unchanged, its peripherals
kept up to date upstream, and the first Blink would come sooner. Rejected
because it moves the clock out of the core: the app would run the AVR, cut
the frame at its edges and call the core between them, so seam 4 would no
longer hold, and an MCU's inputs would be read one slice late by construction.
Every consumer of the circuit — undo, saving, the scope, the review, the
breadboard, the desktop build — would have to learn about a state the core
does not have, and an MCU circuit could only be tested in a browser.

### avr8js in JavaScript, the core calling back into it — rejected

The core keeps the clock and calls a JavaScript function each step to run the
AVR. Rejected as the worst of both: the core depends on the browser, cargo
cannot test it, and the boundary is crossed inside the solver's innermost loop.

### An existing Rust or C emulator — rejected

simavr is GPL-3.0, which ADR 0001 rules out. `avr-emulator` (0.1.2) is a CPU
without the peripherals an Arduino program depends on. Neither has avr8js's
record of running real Arduino builds on Wokwi.

### Averaging PWM (duty × VCC) by default — rejected

What Velxio does, and cheap. Rejected because the ripple, the pulses an LED
really sees and the filter that smooths them are exactly what a beginner
should see, and the measurement says the solver can afford them at the
Arduino's own frequencies.

## Consequences

1. **The port is real work and is ours to maintain.** About 4,600 lines with
   their tests; avr8js's later fixes have to be read and brought across by
   hand. Keeping its structure and names is what keeps that possible.
2. **The design doc's §8 is updated** to say the AVR is a port of avr8js in the
   core, and why, with a pointer here.
3. **The run always has a past when an Uno is in the circuit**, even with
   nothing reactive in it: the AVR is a state that moves with time, as a 555's
   latch is.
4. **A frame's cost now includes the program.** The pace (C-83) measures it
   with the rest, and C-15's gate gets an MCU circuit once there is one.
5. **Compiling a sketch is not decided here** — A-07 to A-09 are.
6. **Revisit** if the port's cost outgrows its benefit, or if a second
   emulator (ESP32, Phase 3 later) needs the external-engine coupling the
   rejected option describes; Renode (Phase 4) will need it regardless, with a
   fixed quantum, and this decision does not stand in its way.
