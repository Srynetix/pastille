#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Builds Pastille for Linux from the Mac, in Docker (ADR 0022): a .deb
# (Debian, Ubuntu and their derivatives) and an AppImage (any other
# distribution, a single executable file).
#
#   scripts/linux.sh [--arch amd64|arm64] [--bundles deb,appimage,pacman] [--no-smoke] [--allow-dirty]
#
# - amd64 (x86_64 PCs) by default; on an Apple Silicon Mac it runs emulated,
#   which is slow. arm64 builds natively, for ARM machines.
# - --bundles: deb,appimage by default; pacman adds a package for Arch Linux
#   and its derivatives, the .deb repackaged in an archlinux container.
#   amd64 only: Arch Linux ARM is a distribution of its own.
# - The web app is built here first (`just build`), and the container builds
#   only the shell around it.
# - Then the app is opened once on a virtual display and has to simulate, as
#   the Mac's is (`just desktop-smoke`); --no-smoke leaves that out.
# - Refuses uncommitted changes, so the packages are of a commit;
#   --allow-dirty builds anyway.
#
# Out: desktop/target/linux-<arch>/Pastille_<version>_<arch>.deb,
# .AppImage (and .pkg.tar.zst), and .commit, the commit they are of (scripts/release.sh reuses
# them). The first build takes a while (the image, then every crate); the
# next reuse a Docker volume, pastille-linux-<arch>: `docker volume rm
# pastille-linux-<arch>` starts over.
set -euo pipefail

cd "$(dirname "$0")/.."

arch=amd64
bundles=deb,appimage
smoke=true
dirty=false
while [[ $# -gt 0 ]]; do
	case "$1" in
		--arch) arch=${2:-}; shift ;;
		--bundles) bundles=${2:-}; shift ;;
		--no-smoke) smoke=false ;;
		--allow-dirty) dirty=true ;;
		-h | --help) sed -n '4,26p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
		*) echo "unknown option: $1 (see --help)" >&2; exit 2 ;;
	esac
	shift
done

step() { printf '\n\033[1m▸ %s\033[0m\n' "$*"; }
fail() { printf '\033[31m✗ %s\033[0m\n' "$*" >&2; exit 1; }

[[ "$arch" == amd64 || "$arch" == arm64 ]] || fail "--arch: amd64 or arm64"
[[ "$bundles" =~ ^(deb|appimage|pacman)(,(deb|appimage|pacman))*$ ]] ||
	fail "--bundles: deb, appimage and/or pacman, comma-separated"
# The Arch package is the .deb repackaged: Tauri builds the others, and the
# .deb along with them.
pacman=false
tauri_bundles=$(tr , '\n' <<<"$bundles" | { grep -vx pacman || true; } | paste -sd, -)
if [[ ",$bundles," == *,pacman,* ]]; then
	pacman=true
	[[ "$arch" == amd64 ]] || fail "--bundles pacman: amd64 only (Arch Linux ARM is another distribution)"
	[[ ",$tauri_bundles," == *,deb,* ]] || tauri_bundles=${tauri_bundles:+$tauri_bundles,}deb
fi
command -v docker >/dev/null || fail "docker not found (OrbStack, Docker Desktop…)"
docker info >/dev/null 2>&1 || fail "Docker is not running"

version=$(node -p "require('./desktop/tauri.conf.json').version")
commit=$(git rev-parse HEAD)
if [[ -n "$(git status --porcelain --untracked-files=no)" ]]; then
	$dirty || fail "uncommitted changes: commit them, or --allow-dirty"
	echo "uncommitted changes: building anyway (--allow-dirty)"
	commit="$commit-dirty"
fi

step "Pastille $version (${commit:0:7}), linux $arch, $bundles"

step "The web app, built here"
just build

step "Build image (cached after the first time)"
image=pastille-linux-builder:$arch
docker build --platform "linux/$arch" -t "$image" scripts/linux

step "The shell and its packages, in the container"
out=desktop/target/linux-$arch
rm -rf "$out"
mkdir -p "$out"
work=$(mktemp -d "${TMPDIR:-/tmp}/pastille-linux.XXXXXX")
trap 'rm -rf "$work"' EXIT
docker run --rm --platform "linux/$arch" \
	-v "$PWD:/src:ro" -v "pastille-linux-$arch:/cache" -v "$work:/out" \
	-e BUNDLES="$tauri_bundles" -e SMOKE="$($smoke && echo 1 || true)" \
	"$image" /src/scripts/linux/build.sh

if $pacman; then
	step "Arch package, from the .deb"
	in=$(mktemp -d "${TMPDIR:-/tmp}/pastille-arch.XXXXXX")
	trap 'rm -rf "$work" "$in"' EXIT
	debs=("$work"/*.deb)
	cp "${debs[0]}" "$in/pastille.deb"
	cp LICENSE-MIT LICENSE-APACHE "$in/"
	# pacman's versions have no `-`, which separates the package's release:
	# 0.2.0-beta.1 is written 0.2.0_beta.1.
	docker run --rm --platform linux/amd64 \
		-v "$PWD/scripts/linux/arch:/pkg:ro" -v "$in:/in:ro" -v "$work:/out" \
		-e PASTILLE_PKGVER="${version//-/_}" \
		archlinux:base-devel /pkg/build.sh
	# Asked for the Arch package alone, the .deb it came from is not an output.
	[[ ",$bundles," == *,deb,* ]] || rm -f "$work"/*.deb
fi

step "Done"
shopt -s nullglob
for f in "$work"/*; do
	case "$f" in
		*.deb) name="Pastille_${version}_${arch}.deb" ;;
		*.AppImage) name="Pastille_${version}_${arch}.AppImage" ;;
		*.pkg.tar.zst) name="Pastille_${version}_${arch}.pkg.tar.zst" ;;
		*) continue ;;
	esac
	cp "$f" "$out/$name"
	echo "$out/$name ($(du -h "$out/$name" | cut -f1))"
done
echo "$commit" >"$out/.commit"
cat <<'TXT'

On the Linux machine:
  .deb       sudo apt install ./Pastille_<version>_<arch>.deb
  AppImage   chmod +x Pastille_<version>_<arch>.AppImage && ./Pastille_<version>_<arch>.AppImage
             (needs libfuse2: sudo apt install libfuse2, or libfuse2t64 on Ubuntu 24.04)
  Arch       sudo pacman -U Pastille_<version>_amd64.pkg.tar.zst
TXT
