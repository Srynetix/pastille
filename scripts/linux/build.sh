#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Runs in scripts/linux/Dockerfile's image, started by scripts/linux.sh: /src
# the sources with app/dist already built (read only), /cache what is kept
# between builds, /out the packages. BUNDLES says which; SMOKE=1 opens the
# built app once on a virtual display and fails unless it simulates.
set -euo pipefail

# A copy of the sources, updated in place, so cargo and pnpm redo only what
# changed. app/dist comes along: the web app is built on the Mac, wasm and
# all, and the container only builds the shell around it.
rsync -a --delete \
	--exclude node_modules --exclude /target --exclude /desktop/target \
	--exclude /app/src/lib/wasm --exclude /.git \
	/src/ /cache/work/
cd /cache/work
[[ -f app/dist/index.html ]] || { echo "no app/dist: build the app first" >&2; exit 1; }

export CARGO_HOME=/cache/cargo CARGO_TARGET_DIR=/cache/target
# linuxdeploy, which makes the AppImage, is an AppImage itself: without FUSE
# in a container, it unpacks itself instead.
export APPIMAGE_EXTRACT_AND_RUN=1
pnpm config set store-dir /cache/pnpm-store >/dev/null
pnpm install --frozen-lockfile --filter desktop

bundle=$CARGO_TARGET_DIR/release/bundle
# The volume keeps the packages of earlier builds: only this one's go out.
rm -rf "$bundle"
pnpm --filter desktop tauri build --bundles "$BUNDLES"

if [[ -n "${SMOKE:-}" ]]; then
	# The app as built, opened once: it has to simulate a second of a circuit
	# and exit, as `just desktop-smoke` asks of the Mac's (ADR 0013).
	echo "smoke run, on a virtual display"
	PASTILLE_SMOKE=1 WEBKIT_DISABLE_COMPOSITING_MODE=1 timeout 120 \
		dbus-run-session -- xvfb-run -a "$CARGO_TARGET_DIR/release/pastille-desktop"
fi

shopt -s nullglob
files=("$bundle"/deb/*.deb "$bundle"/appimage/*.AppImage)
[[ ${#files[@]} -gt 0 ]] || { echo "no package in $bundle" >&2; exit 1; }
for f in "${files[@]}"; do
	cp "$f" /out/
	echo "/out/$(basename "$f")"
done
