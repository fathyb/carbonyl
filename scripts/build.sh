#!/usr/bin/env bash

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- "$(pwd)")

cd $CARBONYL_ROOT
source scripts/env.sh

triple=$(scripts/platform-triple.sh "$2")

if [ -z "$MACOSX_DEPLOYMENT_TARGET" ]; then
    export MACOSX_DEPLOYMENT_TARGET=10.13
fi

cargo build --target "$triple" --release

if [ -f "build/$triple/release/libcarbonyl.dylib" ]; then
    cp "build/$triple/release/libcarbonyl.dylib" "$CHROMIUM_SRC/out/$1"
    install_name_tool \
        -id @executable_path/libcarbonyl.dylib \
        "build/$triple/release/libcarbonyl.dylib"
else
    cp "build/$triple/release/libcarbonyl.so" "$CHROMIUM_SRC/out/$1"
fi

cd "$CHROMIUM_SRC/out/$1"

ninja headless:headless_shell
