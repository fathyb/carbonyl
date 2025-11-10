#!/usr/bin/env bash
set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- "$(pwd)")
export INSTALL_DEPOT_TOOLS="true"

source "$CARBONYL_ROOT/scripts/env.sh"

# Validate CHROMIUM_ROOT directory exists
if [ ! -d "$CHROMIUM_ROOT" ]; then
    echo "Error: CHROMIUM_ROOT directory $CHROMIUM_ROOT does not exist" >&2
    echo "Create the directory first: mkdir -p $CHROMIUM_ROOT" >&2
    exit 1
fi

(
    cd "$CHROMIUM_ROOT" &&
    gclient "$@"
)
