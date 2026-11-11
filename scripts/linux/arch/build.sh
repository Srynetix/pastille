#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Runs in an archlinux:base-devel container, started by scripts/linux.sh:
# /in the .deb and the licences, /pkg this folder (read only), /out the Arch
# package. makepkg refuses to run as root, so a user of its own builds it.
set -euo pipefail

useradd -m builder
work=/home/builder/pkg
mkdir -p "$work"
cp /pkg/PKGBUILD "$work/"
cp /in/pastille.deb /in/LICENSE-MIT /in/LICENSE-APACHE "$work/"
chown -R builder: "$work"
# --nodeps: its dependencies are what installing it pulls in, not what
# repackaging it needs.
su builder -c "cd $work && PASTILLE_PKGVER='$PASTILLE_PKGVER' makepkg --nodeps --noconfirm" >&2
cp "$work"/pastille-*.pkg.tar.zst /out/
