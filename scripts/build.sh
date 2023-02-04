#!/usr/bin/env bash

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- "$(pwd)")

source "$CARBONYL_ROOT/scripts/env.sh"

triple=$("$CARBONYL_ROOT/scripts/platform-triple.sh" "$2")

export MACOSX_DEPLOYMENT_TARGET=10.13
cargo build --target "$triple" --release

cd "$CHROMIUM_SRC/out/$1"

ninja headless:headless_shell
