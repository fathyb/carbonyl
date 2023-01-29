#!/usr/bin/env bash

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

source "$CARBONYL_ROOT/scripts/env.sh"

target="$1"
shift

"$CHROMIUM_SRC/out/$target/headless_shell" "$@"
