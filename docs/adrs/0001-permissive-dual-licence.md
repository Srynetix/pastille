# 1. Dual MIT / Apache-2.0 licence, no vendored copyleft

- **Status:** accepted — the desktop shell takes MPL-2.0 dependencies under a
  narrow exception, recorded in ADR 0013; consequences 3 and 4 are now checked
  by `just ci` (`deny`, `npm-licences`, `spdx`), since C-106
- **Date:** 2026-09-17
- **Ticket:** C-01
- **Deciders:** Denis Bourge

## Context and problem statement

The licence has to be fixed before any code lands, because reversing it later means
re-obtaining agreement from every contributor. Two things constrain the choice.

**Downstream dependencies must stay reachable.** The extension roadmap names three
dependencies, and their licences were verified rather than assumed:

| Dependency | Licence | Phase |
|---|---|---|
| `wokwi/avr8js` | MIT (Uri Shaked) | 3 — MCU |
| `renode/renode` | MIT (Antmicro) | 4 — co-simulation |
| `ngspice` | Modified BSD **overall**, with exceptions (below) | 5 — accurate mode |

ngspice is not uniformly BSD. Per its `COPYING`:

| Path | Licence |
|---|---|
| `src/maths/KLU` | LGPLv2 |
| `src/frontend/numparam` | LGPLv2+ |
| `src/tclspice.c` | LGPLv2 |
| `src/osdi` | MPL-2.0 |
| `src/xspice/icm/table` | **GPLv2+** |
| `src/xspice`, `src/spicelib/devices/ndev` | Public domain |
| manual | CC-BY-SA-4.0 |

**Prior art was read, and some of it is copyleft.** M-00 involved reading
`pfalstad/circuitjs1` (GPL-2.0) and `fritzing/fritzing-app` (GPL-3) at source level, for the author's notes on prior art.

## Decision

**Dual-licensed MIT OR Apache-2.0**, at the user's option. `LICENSE-MIT` and
`LICENSE-APACHE` at the repository root.

No copyleft code is vendored, linked, or copied into this repository.

## Options considered

### MIT OR Apache-2.0 dual — chosen

The Rust ecosystem default, so crate consumers meet no surprise. Apache-2.0 carries an
explicit patent grant and a contribution clause that MIT lacks; MIT stays available for
anyone whose tooling or policy dislikes Apache-2.0's extra terms. Keeping both means the
project can be consumed by MIT-only and Apache-only downstreams alike, and upstream
contributions to Renode and avr8js (both MIT) stay frictionless in both directions.

Cost: two files instead of one, and a `license = "MIT OR Apache-2.0"` field to keep
consistent across the Rust crates and the npm package.

### MPL-2.0 — rejected

File-level copyleft would keep improvements to the solver itself public, which is
genuinely attractive for a project whose core value is the solver. Rejected because it
adds a compliance question for every downstream that embeds the wasm bundle, and because
the audience (§4 of the design doc) is beginners and hobbyists, not vendors who would
fork the solver and hide it. The defensive value does not pay for the adoption friction.

### GPL-3 — rejected

Would match CircuitJS1 and Fritzing, and would be the natural choice for a project that
sits in their lineage. Rejected because it forecloses exactly the paths §8 depends on:
GPL code cannot be contributed upstream into MIT-licensed Renode or avr8js, and any
downstream wanting to embed the core faces the whole-work obligation. The design doc
already recorded this in §9; verification of the three dependency licences confirms it.

## Consequences

1. **Clean-room discipline is now a rule, not an intention.** Concepts, algorithms and
   published techniques may be reused: MNA, companion models, SPICE's `pnjlim` junction
   limiting, gmin stepping. These come from public literature — SPICE2/SPICE3 (Berkeley),
   Vlach & Singhal, Najm — and that is where they get reimplemented from. No code
   fragment, no structural transcription, no comment carried over from circuitjs1 or
   Fritzing. The prior-art study describes concepts only and contains no code.

2. **ngspice integration (L-04, phase 5) is not a licence-free action.** Before shipping
   an ngspice-backed desktop build: exclude `src/xspice/icm/table` (GPLv2+), and treat
   KLU and numparam (LGPL) as dynamic-link-with-relink-rights, not static inclusion.
   Re-open this ADR at that point rather than assuming "ngspice is BSD".

3. **Dependency licences need enforcing, not trusting.** C-02 should add `cargo-deny`
   with an allow-list (MIT, Apache-2.0, BSD-2/3, ISC, Unicode-3.0, Zlib) and a matching
   check on the npm side, failing CI on anything else. A GPL crate arriving three levels
   deep in a transitive tree is the realistic failure mode, not a deliberate copy.

4. **Every source file gets an SPDX header**: `// SPDX-License-Identifier: MIT OR Apache-2.0`.
   Cheap now, tedious to retrofit.

5. **Contributions.** No CLA. Apache-2.0 §5 already states that a contribution is
   submitted under the licence of the work, which is sufficient for a project this size.
   Revisit only if a corporate contributor asks for one.

## Open point

The repository's git identity was at first a work address. This is a personal project;
committing it under a work identity invites an ownership question later that is much
easier to avoid now than to argue afterwards. Recommend setting a personal `user.email`
on this repository before the first commit. Copyright is attributed to "Denis Bourge" in
both licence files on the assumption that this is a personal work — say so if that
assumption is wrong.

*Updated 2026-11-07:* resolved. Every commit in the history is under a personal address.
