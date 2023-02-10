#!/usr/bin/env bash

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- "$(pwd)")

source "$CARBONYL_ROOT/scripts/env.sh"

cd "$CHROMIUM_SRC"

chromium_upstream="92da8189788b1b373cbd3348f73d695dfdc521b6"
skia_upstream="486deb23bc2a4d3d09c66fef52c2ad64d8b4f761"

if [[ "$1" == "apply" ]]; then
    echo "Stashing Chromium changes.."
    git add -A .
    git stash

    echo "Applying Chromium patches.."
    git checkout "$chromium_upstream"
    for patch in "$CARBONYL_ROOT/chromium/patches/chromium"/*.patch; do
        git am --committer-date-is-author-date "$patch"
    done
    "$CARBONYL_ROOT/scripts/restore-mtime.sh" "$chromium_upstream"

    echo "Stashing Skia changes.."
    cd third_party/skia
    git add -A .
    git stash

    echo "Applying Skia patches.."
    git checkout "$skia_upstream"
    for patch in "$CARBONYL_ROOT/chromium/patches/skia"/*.patch; do
        git am --committer-date-is-author-date "$patch"
    done
    "$CARBONYL_ROOT/scripts/restore-mtime.sh" "$skia_upstream"

    echo "Patches successfully applied"
elif [[ "$1" == "save" ]]; then
    if [[ -d carbonyl ]]; then
        git add -A carbonyl
    fi

    echo "Updating Chromium patches.."
    git format-patch --output-directory "$CARBONYL_ROOT/chromium/patches/chromium" "$chromium_upstream"

    echo "Updating Skia patches.."
    cd third_party/skia
    git format-patch --output-directory "$CARBONYL_ROOT/chromium/patches/skia" "$skia_upstream"

    echo "Patches successfully updated"
else
    echo "Unknown argument: $1"

    exit 2
fi
