# Patterns

The patterns Pastille's code relies on. Each section says what problem the pattern solves here,
where it lives, and how to add one more case without breaking it. When a new pattern enters the
code, it gets a section in the same commit.

## One circuit, held by the core

**The problem.** A circuit is edited in the app, simulated in the core, saved, undone, exported.
Two copies (one in the app for drawing, one in the core for solving) would disagree, and a
reload would lose whichever was not saved.

**Where.** [`Circuit`](../../crates/core/src/circuit.rs) holds the netlist, where each part is
drawn, where wires are bent, the undo history and the run
([ADR 0006](../adrs/0006-layout-beside-netlist.md)).
[`Projection`](../../app/src/lib/projection.svelte.ts) is the app's read-only view of it.

**How it works.** The app holds a handle to the wasm `Circuit` and asks it what to draw
(`elements()`, `nets()`, `state()`). `Projection.project()` re-reads the structure every frame,
compares it with what it published last, and publishes it only if it changed, bumping `edits`.
Anything that should happen per edit rather than per frame (the autosave, the inspector) reads
`edits`.

**Adding a case.** A new fact about the circuit goes in `Circuit` (or the netlist), is saved
in `Document`, and reaches the app through a view in `views.rs` and a read in `project()`.

**Avoid.** A `$state` in a component that holds part of the circuit; an edit that updates the
app's view before the core has it.

## The core decides, the app words and draws

**The problem.** The facts must be the same in the browser, in the runner and in a saved file;
the words are for a beginner, in one language, and the drawing is pixels.

**Where.** Facts: [`registry.rs`](../../crates/core/src/registry.rs), the solver, the linter,
[`FailureCode`](../../crates/core/src/engine/mod.rs). Look and words:
[`symbols.ts`](../../app/src/lib/symbols.ts), [`palette.ts`](../../app/src/lib/palette.ts),
[`failures.ts`](../../app/src/lib/failures.ts), [`units.ts`](../../app/src/lib/units.ts),
[`voltage.ts`](../../app/src/lib/voltage.ts). [ADR 0008](../adrs/0008-component-registry.md).

**How it works.** The core answers with values and closed codes: a part's pins and ratings, a
`FailureCode`, a `Finding` with its rule name. The app maps each to what is shown. Because the
two live in two languages, tests hold them together: `canvas.test.ts` checks that the palette
offers exactly what the core offers, in the same order, and that every symbol names its pins as
the core does; the kinds are a generated TypeScript type (`ComponentKind`), so a kind the core
drops is a compile error in the app.

**Adding a case.** A new failure: a `FailureCode` variant (the match in `EngineError::code` is
exhaustive on purpose), then its sentence in `failures.ts`. A new number shown: computed in the
core if it is a fact of the circuit, formatted in `units.ts`.

**Avoid.** A sentence for the user built in Rust from a solver error; a physical rule in
TypeScript ("an LED drops 2 V").

## A part in three places

**The problem.** Adding a part touches the solver, the file format, the drawing, the palette and
the breadboard. Without a recipe, one of them is forgotten and the part half works.

**Where.** [`ElementKind`](../../crates/core/src/netlist/mod.rs), [`REGISTRY`](../../crates/core/src/registry.rs),
a model in [`solver/device/`](../../crates/core/src/solver/device/mod.rs), its stamping in
[`mna.rs`](../../crates/core/src/solver/mna.rs); in the app, `symbols.ts` and `palette.ts`; an
example in `app/src/examples/` listed in [`examples.ts`](../../app/src/lib/examples.ts).

**How it works.** The registry is the single list of kinds. The solver indexes a part's pins by
number, and `device/mod.rs`'s `pin_order` test holds each index to the registry's name. The
app's tests hold the palette and symbols to the registry.

**Adding a part.**
1. An `ElementKind` variant with its values; its registry entry (pins, unit, defaults, ratings,
   thermal data, whether it is offered).
2. Its model in `solver/device/`, as a [companion](#companions-the-solver-never-knows-a-part),
   and its stamping in `mna.rs`; its pin indices in the `pin_order` test.
3. Its words (`labels.rs`); how it sits on the breadboard (`breadboard.rs`: `dip`, `ties`,
   `off_board`); its KiCad `mapping` and its `catalogue` part if it is sold; any lint rule it
   needs.
4. Its symbol in `symbols.ts`, its name and help in `palette.ts`.
5. An example circuit, whose `look` line says what to watch, ideally held by a test in
   `examples.test.ts`.

`just ci` fails on most forgotten steps; review catches the rest.

**Avoid.** A part offered with no example; pin names typed twice by hand.

## Companions: the solver never knows a part

**The problem.** Fifty-six parts, some nonlinear (a diode), some reactive (a capacitor), some
with memory (a 555, a counter). A matrix that knew each would grow a branch per part.

**Where.** [`Companion`](../../crates/core/src/solver/newton.rs) and the `Nonlinear` trait
(two terminals), `CoupledCompanion` and `Coupled` (up to six terminals),
`Capacitor::companion` and the other reactive models in
[`solver/device/`](../../crates/core/src/solver/device/mod.rs).

**How it works.** At each Newton iteration, a nonlinear device is asked to linearise itself
about the current guess and returns a conductance and a current; a reactive device does the
same about the previous step, by the integration method of
[`integration.rs`](../../crates/core/src/solver/integration.rs). The matrix stamps both the
same way. A device is a pure function of its arguments: the loop owns its history
(`previous_point`), so a device keeps no state between calls.

**Adding a case.** Implement `Nonlinear` or `Coupled` for the model; test it alone in its file
(its curve, its limits); then stamp it. A chip is modelled by what its datasheet promises, not
transistor by transistor ([ADR 0010](../adrs/0010-behavioural-parts.md)).

**Avoid.** `if kind == …` inside Newton or the LU; a device that mutates itself while
linearising.

## Wall time in, fixed steps out

**The problem.** A simulation advanced inside the render loop runs at whatever rate the browser
gives it, cannot run headless, and freezes the page when a frame costs more than it lasts.

**Where.** [`Scheduler`](../../crates/core/src/scheduler.rs) in the core;
[`Pace`](../../app/src/lib/pace.ts) in the app; the loop in `App.svelte`.

**How it works.** The frame reports the wall time that passed. `Pace` grants only as much as the
last frames could afford. The scheduler cuts what it is given into steps of fixed size, carries
the remainder to the next call, and steps the engine. The step never depends on the caller's
timing, so a run is reproducible, and `pastille-run` and the tests drive the same code with no
frame at all. The circuit's clock (`elapsed`) belongs to the circuit, not to the engine, so an
edit does not rewind it.

**Adding a case.** Something that happens at a time (a button's bounce, a momentary press)
is scheduled in simulated seconds inside `Circuit::advance`, which cuts the frame at it.

**Avoid.** `setTimeout` or `Date.now()` driving anything electrical; a step size derived from
the frame.

## Edit, then rebuild on the next advance

**The problem.** An edit can make the circuit unsolvable for a moment (half a wire is the
normal state of a circuit being built), and what the run holds (a capacitor's charge, the
Uno's memory) must survive edits.

**Where.** `revision` and `simulated` in `Circuit`; `rebuild` in
[`circuit/read.rs`](../../crates/core/src/circuit/read.rs); `Engine::adopt`.

**How it works.** Every edit bumps `revision` and nothing else. The next `advance` sees the
revision has moved, and hands the new netlist to the existing engine (`adopt`) rather than
building a new one. A circuit that failed is not retried until something changes; the failure
is reported in `state()` and the app explains it.

**Adding a case.** A new edit: a `try_*` method that checks, calls `remember()`, changes the
netlist or the layout, and bumps `revision`. Its wasm wrapper calls it. Never solve inside an
edit.

**Avoid.** Rebuilding or solving from the edit itself; throwing away the engine on an edit.

## A gesture is one step of undo

**The problem.** A drag is sixty moves; deleting a selection is several removals. Each as its
own undo step would make Undo useless.

**Where.** `beginGesture` and `endGesture` on `Circuit`, `remember()` in
[`circuit/edit.rs`](../../crates/core/src/circuit/edit.rs), `Snapshot` in `circuit.rs`.

**How it works.** Every mutation calls `remember()`. Outside a gesture it pushes a snapshot of
the netlist, layout, board and wires. Inside one, only the first call does. What the run does
on its own (a fuse that blows, a button that bounces) changes the circuit without a step of
undo.

**Adding a case.** A UI action that makes several calls wraps them in `beginGesture` and
`endGesture`, ending it in a `finally` or on every exit path.

**Avoid.** Undo state in the app.

## Native first, wasm as a thin wrapper

**The problem.** A test in a browser is slow and hard to debug; a core reachable only through
wasm can only be checked by clicking.

**Where.** The `try_*` methods of `Circuit` (`edit.rs`, `read.rs`, `exports.rs`) and their
wrappers in [`circuit/wasm.rs`](../../crates/core/src/circuit/wasm.rs);
[`pastille-run`](../../crates/runner/src/main.rs).

**How it works.** Each wrapper converts arguments and errors (`CircuitError` to `JsError`, views
through `tsify`) and calls a native method. `cargo test` covers the core's paths; the app's
tests, in a real browser, cover the boundary and the shell.

**Adding a case.** Write the `try_*` method and its native test first; then a wrapper of a few
lines, with a `js_name` in camelCase.

**Avoid.** Logic in a wrapper.

## One door to the wasm glue

**The problem.** Imports of the generated glue spread across the app would make the boundary
impossible to change.

**Where.** [`lib/core.ts`](../../app/src/lib/core.ts), checked by `just boundary`
([`check-import-boundary.mjs`](../../scripts/check-import-boundary.mjs)).

**How it works.** `core.ts` instantiates the wasm module once and re-exports what the app
uses. Every other file imports from `./core`. The script resolves every import path, so a
relative import into `lib/wasm/` is caught.

**Adding a case.** A new export of the core is re-exported from `core.ts`.

**Avoid.** Importing `./wasm/pastille_core.js` anywhere else; editing `lib/wasm/`.

## Rules as a list

**The problem.** The linter grows a rule every few parts. Rules that each walk the circuit in
their own way would be slow and hard to order.

**Where.** [`lint/mod.rs`](../../crates/core/src/lint/mod.rs) (the engine, `Context`, `Finding`),
[`lint/rules.rs`](../../crates/core/src/lint/rules.rs) (the rules and `RULES`).

**How it works.** A rule is a function over one element that returns something to say or
nothing. It is either `Check::Wiring` (reads the netlist alone, so it also runs on a circuit
that did not solve) or `Check::Solved` (reads the solution through `Context`). Its place in
`RULES` is the order findings come out in. A finding may carry the one edit that fixes it.

**Adding a rule.** One function, one entry in `RULES` with a stable name, its tests. Its message
fits on one line and says what to do, in a beginner's words.

**Avoid.** A rule that changes the circuit; a rule written in the app.

## Data, not branches, in the app

**The problem.** Fifty-six parts, dozens of shortcuts. A renderer that knew each part, or a key
handler with a branch per key, would grow with every feature and drift from what is announced.

**Where.** [`symbols.ts`](../../app/src/lib/symbols.ts) (a symbol is shapes and pins),
[`render.ts`](../../app/src/lib/render.ts) (draws shapes), [`glyph.ts`](../../app/src/lib/glyph.ts)
(the same shapes as SVG for the palette), [`scene.ts`](../../app/src/lib/scene.ts),
[`keys.ts`](../../app/src/lib/keys.ts) (`GESTURES`, `actionFor`).

**How it works.** The canvas draws a scene and hit-tests it; it does not know what a netlist
is. Key presses are dispatched through `actionFor`, and the shortcuts list is drawn from the
same table, so what is announced is what is bound.

**Adding a case.** A part's look: an entry in `symbols.ts`, no draw call. A shortcut: an entry
in the table.

**Avoid.** `if (kind === 'Resistor')` in `render.ts` or a component; a key handled outside
`actionFor`.

## The AVR's peripherals through a trait, not closures

**The problem.** avr8js, which the AVR is ported from, lets each peripheral claim registers with
closures stored in the CPU. In Rust, closures that capture both the peripheral and the CPU
need `Rc<RefCell<…>>` and a run-time borrow check on every I/O instruction.

**Where.** The `Hooks` trait in [`avr/cpu.rs`](../../crates/core/src/avr/cpu.rs), implemented by
whoever owns the peripherals; [`avr/mod.rs`](../../crates/core/src/avr/mod.rs).
[ADR 0017](../adrs/0017-the-avr-in-the-core.md).

**How it works.** The CPU keeps, per register, which `Hook` claims it (`read_hooks`,
`write_hooks`); on a read, a write or a due
clock event it calls the owner's `Hooks` with itself as an argument. Each ported file keeps
avr8js's shape and names, so a fix upstream can be found here and brought across.

**Adding a case.** A new peripheral: its file in `avr/`, after avr8js's, its registers claimed
in the owner's `Hooks`, its tests in `avr/tests/` (against a real compiled program where it
can be).

**Avoid.** Renaming what avr8js names; `Rc<RefCell<…>>` between the CPU and a peripheral.
