# 19. ESLint lints the app, with types, in the gate

- **Status:** accepted
- **Date:** 2026-11-07
- **Ticket:** —
- **Deciders:** Denis Bourge

## Context and problem statement

The gate type-checks the app (`svelte-check`, `tsc`) and, since ADR 0018, formats it. Neither
catches what the types allow but is still a mistake: a promise nobody waits for, an import
left behind, a value read from `JSON.parse` and used as if it had been checked, a Svelte
`Set` mutated where nothing will notice. The Rust side has had `clippy -D warnings` from the
start; the app had nothing in that place.

## Decision

**ESLint, with the recommended rules of `@eslint/js`, of `typescript-eslint` with type
information (`recommendedTypeChecked`), and of `eslint-plugin-svelte`; run by `just lint`,
which `just ci` runs. Every finding an error.**

- `app/eslint.config.js`. No layout rules: layout is Prettier's.
- Turned off only with the reason beside it, in the config or on the line:
  - The `no-unsafe-*` rules in `.svelte` files. The linter's TypeScript cannot read the
    types of an imported `.svelte` component, so in a component everything that comes from
    a child — its exports, its callbacks' arguments — is `any` to it. `svelte-check`,
    already in the gate, reads them. In `.ts` files the rules stay on.
  - `svelte/prefer-svelte-reactivity` on five lines, each a `Map` or `Set` that is
    deliberately not reactive: a gesture's bookkeeping, a private decoder table, a copy
    handed back whole.
- `_` names a value a pattern must bind and nobody reads.

## What it found

118 findings on the first run, in 21 files. 55 were the `no-unsafe-*` rules on component
values in `App.svelte`, the blind spot above. Of the other 63, none was a bug in what the
app does, and each is fixed:

- floating promises: the core's `start()`, and Svelte's `unmount()` in tests;
- unused imports and variables, among them a test's leftover `Projection`;
- `JSON.parse` results used untyped, now typed where they are read;
- unneeded type assertions, and `async` test bodies with nothing to await;
- arrays interpolated into test messages, now joined;
- a `svelte-ignore` the compiler no longer needed.

## Options considered

- **Biome's linter**: one tool with ADR 0018's alternative, and fast. It lints a `.svelte`
  file's script but not its markup, and has no rule that knows Svelte's reactivity. Worth
  revisiting with the formatter.
- **oxlint**: fast, but its Svelte support is the script alone, and it has no rule that
  knows Svelte's reactivity either.
- **ESLint without type information**: faster, but it loses `no-floating-promises` and the
  `no-unsafe-*` family, which were most of what it found.
- **Nothing**: the app keeps no equivalent of `clippy`.

## Consequences

- `just ci` takes about fifteen seconds longer: type-aware linting builds the program.
- Five dev dependencies. ESLint brings `minimatch` under BlueOak-1.0.0, a permissive
  licence not on ADR 0001's list; it is named in `scripts/check-npm-licences.mjs` as a
  development tool, none of which is in the bundle.
