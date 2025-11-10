#!/usr/bin/env bash
set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- "$(pwd)")

source "$CARBONYL_ROOT/scripts/env.sh"

target="$1"

# Validate target argument
if [ -z "$target" ]; then
    echo "Error: target argument is required" >&2
    echo "Usage: $0 <target> [additional-args]" >&2
    exit 1
fi

shift

# Validate headless_shell exists
if [ ! -f "$CHROMIUM_SRC/out/$target/headless_shell" ]; then
    echo "Error: headless_shell not found at $CHROMIUM_SRC/out/$target/headless_shell" >&2
    echo "Run scripts/build.sh first" >&2
    exit 1
fi

"$CHROMIUM_SRC/out/$target/headless_shell" "$@"
