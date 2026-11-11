# 11. A source may change with time, and it is a part of its own

- **Status:** accepted
- **Date:** 2026-09-24
- **Ticket:** L-08
- **Deciders:** Denis Bourge

## Context and problem statement

Every v1 source is DC. The scope (C-22), the companion models (C-06, C-07)
and the adaptive stepper (C-10) were all built for a circuit that changes over
time, and all the circuit could ever do was settle once, be switched, or have
a knob turned. L-08 asked whether v1 needs a source whose voltage changes, and
said what yes would cost: the first element whose value moves between steps,
a stepper that must not stride over an edge, and a property that is neither a
number nor a closed set.

Asked on 2026-09-24, the answer is yes. The comparator (C-56), the capacitor
and the coil have their most instructive behaviour only under a signal, and
"what does alternating current mean" is a question §4's beginner asks.

## Decision

**A signal generator, as its own component**, beside the battery rather than
inside it.

1. **Its own kind, `SignalGenerator`.** A battery stays a battery. Giving the
   DC source a waveform would put a "shape" chooser on the one part every
   beginner places first, for a feature they meet later. A signal generator is
   bench equipment, like the meters: it has no part number and needs none, as
   the voltmeter has none (ADR 0010's term 3 is about integrated circuits).

2. **Three properties, in the shapes that already exist.** The waveform —
   square, sine, triangle — is a choice (C-46). The amplitude is its value, in
   volts, exactly as a battery's is. The frequency is a second number, edited
   the way a battery's internal resistance is (C-17). So the "third shape of
   property" L-08 anticipated is not needed: a waveform is a choice *and* two
   numbers.

3. **It swings symmetrically.** V(+) − V(−) = A·w(2πft), with w between −1 and
   +1. That is what makes it alternating, which is the idea it exists to show;
   a battery in series with it offsets it, and that is a lesson in itself. A
   square wave from 0 to A is exactly that, and an example says so.

4. **Time reaches the stamp.** A step carries the time it ends at, and the
   generator is stamped at that time — the implicit integration rules evaluate
   sources at the end of the step, and doing otherwise lags every waveform by
   a step. The DC operating point is taken at t = 0.

5. **The stepper lands on edges.** A square wave's jumps and a triangle's
   corners are breakpoints: no step crosses one, and a step that reaches one
   ends there. Without this an edge is smeared across whatever step happens to
   straddle it, and the error estimator shrinks the step only after the damage.

6. **Slow by default, and the live loop says so.** 1 Hz, so an LED on it
   blinks and the scope's ten-second window shows ten cycles. The live loop
   runs in real time; a frequency beyond what 32 steps a frame can follow is
   stepped at the shortest step allowed and counted as `forced`, as any fast
   circuit already is.

## Options considered

### A waveform on the DC source — rejected

One component fewer in the palette, and every battery would carry controls
nobody placing their first one needs.

### A new kind of inspector control for a waveform — rejected

A waveform editor (points, a drawn curve) is what L-08 was worried about. Three
fixed shapes with an amplitude and a frequency cover what a beginner's bench
generator does, and every one of those fits a control the inspector has.

### Signals from 0 to A — rejected as the default

The shape a 555 or a microcontroller pin gives, and one an LED blinks on. But
it is DC with ripple, not alternating current, and the part would then teach
the wrong half of the thing it was added for. The 0-to-A square is one battery
in series away, and ships as an example.

## Consequences

1. **`Timestep` carries the time.** The transient runs, the live stepper and
   the fixed-step `transient` all count time from the start of the run, and
   pass it down.
2. **The stepper asks the netlist for its next breakpoint** before each step,
   and clips the step to it.
3. **§5 gains a row**, and L-08 is closed.
4. **A 555 becomes possible later**, since there is now something it would be
   for; it is still its own decision (ADR 0010 left it out for its state).
