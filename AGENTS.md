# Pastille, for agents

Pastille is an electronics simulator for people who have never built a circuit: a Rust core
compiled to WebAssembly, a Svelte app around it, in the browser and in a Tauri desktop shell.
It is part of the [P suite](https://github.com/Srynetix/p-suite): read its `SUITE.md` once;
this file says only what is particular to Pastille.

## Where things are

- `crates/core/` — the simulator: parts, solver, the Arduino's AVR. Compiled to wasm.
- `crates/runner/` — `pastille-run`, a saved circuit run without a browser (`just run --help`).
- `app/` — the Svelte app. `app/src/lib/core.ts` is the only file that may reach the wasm glue
  (`just boundary`); `app/src/examples/` holds the example circuits.
- `desktop/` — the Tauri shell, a Cargo workspace of its own; `just desktop-check` is its gate.
- `programs/` — the Uno's example programs, built from their sketches (`just programs`).
- `docs/dev/` — how the code is built: architecture, patterns, building; read it before changing
  the structure.
- `docs/adrs/` — every technical decision, indexed in its `README.md`.
- `scripts/` — checks, packaging and release; called by the justfile, not by hand.

## Commands

```bash
just doctor   # what this machine is missing
just install
just dev      # the app on http://localhost:5173
just fmt
just check    # format, lints, types: the fast loop
just test     # the core's tests, and the app's in a real browser
just ci       # the gate: run it before saying a change is done
```

`just` lists the rest: packaging (`desktop`, `linux`, `windows`, `pages`), `bench`, `coldload`.

## Done means

- `just ci` passed. If it could not run, say so and why; never call a change done without it.
- The change comes with its tests; a fixed bug with a test that fails without the fix.
- A decision taken on the way is recorded in `docs/adrs/` (copy the latest record's shape, add
  it to the index). A change that goes against a record comes with a new record.
- The code follows [SUITE.md § 12](https://github.com/Srynetix/p-suite/blob/main/SUITE.md#12-code):
  a person could take it over by hand.
- The README and `docs/dev/` say what is now true: a new module, flow or pattern is documented
  in the same commit.
- The commit follows Conventional Commits (`feat(core): …`, `fix(app): …`); its subject is
  written for a user reading the release notes. `just check-commits` checks it.

## Rules

- Read the ADR behind a part of the code before proposing to change it: most of what looks odd
  was chosen.
- **The core decides, the app words and draws.** `crates/core` knows no canvas, DOM or file
  system; a physical rule found in `app` moves to the core, with its test. The app keeps no copy
  of the circuit ([docs/dev/patterns.md](docs/dev/patterns.md)).
- Comments say why, never what or when: no comment that repeats the code, narrates the change
  or speaks to a reviewer; no code commented out; no `TODO` without an issue. Do not add ticket
  ids (`C-24`, `M-21`) to new comments; drop them from a comment you touch when the sentence
  stands without them. An explanation longer than a few lines goes in `docs/dev/`.
- Fix the cause, not the check: never weaken a test, a lint or a gate to make it pass.
- **The benchmark in `just ci` needs mains power.** It measures the solver's cost a frame
  against a baseline recorded on this Mac, and fails both ways. Before believing a red run,
  check `pmset -g batt` and `pmset -g | grep lowpowermode`, and compare against `main`. Never
  move the baseline except from a run on mains power with the machine idle, saying why in the
  commit. When the author says to go on without it: run every other step of `just ci`, and
  say in the commit message that the benchmark was not run, and why.
- **A new part comes with an example** in `app/src/examples/` and its entry in
  `app/src/lib/examples.ts`; more examples is better. Each example's "look" line must be true,
  ideally asserted by a test.
- A new part's electrical facts go in `crates/core`, its symbol and words in `app`
  ([ADR 0008](docs/adrs/0008-component-registry.md)).
- Every source file starts with `SPDX-License-Identifier: MIT OR Apache-2.0` in its first three
  lines (`just spdx`).
- Dependencies only under permissive licences, no copyleft code (ADR 0001); MPL-2.0 is allowed
  in `desktop/` alone (ADR 0013).
- Never edit `app/src/lib/wasm/`: wasm-pack generates it.
- Everything is written in English: code, comments, docs, commits.

## Ask before

- adding, removing or upgrading a dependency;
- changing the circuit file format, or anything a saved circuit depends on;
- running `just release` or `just pages-publish`, pushing a tag, or anything that publishes;
- deleting files you did not create.
