#!/usr/bin/env bash

set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- "$(pwd)")

source "$CARBONYL_ROOT/scripts/env.sh"

target="$1"

if [ -z "$target" ]; then
    echo "Error: target argument is required"
    echo "Usage: $0 <target> [args...]"
    exit 1
fi

shift

if [ ! -f "$CHROMIUM_SRC/out/$target/headless_shell" ]; then
    echo "Error: headless_shell not found at $CHROMIUM_SRC/out/$target/headless_shell"
    echo "Build it first with scripts/build.sh"
    exit 1
fi

"$CHROMIUM_SRC/out/$target/headless_shell" "$@"
