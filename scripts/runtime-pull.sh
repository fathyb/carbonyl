#!/usr/bin/env bash
set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

# Validate required commands
if ! command -v curl &> /dev/null; then
    echo "Error: curl is required but not installed" >&2
    exit 1
fi

if ! command -v tar &> /dev/null; then
    echo "Error: tar is required but not installed" >&2
    exit 1
fi

echo "Computing Chromium patches sha.."

sha="$(scripts/runtime-hash.sh)"
triple="$(scripts/platform-triple.sh "$@")"

if [ ! -f "build/pre-built/$triple.tgz" ]; then
    url="https://carbonyl.fathy.fr/runtime/$sha/$triple.tgz"

    echo "Downloading pre-built binaries from $url"

    mkdir -p build/pre-built

    if ! curl --silent --fail --output "build/pre-built/$triple.tgz" "$url"; then
        echo "Error: Pre-built binaries not available at $url" >&2
        echo "You may need to build locally or wait for CI to publish binaries" >&2
        exit 1
    fi
fi

echo "Pre-built binaries available, extracting.."

cd build/pre-built

# Validate archive exists before extraction
if [ ! -f "$triple.tgz" ]; then
    echo "Error: Archive $triple.tgz not found" >&2
    exit 1
fi

rm -rf "$triple"
tar -xzf "$triple.tgz"

# Validate extraction succeeded
if [ ! -d "$triple" ]; then
    echo "Error: Extraction failed - directory $triple not found" >&2
    exit 1
fi

echo "✓ Pre-built binaries extracted successfully to build/pre-built/$triple"
