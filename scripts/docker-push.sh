#!/usr/bin/env bash

set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- "$(pwd)")

source "$CARBONYL_ROOT/scripts/env.sh"

version="$1"

if [ -z "$version" ]; then
    echo "Error: Version argument is required"
    echo "Usage: $0 <version>"
    exit 1
fi

# Check if docker is available
if ! command -v docker >/dev/null 2>&1; then
    echo "Error: docker command not found"
    exit 1
fi

tag="fathyb/carbonyl"

# Check if images exist locally
if ! docker image inspect "$tag:arm64" >/dev/null 2>&1; then
    echo "Error: arm64 image not found. Build it first with scripts/docker-build.sh arm64"
    exit 1
fi

if ! docker image inspect "$tag:amd64" >/dev/null 2>&1; then
    echo "Error: amd64 image not found. Build it first with scripts/docker-build.sh amd64"
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

echo "Successfully pushed Docker images for version $version"
