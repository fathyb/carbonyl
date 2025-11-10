#!/usr/bin/env bash
set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- "$(pwd)")

source "$CARBONYL_ROOT/scripts/env.sh"

tag="fathyb/carbonyl"
version="$1"

# Validate version argument
if [ -z "$version" ]; then
    echo "Error: version argument is required" >&2
    echo "Usage: $0 <version>" >&2
    echo "Example: $0 1.0.0" >&2
    exit 1
fi

# Validate docker is available
if ! command -v docker &> /dev/null; then
    echo "Error: docker is required but not installed" >&2
    exit 1
fi

# Validate arm64 image exists
if ! docker image inspect "$tag:arm64" &> /dev/null; then
    echo "Error: Docker image $tag:arm64 not found" >&2
    echo "Run scripts/docker-build.sh arm64 first" >&2
    exit 1
fi

# Validate amd64 image exists
if ! docker image inspect "$tag:amd64" &> /dev/null; then
    echo "Error: Docker image $tag:amd64 not found" >&2
    echo "Run scripts/docker-build.sh amd64 first" >&2
    exit 1
fi

echo "Pushing arm64 image as $tag:$version-arm64"
docker tag "$tag:arm64" "$tag:$version-arm64"
docker push "$tag:$version-arm64"

echo "Pushing amd64 image as $tag:$version-amd64"
docker tag "$tag:amd64" "$tag:$version-amd64"
docker push "$tag:$version-amd64"

docker manifest create "$tag:$version" \
    --amend "$tag:$version-arm64" \
    --amend "$tag:$version-amd64"

docker manifest push "$tag:$version" --purge

echo "✓ Docker images and manifest pushed successfully"
