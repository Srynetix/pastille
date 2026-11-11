# 3. A circuit with no ground symbol still simulates

- **Status:** accepted — **overrides the written acceptance of ticket M-05**
- **Date:** 2026-09-17
- **Ticket:** M-05
- **Deciders:** Denis Bourge

## Context and problem statement

Nodal analysis needs exactly one reference node. The question is what to do when
the user has not said which one.

M-05's acceptance, as written on the board, says: *"a netlist with no ground is rejected
with a named error."* That is the correct instinct for a tool aimed at people who know
what a ground is. It is the wrong one here.

The primary audience (design doc §4) is *someone who has never built a circuit*. "Ground"
is not a component they forgot to place; it is a concept they do not have yet. A battery,
a resistor and an LED in a loop is a complete, working, physically real circuit, and it
is the exact circuit of the first milestone (§10). Refusing to simulate it until an extra
symbol appears fails the first thirty seconds.

CircuitJS1 resolved this the same way, and it is one of the reasons it feels friendly:
with no ground element, it takes terminal 0 of the first voltage source as the reference
(see the prior-art study, §1).

## Decision

**Resolve the reference in this order:**

1. An explicitly placed `Ground` element, if it is wired to anything.
2. Otherwise, the negative terminal of the first DC source.
3. Otherwise, refuse — with `CompileError::NoReferenceNode`, or
   `CompileError::GroundNotConnected` when a ground symbol exists but is wired to
   nothing.

**Record how the choice was made.** `NodeMap::ground_origin()` returns `Explicit` or
`ImpliedBySource`, so the rest of the system knows the difference.

## Options considered

### Reject without a ground — rejected

What the board originally said. Honest, unambiguous, and exactly how a professional tool
behaves. Rejected on audience grounds: it turns the first circuit a beginner builds into
an error message about a word they have not learnt. The teaching value is near zero,
because the error cannot explain *why* without explaining nodal analysis.

### Implicit ground, silently — rejected

Falstad's behaviour. Works, but the user never finds out that grounding is a thing, and
when a later circuit needs an explicit reference they have no way to reason about it.

### Implicit ground, recorded — chosen

The circuit runs, and the system knows the reference was inferred. That fact is available
to the linter (C-05, B-05) and to the error surface (C-25), so the app can raise it at
the moment it starts to matter — a second source, a floating subcircuit — instead of at
the moment the user is least equipped to hear it.

## Consequences

1. **M-05's acceptance on the board is now wrong and has been rewritten.** The named
   error still exists, but it fires only when there is genuinely no candidate, which is a
   much rarer and more honest failure.

2. **`ElementKind::Ground` exists** as a one-pin element, so the explicit path is
   available from the start rather than bolted on when someone needs it.

3. **A linter rule is owed.** `GroundOrigin::ImpliedBySource` is a fact nothing consumes
   yet. It should become a quiet annotation once B-01 exists — not a warning, since the
   circuit is correct, but the hook for explaining grounding the first time it bites.

4. **Reversal is cheap and local.** It is the `find_reference_net` fallback arm in
   `netlist/compile.rs` plus this ADR. If the beginner test (M-22) shows that implicit
   grounding confuses more than it helps, flipping it is a few lines.

5. **Nets with no pins are not nodes.** A wire joined to nothing is skipped during
   numbering rather than producing an isolated row in a matrix that cannot be solved.

6. *Updated 2026-09-18 (C-34):* the palette no longer offers a ground. §5 of the design
   doc never listed one — it was added during M-15 on the way past — and this ADR exists
   so that a beginner never has to know what a ground is. Offering one anyway asks the
   question the decision was made to avoid, and the first M-22 tester asked it: *"we don't
   understand what the ground is for"*. The element, its symbol and the explicit-reference
   rule above all remain; nothing in v1 produces one, which is the same discipline C-03
   applies to pin fields. Reversing it is one line in `app/src/lib/palette.ts`.

7. *Updated 2026-09-18 (C-35):* one reference for the whole canvas turned out to be
   wrong. A canvas can hold several circuits, and a reference in one says nothing about
   potentials in another, so the rule above is applied **per circuit** rather than once.
   A part of the canvas with neither a ground nor a source now picks any of its nets as
   its zero, which is the same argument one step further: an isolated passive network has
   no defined potential, so choosing one is honest and refusing to solve the rest of the
   canvas because of it is not.

8. *Updated 2026-11-07:* the names above have moved on with the code. The reference is
   picked by `pick_reference` in `netlist/compile.rs`, and `NodeMap::references()` gives
   one `Reference` per circuit, each with its `GroundOrigin`: `Explicit`,
   `ImpliedBySource` or `Arbitrary`. `GroundNotConnected` is gone — an unwired ground is
   simply not a candidate — and `NoReferenceNode` now means nothing is wired at all. The
   sources that imply a reference are a DC source's `−`, a signal generator's (ADR 0011)
   and an Uno's `GND` (ADR 0017), the board being a supply as a battery is. Consequence 3
   still stands: nothing consumes `GroundOrigin` yet.
