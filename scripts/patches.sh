#!/usr/bin/env bash

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- "$(pwd)")

source "$CARBONYL_ROOT/scripts/env.sh"

cd "$CHROMIUM_SRC"

chromium_upstream="111.0.5511.1"
skia_upstream="486deb23bc2a4d3d09c66fef52c2ad64d8b4f761"

if [[ "$1" == "apply" ]]; then
    if [[ -d carbonyl ]]; then
        git add -A carbonyl
    fi

    echo "Stashing Chromium changes.."

    git stash
    git checkout "$chromium_upstream"
    echo "Applying Chromium patches.."
    git apply < ../../src/chromium.patch

    cd third_party/skia
    echo "Stashing Chromium changes.."
    git stash
    git checkout "$skia_upstream"
    echo "Applying Skia patches.."
    git apply < ../../../../src/skia.patch

    echo "Patches successfully applied"
elif [[ "$1" == "save" ]]; then
    if [[ -d carbonyl ]]; then
        git add -A carbonyl
    fi

    echo "Updating Chromium patch.."
    git diff "$chromium_upstream" > ../../src/chromium.patch

    echo "Updating Skia patch.."
    cd third_party/skia
    git diff "$skia_upstream" > ../../../../src/skia.patch

    echo "Patches successfully updated"
else
    echo "Unknown argument: $1"

    exit 2
fi
