#!/usr/bin/env bash

set -eo pipefail

export CHROMIUM_ROOT="$CARBONYL_ROOT/chromium"
export CHROMIUM_SRC="$CHROMIUM_ROOT/src"
export DEPOT_TOOLS_ROOT="$CHROMIUM_ROOT/depot_tools"
export PATH="$PATH:$DEPOT_TOOLS_ROOT"

if [ ! -d "$DEPOT_TOOLS_ROOT" ]; then
    echo "depot_tools not found, fetching submodule.."

    git -C "$CARBONYL_ROOT" submodule update --init --recursive
fi
