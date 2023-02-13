#!/usr/bin/env bash

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- "$(pwd)")

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

cpu="$1"

triple=$(scripts/platform-triple.sh "$cpu")
build_dir="build/docker/$triple"

rm -rf "$build_dir"
mkdir -p "build/docker"
cp -r "$CARBONYL_ROOT/build/pre-built/$triple" "$build_dir"
cp "$CARBONYL_ROOT/Dockerfile" "$build_dir"

tag="fathyb/carbonyl:$cpu"

docker buildx build "$build_dir" --load --platform "linux/$cpu" --tag "$tag"

echo "Image tag: $tag"
