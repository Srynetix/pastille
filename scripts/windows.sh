#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Pastille for Windows, built on the Mac with cargo-xwin (ADR 0022): x64
# (x86_64-pc-windows-msvc) by default, arm64 (aarch64-pc-windows-msvc) with
# WINDOWS_ARCH=arm64.
#
#   scripts/windows.sh check [cargo args]   cargo check of the desktop shell
#   scripts/windows.sh build [tauri args]   the NSIS installer, unsigned, in
#                                           desktop/target/<target>/release/bundle/nsis
#
# The app it carries is app/dist, built first for the root as `just build`
# builds it. Needs clang-cl, lld-link and llvm-lib (`brew install llvm lld`),
# makensis to build (`brew install makensis`), cargo-xwin (`cargo install
# --locked cargo-xwin`) and the Rust targets (`rustup target add
# x86_64-pc-windows-msvc aarch64-pc-windows-msvc`). cargo-xwin downloads
# Microsoft's CRT and Windows SDK on its first run: XWIN_ACCEPT_LICENSE=1
# accepts their licence.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
case "${WINDOWS_ARCH:-x64}" in
	x64) target=x86_64-pc-windows-msvc ;;
	arm64) target=aarch64-pc-windows-msvc ;;
	*) echo "windows: WINDOWS_ARCH x64 or arm64, not ${WINDOWS_ARCH}" >&2; exit 2 ;;
esac

usage() { sed -n '4,18p' "$0" | sed 's/^# \{0,1\}//'; }
mode="${1:-}"
case "$mode" in
	check | build) shift ;;
	-h | --help) usage; exit 0 ;;
	*) usage >&2; exit 2 ;;
esac

# Homebrew's LLVM is keg-only: its clang-cl and llvm-lib are not on the PATH.
for brew in /opt/homebrew/opt /usr/local/opt; do
	[ -d "$brew/llvm/bin" ] && PATH="$brew/llvm/bin:$PATH"
	[ -d "$brew/lld/bin" ] && PATH="$brew/lld/bin:$PATH"
done
export PATH
tools="cargo-xwin clang-cl lld-link"
[ "$mode" = build ] && tools="$tools makensis llvm-lib pnpm"
for tool in $tools; do
	command -v "$tool" >/dev/null || { echo "windows: $tool not found (scripts/windows.sh --help)" >&2; exit 1; }
done
rustup target list --installed | grep -qx "$target" ||
	{ echo "windows: no Rust target $target (rustup target add $target)" >&2; exit 1; }
if [ -z "${XWIN_ACCEPT_LICENSE:-}" ]; then
	echo "windows: set XWIN_ACCEPT_LICENSE=1 to accept the licence of Microsoft's CRT and SDK, which cargo-xwin downloads" >&2
	exit 1
fi

if [ "$mode" = check ]; then
	cd "$root/desktop"
	exec cargo xwin check --target "$target" "$@"
fi

[ -f "$root/app/dist/index.html" ] || { echo "windows: no app/dist: run \`just build\` first" >&2; exit 1; }
cd "$root"
pnpm --filter desktop tauri build --runner cargo-xwin --target "$target" --bundles nsis "$@"
ls "$root/desktop/target/$target/release/bundle/nsis/"*-setup.exe
