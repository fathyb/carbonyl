#!/usr/bin/env bash
set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- "$(pwd)")

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

cpu="$1"

# Validate cpu argument
if [ -z "$cpu" ]; then
    echo "Error: cpu argument is required" >&2
    echo "Usage: $0 <cpu>" >&2
    echo "Example: $0 amd64" >&2
    exit 1
fi

# Validate docker is available
if ! command -v docker &> /dev/null; then
    echo "Error: docker is required but not installed" >&2
    exit 1
fi

triple=$(scripts/platform-triple.sh "$cpu" linux)
build_dir="build/docker/$triple"

# Validate pre-built binaries exist
if [ ! -d "$CARBONYL_ROOT/build/pre-built/$triple" ]; then
    echo "Error: Pre-built binaries not found for $triple" >&2
    echo "Run scripts/copy-binaries.sh first" >&2
    exit 1
fi

# Validate Dockerfile exists
if [ ! -f "$CARBONYL_ROOT/Dockerfile" ]; then
    echo "Error: Dockerfile not found" >&2
    exit 1
fi

rm -rf "$build_dir"
mkdir -p "build/docker"
cp -r "$CARBONYL_ROOT/build/pre-built/$triple" "$build_dir"
cp "$CARBONYL_ROOT/Dockerfile" "$build_dir"

tag="fathyb/carbonyl:$cpu"

docker buildx build "$build_dir" --load --platform "linux/$cpu" --tag "$tag"

echo "✓ Docker image built successfully"
echo "Image tag: $tag"
