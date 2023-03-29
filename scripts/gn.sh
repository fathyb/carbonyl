#!/usr/bin/env bash

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- "$(pwd)")
export INSTALL_DEPOT_TOOLS="true"

source "$CARBONYL_ROOT/scripts/env.sh"

(
    cd "$CHROMIUM_SRC" &&
    gn "$@"
)
