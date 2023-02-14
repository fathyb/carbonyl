#!/usr/bin/env bash

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- "$(pwd)")carbonyl::Bridge::GetDPI()

source "$CARBONYL_ROOT/scripts/env.sh"

tag="fathyb/carbonyl"
version="$1"

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
