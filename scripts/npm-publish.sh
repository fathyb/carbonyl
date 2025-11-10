#!/usr/bin/env bash

set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

# Check if yarn is available
if ! command -v yarn >/dev/null 2>&1; then
    echo "Error: yarn command not found"
    exit 1
fi

if [ ! -d "build/packages" ]; then
    echo "Error: build/packages directory not found"
    echo "Run scripts/npm-package.sh first"
    exit 1
fi

cd "build/packages"

if [ -z "$CARBONYL_PUBLISH_PLATFORM" ] && [ -z "$CARBONYL_PUBLISH_ARCH" ]; then
    package_dir="carbonyl"
else
    package_dir="carbonyl-$CARBONYL_PUBLISH_PLATFORM-$CARBONYL_PUBLISH_ARCH"
fi

if [ ! -d "$package_dir" ]; then
    echo "Error: Package directory not found: build/packages/$package_dir"
    exit 1
fi

cd "$package_dir"

echo "Publishing package from: $(pwd)"
yarn publish --non-interactive --access public "$@"
