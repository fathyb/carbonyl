#!/usr/bin/env bash

set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- "$(pwd)")

source "$CARBONYL_ROOT/scripts/env.sh"

if [ -z "$1" ]; then
    echo "Error: Missing argument"
    echo "Usage: $0 <apply|save>"
    echo "  apply - Apply patches to Chromium, Skia, and WebRTC"
    echo "  save  - Save current changes as patches"
    exit 1
fi

if [ ! -d "$CHROMIUM_SRC" ]; then
    echo "Error: CHROMIUM_SRC directory not found: $CHROMIUM_SRC"
    exit 1
fi

cd "$CHROMIUM_SRC"

chromium_upstream="92da8189788b1b373cbd3348f73d695dfdc521b6"
skia_upstream="486deb23bc2a4d3d09c66fef52c2ad64d8b4f761"
webrtc_upstream="727080cbacd58a2f303ed8a03f0264fe1493e47a"

if [[ "$1" == "apply" ]]; then
    # Check that patch directories exist
    if [ ! -d "$CARBONYL_ROOT/chromium/patches/chromium" ] || [ -z "$(ls -A "$CARBONYL_ROOT/chromium/patches/chromium" 2>/dev/null)" ]; then
        echo "Error: Chromium patches directory not found or empty: $CARBONYL_ROOT/chromium/patches/chromium"
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
    if [ ! -d "$CARBONYL_ROOT/chromium/patches/skia" ] || [ -z "$(ls -A "$CARBONYL_ROOT/chromium/patches/skia" 2>/dev/null)" ]; then
        echo "Warning: Skia patches directory not found or empty, skipping"
    else
        git checkout "$skia_upstream"
        git am --committer-date-is-author-date "$CARBONYL_ROOT/chromium/patches/skia"/*
        "$CARBONYL_ROOT/scripts/restore-mtime.sh" "$skia_upstream"
    fi

    echo "Stashing WebRTC changes.."
    cd "$CHROMIUM_SRC/third_party/webrtc"

    if [ ! -d "$CHROMIUM_SRC/third_party/webrtc" ]; then
        echo "Warning: WebRTC directory not found, skipping"
    else
        git add -A .
        git stash

        echo "Applying WebRTC patches.."
        if [ ! -d "$CARBONYL_ROOT/chromium/patches/webrtc" ] || [ -z "$(ls -A "$CARBONYL_ROOT/chromium/patches/webrtc" 2>/dev/null)" ]; then
            echo "Warning: WebRTC patches directory not found or empty, skipping"
        else
            git checkout "$webrtc_upstream"
            git am --committer-date-is-author-date "$CARBONYL_ROOT/chromium/patches/webrtc"/*
            "$CARBONYL_ROOT/scripts/restore-mtime.sh" "$webrtc_upstream"
        fi
    fi

    echo "Patches successfully applied"
elif [[ "$1" == "save" ]]; then
    if [[ -d carbonyl ]]; then
        git add -A carbonyl
    fi

    echo "Updating Chromium patches.."
    mkdir -p "$CARBONYL_ROOT/chromium/patches/chromium"
    rm -rf "$CARBONYL_ROOT/chromium/patches/chromium"
    git format-patch --no-signature --output-directory "$CARBONYL_ROOT/chromium/patches/chromium" "$chromium_upstream"

    echo "Updating Skia patches.."
    if [ -d "$CHROMIUM_SRC/third_party/skia" ]; then
        cd "$CHROMIUM_SRC/third_party/skia"
        mkdir -p "$CARBONYL_ROOT/chromium/patches/skia"
        rm -rf "$CARBONYL_ROOT/chromium/patches/skia"
        git format-patch --no-signature --output-directory "$CARBONYL_ROOT/chromium/patches/skia" "$skia_upstream"
    else
        echo "Warning: Skia directory not found, skipping"
    fi

    echo "Updating WebRTC patches.."
    if [ -d "$CHROMIUM_SRC/third_party/webrtc" ]; then
        cd "$CHROMIUM_SRC/third_party/webrtc"
        mkdir -p "$CARBONYL_ROOT/chromium/patches/webrtc"
        rm -rf "$CARBONYL_ROOT/chromium/patches/webrtc"
        git format-patch --no-signature --output-directory "$CARBONYL_ROOT/chromium/patches/webrtc" "$webrtc_upstream"
    else
        echo "Warning: WebRTC directory not found, skipping"
    fi

    echo "Patches successfully updated"
else
    echo "Error: Unknown argument: $1"
    echo "Usage: $0 <apply|save>"
    exit 1
fi
