# 21. GitHub Pages serves a build made and checked on this machine

- **Status:** accepted
- **Date:** 2026-11-11
- **Ticket:** —
- **Deciders:** Denis Bourge

## Context and problem statement

Pastille is to be published as open source, and its first promise is that it runs in a
browser. Someone reading the README should be able to open it there and then, without
cloning and building it. GitHub Pages serves a static site from the repository for free, and
the app is a static site: a document, its bundles, the wasm core and a service worker.

Two things stood in the way. The app assumed it was served at the root of a server, where
Pages serves a project at `https://srynetix.github.io/pastille/`. And ADR 0014 keeps the
gate on this machine: a build made somewhere else would be one nothing here has checked.

## Decision

**`just pages` builds the app for `/pastille/` and runs the offline check against it there;
`just pages-publish` pushes that build to the `gh-pages` branch, which Pages serves. Nothing
builds or publishes on GitHub.**

- The build takes its base from `PASTILLE_BASE`, `/` when unset: the dev server, `just ci`
  and the desktop shell are unchanged.
- What the app names itself is relative, so one build works under any base: the manifest's
  `id`, `start_url`, `scope` and icons; the service worker's precache list and the document
  it answers navigations with, both resolved against the worker's own address. The worker's
  registration and the core's preload take the base from Vite.
- `scripts/check-offline.mjs` serves the build under `PASTILLE_BASE` and answers nothing
  outside it, as Pages does, so a path left absolute fails the check rather than a visitor.
- `scripts/publish-pages.sh` refuses a build that is not for `/pastille/`, an `origin` other
  than `Srynetix/pastille`, and a working tree with uncommitted changes, so what is published
  is a commit, which its message names. The
  branch holds that one build in one commit, replaced each time, with `.nojekyll`.

## Options considered

- **A GitHub Actions workflow that builds and deploys.** The usual way, and it would
  publish on every push. But it means the Rust toolchain, `wasm-pack` and pnpm set up on
  GitHub's runners, and a build ADR 0014's gate never saw, unless the workflow runs the gate
  too, which is the hosted pipeline ADR 0014 declined.
- **A custom domain at the root.** No base path to handle at all. It needs a domain; the
  relative paths here make it a setting to add later, not a change.
- **Pages from `/docs` on `main`.** Would put minified bundles in the source history at
  every release.

## Consequences

- Publishing is a deliberate act, `just pages-publish`, never a side effect of a push. What
  is online can lag behind `main`, by as much as its author lets it.
- Once, in the repository's settings: Pages › Build and deployment › Deploy from a branch,
  `gh-pages`, `/ (root)`.
- `just pages` leaves `app/dist` built for `/pastille/`; the next `just build` or `just ci`
  builds it for the root again.
- An install made before this change has the old manifest's identity (`/`). There are no
  installs to keep besides the author's (ADR 0020).
