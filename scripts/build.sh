#!/usr/bin/env bash

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- "$(pwd)")
export INSTALL_DEPOT_TOOLS="true"

cd "$CARBONYL_ROOT"
source scripts/env.sh

if [[ $# -lt 2 ]]; then
    echo "Error: Expected at least two arguments."
    exit 1
fi

target="$1"; shift
cpu="$2"; shift

triple=$(scripts/platform-triple.sh "$cpu")

[ -z "$CARBONYL_SKIP_CARGO_BUILD" ] && {
    [ -z "$MACOSX_DEPLOYMENT_TARGET" ] && export MACOSX_DEPLOYMENT_TARGET=10.13
    cargo build --target "$triple" --release
}

lib_file="build/$triple/release/libcarbonyl"
if [ -f "${lib_file}.dylib" ]; then
    cp "${lib_file}.dylib" "$CHROMIUM_SRC/out/$target"
    install_name_tool -id @executable_path/libcarbonyl.dylib "${lib_file}.dylib"
else
    cp "${lib_file}.so" "$CHROMIUM_SRC/out/$target"
fi

cd "$CHROMIUM_SRC/out/$target"
ninja headless:headless_shell "$@"
