#!/usr/bin/env bash
set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

# Validate required commands
if ! command -v node &> /dev/null; then
    echo "Error: node is required but not installed" >&2
    exit 1
fi

if ! command -v git &> /dev/null; then
    echo "Error: git is required but not installed" >&2
    exit 1
fi

# Validate npm-package.mjs exists
if [ ! -f "$CARBONYL_ROOT/scripts/npm-package.mjs" ]; then
    echo "Error: npm-package.mjs not found" >&2
    exit 1
fi

VERSION_ID="$(git rev-parse --short HEAD)" \
    node "$CARBONYL_ROOT/scripts/npm-package.mjs"

echo "✓ NPM package created successfully"
