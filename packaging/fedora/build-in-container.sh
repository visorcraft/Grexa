#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 VisorCraft LLC
# SPDX-License-Identifier: GPL-3.0-only
#
# Host-side wrapper that builds the grexa Fedora 44 RPM inside a
# podman container. Use this instead of running rpmbuild on the host
# when you need an RPM that installs on Fedora 44 - the on-host
# rpmbuild path links against whatever Qt the host has (Qt 6.11 on
# CachyOS), and Qt's AOT-compiled QML binds to private symbols that
# only exist on the matching Qt minor version. See
# Containerfile.fedora44 for the longer explanation.
#
# Output:
#   packaging/fedora/_rpmbuild-fedora44/RPMS/x86_64/grexa-<ver>-1.fc44.x86_64.rpm
#
# First run builds the image (~2-3 min). Subsequent runs reuse it.
# Pass --rebuild-image to force a fresh image build (after editing
# the Containerfile, or to pick up newer Fedora base updates).

set -euo pipefail

self_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${self_dir}/../.." && pwd)"
image_tag="grexa-rpm-fedora44"
target_volume="grexa-fedora44-target"
cargo_volume="grexa-fedora44-cargo"
output_dir="${self_dir}/_rpmbuild-fedora44/RPMS/x86_64"

rebuild_image=false
for arg in "$@"; do
    case "$arg" in
        --rebuild-image) rebuild_image=true ;;
        -h|--help)
            sed -n '2,/^$/p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "Unknown argument: $arg" >&2
            exit 2
            ;;
    esac
done

if ! command -v podman >/dev/null 2>&1; then
    echo "podman is required but not installed." >&2
    echo "Install: sudo pacman -S podman   (Arch/CachyOS)" >&2
    exit 1
fi

cd "$self_dir"

# Build the image if it doesn't exist or if --rebuild-image was passed.
if $rebuild_image || ! podman image exists "$image_tag"; then
    echo "==> Building container image (${image_tag})"
    podman build -f Containerfile.fedora44 -t "$image_tag" .
else
    echo "==> Reusing existing container image (${image_tag})"
    echo "    Pass --rebuild-image to force a rebuild after Containerfile or base-image changes."
fi

mkdir -p "$output_dir"

echo "==> Building RPM"
echo "    Source : ${repo_root}"
echo "    Output : ${output_dir}"
echo

# --security-opt label=disable bypasses SELinux relabeling on the bind
# mount. CachyOS doesn't run SELinux, but the flag keeps the script
# portable to hosts that do without rewriting file labels on the
# user's repo.
# --userns=keep-id maps the container's `builder` user (uid 1000) to
# the host's user (uid 1000) so the bind-mounted source is readable
# without --rootless quirks.
podman run --rm \
    --security-opt label=disable \
    --userns=keep-id \
    -v "${repo_root}:/src:ro" \
    -v "${output_dir}:/output" \
    -v "${target_volume}:/home/builder/target-cache" \
    -v "${cargo_volume}:/home/builder/.cargo" \
    "$image_tag"

echo
echo "==> Built RPMs:"
ls -l "$output_dir"/*.rpm 2>/dev/null || {
    echo "    (none - the container build did not produce an RPM)" >&2
    exit 1
}
