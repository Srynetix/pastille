# Building and releasing

Prerequisites: [Rust](https://rustup.rs) with the `wasm32-unknown-unknown` target,
[wasm-pack](https://rustwasm.github.io/wasm-pack/), Node.js with [pnpm](https://pnpm.io), and
[just](https://just.systems). For the desktop app, the
[Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) too. `just doctor` says what is
missing.

## The development loop

Recipes live in the `justfile`; `just` on its own lists them.

```bash
just install  # the app's dependencies
just dev      # build the core (unoptimised) and serve the app on http://localhost:5173
just dev-lan  # the same, reachable from a phone on your network
just fmt      # format Rust and the app
just check    # format, clippy, types, ESLint: the fast loop
just test     # the core's tests, and the app's in a real browser
just run app/src/examples/milestone.json   # a saved circuit, simulated without a browser
```

`just dev` rebuilds the wasm core with `wasm-pack` into `app/src/lib/wasm/`, which is generated:
never edit it. A change to the core needs `just dev` again (or `just wasm-dev`) before the app
sees it.

## The gate

```bash
just ci
```

In order, fastest failure first: commit messages, formatting, clippy, types, ESLint, SPDX
headers, the core's tests, `cargo deny`, npm licences, the optimised wasm build, the import
boundary, the app's tests in a real browser with the benchmark, the production build, and an
offline run. There is no hosted pipeline ([ADR 0014](../adrs/0014-the-gate-stays-local.md)):
`just ci` passing is what says a change is good.

**Run it on mains power.** The benchmark measures the solver's cost a frame against a baseline
recorded on the author's Mac, and fails both ways. Before believing a red run, check
`pmset -g batt` and `pmset -g | grep lowpowermode`, and compare against `main`.

The desktop shell has its own gate, `just desktop-check`: its lints, its dependency policy, its
third-party notice, and one run of the bundled app that must simulate a second of a circuit
before it exits.

## Packaging

```bash
just desktop             # the desktop app for this machine, in desktop/target/release/bundle
just desktop-universal   # one Mac app for Apple Silicon and Intel
just windows             # the Windows installer, from the Mac: scripts/windows.sh --help
just linux               # the Linux .deb and AppImage, from the Mac, in Docker
just pages               # the web app as GitHub Pages serves it, checked offline
just coldload            # the optimised bundle's cold load, against its budget
```

Windows and Linux are built on the Mac ([ADR 0022](../adrs/0022-windows-and-linux-from-the-mac.md));
the desktop shell is unsigned ([ADR 0013](../adrs/0013-desktop-shell.md)). GitHub Pages serves
a build made and checked here, under `/pastille/`
([ADR 0021](../adrs/0021-github-pages-from-a-local-build.md)).

## The Uno's programs

`programs/` holds the example sketches (`*.ino`) beside the Intel HEX the app bundles.
`just programs` rebuilds them with `programs/build.sh`, against Arduino's AVR core. The HEX files carry Arduino's
core, under LGPL-2.1: `programs/NOTICE.md` says which version and where its source is.

## Releasing

```bash
just changelog           # what changed since the last tag, from the commits
just release --dry-run   # what a release would publish: just release --help
```

A release is tagged `pastille-v*`, and every file it publishes ships with its `.sha256` and a
minisign `.minisig` ([ADR 0023](../adrs/0023-in-line-with-the-p-suite.md)). `just release` and
`just pages-publish` publish: an agent runs them only when asked.
