#!/usr/bin/env bash

set -eo pipefail

if [ -z "$CARBONYL_ROOT" ]; then
    echo "CARBONYL_ROOT should be defined"

    exit 2
fi

if [ -z "$CHROMIUM_ROOT" ]; then
    export CHROMIUM_ROOT="$CARBONYL_ROOT/chromium"
fi
if [ -z "$CHROMIUM_SRC" ]; then
    export CHROMIUM_SRC="$CHROMIUM_ROOT/src"
fi
if [ -z "$DEPOT_TOOLS_ROOT" ]; then
    export DEPOT_TOOLS_ROOT="$CHROMIUM_ROOT/depot_tools"
fi

export PATH="$PATH:$DEPOT_TOOLS_ROOT"

if [ "$INSTALL_DEPOT_TOOLS" = "true"] && [ ! -f "$DEPOT_TOOLS_ROOT/README.md" ]; then
    echo "depot_tools not found, fetching submodule.."

    git -C "$CARBONYL_ROOT" submodule update --init --recursive
fi
