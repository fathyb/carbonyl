#!/usr/bin/env bash

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- "$(pwd)")

source "$CARBONYL_ROOT/scripts/env.sh"

target="$1"
cpu="$2"

build_dir="$CARBONYL_ROOT/build/browser/$cpu"

rm -rf "$build_dir"
mkdir -p "$build_dir"
cd "$build_dir"

cp "$CARBONYL_ROOT/Dockerfile" .
cp "$CHROMIUM_SRC/out/$target/headless_shell" carbonyl
cp "$CHROMIUM_SRC/out/$target/icudtl.dat" .
cp "$CHROMIUM_SRC/out/$target/libEGL.so" .
cp "$CHROMIUM_SRC/out/$target/libGLESv2.so" .
cp "$CHROMIUM_SRC/out/$target/v8_context_snapshot.bin" .

if [[ "$cpu" == "arm64" ]]; then
    aarch64-linux-gnu-strip carbonyl *.so
else
    strip carbonyl *.so
fi

tag="fathyb/carbonyl:$cpu"

docker buildx build . --load --platform "linux/$cpu" --tag "$tag"

echo "Image tag: $tag"
