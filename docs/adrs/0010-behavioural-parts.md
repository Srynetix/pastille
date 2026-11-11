# 10. An integrated circuit is modelled by what its datasheet promises

- **Status:** accepted — its "left out" list is overtaken: the 555 by ADR 0012;
  and nine chips have since been modelled under this ADR's own terms, in
  the analog solver with their state carried between steps rather than as
  logic gates — the CD4511 (C-77), CD4026 (C-78), L293D (C-91), 74HC595
  (C-93), CD4017 (C-110), CD4013 (C-111), LM3914 (C-119), and the
  74HC00/08/32/86/04 gates (C-108) and 74HC14 (C-109), the same shape with
  the outputs as what is carried. The `logic` pin discipline is still Phase 3's.
- **Date:** 2026-09-24
- **Ticket:** C-52
- **Deciders:** Denis Bourge

## Context and problem statement

Every v1 component is modelled from its physics. A junction is Shockley, a
transistor is Ebers-Moll, a capacitor is a companion model of its own equation.
Nothing in the solver is there because a datasheet said so.

The next parts worth having are not like that. A linear regulator is the part
B-06's rule was written about and the component set never had; an op-amp and a
comparator are what a hobbyist reaches for the moment a circuit has to *decide*
something. Each is a few dozen transistors on one die, and modelling them that
way is the "transistor-level modelling" §3 defers to Phase 5 — slow, hard to
converge, and more fidelity than the rest of v1 claims.

So the question is whether the solver may hold a second kind of truth: a model
built from what a part is sold as doing rather than from what it is made of,
and if so, on what terms.

## Decision

**Yes, for integrated circuits, on four terms.**

1. **A behavioural model is for a part whose inside is a die.** Discrete parts
   keep physical models. A regulator, an op-amp, a comparator are bought and
   used as blocks with a datasheet, and the beginner of §4 never sees their
   insides; a transistor is the thing inside, and stays Ebers-Moll.

2. **Each model writes down what it honours and what it ignores.** In the
   module's own doc comment, figure by figure, so that leaving something out is
   a sentence rather than an accident. The table below is the first cut. What a
   model ignores it ignores silently on screen: §3 and C-26 put the accuracy
   claim in one sentence, `ACCURACY` in `lib.rs`, and that sentence already
   covers these parts. A second caveat beside every op-amp is exactly the
   repetition C-26 refused.

3. **Each part is one real part a person can buy.** A default part number, a
   package and ratings taken from that part's datasheet. The bridge of §6 needs
   it: the BOM (B-11) has to list something orderable and the KiCad export
   (B-17) has to name a symbol and a footprint. This is why user-drawn
   subcircuits are not this decision (L-09).

4. **They are all built on one primitive.** A voltage-controlled source whose
   output saturates smoothly between two node voltages (C-53). Written once,
   tested against closed-form circuits, and shared; not a stamp per part, each
   inventing its own way of meeting a rail.

### What each model honours

| Part (default) | Honours | Ignores |
|---|---|---|
| Linear regulator (7805, TO-220) | output voltage; dropout, below which it follows the input; quiescent current in the ground pin; short-circuit current limit | line and load regulation; ripple rejection; thermal shutdown; start-up |
| Op-amp (LM358-like) | open-loop gain; output limited short of each rail by the part's headroom; input and output resistance; output current limit | bandwidth; slew rate; offset voltage; input bias current; common-mode range |
| Comparator (LM393-like) | threshold at the inputs' crossing; open-collector output that sinks and never sources; saturation voltage | propagation delay; hysteresis (the part has none; a circuit adds it) |

The regulator's current limit is honoured because a shorted output is a
mistake a beginner will make, and the real part survives it: an ideal clamp
would instead ask the solver for an unbounded current, which is C-17's problem
wearing a new component.

### What is left out, and why

- **The 555.** It has internal state — a flip-flop — which is a new kind of
  thing for an element to carry, and it only earns its place if L-08 says yes
  to a signal that changes. Revisit after L-08.
- **Logic gates.** A gate is where the `logic` pin discipline of §7.1.3 begins,
  and that seam is Phase 3's to open. The comparator's pins stay
  `Electrical`: its output is an open collector in the analog solver, not a
  logic level.
- **Subcircuits somebody draws.** No part number, no footprint (L-09).
- **Anything with firmware.** L-01.

## Options considered

### No integrated circuits in v1 — rejected

Keeps §5 exactly as short as it is. It also leaves B-06 as a rule with nothing
to fire on, and leaves out the two parts most starter kits include beside the
LEDs and resistors v1 already has. The audience is §4's hobbyist as well as the
beginner, and a regulator is where "why is my Arduino's 5 V pin hot" starts.

### Transistor-level models — rejected

The honest model, and the one §3 defers on purpose. An LM358 stage is on the
order of a dozen transistors, each a Newton-linearised device: the C-15 frame
budget would not hold it, and convergence is §11's first risk. It is also
precisely what ngspice as accurate mode (L-04) is for, behind the solver seam
that already exists.

### SPICE macromodels (Boyle and its descendants) — rejected for now

A middle road: a few controlled sources, diodes and a pole or two, fitted to
reproduce gain, bandwidth and slew. More fidelity than a beginner can see on a
DC and transient canvas, and each extra pole is a timestep constraint. If a
model outgrows the table above, this is where it grows to — but by a later
decision, not by drift.

### Datasheet behaviour on one saturating primitive — chosen

As little as reproduces what the part is sold as doing, in the words a
datasheet uses, built on one tested piece of solver.

## Consequences

1. **`Rating` grows a thermal figure.** Today it holds current and power. The
   regulator needs the package's thermal resistance and a maximum junction
   temperature, or B-06 has nothing to divide by. C-54 adds them; every other
   part leaves them `None`.

2. **§5 of the design doc gains three rows and a sentence**, saying the parts
   are behavioural and pointing here. The palette grows by three, which §5 kept
   short on purpose; that is the cost this record accepts.

3. **A bug in C-53 is a bug in three parts.** That is the point of sharing it,
   and why C-53 carries the tests and the C-15 gate rather than the parts
   built on it.

4. **The supply pins are drawn.** An op-amp or a comparator shows its V+ and
   V− like any other pin. Leaving them off is the textbook convention and is
   how a circuit works on screen and not on a breadboard (§6), which B-20 then
   checks for.

5. **Fidelity has a ceiling, and it is written down.** A circuit that depends
   on something in the "ignores" column — an oscillator made of an op-amp's
   slew rate, say — will simulate wrongly, and the table above is the place to
   find out why.
