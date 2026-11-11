# 13. A Tauri shell beside the PWA, with MPL-2.0 allowed in it alone

- **Status:** accepted
- **Date:** 2026-09-24
- **Ticket:** C-68
- **Deciders:** Denis Bourge

## Context and problem statement

The app ships as a PWA (§7.3, ADR 0009): a link, and after one visit it works offline.
That needs a host and an HTTPS address, and the obvious workaround — a zip of
`app/dist` — does not work by double-click, because no browser loads a wasm module or a
service worker from `file://`. Handing a tester one file is the gap C-68 names.

The browser build is not being replaced. Whatever wraps it has to wrap the same
`app/dist`, with no second copy of the UI, and the browser has to keep behaving exactly
as it does.

## Decision

**A Tauri 2 shell in `desktop/`**, showing the same `app/dist` in the system's web view.

1. **Its own Cargo workspace**, excluded from the root one. A web view and a windowing
   stack are hundreds of crates; `just ci` never compiles them. `just desktop-check` is
   the shell's gate: format, clippy, its dependency policy, its notice, and one run of the
   bundled app that has to simulate a second of a circuit before it exits.

2. **Three things differ from the browser, and only three.** The macOS web view ignores
   download links, so saving and opening go through the platform's dialogs, as two shell
   commands. It opens no second window either, so a link meant for a tab of its own — a
   datasheet in the Inspector — goes to the system's browser through a third, which
   follows `https:` and nothing else. The service worker is not registered. All three are
   decided by `app/src/lib/desktop.ts`, and the browser build takes none of these paths.

3. **MPL-2.0 is allowed in the shell's dependency tree, and nowhere else.** Tauri depends,
   through `tauri-utils`, on Servo's CSS parser (`cssparser`, `cssparser-macros`,
   `selectors`, `dtoa-short`), and on `option-ext` through `dirs`. All five are MPL-2.0,
   which ADR 0001's "no copyleft linked" rule forbids as written. `desktop/deny.toml`
   allows MPL-2.0 for this workspace alone; the root policy is unchanged, so the core,
   the wasm bundle and the web app stay strictly permissive.

## Options considered

### Tauri 2 — chosen

A window of its own for ~9 MB (a 4.4 MB `.dmg`), a Rust shell beside a Rust core, and the
system web view rather than a second browser. The cost is three engines to render on
(WebKit on macOS and Linux, WebView2 on Windows) instead of one, and the licence
exception above.

### Electron — rejected

Chromium in the box, so one engine everywhere, at ~100 MB or more per platform and a Node
packaging toolchain. It does not avoid the licence question either: Electron ships LGPL
components such as ffmpeg.

### A small Rust binary serving `dist` to the default browser — rejected

The lightest, ~1–2 MB, with a permissive tree. It is a browser tab rather than an app:
the port and the browser are the user's, and there is no window to put in a dock. Kept in
mind as the fallback if Tauri turns out to render badly somewhere.

## Consequences

1. **MPL-2.0's obligation is met by a notice.** The five crates are linked unmodified;
   the licence asks that their source stays available and that the build says so.
   `desktop/THIRD-PARTY.md` names them with their versions and sources, ships inside the
   bundle, and `scripts/check-third-party.mjs` fails `desktop-check` the day a
   `cargo update` changes the list without the notice. Modifying any of those files would
   put the modified file under MPL-2.0; nothing here does, and this ADR has to be reopened
   before anything would.

2. **Six unmaintained-crate advisories are ignored by name**, in `desktop/deny.toml`
   with a reason each: `proc-macro-error` through gtk on Linux, and the rust-unic crates
   through `urlpattern`. They are maintenance notices, not vulnerabilities, and they
   leave when Tauri does. A new advisory still fails the check.

3. **Builds are unsigned, but sealed.** The macOS bundle is signed ad hoc
   (`signingIdentity: "-"`), which costs nothing and proves nothing about who built it,
   but seals the bundle. Without it only the executable carries the linker's signature,
   `codesign --verify` fails, and a downloaded copy is reported as "damaged and can't be
   opened", with no way past it in System Settings. Sealed, it is the ordinary
   unidentified-developer refusal, which Privacy & Security's "Open Anyway" lifts;
   Windows SmartScreen warns likewise. For a few testers a
   written instruction is enough; signing (an Apple Developer ID, a Windows certificate)
   is its own ticket when there are more.

4. **Each platform is built on itself.** `just desktop` builds for the machine it runs
   on; `just desktop-universal` builds the one macOS bundle to hand out, which runs on
   Apple Silicon and on Intel.
   Windows and Linux builds need those machines, or a hosted runner, which ADR 0005 has so
   far said no to.

5. **Autosave is the web view's own.** The shell keeps its circuit in its own storage, so
   the browser and the desktop app do not see each other's autosaved canvas. Files are
   the same format either way.
