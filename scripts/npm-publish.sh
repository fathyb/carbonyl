#!/usr/bin/env bash
set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

# Validate yarn is available
if ! command -v yarn &> /dev/null; then
    echo "Error: yarn is required but not installed" >&2
    echo "Install from: https://yarnpkg.com/" >&2
    exit 1
fi

# Validate packages directory exists
if [ ! -d "build/packages" ]; then
    echo "Error: build/packages directory does not exist" >&2
    echo "Run scripts/npm-package.sh first" >&2
    exit 1
fi

cd "build/packages"

if [ -z "$CARBONYL_PUBLISH_PLATFORM" ] && [ -z "$CARBONYL_PUBLISH_ARCH" ]; then
    package_dir="carbonyl"
else
    package_dir="carbonyl-$CARBONYL_PUBLISH_PLATFORM-$CARBONYL_PUBLISH_ARCH"
fi

# Validate package directory exists
if [ ! -d "$package_dir" ]; then
    echo "Error: Package directory $package_dir does not exist" >&2
    exit 1
fi

cd "$package_dir"

yarn publish --non-interactive --access public "$@"

echo "✓ Package published successfully"
