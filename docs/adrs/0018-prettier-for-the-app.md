# 18. Prettier formats the app, and the gate holds it

- **Status:** accepted
- **Date:** 2026-11-07
- **Ticket:** —
- **Deciders:** Denis Bourge

## Context and problem statement

The Rust is formatted by `cargo fmt` and `just ci` fails on a diff. The app — TypeScript,
Svelte and CSS, some 27 000 lines — has had nothing but an `.editorconfig`, so its style
is whatever each editor did: tabs, single quotes and no semicolons hold throughout, but
line breaking does not, and 491 lines run past 100 columns. A formatter nobody runs is the
same as none, so the question is which one, and whether the gate enforces it.

## Decision

**Prettier, with `prettier-plugin-svelte`, configured to the style the code already has,
run by `just fmt` and checked by `just fmt-check`, which `just ci` runs first.**

- `app/.prettierrc.json`: tabs, single quotes, no semicolons, 100 columns.
- It covers `app/src/**/*.{ts,svelte,css}` and the app's own config files. The example
  circuits (JSON) are left as they are: they are written by hand to be read, one element
  to a line, and a formatter would spread each over a dozen.
- The first run is a commit of its own, listed in `.git-blame-ignore-revs`, so `git blame`
  looks through it.

## Options considered

Measured on the code as it was, at the same settings.

| | Files changed | App tests after | Time |
|---|---|---|---|
| **Prettier 3.9 + plugin-svelte** | 41 of 88 (40 once `symbols.ts` is left out) | 885 pass | 4.1 s |
| Biome 2.6, Svelte support enabled | 44 of 89 | 3 fail | 0.2 s |
| Nothing (status quo) | — | — | — |

- **Biome** is twenty times faster and one tool for formatting and linting. But its
  Svelte support is still behind an experimental flag, and on this code it reflowed
  markup text — whitespace inside a paragraph is part of what it says — so three tests
  that read what the app says on screen failed. A formatter that can change what a user
  reads cannot be run blind. The width it formats Rust-free code to is the same, so moving
  later is a reformat, not a rewrite: worth revisiting once its Svelte support is stable.
- **dprint** formats Svelte only through a general markup plugin, with the same risk.
- **Nothing** leaves the gate asymmetric: a Rust change is held to a style, an app change
  is not.

## Consequences

- `just ci` fails on an unformatted app file, as it does on an unformatted Rust one; the
  fix is `just fmt`.
- Two dev dependencies, both MIT, both checked by `just npm-licences`.
- `app/src/lib/symbols.ts` is in `.prettierignore`. It is drawing data, written one shape
  to a line so a symbol reads as its outline; Prettier spreads every pair of points over
  seven lines and would have added a thousand. Excluded for the same reason the example
  circuits are.
- No linter is added here; that is ADR 0019.
- *Updated 2026-11-11:* the history was squashed into one commit before the repository was
  opened, so the formatting commit and `.git-blame-ignore-revs` are gone with it.
