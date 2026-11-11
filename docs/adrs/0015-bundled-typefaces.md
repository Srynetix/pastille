# 15. Two typefaces in the bundle, under the SIL Open Font License

- **Status:** accepted
- **Date:** 2026-10-02
- **Ticket:** C-126
- **Deciders:** Denis Bourge

## Context and problem statement

The redesign (design doc §15) sets the app in two typefaces: IBM Plex Sans for text, and
Recursive, whose `MONO` axis gives the readings a monospace that does not jitter as the
digits change and whose `CASL` axis gives the headings their character. Until now the app
used `system-ui` and shipped no font at all.

A typeface loaded from a CDN would break C-27: the app has to open and work with no
network, and the canvas's labels would fall back to whatever the machine has. So the
fonts go in the bundle and in the service worker's precache, like everything else.

Both are licensed under the SIL Open Font License 1.1, which is not on ADR 0001's list.
`just npm-licences` refuses them, as it should.

## Decision

**The two typefaces ship in the app, unmodified, under the OFL, as named exceptions in
`scripts/check-npm-licences.mjs`.**

1. **From npm, at a pinned version**: `@fontsource-variable/recursive` and
   `@fontsource-variable/ibm-plex-sans`, imported by `app/src/fonts.css`. The packages
   carry the font files and the licence; nothing is copied into the repository by hand.
2. **Only the faces used.** Recursive's `full` files (they hold the `MONO` and `CASL`
   axes; no other Fontsource file holds both) and Plex Sans's upright `wght` files. Each
   comes in unicode-range subsets, so a browser fetches only the ones its text needs;
   the service worker precaches all of them, which is what keeps them offline.
3. **The licence travels with the files.** Each package's `LICENSE` is copied into
   `app/dist/fonts/` at build time beside a short `README`, so a copy of the app carries
   the notice the OFL asks for.

## Why the OFL is acceptable here

ADR 0001's concern is copyleft *code* reaching the work through linking or copying. A font
is neither linked nor part of the program: the OFL says in its preamble that fonts may be
"bundled, embedded, redistributed and/or sold with any software", and that doing so does
not subject the software to the OFL. Its conditions bind the font alone — it may not be
sold by itself, a modified version must keep the OFL and drop any Reserved Font Name, and
the copyright and licence must accompany it. The first two never arise for unmodified
files inside an app; the third is point 3 above.

The code of the app, the core and the wasm bundle stay MIT OR Apache-2.0, unchanged.

## Options considered

### Bundle both under the OFL — chosen

The look the redesign was drawn with, offline, at the cost of a few hundred kilobytes in
the precache (Recursive's variable font is the larger) and this exception.

### `system-ui` and `ui-monospace` — rejected

No licence question and nothing to download, but the monospace differs on every platform
(SF Mono, Cascadia, DejaVu), and the casual headings that give the app its character do
not exist at all.

### A CDN (Google Fonts) — rejected

Breaks C-27 outright, and sends every visitor's address to a third party for a font.

## Consequences

- Updating either package is an ordinary dependency update; the exception names the
  package, not a version, because the licence is the font family's, not a release's.
- A third typeface would need the same: a named exception with its reason, and its licence
  in `dist/fonts/`.
- Subsetting the files further (to Latin only) would be a modification under the OFL's
  terms — allowed, but it would make the fonts "Modified Versions"; leave them as shipped.
