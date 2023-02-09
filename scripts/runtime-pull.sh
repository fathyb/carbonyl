#!/usr/bin/env bash

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))
export SKIP_DEPOT_TOOLS="true"

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

echo "Computing Chromium patches sha.."

sha=$(scripts/runtime-hash.sh)

triple="$1"

if [ ! -f "build/pre-built/$triple.tgz" ]; then
    url="https://carbonyl.fathy.fr/runtime/$sha/$triple.tgz"

    echo "Downloading pre-built binaries from $url"

    mkdir -p build/pre-built

    if ! curl --silent --fail --output "build/pre-built/$triple.tgz" "$url"; then
        echo "Pre-built binaries not available"

        exit 1
    fi
fi

echo "Pre-build binaries available, extracting.."

cd build/pre-built
rm -rf "$triple"
tar -xvzf "$triple.tgz"
