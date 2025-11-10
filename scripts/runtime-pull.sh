#!/usr/bin/env bash

set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

# Check if curl is available
if ! command -v curl >/dev/null 2>&1; then
    echo "Error: curl command not found"
    exit 1
fi

echo "Computing Chromium patches sha.."

sha="$(scripts/runtime-hash.sh)"

if [ -z "$sha" ]; then
    echo "Error: Failed to compute runtime hash"
    exit 1
fi

triple="$(scripts/platform-triple.sh "$@")"

if [ -z "$triple" ]; then
    echo "Error: Failed to determine platform triple"
    exit 1
fi

if [ ! -f "build/pre-built/$triple.tgz" ]; then
    url="https://carbonyl.fathy.fr/runtime/$sha/$triple.tgz"

    echo "Downloading pre-built binaries from $url"

    mkdir -p build/pre-built

    if ! curl --silent --fail --output "build/pre-built/$triple.tgz" "$url"; then
        echo "Error: Pre-built binaries not available at $url"
        echo "You may need to build from source using scripts/build.sh"
        exit 1
    fi

    echo "Download completed successfully"
fi

echo "Pre-built binaries available, extracting.."

if [ ! -f "build/pre-built/$triple.tgz" ]; then
    echo "Error: Archive not found: build/pre-built/$triple.tgz"
    exit 1
fi

cd build/pre-built
rm -rf "$triple"
tar -xvzf "$triple.tgz"

if [ ! -d "$triple" ]; then
    echo "Error: Failed to extract binaries"
    exit 1
fi

echo "Binaries extracted successfully to build/pre-built/$triple"
