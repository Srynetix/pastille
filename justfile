# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Everything the project can be asked to do. `just ci` is the gate: it runs the
# same checks in the same order a hosted pipeline would, ordered so the fastest
# thing that can fail, fails first.

WASM_OUT := "../../app/src/lib/wasm"

# List the recipes.
default:
    @just --list --unsorted

# ---------------------------------------------------------------- development

# This machine only: a dev server serves more than the app, and its file
# access rules have had holes (GHSA-9jrq-w75r-8gcw), so it is not left open
# to whoever shares the network. `dev-lan` is the phone's way in.
# Serve the app, rebuilding the wasm bundle first.
dev: wasm-dev
    pnpm --filter app dev

# On every interface, so a phone on the same network can open it at this
# machine's address (C-138). Only on a network you trust.
# Serve the app to the local network as well.
dev-lan: wasm-dev
    pnpm --filter app dev --host 0.0.0.0

# Rebuild the Uno's programs (programs/*.hex) from their sketches (A-07, A-04, A-06).
# Needs Arduino's AVR core and an avr-gcc: see programs/build.sh.
programs:
    cd programs && ./build.sh Blink Button TrafficLight Fade Smoothed Sweep AnalogInput Dimmer NightLight Thermometer AnalogReadSerial SerialLed SerialThermometer TwoPots

# Run a saved circuit without a browser. `just run --help` for the options.
run *ARGS:
    cargo run --quiet -p pastille-run -- {{ARGS}}

# Install everything the workspace needs, the app tests' browser included:
# a Playwright update wants its own build of Chromium, and says so only when
# the tests are run.
install:
    pnpm install
    pnpm --filter app exec playwright install chromium

# Reformat the Rust sources, and the app's (ADR 0018).
fmt:
    cargo fmt --all
    pnpm --filter app format

# ---------------------------------------------------------------------- build

# Build the wasm bundle, optimised.
wasm:
    wasm-pack build crates/core --release --target web --out-dir {{WASM_OUT}} --out-name pastille_core

# Build the wasm bundle, unoptimised and much faster.
wasm-dev:
    wasm-pack build crates/core --dev --target web --out-dir {{WASM_OUT}} --out-name pastille_core

# Build the production app into app/dist.
build: wasm
    pnpm --filter app build

# On every interface, as `dev-lan` is: a preview serves the built `dist/` and
# nothing else, so it does not carry the dev server's exposure.
# Serve the production build, which is what a cold load actually loads.
preview: build
    pnpm --filter app preview --host 0.0.0.0 --port 4173

# ---------------------------------------------------------------------- check

# Fail if anything is unformatted.
fmt-check:
    cargo fmt --all --check
    pnpm --filter app format:check

# Warnings are errors here: an ignored warning is a warning that never goes away.

# Lint the core, natively and as the browser gets it: code behind
# `cfg(target_arch = "wasm32")` is only ever compiled by the second.
clippy:
    cargo clippy --all-targets -- -D warnings
    cargo clippy --target wasm32-unknown-unknown -p pastille-core -- -D warnings

# Run the core's tests.
test-core:
    cargo test

# Licences and advisories across the Rust tree (ADR 0001).
deny:
    cargo deny check

# Only app/src/lib/core.ts may reach into the generated wasm glue (seam #1).
boundary:
    node scripts/check-import-boundary.mjs

# The fast static checks: format, lint, types. The types need the wasm glue,
# so its unoptimised build comes first.
check: fmt-check clippy wasm-dev types lint

# Every commit since the last release follows Conventional Commits: the release
# notes are built from them (ADR 0023). A range checks those commits instead.
check-commits *ARGS:
    scripts/check-commits.sh {{ARGS}}

# Every source file carries the SPDX line (ADR 0001).
spdx:
    node scripts/check-spdx.mjs

# Licences across the npm tree, the side `deny` does not see (ADR 0001).
npm-licences:
    node scripts/check-npm-licences.mjs

# Install the app, take the network away, and see whether it still works (C-27).
offline:
    node scripts/check-offline.mjs

# Needs Docker. `--help` for the options; amd64 is emulated on Apple Silicon.
# Build the Linux packages, a .deb and an AppImage, from this Mac (ADR 0022).
linux *ARGS:
    scripts/linux.sh {{ARGS}}

# Needs cargo-xwin, LLVM and makensis: `scripts/windows.sh --help`. WINDOWS_ARCH=arm64 for ARM.
# Build the Windows installer from this Mac (ADR 0022).
windows: build
    XWIN_ACCEPT_LICENSE=1 scripts/windows.sh build

# Check that the desktop shell compiles for Windows, without building the installer.
windows-check:
    XWIN_ACCEPT_LICENSE=1 scripts/windows.sh check

# What changed since the last release, from the commit subjects. `--help` for the options.
changelog *ARGS:
    scripts/changelog.sh {{ARGS}}

# Checks, builds and publishes a GitHub release of the Mac app. `--help` for the options.
release *ARGS:
    scripts/release.sh {{ARGS}}

# Leaves app/dist built for Pages: the next `just build` or `just ci` builds
# it for the root again.
# Build the app as GitHub Pages serves it, under /pastille/, and check it offline there (ADR 0021).
pages: wasm
    PASTILLE_BASE=/pastille/ pnpm --filter app build
    PASTILLE_BASE=/pastille/ node scripts/check-offline.mjs

# Replaces what is published, so it is run on purpose and never by `ci`.
# Build for Pages, check it, and push it to origin's gh-pages branch.
pages-publish: pages
    scripts/publish-pages.sh

# A first visit to the production build on a throttled phone, timed until the
# core is ready (M-21's three seconds). Not part of `ci` yet: it does not pass.
# Time a cold load against M-21's budget.
coldload: build
    node scripts/check-coldload.mjs

# Redraw the installed icons from the logo. Only needed when the logo changes.
icons:
    node scripts/render-icons.mjs

# Typecheck the app.
types:
    pnpm --filter app check

# Lint the app (ADR 0019): what the types allow but is still a mistake.
lint:
    pnpm --filter app lint

# Run the app's tests in a real browser.
test-app: wasm-dev
    pnpm --filter app test

# Measure what the simulation costs, where it runs. `just ci` runs this too.
#
# Against the optimised bundle, which is the only one whose numbers mean
# anything: the unoptimised one is an order of magnitude slower, and the
# benchmark skips itself unless VITE_BENCH says the build is the real one.
bench: wasm
    VITE_BENCH=1 pnpm --filter app test bench

# Everything, core and app.
test: test-core test-app

# -------------------------------------------------------------------- desktop
#
# The same app in a window of its own (C-68, ADR 0013). The shell is a workspace
# of its own, so `just ci` never waits on it; `desktop-check` is its gate.

# Run the desktop shell against the dev server, rebuilt on every edit.
desktop-dev:
    pnpm --filter desktop tauri dev

# A .app and .dmg on macOS, an installer on Windows, an AppImage on Linux —
# unsigned (ADR 0013).
# Build the desktop app for this machine.
desktop: build
    pnpm --filter desktop tauri build

# The build to hand a tester on a Mac: one .app and .dmg that run on Apple
# Silicon and on Intel, each half compiled for its own processor. Needs the
# Intel target once: `rustup target add x86_64-apple-darwin`.
# Build the desktop app for every Mac.
desktop-universal: build
    pnpm --filter desktop tauri build --target universal-apple-darwin

# Lint, its dependency policy, its notice, and one run of the bundled app that
# has to simulate a second of a circuit before it exits.
# The desktop shell's gate.
desktop-check: build
    cd desktop && cargo fmt --check && cargo clippy -- -D warnings && cargo deny check
    node scripts/check-third-party.mjs
    pnpm --filter desktop tauri build --bundles app
    just desktop-smoke

# macOS only: the path is the .app's.
# Open the bundled app once and have it report whether it simulates.
desktop-smoke:
    PASTILLE_SMOKE=1 desktop/target/release/bundle/macos/Pastille.app/Contents/MacOS/pastille-desktop

# ------------------------------------------------------------------------- ci

# The gate. Everything a hosted pipeline would run, fastest failure first. The
# app's tests run here against the optimised bundle, with the benchmark: the
# stricter form of `test-app`, so `test` is not run on top of it.
ci: check-commits check spdx test-core deny npm-licences wasm boundary
    VITE_BENCH=1 pnpm --filter app test
    pnpm --filter app build
    just offline
    @echo "\n  ✓ all checks passed"

# There is no hosted pipeline, so this machine is the pipeline: a missing tool is
# a check that silently never runs.

# Report anything missing from the toolchain.
doctor:
    #!/usr/bin/env bash
    set -uo pipefail
    missing=0
    for tool in cargo rustc wasm-pack cargo-deny node pnpm; do
        if command -v "$tool" > /dev/null 2>&1; then
            printf '  %-12s %s\n' "$tool" "$($tool --version 2>&1 | head -1)"
        else
            printf '  %-12s MISSING\n' "$tool"
            missing=1
        fi
    done
    if ! rustup target list --installed 2>/dev/null | grep -q wasm32-unknown-unknown; then
        printf '  %-12s MISSING (rustup target add wasm32-unknown-unknown)\n' "wasm32"
        missing=1
    fi
    if [ "$missing" -ne 0 ]; then
        echo
        echo "  Install the missing tools before trusting 'just ci'."
        echo "    cargo install wasm-pack cargo-deny"
        exit 1
    fi
    echo
    echo "  ✓ toolchain complete"
