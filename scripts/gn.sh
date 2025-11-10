#!/usr/bin/env bash

set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- "$(pwd)")
export INSTALL_DEPOT_TOOLS="true"

source "$CARBONYL_ROOT/scripts/env.sh"

if [ ! -d "$CHROMIUM_SRC" ]; then
    echo "Error: CHROMIUM_SRC directory not found: $CHROMIUM_SRC"
    exit 1
fi

(
    cd "$CHROMIUM_SRC" &&
    gn "$@"
)
