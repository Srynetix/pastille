# 22. The Windows and Linux apps are built on the Mac

- **Status:** accepted — revises consequence 4 of ADR 0013, "each platform is built on
  itself"
- **Date:** 2026-11-11
- **Ticket:** —
- **Deciders:** Denis Bourge

## Context and problem statement

The desktop shell (ADR 0013) was only ever built for the Mac, though its configuration
already named the Windows and Linux bundles: ADR 0013 left those to "those machines, or a
hosted runner", and ADR 0014 keeps the gate on this one. Published as open source, Pastille
reaches people on every system, and most beginners are on Windows. The web app runs on all
of them, but an installed app is what many expect.

The shell is small: Tauri, two of its plugins and the commands that save and open files.
Nothing in it is native code of its own, so nothing in it cares which machine compiles it.

## Decision

**The Mac builds all three, as Prism's are built: Windows with cargo-xwin, Linux in
Docker. `just windows` and `just linux` build them, and `just release --windows --linux`
publishes them with the Mac's.**

- **Windows** (`scripts/windows.sh`): `tauri build --runner cargo-xwin` for
  `x86_64-pc-windows-msvc`, or `aarch64-pc-windows-msvc` with `WINDOWS_ARCH=arm64`, linked
  against Microsoft's CRT and SDK, which cargo-xwin downloads once, and packed by Homebrew's
  makensis into an NSIS installer. Unsigned: SmartScreen warns, as Gatekeeper does
  (ADR 0013).
- **Linux** (`scripts/linux.sh`): an Ubuntu 22.04 image (glibc 2.35, WebKitGTK 4.1),
  `amd64` or `arm64`, builds a `.deb` and an AppImage. The web app is built on the Mac first,
  wasm and all, and only the shell is built in the container, which keeps the image to
  Tauri's prerequisites. For Arch Linux, `amd64` only, the `.deb` is repackaged by a PKGBUILD
  in an `archlinux:base-devel` container into a `.pkg.tar.zst`: the same files, Arch's names
  for its two dependencies, WebKitGTK 4.1 and GTK 3.
- **Checked as far as this machine can.** Both compile under the gate's dependency policy,
  which already listed their targets and now lists the ARM ones too. The Linux app is opened
  once on a virtual display (Xvfb) in the container and has to simulate, as the Mac's is.
  The Windows app is not run: nothing here can run it.
- The AppImage carries Ubuntu's GTK and WebKitGTK, mostly LGPL, unmodified and dynamically
  linked; `desktop/THIRD-PARTY.md` says so and where their source is. The `.deb`, the Arch
  package and the Windows installer carry no system library.

## Options considered

- **A hosted runner per system** (GitHub Actions' `windows-latest`, `ubuntu-latest`). Native
  builds, and the Windows app could be run. But it is the hosted pipeline ADR 0014 declined,
  with the gate then split between two places.
- **Building on a Windows and a Linux machine by hand.** What ADR 0013 assumed. There are
  none at hand, and a release would wait on three machines.
- **Web only for Windows and Linux.** The web app works there, offline once installed, and
  costs nothing. But it keeps its circuits in the browser's storage, and opening and saving
  files is the browser's dialogs; worth more as the other way in than as the only one.

## Consequences

- One machine builds and publishes everything, with Docker and the Windows toolchain
  installed: `scripts/windows.sh --help` lists it.
- Cross-compiling for Windows is what Tauri calls experimental. A Windows-only problem shows
  on a Windows machine, not here: the first Windows users are also its first testers.
- Linux `amd64` is emulated on Apple Silicon: the first build of the image and the crates
  is slow; a Docker volume keeps them for the next ones.
- The Windows installer downloads WebView2 if Windows lacks it. Windows 11 has it, and
  Windows 10 has been given it by its updates, so it is mostly a machine kept offline that
  needs the download.
