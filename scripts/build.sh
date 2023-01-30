#!/usr/bin/env bash

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

source "$CARBONYL_ROOT/scripts/env.sh"

platform="linux"
cpu=$(uname -m)

if [[ "$cpu" == "arm64" ]]; then
    cpu="aarch64"
fi

if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    platform="unknown-linux-gnu"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    platform="apple-darwin"
else
    echo "Unsupported platform: $OSTYPE"

    exit 2
fi

target="$cpu-$platform"

cargo build --target "$target" --release

cd "$CHROMIUM_SRC/out/$1"

ninja headless:headless_shell
