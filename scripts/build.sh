#!/usr/bin/env bash

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- "$(pwd)")

cd $CARBONYL_ROOT
source scripts/env.sh

triple=$(scripts/platform-triple.sh "$2")

MACOSX_DEPLOYMENT_TARGET=11.0 cargo build --target "$triple" --release

if [ -f "build/$triple/release/libcarbonyl.dylib" ]; then
    install_name_tool \
        -id @executable_path/libcarbonyl.dylib \
        "build/$triple/release/libcarbonyl.dylib"
fi

cd "$CHROMIUM_SRC/out/$1"

ninja headless:headless_shell
