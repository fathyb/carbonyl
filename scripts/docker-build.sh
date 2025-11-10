#!/usr/bin/env bash

set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- "$(pwd)")

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

cpu="$1"

if [ -z "$cpu" ]; then
    echo "Error: CPU architecture argument is required"
    echo "Usage: $0 <arm64|amd64>"
    exit 1
fi

# Check if docker is available
if ! command -v docker >/dev/null 2>&1; then
    echo "Error: docker command not found"
    exit 1
fi

triple=$(scripts/platform-triple.sh "$cpu" linux)

if [ -z "$triple" ]; then
    echo "Error: Failed to determine platform triple"
    exit 1
fi

build_dir="build/docker/$triple"

# Check if pre-built binaries exist
if [ ! -d "$CARBONYL_ROOT/build/pre-built/$triple" ]; then
    echo "Error: Pre-built binaries not found at $CARBONYL_ROOT/build/pre-built/$triple"
    echo "Run scripts/copy-binaries.sh first"
    exit 1
fi

# Check if Dockerfile exists
if [ ! -f "$CARBONYL_ROOT/Dockerfile" ]; then
    echo "Error: Dockerfile not found at $CARBONYL_ROOT/Dockerfile"
    exit 1
fi

rm -rf "$build_dir"
mkdir -p "build/docker"
cp -r "$CARBONYL_ROOT/build/pre-built/$triple" "$build_dir"
cp "$CARBONYL_ROOT/Dockerfile" "$build_dir"

tag="fathyb/carbonyl:$cpu"

docker buildx build "$build_dir" --load --platform "linux/$cpu" --tag "$tag"

echo "Image tag: $tag"
