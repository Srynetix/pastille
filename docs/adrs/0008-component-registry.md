# 8. A component is defined in three places, and each owns a different kind of fact

- **Status:** accepted
- **Date:** 2026-09-19
- **Ticket:** C-16
- **Deciders:** Denis Bourge

## Context and problem statement

A resistor is described in six places. The `ElementKind` enum carries its ohms;
`parse_kind` knows a fresh one is 220 Ω; `pin_names` knows it has `a` and `b`;
`unit_of` knows it is measured in Ω; `symbols.ts` knows how to draw it and names
its pins again; `palette.ts` knows it is called "Resistor" and limits current.
Adding the seventh component meant touching all six, and the pin names appearing
twice is a bug waiting to happen — `Anode` against `anode` connects nothing while
looking perfectly fine on screen.

C-16 asks for one declarative definition per component, "carrying symbol, model
parameters, ratings and defaults", consumed by palette, solver, inspector and
later the linter and the BOM.

The difficulty is the word *symbol*. ADR 0006 already decided that where a wire
is drawn is the app's business, because pin positions come from the symbol and
symbols live in the app. A registry that carried polylines would put the drawing
in the core and reverse that.

## Decision

Three owners, split by what kind of fact each holds.

**The core owns what is true about a component.** `registry.rs` has one entry per
kind: its pin names, the unit of its editable value, a constructor for a fresh
one with its defaults, its ratings, and whether v1 offers it in the palette or
merely knows how to draw it. Everything electrical, and everything a saved file
or an export has to agree about.

**The app owns how it is drawn.** `symbols.ts` keeps the shapes and the pin
positions, as ADR 0006 has it. A symbol is pixels and a right angle; the core has
no business with either.

**The app owns how it is described.** `palette.ts` keeps the label, the one-line
hint and the two sentences of §14 help. That is copy written for a beginner in a
particular language, not a fact about a resistor.

What keeps the three honest is tests rather than types, because they are in two
languages: the app asserts that the kinds it offers are the kinds the core says
are offered, in the same order, that every kind the core can draw has a symbol,
and that every symbol names its pins exactly as the core does. That last test
already existed and already caught this class of bug; the others are the same
idea applied to the rest of the definition.

*Updated 2026-11-07:* the kinds themselves are a type now as well. The core declares
them as `ComponentKind` in its generated TypeScript, held to `REGISTRY` by a test on the
Rust side, and the app's `SymbolKind` is that type rather than its own list — so a kind
the core drops is a compile error in the app, and a misspelt one is too. The tests
above still hold the rest: what is offered, in what order, and the pin names.

## Options considered

### Everything in the core, symbols included — rejected

The literal reading of the ticket, and it would make the registry genuinely
single. Rejected because it reverses ADR 0006 for no gain the ticket actually
needs: the consumers it lists — palette, solver, inspector, linter, BOM — want
pin names, units, defaults and ratings. Not one of them wants a polyline. The
breadboard view of B-14 will want a *different* drawing of the same component,
which is the same argument ADR 0006 made about layout.

### Everything in the app, with the core kept purely electrical — rejected

Symmetrical, and wrong in the same way for the opposite reason. Defaults and
ratings have to survive a save (C-04) and an export (B-11, B-16), which are core
concerns, and a linter rule about exceeding a rating (B-03) runs where the
solver's numbers are.

### Split by the kind of fact, agreement enforced by test — chosen

Each fact lives where it is used and where it has to survive. The one thing that
genuinely must not diverge — the pin names — is already held together by a test
the repo trusts, and the same mechanism extends to the rest.

## Consequences

1. **Adding a component starts in the registry.** The core will not place a kind
   that is not there, so a missing entry is an error rather than a component that
   half exists.

2. **`offered` and `drawable` are two facts, not one.** A ground is drawable and
   not offered, which is exactly the state C-34 left it in: the element remains
   for circuits that contain one, and the palette does not ask the question. A
   capacitor is currently neither, and C-18 and C-28 are what flip it.

   *Updated 2026-11-07:* they did. Every kind is drawable now, and every one but the
   ground is offered.

3. **The agreement tests are load-bearing.** They are the only thing standing
   between the two languages, so weakening one to make a change land is
   reintroducing the bug the registry exists to prevent.

4. **Prose stays out of the core.** When the app is translated, nothing in
   `registry.rs` needs to change — which is the test of whether the line was
   drawn in the right place.
