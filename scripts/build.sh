#!/usr/bin/env bash

set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- "$(pwd)")
export INSTALL_DEPOT_TOOLS="true"

cd "$CARBONYL_ROOT"
source scripts/env.sh

target="$1"
cpu="$2"

if [ ! -z "$target" ]; then
    shift
fi
if [ ! -z "$cpu" ]; then
    shift
fi

triple=$(scripts/platform-triple.sh "$cpu")

if [ -z "$triple" ]; then
    echo "Error: Failed to determine platform triple"
    exit 1
fi

if [ -z "$CARBONYL_SKIP_CARGO_BUILD" ]; then
    if [ -z "$MACOSX_DEPLOYMENT_TARGET" ]; then
        export MACOSX_DEPLOYMENT_TARGET=10.13
    fi

    cargo build --target "$triple" --release
fi

if [ -f "build/$triple/release/libcarbonyl.dylib" ]; then
    if [ -z "$target" ] || [ ! -d "$CHROMIUM_SRC/out/$target" ]; then
        echo "Error: Invalid target directory: $CHROMIUM_SRC/out/$target"
        exit 1
    fi
    cp "build/$triple/release/libcarbonyl.dylib" "$CHROMIUM_SRC/out/$target"
    install_name_tool \
        -id @executable_path/libcarbonyl.dylib \
        "build/$triple/release/libcarbonyl.dylib"
elif [ -f "build/$triple/release/libcarbonyl.so" ]; then
    if [ -z "$target" ] || [ ! -d "$CHROMIUM_SRC/out/$target" ]; then
        echo "Error: Invalid target directory: $CHROMIUM_SRC/out/$target"
        exit 1
    fi
    cp "build/$triple/release/libcarbonyl.so" "$CHROMIUM_SRC/out/$target"
else
    echo "Error: libcarbonyl library not found at build/$triple/release/"
    exit 1
fi

if [ ! -d "$CHROMIUM_SRC/out/$target" ]; then
    echo "Error: Target directory does not exist: $CHROMIUM_SRC/out/$target"
    exit 1
fi

cd "$CHROMIUM_SRC/out/$target"

ninja headless:headless_shell "$@"
