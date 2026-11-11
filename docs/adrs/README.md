# Decisions

Every technical decision Pastille has taken so far, one record each, numbered in the order
they were taken. Each says what was decided, what else was considered and why it lost, and
what follows from it. Read the one behind a part of the code before proposing to change it:
most of what looks odd here was chosen, and the record says against what.

A record is not rewritten when things move on. A later one revises it and says so in its
status, or the record gains a dated *Updated* note. The status line is where to look first.

**Ticket ids** such as C-27, M-22 or A-07 refer to the author's own board, which is not
public, and neither is **the design doc** many records and comments cite by section (§4,
§6.3…), the author's own design document. Both are kept as the trace of where a decision
came from; every record is meant to read without them.

## The simulator

| # | Decided | In force |
|---|---|---|
| [4](0004-own-lu-solver.md) | The linear solver is our own dense LU, about ninety lines, rather than a linear algebra crate. | Yes |
| [3](0003-implicit-ground.md) | A circuit with no ground symbol still simulates: the first battery's − is the reference. | Yes |
| [10](0010-behavioural-parts.md) | A chip is modelled by what its datasheet promises, not transistor by transistor. | Yes; its list of chips left out is overtaken |
| [11](0011-signal-generator.md) | A source that changes with time is a part of its own, the signal generator, beside the battery. | Yes |
| [12](0012-the-555.md) | The 555 carries one bit of state between steps and shows six of its eight pins. | Yes |
| [17](0017-the-avr-in-the-core.md) | The Arduino's AVR is avr8js ported to Rust, running inside the core. | Yes |

## The core and the app

| # | Decided | In force |
|---|---|---|
| [2](0002-wasm-boundary-serialization.md) | The wasm boundary is typed structs for commands and a zero-copy view for the solver's output. | Yes, with a dated note on the save format |
| [6](0006-layout-beside-netlist.md) | Where parts are drawn lives beside the netlist, in the core; the app keeps no copy of the circuit. | Yes |
| [7](0007-wire-waypoints.md) | A wire's route is derived from its ends, except where the user has bent it. | Yes |
| [8](0008-component-registry.md) | A component is defined in three places: its electrical facts in the core, its look and its words in the app. | Yes |
| [16](0016-examples-over-work.md) | An example can be opened over work at any time, one Undo away. | Yes |

## Where it runs

| # | Decided | In force |
|---|---|---|
| [9](0009-own-offline-shell.md) | The service worker that makes the app work offline is ours, about sixty lines, not a PWA plugin. | Yes |
| [13](0013-desktop-shell.md) | A Tauri shell shows the same build as a desktop app, and only it may use MPL-2.0 dependencies. | Yes; where it is built, revised by 22 |
| [21](0021-github-pages-from-a-local-build.md) | GitHub Pages serves a build made and checked on the author's machine, under `/pastille/`. | Yes |
| [22](0022-windows-and-linux-from-the-mac.md) | The Windows and Linux apps are built on the Mac: cargo-xwin for Windows, Docker for Linux. | Yes |

## The project

| # | Decided | In force |
|---|---|---|
| [1](0001-permissive-dual-licence.md) | Dual-licensed MIT OR Apache-2.0, and no copyleft code in the repository. | Yes, with an exception for the desktop shell (13) |
| [5](0005-no-hosted-ci.md) | The checks live in a Justfile, and `just ci` runs them all, fastest failure first. | Yes; its reason for no hosted CI is revisited by 14 |
| [14](0014-the-gate-stays-local.md) | The gate stays on the author's machine even with a remote: the remote keeps the work, it does not check it. | Yes |
| [15](0015-bundled-typefaces.md) | Two typefaces ship with the app, under the SIL Open Font License. | Yes |
| [18](0018-prettier-for-the-app.md) | Prettier formats the app, and the gate checks it. | Yes |
| [19](0019-eslint-for-the-app.md) | ESLint lints the app with type information, every finding an error. | Yes |
| [20](0020-the-app-is-pastille.md) | The app is called Pastille everywhere, with nothing migrated from its old names. | Yes |
| [23](0023-in-line-with-the-p-suite.md) | Pastille follows the P suite's conventions: commits checked by the gate, every release file signed with minisign, the contract for agents in `AGENTS.md`. | Yes |

## Writing one

Copy the shape of the latest record: a title that states the decision, the status, date,
ticket and deciders, then *Context and problem statement*, *Decision*, *Options
considered* and *Consequences*. Take the next number, and add a line here.
