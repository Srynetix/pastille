# Developing Pastille

How Pastille is built, for a developer who knows Rust and TypeScript but not this code. The
[README](../../README.md) says what Pastille does; [the decisions](../adrs/README.md) say why it
was built this way; these documents say how it is built now.

## Where to start

1. [Architecture](architecture.md): the core, the boundary, the app and the shells, what each may
   depend on, and how a frame becomes a moving circuit. Read it first.
2. [Patterns](patterns.md): the patterns the code relies on (one circuit, held by the core; a
   frame that advances then reads back; companions; rules as a list…), and how to add a case to
   each.
3. [Building and releasing](building.md): the development loop, the gate, the desktop and
   web builds, the release.
4. `just doctor`, `just install`, then `just dev`. `just` lists every command.

## Vocabulary

| Term | Meaning | In the code |
|---|---|---|
| Netlist | Elements, their pins, and the nets the pins sit on: what is joined to what, with no coordinates | [`Netlist`](../../crates/core/src/netlist/mod.rs) |
| Element kind | What a part is electrically, with its values: a 220 Ω resistor, a red LED | [`ElementKind`](../../crates/core/src/netlist/mod.rs) |
| Registry | One entry per kind: pin names, unit, defaults, ratings, whether the palette offers it | [`REGISTRY`](../../crates/core/src/registry.rs) |
| Circuit | The netlist, where each part is drawn, the undo history and the running simulation: the only object the app talks to | [`Circuit`](../../crates/core/src/circuit.rs) |
| Document | A circuit as a saved file, versioned by `SCHEMA` | [`Document`](../../crates/core/src/document.rs) |
| Engine | What the circuit asks to simulate: step, read a net's voltage, adopt an edited netlist | [`Engine`](../../crates/core/src/engine/mod.rs), [`MnaEngine`](../../crates/core/src/engine/mna_engine.rs) |
| Scheduler | Turns wall time into fixed-size simulation steps | [`Scheduler`](../../crates/core/src/scheduler.rs) |
| MNA | Modified nodal analysis: the circuit as a matrix `A·x = z` | [`MnaSystem`](../../crates/core/src/solver/mna.rs) |
| Companion | A conductance and a current that stand in for a nonlinear or reactive part at one point | [`Companion`](../../crates/core/src/solver/newton.rs) |
| Finding | What the linter says about a part: severity, a sentence, the parts, an optional fix | [`Finding`](../../crates/core/src/lint/mod.rs) |
| Failure | Why a circuit did not solve, as a code the app puts into words | [`FailureCode`](../../crates/core/src/engine/mod.rs), [`failures.ts`](../../app/src/lib/failures.ts) |
| Projection | The app's read-only copy of what the core says to draw, refreshed every frame | [`Projection`](../../app/src/lib/projection.svelte.ts) |
| Scene | What the canvas draws and hit-tests, as data | [`scene.ts`](../../app/src/lib/scene.ts) |
| Symbol | A part's schematic drawing, as shapes and pins | [`symbols.ts`](../../app/src/lib/symbols.ts) |
| Pace | How much simulated time a frame can afford, from what the last frames cost | [`Pace`](../../app/src/lib/pace.ts) |
| AVR | The Uno's ATmega328P, emulated inside the core | [`avr`](../../crates/core/src/avr/mod.rs) |

## The documents

| Document | Covers |
|---|---|
| [Architecture](architecture.md) | Layers, dependency rule, the boundary, main flows, platform code, departures |
| [Patterns](patterns.md) | The patterns used, where, and how to extend them |
| [Building and releasing](building.md) | Development loop, the gate, packaging, releases |
