#!/usr/bin/env bash
# Runs inside the Linux release container (see Containerfile).
#   /src   - the host repository, mounted read-only
#   /out   - receives the deb, rpm and AppImage bundles
#   /cache - persistent volume for Cargo registry, build target and npm cache
set -euo pipefail

cargo_registry_cache=/cache/cargo-registry
export CARGO_TARGET_DIR=/cache/target
export npm_config_cache=/cache/npm
mkdir -p "$cargo_registry_cache" "$CARGO_TARGET_DIR" "$npm_config_cache"
ln -sfn "$cargo_registry_cache" "${CARGO_HOME}/registry"

# Bundles from earlier runs persist in the target volume; clear them so only
# this build's artifacts are copied out.
bundle_dir="${CARGO_TARGET_DIR}/release/bundle"
rm -rf "$bundle_dir"

# Build from a fresh copy of the working tree: tracked plus untracked,
# non-ignored files only, so host node_modules/target never leak in.
git config --global --add safe.directory /src
rm -rf /build
mkdir -p /build
git -C /src ls-files -z --cached --others --exclude-standard \
    | rsync -a --from0 --files-from=- --ignore-missing-args /src/ /build/

cd /build
npm ci
npm run tauri:build

# The deb/rpm desktop template renders a Hidden stub (the real launcher ships
# as info.brumley.rustyg6.desktop), and the AppImage reuses that template.
# tauri.conf.json bundle.linux.appimage.files overrides the stub by the
# bundler's generated filename; fail if that drifts and the AppImage's
# top-level launcher is the hidden stub.
for appdir in "${bundle_dir}"/appimage/*.AppDir; do
    for launcher in "$appdir"/*.desktop; do
        if grep -qx 'Hidden=true' "$launcher"; then
            echo "AppImage launcher check failed: $launcher is the hidden bundler stub." >&2
            echo "Update bundle.linux.appimage.files in tauri.conf.json to the bundler's generated desktop filename." >&2
            exit 1
        fi
        echo "AppImage launcher check passed: $(basename "$launcher")"
    done
done

# linuxdeploy bundles this container's libwayland (1.20, Ubuntu 22.04), but the
# GPU stack (Mesa's libEGL and drivers) always comes from the host. A newer
# host Mesa calls libwayland symbols 1.20 lacks (wl_display_create_queue_with_name,
# 1.23+), so EGL fails with EGL_BAD_PARAMETER and the window stays white
# (GitHub issue #1, CachyOS). libwayland keeps a stable ABI and every GTK
# desktop installs it, so the AppImage must use the host's copy: remove the
# bundled one and repack the AppImage with its original runtime.
strip_bundled_wayland() {
    local appimage="$1"
    local work
    work="$(mktemp -d)"
    # The squashfs payload starts right after the runtime's ELF section
    # header table: e_shoff (offset 0x28) + e_shentsize * e_shnum (0x3A, 0x3C).
    local section_headers_offset section_header_size section_header_count payload_offset
    section_headers_offset=$(od -An -t u8 -j 40 -N 8 "$appimage" | tr -d ' ')
    section_header_size=$(od -An -t u2 -j 58 -N 2 "$appimage" | tr -d ' ')
    section_header_count=$(od -An -t u2 -j 60 -N 2 "$appimage" | tr -d ' ')
    payload_offset=$((section_headers_offset + section_header_size * section_header_count))

    unsquashfs -quiet -no-progress -offset "$payload_offset" -dest "${work}/root" "$appimage"
    rm -f "${work}"/root/usr/lib/libwayland-{client,cursor,egl,server}.so*

    head -c "$payload_offset" "$appimage" > "${work}/repacked.AppImage"
    mksquashfs "${work}/root" "${work}/payload.squashfs" \
        -quiet -no-progress -root-owned -noappend -comp zstd -b 131072 -mkfs-time 0
    cat "${work}/payload.squashfs" >> "${work}/repacked.AppImage"
    chmod 0755 "${work}/repacked.AppImage"
    mv "${work}/repacked.AppImage" "$appimage"
    rm -rf "$work"

    if unsquashfs -lls -offset "$payload_offset" "$appimage" | grep -q 'usr/lib/libwayland-'; then
        echo "AppImage libwayland check failed: $(basename "$appimage") still bundles libwayland." >&2
        exit 1
    fi
    echo "AppImage libwayland check passed: $(basename "$appimage") uses the host's libwayland."
}

for appimage in "${bundle_dir}"/appimage/*.AppImage; do
    strip_bundled_wayland "$appimage"
done

for bundle in deb rpm appimage; do
    rm -rf "/out/${bundle}"
    mkdir -p "/out/${bundle}"
done
cp "${bundle_dir}"/deb/*.deb /out/deb/
cp "${bundle_dir}"/rpm/*.rpm /out/rpm/
cp "${bundle_dir}"/appimage/*.AppImage /out/appimage/

echo "Linux release bundles written to /out (glibc $(ldd --version | head -n1 | awk '{print $NF}'))"
