# 6. Layout lives beside the netlist, in the core, not inside either

- **Status:** accepted
- **Date:** 2026-09-18
- **Ticket:** M-16
- **Deciders:** Denis Bourge

## Context and problem statement

Dropping a component on the canvas produces two facts: *a resistor now exists*, which is
electrical, and *it is at (40, −20)*, which is not. The design doc is emphatic that the
netlist is decoupled from the canvas (§7.1.1) — a netlist carrying pixel coordinates is
the coupling it warns about. But the position cannot live in the app either: C-04 has to
save and reload a circuit, and a reload that restores the connections while scattering
the drawing is not a reload.

There is also a second geometry coming. B-14 puts the same netlist on a breadboard, where
the same resistor is in a different place. Whatever holds schematic positions has to
tolerate a sibling that holds breadboard ones.

Fritzing answered this by giving each of its three views its own geometry *and its own
connections*, which is the thing the prior-art study identifies as the structural mistake to
avoid: three graphs to keep in agreement instead of one.

## Decision

`Circuit` owns a `Netlist` and, separately, a layout mapping element ids to positions.
Neither knows about the other; the element id is the only thing they share.

The app holds a `Circuit` handle and asks it what to draw. It keeps no copy of the
circuit — `elements()` is always what the core last said.

## Options considered

### Positions as fields on the element — rejected

Simplest to write and the easiest thing to reach for. Rejected because every consumer of
the netlist — solver, linter, BOM, KiCad export — would then carry coordinates it has no
use for, and the breadboard view would need a second set of the same fields with a
different meaning. It is the Fritzing shape arrived at one convenience at a time.

### Positions in the app — rejected

Keeps the core purely electrical, which is appealing. Rejected on save/load: the drawing
would have to be serialized separately from the circuit and rejoined by id on load, and
any divergence between the two files is a corrupted drawing. It also puts state the core
owns (which elements exist) in two places.

### A layout beside the netlist, both in the core — chosen

The netlist stays electrical. The layout is a separate map, so the breadboard's layout in
B-12 is another map rather than a second meaning for the same fields. One save file
covers both because one type owns both.

## Consequences

1. **The app is a projection and nothing else.** `toScene` turns `elements()` into
   drawables in one direction, and there is no path back. A component's position changes
   by asking the core to change it, not by the canvas remembering something different.

2. **C-04 serializes `Circuit`, not `Netlist`.** Otherwise a saved circuit reloads with
   every component at the origin.

3. **Removing an element must forget its position.** Otherwise the layout accumulates
   entries for elements that no longer exist, and a save file grows debris. There is a
   test.

4. **Ids cross the wasm boundary as strings.** A slotmap key is 64 bits and a JavaScript
   number is not; a handle silently losing its generation bits would reintroduce exactly
   the stale-id bug the generational key exists to prevent.
