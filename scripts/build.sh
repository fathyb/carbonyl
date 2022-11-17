#!/usr/bin/env bash

set -eo pipefail

target="$1"

shift

scripts/ninja.sh "$target"
electron/src/electron/script/strip-binaries.py -d "electron/src/out/$target" "$@"
ninja -C "electron/src/out/$target" electron:electron_dist_zip

