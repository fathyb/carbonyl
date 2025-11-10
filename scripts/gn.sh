#!/usr/bin/env bash
set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- "$(pwd)")
export INSTALL_DEPOT_TOOLS="true"

source "$CARBONYL_ROOT/scripts/env.sh"

# Validate CHROMIUM_SRC directory exists
if [ ! -d "$CHROMIUM_SRC" ]; then
    echo "Error: CHROMIUM_SRC directory $CHROMIUM_SRC does not exist" >&2
    echo "Run gclient sync first to set up the Chromium source" >&2
    exit 1
fi

(
    cd "$CHROMIUM_SRC" &&
    gn "$@"
)
