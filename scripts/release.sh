#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Publishes this commit as a GitHub release of Pastille: the Mac app, for
# Apple Silicon and Intel; with --linux and --windows the apps for those,
# built on the Mac too (ADR 0022); with --pages the web app on GitHub Pages.
#
#   scripts/release.sh [--to owner/repo] [--linux] [--windows] [--pages] [--skip-checks]
#                      [--rebuild] [--notes "text" | --notes-file file] [--no-tag] [--dry-run]
#
# - --to: the repository, Srynetix/pastille by default (PASTILLE_RELEASE_TO).
# - The release is tagged pastille-v<version>, the version of
#   desktop/tauri.conf.json, which Cargo.toml, desktop/Cargo.toml and
#   package.json must agree with: bump them together first. A version
#   already released is refused.
# - A release is a commit: no uncommitted changes, and the commit already on
#   GitHub, so the tag the release makes there is on it.
# - Checks first, as every change is checked (ADR 0014): `just ci`, then the
#   built app opened once and made to simulate (`just desktop-smoke`'s
#   check, on the universal app). Run it on mains power, as `just ci` is.
#   --skip-checks leaves `just ci` out, for a commit it has just passed on.
# - Files: Pastille_<version>_macos-universal.dmg, the app `just
#   desktop-universal` builds, in a disk image made with hdiutil, and its
#   .sha256. Unsigned (ADR 0013): the notes say how to open it. A .dmg of
#   this commit already built is published as is; --rebuild builds it again.
# - --linux: Pastille_<version>_{amd64,arm64}.{deb,AppImage}, and
#   Pastille_<version>_amd64.pkg.tar.zst for Arch Linux, built in Docker by
#   scripts/linux.sh, which opens each app once on a virtual display. amd64
#   is emulated on Apple Silicon, so slow the first time.
# - --windows: Pastille_<version>_windows-{x64,arm64}-setup.exe, NSIS
#   installers built with cargo-xwin by scripts/windows.sh. Unsigned:
#   SmartScreen warns, and the notes say how past it. Not opened before
#   publishing: nothing here can run them.
#   Builds of this commit already there are reused, as the .dmg is.
# - Notes: the changelog since the previous release (scripts/changelog.sh,
#   from the Conventional commit subjects), then how to open the app;
#   --notes or --notes-file replace the changelog.
# - Once published, the commit is tagged pastille-v<version> in this clone
#   too, so the next changelog starts from it. --no-tag leaves it out.
# - --pages: then `just pages-publish` (ADR 0021), so the web app is the
#   release too.
# - --dry-run builds and prints what would be published, without publishing.
set -euo pipefail

cd "$(dirname "$0")/.."

to=${PASTILLE_RELEASE_TO:-Srynetix/pastille}
linux=false
windows=false
pages=false
checks=true
dry=false
rebuild=false
tag_it=true
notes=""
notes_file=""
while [[ $# -gt 0 ]]; do
	case "$1" in
		--to) to=${2:-}; shift ;;
		--linux) linux=true ;;
		--windows) windows=true ;;
		--pages) pages=true ;;
		--skip-checks) checks=false ;;
		--notes) notes=${2:-}; shift ;;
		--notes-file) notes_file=${2:-}; shift ;;
		--no-tag) tag_it=false ;;
		--dry-run) dry=true ;;
		--rebuild) rebuild=true ;;
		-h | --help) sed -n '4,42p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
		*) echo "unknown option: $1 (see --help)" >&2; exit 2 ;;
	esac
	shift
done

step() { printf '\n\033[1m▸ %s\033[0m\n' "$*"; }
fail() { printf '\033[31m✗ %s\033[0m\n' "$*" >&2; exit 1; }

[[ "$(uname -s)" == Darwin ]] || fail "a release is built on a Mac: the app is a macOS one"
[[ $to == */* ]] || fail "--to owner/repo, not '${to}'"
command -v gh >/dev/null || fail "gh not found (brew install gh)"
[[ -z "$notes_file" || -f "$notes_file" ]] || fail "--notes-file: no $notes_file"

# One version, written in four places.
version=$(node -p "require('./desktop/tauri.conf.json').version")
for file in Cargo.toml desktop/Cargo.toml; do
	found=$(sed -n 's/^version = "\(.*\)"/\1/p' "$file" | head -1)
	[[ $found == "$version" ]] || fail "$file says $found, desktop/tauri.conf.json $version: bump them together"
done
found=$(node -p "require('./package.json').version")
[[ $found == "$version" ]] || fail "package.json says $found, desktop/tauri.conf.json $version: bump them together"
tag="pastille-v$version"

step "Pastille $version → $to ($tag)"
gh repo view "$to" >/dev/null || fail "no access to $to with gh"
if gh release view "$tag" -R "$to" >/dev/null 2>&1; then
	fail "$tag is already released in $to: bump the version first"
fi

[[ -z "$(git status --porcelain --untracked-files=no)" ]] || fail "uncommitted changes: commit them first"
commit=$(git rev-parse HEAD)
short=$(git rev-parse --short HEAD)
# Without this, GitHub would tag its own default branch, which may be another commit.
gh api "repos/$to/commits/$commit" >/dev/null 2>&1 || fail "$short is not on $to yet: push it first"

if $checks; then
	step "The gate"
	just ci
fi

step "macOS"
built=desktop/target/universal-apple-darwin/release/bundle
app="$built/macos/Pastille.app"
dmg="$built/release/Pastille_${version}_universal.dmg"
stamp="$built/release/.commit"
if [[ -f "$dmg" && -f "$stamp" && "$(cat "$stamp")" == "$commit" ]] && ! $rebuild; then
	echo "$(basename "$dmg") of $short already built: reused (--rebuild to build it again)"
else
	# The .app only, for both processors, as `just desktop-universal` builds
	# it; the disk image is made below. Tauri's own lays its window out by
	# driving the Finder with AppleScript, which times out wherever the
	# terminal may not control it, and the window's layout is not worth that.
	just build
	pnpm --filter desktop tauri build --target universal-apple-darwin --bundles app
	[[ -d "$app" ]] || fail "no $app after the build"
	[[ "$(lipo -archs "$app/Contents/MacOS/pastille-desktop")" == *x86_64*arm64* ]] ||
		fail "$app is not for both Intel and Apple Silicon"

	# The app beside a link to Applications, the one thing to drag it onto.
	mkdir -p "$(dirname "$dmg")"
	stage=$(mktemp -d "${TMPDIR:-/tmp}/pastille-dmg.XXXXXX")
	trap 'rm -rf "$stage"' EXIT
	cp -R "$app" "$stage/"
	ln -s /Applications "$stage/Applications"
	hdiutil create -quiet -volname "Pastille $version" -srcfolder "$stage" -ov -format UDZO "$dmg" ||
		fail "hdiutil could not make $dmg"
	rm -rf "$stage"
	echo "$commit" >"$stamp"
fi
if $checks; then
	# The app about to be published, opened once: it has to simulate a second
	# of a circuit before it exits, as `just desktop-smoke` asks of the
	# machine's own build.
	PASTILLE_SMOKE=1 "$built/macos/Pastille.app/Contents/MacOS/pastille-desktop" ||
		fail "the universal app did not simulate"
fi

if $linux; then
	for arch in amd64 arm64; do
		step "Linux $arch"
		built_linux=desktop/target/linux-$arch
		# Arch Linux is x86_64 only: its package is amd64's.
		kinds=(deb AppImage)
		[[ $arch == amd64 ]] && kinds+=(pkg.tar.zst)
		complete=true
		for kind in "${kinds[@]}"; do
			[[ -f "$built_linux/Pastille_${version}_${arch}.$kind" ]] || complete=false
		done
		if [[ -f "$built_linux/.commit" && "$(cat "$built_linux/.commit")" == "$commit" ]] && $complete && ! $rebuild; then
			echo "the $arch packages of $short already built: reused (--rebuild to build them again)"
		else
			scripts/linux.sh --arch "$arch" --bundles "deb,appimage$([[ $arch == amd64 ]] && echo ,pacman)"
		fi
	done
fi

if $windows; then
	export XWIN_ACCEPT_LICENSE=${XWIN_ACCEPT_LICENSE:-1}
	for arch in x64 arm64; do
		step "Windows $arch"
		target=$([[ $arch == x64 ]] && echo x86_64 || echo aarch64)-pc-windows-msvc
		installer=desktop/target/$target/release/bundle/nsis/Pastille_${version}_${arch}-setup.exe
		winstamp=desktop/target/$target/release/bundle/nsis/.commit
		if [[ -f "$installer" && -f "$winstamp" && "$(cat "$winstamp")" == "$commit" ]] && ! $rebuild; then
			echo "$(basename "$installer") of $short already built: reused (--rebuild to build it again)"
		else
			# The web app the installer carries: this commit's, for the root.
			just build
			WINDOWS_ARCH=$arch scripts/windows.sh build
			[[ -f "$installer" ]] || fail "no $installer after scripts/windows.sh"
			echo "$commit" >"$winstamp"
		fi
	done
fi

out=desktop/target/release-$version
rm -rf "$out"
mkdir -p "$out"
cp "$dmg" "$out/Pastille_${version}_macos-universal.dmg"
if $linux; then
	for arch in amd64 arm64; do
		cp "desktop/target/linux-$arch/Pastille_${version}_${arch}."{deb,AppImage} "$out/"
	done
	cp "desktop/target/linux-amd64/Pastille_${version}_amd64.pkg.tar.zst" "$out/"
fi
if $windows; then
	cp "desktop/target/x86_64-pc-windows-msvc/release/bundle/nsis/Pastille_${version}_x64-setup.exe" \
		"$out/Pastille_${version}_windows-x64-setup.exe"
	cp "desktop/target/aarch64-pc-windows-msvc/release/bundle/nsis/Pastille_${version}_arm64-setup.exe" \
		"$out/Pastille_${version}_windows-arm64-setup.exe"
fi
(cd "$out" && for f in Pastille_*; do [[ $f == *.sha256 ]] || shasum -a 256 "$f" >"$f.sha256"; done)
files=("$out"/Pastille_*)

changes=""
if [[ -n "$notes_file" ]]; then
	changes=$(cat "$notes_file")
elif [[ -n "$notes" ]]; then
	changes=$notes
else
	changes=$(scripts/changelog.sh)
fi
notes="$changes

### Opening the Mac app

The app is not signed, so macOS refuses to open a copy that was downloaded. Drag it to
Applications, then remove the quarantine flag once:

\`\`\`sh
xattr -dr com.apple.quarantine /Applications/Pastille.app
\`\`\`

or, after a first refused launch, choose **Open Anyway** in System Settings › Privacy & Security."

if $windows; then
	notes="$notes

### Installing on Windows

The installer is not signed either: SmartScreen stops it the first time, and **More info ›
Run anyway** installs it. \`windows-x64\` is for most PCs, \`windows-arm64\` for ARM ones."
fi
if $linux; then
	notes="$notes

### Installing on Linux

\`amd64\` is for most PCs, \`arm64\` for ARM boards such as a Raspberry Pi 4 or 5.

- Debian, Ubuntu and their derivatives: \`sudo apt install ./Pastille_${version}_<arch>.deb\`
- Arch Linux and its derivatives: \`sudo pacman -U Pastille_${version}_amd64.pkg.tar.zst\`
- Any other distribution: \`chmod +x Pastille_${version}_<arch>.AppImage\`, then run it
  (it needs libfuse2: \`sudo apt install libfuse2\`, or \`libfuse2t64\` on Ubuntu 24.04)."
fi
notes="$notes

Pastille $version, built from $short."

step "Files"
for f in "${files[@]}"; do echo "  $(basename "$f") ($(du -h "$f" | cut -f1))"; done
if $dry; then
	step "Dry run: nothing published"
	printf 'Would publish %s to %s%s, notes:\n\n%s\n' "$tag" "$to" "$($pages && echo ', and the web app to Pages')" "$notes"
	exit 0
fi

step "Publishing"
gh release create "$tag" -R "$to" --target "$commit" --title "Pastille $version" --notes "$notes" "${files[@]}"

if $tag_it; then
	step "Tag"
	if git rev-parse -q --verify "refs/tags/$tag" >/dev/null; then
		echo "$tag already tagged here: left as is"
	else
		# Verbatim: the changelog's headings start with #, which git would
		# otherwise take for comments.
		git tag -a --cleanup=verbatim "$tag" -m "Pastille $version" -m "$changes"
		echo "$tag on $short, in this clone (GitHub has its own, from the release)"
	fi
fi

if $pages; then
	step "GitHub Pages"
	just pages-publish
fi

step "Done"
echo "Pastille $version: https://github.com/$to/releases/tag/$tag"
