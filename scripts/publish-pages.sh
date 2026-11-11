#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Publishes app/dist, as `just pages` built it, to the gh-pages branch of
# origin, which GitHub Pages serves at https://srynetix.github.io/pastille/
# (ADR 0021). Built and checked here, as everything is (ADR 0014): the branch
# only ever receives a build that passed `just pages` on this machine.
#
# The branch holds the build and nothing else, one commit, replaced each time:
# its history would be the history of minified bundles, which nobody reads.
set -euo pipefail

root=$(git rev-parse --show-toplevel)
dist="$root/app/dist"
base=/pastille/

# A build for the root would publish an app whose every path misses.
if ! grep -q "src=\"${base}assets/" "$dist/index.html"; then
	echo "app/dist was not built for ${base}: run \`just pages\` first." >&2
	exit 1
fi

# What is published should be what is committed, so the page can say which
# commit it is.
if [ -n "$(git -C "$root" status --porcelain)" ]; then
	echo "Uncommitted changes: commit them first, so the page is a commit." >&2
	exit 1
fi

commit=$(git -C "$root" rev-parse --short HEAD)
remote=$(git -C "$root" remote get-url origin)
# Pages serves the build from the repository it is pushed to, and its paths
# are /pastille/'s: any other origin would publish an app whose every path misses.
case "$remote" in
	*[:/]Srynetix/pastille | *[:/]Srynetix/pastille.git) ;;
	*)
		echo "origin is $remote, not Srynetix/pastille: point it there first." >&2
		exit 1
		;;
esac
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

cp -R "$dist"/. "$work"
# GitHub Pages would otherwise run the files through Jekyll, which drops
# anything whose name starts with an underscore.
touch "$work/.nojekyll"

git -C "$work" init --quiet --initial-branch=gh-pages
git -C "$work" add --all
git -C "$work" commit --quiet --message "Pastille, built from ${commit}"
git -C "$work" push --force --quiet "$remote" gh-pages
echo "Published ${commit} to gh-pages: https://srynetix.github.io/pastille/"
