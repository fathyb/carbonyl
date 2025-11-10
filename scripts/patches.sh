#!/usr/bin/env bash
set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- "$(pwd)")

source "$CARBONYL_ROOT/scripts/env.sh"

# Validate git is available
if ! command -v git &> /dev/null; then
    echo "Error: git is required but not installed" >&2
    exit 1
fi

# Validate CHROMIUM_SRC exists
if [ ! -d "$CHROMIUM_SRC" ]; then
    echo "Error: CHROMIUM_SRC directory $CHROMIUM_SRC does not exist" >&2
    exit 1
fi

cd "$CHROMIUM_SRC"

chromium_upstream="92da8189788b1b373cbd3348f73d695dfdc521b6"
skia_upstream="486deb23bc2a4d3d09c66fef52c2ad64d8b4f761"
webrtc_upstream="727080cbacd58a2f303ed8a03f0264fe1493e47a"

if [[ "$1" == "apply" ]]; then
    # Validate patch directories exist
    if [ ! -d "$CARBONYL_ROOT/chromium/patches/chromium" ]; then
        echo "Error: Chromium patches directory not found" >&2
        exit 1
    fi
    if [ ! -d "$CARBONYL_ROOT/chromium/patches/skia" ]; then
        echo "Error: Skia patches directory not found" >&2
        exit 1
    fi
    if [ ! -d "$CARBONYL_ROOT/chromium/patches/webrtc" ]; then
        echo "Error: WebRTC patches directory not found" >&2
        exit 1
    fi

    echo "Stashing Chromium changes.."
    git add -A .
    git stash

    echo "Applying Chromium patches.."
    git checkout "$chromium_upstream"
    git am --committer-date-is-author-date "$CARBONYL_ROOT/chromium/patches/chromium"/*
    "$CARBONYL_ROOT/scripts/restore-mtime.sh" "$chromium_upstream"

    echo "Stashing Skia changes.."
    cd "$CHROMIUM_SRC/third_party/skia"
    git add -A .
    git stash

    echo "Applying Skia patches.."
    git checkout "$skia_upstream"
    git am --committer-date-is-author-date "$CARBONYL_ROOT/chromium/patches/skia"/*
    "$CARBONYL_ROOT/scripts/restore-mtime.sh" "$skia_upstream"

    echo "Stashing WebRTC changes.."
    cd "$CHROMIUM_SRC/third_party/webrtc"
    git add -A .
    git stash

    echo "Applying WebRTC patches.."
    git checkout "$webrtc_upstream"
    git am --committer-date-is-author-date "$CARBONYL_ROOT/chromium/patches/webrtc"/*
    "$CARBONYL_ROOT/scripts/restore-mtime.sh" "$webrtc_upstream"

    echo "✓ Patches successfully applied"
elif [[ "$1" == "save" ]]; then
    if [[ -d carbonyl ]]; then
        git add -A carbonyl
    fi

    echo "Updating Chromium patches.."
    rm -rf "$CARBONYL_ROOT/chromium/patches/chromium"
    mkdir -p "$CARBONYL_ROOT/chromium/patches/chromium"
    git format-patch --no-signature --output-directory "$CARBONYL_ROOT/chromium/patches/chromium" "$chromium_upstream"

    echo "Updating Skia patches.."
    cd "$CHROMIUM_SRC/third_party/skia"
    rm -rf "$CARBONYL_ROOT/chromium/patches/skia"
    mkdir -p "$CARBONYL_ROOT/chromium/patches/skia"
    git format-patch --no-signature --output-directory "$CARBONYL_ROOT/chromium/patches/skia" "$skia_upstream"

    echo "Updating WebRTC patches.."
    cd "$CHROMIUM_SRC/third_party/webrtc"
    rm -rf "$CARBONYL_ROOT/chromium/patches/webrtc"
    mkdir -p "$CARBONYL_ROOT/chromium/patches/webrtc"
    git format-patch --no-signature --output-directory "$CARBONYL_ROOT/chromium/patches/webrtc" "$webrtc_upstream"

    echo "✓ Patches successfully updated"
else
    echo "Error: Unknown argument: $1" >&2
    echo "Usage: $0 {apply|save}" >&2
    exit 1
fi
