#!/usr/bin/env bash

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

platform="$1"; shift
arch="$1"; shift

cd "build/packages"

if [ -z "$CARBONYL_PUBLISH_PLATFORM" ] && [ -z "$CARBONYL_PUBLISH_ARCH" ]; then
    cd "carbonyl"
else
    cd "carbonyl-$CARBONYL_PUBLISH_PLATFORM-$CARBONYL_PUBLISH_ARCH"
fi

yarn publish --non-interactive --access public "$@"
