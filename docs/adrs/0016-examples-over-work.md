# 16. Examples can be opened over work, one Undo away

- **Status:** accepted — reverses the placement C-47 chose
- **Date:** 2026-10-02
- **Ticket:** C-131
- **Deciders:** Denis Bourge

## Context and problem statement

C-47 offered the examples on the empty canvas and nowhere else. Its reason was
written in `app/src/lib/examples.ts`: opening an example threw away what was on the
canvas, history and all, so for a curious click it would be a beginner losing their
first circuit — and an offer that only exists while there is nothing to lose cannot
destroy anything. No dialog, no "are you sure", no undo past an open.

Two things have changed since.

1. **Opening is one step of undo** (C-98). Opening a file, and so an example, over an
   hour of work no longer loses it: one press of Undo brings it back, description and
   all.
2. **The library grew from six examples to 97** (C-58 and every part since), and the
   redesign (design doc §15) made the offer a panel of cards. It has become one of the
   main ways the app teaches what a part does — and it was unreachable from the moment
   somebody placed their first component, which is exactly when "what does a relay
   do?" gets asked.

## Decision

**The header has an Examples button that opens the offer at any time, and an example
opened over work says how to get the work back.**

1. **The button opens the same offer** — the shelves and their cards — in a dialog over
   the board. On an empty board the offer is also on the board itself, as before.
2. **Opening one over work is one step of undo** (C-98's), and the app says so at once:
   "Opened *A lit LED*. Your circuit is one Undo away", with an Undo button beside the
   sentence. A beginner may not know Undo exists, and the sentence is the moment they
   need to.
3. **Nothing is offered unasked.** With work on the board, no example appears unless the
   button is pressed: C-47's protection against a curious click survives where it was
   doing its work.

## Options considered

### Open at any time, one Undo away, said out loud — chosen

Reaches the library when it is wanted, at the cost of one sentence after an open over
work.

### Keep C-47's placement — rejected

Safe, but its reason was C-98's missing undo, and the price is the library being out of
reach for everybody who has started building.

### Ask "are you sure?" before opening over work — rejected

A confirmation is the dialog C-47 was right to avoid: it is read past, and it protects
less than an undo that is offered after the fact.

## Consequences

- `examples.ts`'s comment on placement now points here.
- The autosave (C-29) keeps whatever is on the board, so an example opened over work and
  left there replaces the remembered circuit too; the Undo is the way back, as it is for
  an open file. The notice says so by offering it.
