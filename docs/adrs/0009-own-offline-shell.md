# 9. Write the service worker rather than take a PWA plugin

- **Status:** accepted
- **Date:** 2026-09-22
- **Ticket:** C-27
- **Deciders:** Denis Bourge

## Context and problem statement

The design doc puts distribution at "browser-first, PWA, offline-capable" and says why:
"No install is not a detail for this audience; it is most of the adoption" (§7.3). C-27
asks for installable, offline after the first visit, and a wasm bundle that is cached and
versioned.

The default answer is `vite-plugin-pwa`, which wraps Workbox and is what almost every
Vite project uses. The question this record exists for is whether to take it.

What is actually needed is narrow. The app is one document, a stylesheet, a script and a
wasm bundle, all of which Vite already names by content hash. There is no API to keep
fresh, no user data to sync, no navigation preload worth the complexity, no runtime
routing beyond "serve what was built". A circuit is saved to localStorage and to a file
the user chooses, neither of which the network is involved in.

## Decision

**Own implementation**: `app/src/offline/sw.js`, about sixty lines, emitted into the
build by a plugin in `vite.config.ts` that fills in two placeholders — the list of files
this build produced, and a fingerprint of that list.

Cache-first for everything precached, because every asset but the document is
content-addressed by the build and so a hit cannot be stale. Navigations are answered
with the one document whatever path was asked for. The cache is named after the
fingerprint, so a new build is a new cache and activation drops the old one.

## Options considered

### `vite-plugin-pwa` (Workbox) — rejected

It would work, and it handles cases this does not: runtime caching strategies per route,
precache manifests with revision fields for assets that are not content-hashed, an update
flow with a prompt, navigation preload, cross-browser workarounds accumulated over years.

Rejected because none of those cases exist here, and every one of them is behaviour that
would ship in the bundle on the cold-load path M-21 budgets three seconds for. The deeper
reason is the same one ADR 0004 gives for the LU solver: a service worker is a program
that sits *in front of* the app and can serve a stale one for ever if it is wrong. That
is not a component to take on trust — it is one to be able to read in a sitting.

### A generated precache list, hand-written policy — chosen

The split matters. What must be generated is the list of files, because getting it by
hand would mean a hashed filename going stale silently. What must be written is the
policy — what is cached, what is served from it, what happens to the last build — because
that is the part with consequences and the part a reader needs to see.

### No service worker, manifest only — rejected

Installable but not offline: an installed app that shows the browser's dinosaur is worse
than a tab, because installing it was a promise. The audience of §4 includes people on a
school network or a phone tether, which is exactly where this has to keep working.

## Consequences

1. **The gate runs it rather than trusting it.** `just offline` serves the real build to
   a real browser, installs the worker, switches the network off, reloads, and requires
   the app to open *and* solve a circuit — with the wasm core loaded from the cache. It
   is in `just ci`. A service worker that is only inspected is a service worker nobody
   has seen fail.

2. **Updates land on the next load, not the current one.** The new worker skips waiting
   and claims the page, but the page in front of the user keeps the assets it was served
   with until it is reloaded. Acceptable for an app with no server to disagree with; if a
   "new version, reload?" prompt is ever wanted, this is where it goes.

3. **Anything not in the build is cached as it is fetched.** A same-origin file the build
   did not know about is kept on first use, so a second visit has it offline too. Nothing
   currently produces one; it is there so that adding a font or an example circuit does
   not silently create a resource the offline app lacks.

4. **The dev server has no worker at all.** The plugin is build-only and the registration
   is behind `import.meta.env.PROD`, because a worker serving yesterday's app over the one
   being edited is the classic way to lose an afternoon.

5. **The icons are generated from the logo and committed.** `just icons` re-renders them
   through the same Chromium the tests use, because Safari needs a PNG for the home screen
   and the logo is built out of blur filters that other rasterisers get wrong. Two
   drawings that have to be edited together would be a third source of truth.
