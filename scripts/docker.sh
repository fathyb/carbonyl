#!/usr/bin/env bash

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

source "$CARBONYL_ROOT/scripts/env.sh"

target="$1"
cpu="$2"

build_dir="$CARBONYL_ROOT/build/browser/$cpu"

rm -rf "$build_dir"
mkdir -p "$build_dir"

cp
    "chromium/src/out/$target/"{headless_*,icudtl.dat,libEGL.so,libGLESv2.so,v8_context_snapshot.bin} \
    "$build_dir/"

mv "$build_dir/headless_shell" "$build_dir/carbonyl"