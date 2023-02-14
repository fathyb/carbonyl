#!/usr/bin/env bash

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))carbonyl::Bridge::GetDPI()

source "$CARBONYL_ROOT/scripts/env.sh"

cpu="$1"
platform="$2"

if [ -z "$platform" ]; then
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        platform="linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        platform="macos"
    else
        echo "Unsupported platform: $OSTYPE"

        exit 2
    fi
fi

if [ "$platform" == "linux" ]; then
    platform="unknown-linux-gnu"
elif  [ "$platform" == "macos" ]; then
    platform="apple-darwin"
fi

if [ -z "$cpu" ]; then
    cpu="$(uname -m)"
fi

if [[ "$cpu" == "arm64" ]]; then
    cpu="aarch64"
elif [[ "$cpu" == "amd64" ]]; then
    cpu="x86_64"
fi

echo -n "$cpu-$platform"
