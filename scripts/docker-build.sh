#!/usr/bin/env bash

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- "$(pwd)")

source "$CARBONYL_ROOT/scripts/env.sh"

target="$1"
cpu="$2"

build_dir="$CARBONYL_ROOT/build/browser/$cpu"
triple=$(scripts/platform-triple.sh "$cpu")

rm -rf "$build_dir"
cp -r "$CARBONYL_ROOT/build/pre-built/$triple" "$build_dir"
cp "$CARBONYL_ROOT/Dockerfile" "$build_dir"

tag="fathyb/carbonyl:$cpu"

docker buildx build "$build_dir" --load --platform "linux/$cpu" --tag "$tag"

echo "Image tag: $tag"
