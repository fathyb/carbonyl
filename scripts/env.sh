#!/usr/bin/env bash

set -eo pipefail

if [ -z "$CHROMIUM_ROOT" ]; then
    export CHROMIUM_ROOT="$CARBONYL_ROOT/chromium"
fi
if [ -z "$DEPOT_TOOLS_ROOT" ]; then
    export DEPOT_TOOLS_ROOT="$CHROMIUM_ROOT/depot_tools"
fi

export CHROMIUM_SRC="$CHROMIUM_ROOT/src"
export PATH="$PATH:$DEPOT_TOOLS_ROOT"

if [ ! -f "$DEPOT_TOOLS_ROOT/README.md" ] && [ -z "$SKIP_DEPOT_TOOLS" ]; then
    echo "depot_tools not found, fetching submodule.."

    git -C "$CARBONYL_ROOT" submodule update --init --recursive
fi
