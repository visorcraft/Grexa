#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 VisorCraft LLC
# SPDX-License-Identifier: GPL-3.0-only
#
# Entrypoint that runs inside the Fedora 44 build container.
# Expects:
#   /src              read-only bind mount of the grexa repo root
#   /output           writable bind mount where the finished RPM lands
#   /home/builder/.cargo (volume)        cargo cache
#   /home/builder/target-cache (volume)  container-private cargo target dir
#
# Reads the workspace version from /src/Cargo.toml at run time so this
# script does not need to be touched on version bumps.

set -euo pipefail

# Read the workspace package version from Cargo.toml.
version="$(awk -F'"' '
    /^\[workspace\.package\]/ { in_section = 1; next }
    in_section && /^\[/        { exit }
    in_section && $1 ~ /^version[[:space:]]*=/ { print $2; exit }
' /src/Cargo.toml)"

if [ -z "${version:-}" ]; then
    echo "ERROR: could not parse workspace version from /src/Cargo.toml" >&2
    exit 1
fi

echo "==> Building grexa ${version} RPM inside Fedora 44 container"
echo "    Qt version on this host:"
rpm -q qt6-qtbase qt6-qtdeclarative | sed 's/^/      /'

# Stage a clean copy of the source. Bind mount is read-only; rpmbuild
# needs to write to the spec's _sourcedir and we want to be sure the
# host tree is never modified.
work="$(mktemp -d /tmp/grexa-build.XXXXXX)"
trap 'rm -rf "$work"' EXIT

# Copy source minus host build artefacts and stray test scratch. The
# container's `builder` user can't read host-owned files in target/ or
# 0600-mode test fixtures; cp -a barfs the whole copy on the first
# unreadable file. tar handles permission errors per-file via
# --ignore-failed-read so the source copy proceeds even when host has
# locked-down test fixtures or restricted target/ artefacts.
mkdir -p "$work/repo"
tar -C /src --exclude='./target' --exclude='./.flatpak-builder' \
    --ignore-failed-read -cf - . 2>/dev/null \
    | tar -C "$work/repo" -xf - 2>/dev/null

# Point cargo at the container-private target dir so host builds and
# container builds don't fight over the same compiled artefacts.
export CARGO_TARGET_DIR=/home/builder/target-cache
export CARGO_HOME=/home/builder/.cargo

# Disable the workspace's mold + sccache acceleration inside the
# container. The .cargo/config.toml wires rustc-wrapper=sccache and
# fuse-ld=mold, but sccache's CC wrapper breaks cxx-qt C++ codegen.
# Cargo honours these env vars over config.toml.
export CARGO_BUILD_RUSTC_WRAPPER=""
export RUSTFLAGS=""
export CC=gcc
export CXX=g++

cd "$work/repo"

# Spec uses %setup -q -n %{name}-%{version} which expects a tarball
# at <sourcedir>/grexa-<version>.tar.gz unpacking to a top-level
# grexa-<version>/ dir.
git_available=true
if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    git_available=false
fi

if $git_available; then
    git archive --format=tar.gz \
        --prefix="grexa-${version}/" \
        --output="packaging/fedora/grexa-${version}.tar.gz" HEAD
else
    # Fallback for source trees mounted without .git (rare). Tar the
    # working copy minus build/output artefacts.
    tar --transform "s|^\.|grexa-${version}|" \
        --exclude=./target --exclude=./packaging/fedora/_rpmbuild \
        --exclude=./.git \
        -czf "packaging/fedora/grexa-${version}.tar.gz" .
fi

cd packaging/fedora

# Sync the spec's Version field to Cargo.toml if they've drifted.
spec_version="$(awk '/^Version:/ {print $2; exit}' grexa.spec)"
if [ "$spec_version" != "$version" ]; then
    echo "    Note: grexa.spec Version (${spec_version}) lags Cargo.toml (${version}); using Cargo.toml"
    sed -i "s/^Version:.*/Version:        ${version}/" grexa.spec
fi

rpmbuild --define "_topdir $(pwd)/_rpmbuild" \
         --define "_sourcedir $(pwd)" \
         -bb grexa.spec

# Copy the RPMs into the host-mounted output dir.
mkdir -p /output
find _rpmbuild/RPMS -type f -name '*.rpm' -exec cp -v {} /output/ \;

echo
echo "==> Done. RPM(s) copied to /output (host-mounted):"
ls -l /output/*.rpm 2>/dev/null || echo "    (no RPMs found - build may have failed)"
