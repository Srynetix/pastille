# 2. Split the wasm boundary by traffic rate, not by type

- **Status:** accepted
- **Date:** 2026-09-17
- **Ticket:** M-02
- **Deciders:** Denis Bourge

## Context and problem statement

The netlist lives in Rust and is the single source of truth (design doc 7.1.1). The app
is a projection of it. So everything the UI knows has to cross wasm-bindgen, and the
shape of that crossing is fixed early: M-03 designs its types against it, and M-19 runs
the solver continuously with no Run button.

Two kinds of traffic cross, and they have nothing in common:

| | Control plane | Data plane |
|---|---|---|
| Examples | add element, connect pin, edit a resistor value, load a file | node voltages, branch currents, LED brightness |
| Rate | a few per second, driven by a human | every frame, driven by the solver |
| Shape | heterogeneous, nested, named | a flat block of `f64` |
| What matters | ergonomics, type safety, readable errors | no allocation, no copy, no garbage |

A single mechanism serving both is either clumsy for edits or wasteful per frame.

## Decision

**Two mechanisms, chosen per plane.**

**Control plane — `serde-wasm-bindgen` + `tsify`.** Rust structs derive `Tsify` with
`into_wasm_abi`/`from_wasm_abi` and `#[serde(rename_all = "camelCase")]`. wasm-pack emits
real TypeScript interfaces, so the boundary is typed on both sides from one definition
and the doc comments carry across.

**Data plane — an opaque handle plus a zero-copy view.** Solver output stays in wasm
memory behind a `#[wasm_bindgen]` handle exposing `ptr` and `len`; JS reads it as a
`Float64Array` over `memory.buffer`. Nothing is serialized and nothing is allocated per
frame.

**Errors — `Result<T, JsError>`**, surfacing as a thrown `Error`. No sentinel values, no
`null` meaning failure.

**Persistence (C-04) is a third thing**: versioned `serde_json` on disk, deliberately not
the in-memory representation, so a wire format change and a save format change stay
independent.

*Updated 2026-11-07 (M-21):* the save format is unchanged, but in the browser it is no
longer `serde_json` that reads and writes it. `serde_json` was the largest single thing in
the wasm bundle's code, there only for this, on the cold-load path. `src/json.rs` has the
browser's own `JSON.parse`/`JSON.stringify` do the text, crossing as a JavaScript value
through `serde-wasm-bindgen` — the control plane's mechanism — while native builds keep
`serde_json`. The one visible difference: the browser writes a whole number as `9`, not
`9.0`, and both read either.

## Options considered

### JSON strings both ways — rejected

Simplest and most debuggable: one `serde_json::to_string`, one `JSON.parse`. Rejected on
the data plane only: a string allocation and a parse per frame, for a payload that is
already a contiguous block of doubles, is work done to undo work. Still the right answer
for the save file, where it is chosen deliberately.

### `serde-wasm-bindgen` everywhere — rejected

Would keep one mechanism for everything, which is genuinely worth something. Rejected
because per-frame node voltages would become a fresh JS array per frame, i.e. garbage
proportional to node count times frame rate, on the one path where M-19 needs headroom.

### Opaque handles everywhere — rejected

Fast and allocation-free, but it makes every netlist edit a hand-written getter and
turns the control plane into a chatty RPC surface. The ergonomic cost lands on M-03,
M-04 and C-04 — the parts that will change most.

### Binary schema (bincode, FlatBuffers) — rejected for now

Fastest structured option and the right answer if the control plane ever shows up in a
profile. Rejected because it buys speed we have no measurement for, and costs a schema
toolchain on both sides. Revisit at C-15 with numbers, not before.

## Consequences

1. **A view over wasm memory can be detached at any time.** Any Rust-side allocation can
   grow the memory and invalidate every existing `ArrayBuffer` view. The rule, enforced
   by `viewFrame` in `app/src/lib/core.ts`: acquire the view at the point of reading,
   never store it across calls. This is the defect this design can produce, and it is
   silent when it happens.

2. **Data-plane values are handles, so they have lifetimes.** `SolverFrame` must be
   freed. `core.ts` owns that lifetime; Svelte components never hold a handle. If a
   component ever does, seam #1 has already leaked.

3. **`tsify` is now load-bearing.** It is what keeps the boundary typed rather than `any`.
   It is a proc-macro dependency on a small crate — acceptable, but if it ever goes
   unmaintained the fallback is hand-written `.d.ts` overrides, not an untyped boundary.

   *Updated 2026-09-18:* this happened almost immediately. The first `cargo deny` run
   (C-02) flagged RUSTSEC-2025-0048 — `tsify-next`, the fork originally chosen here, is
   no longer maintained and the advisory points back at `tsify`, which has resumed
   maintenance. Swapped; the only code change was the `use` line. Worth recording as
   evidence for why the advisory check is in CI rather than in someone's memory.

   *Updated 2026-11-07:* `tsify` also carries the names of the component kinds now. The
   core declares a `ComponentKind` union in its generated TypeScript (`registry.rs`, held
   to `REGISTRY` by a test) and types every `kind` field with it, so the app compares kinds
   against names the compiler knows rather than against any string.

4. **A Rust panic still crosses as `unreachable`.** Add `console_error_panic_hook` in a
   debug build before the solver work makes panics likely (M-09 onwards). Not done here
   because nothing panics yet.

   *Updated 2026-11-07:* done under C-31, and in every build rather than in debug only: a
   panic met in the shipped app is the one most worth reading. A panic still leaves the
   instance unusable, so the shell stops its loop on the first error from the core and
   offers a reload, the autosave holding the circuit.

5. **No numbers yet.** Nothing in this ADR is measured; it is reasoning about where the
   costs sit. C-15's benchmark harness is what turns it into evidence, and is also what
   should catch it if this split turns out to be wrong.

   *Updated 2026-11-07:* there are numbers now. C-15's bench (`app/src/lib/bench.test.ts`)
   times `advance` against the optimised bundle in `just ci`, the per-frame read-back of
   the structure included (C-24, under half a millisecond on the fifty-node ladder), and
   `just coldload` times the bundle's cold load against M-21's budget.
