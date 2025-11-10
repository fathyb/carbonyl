#!/usr/bin/env bash

set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

# Check if node is available
if ! command -v node >/dev/null 2>&1; then
    echo "Error: node command not found"
    exit 1
fi

if [ ! -f "$CARBONYL_ROOT/scripts/npm-package.mjs" ]; then
    echo "Error: npm-package.mjs not found"
    exit 1
fi

VERSION_ID="$(git rev-parse --short HEAD)" \
    node "$CARBONYL_ROOT/scripts/npm-package.mjs"

echo "NPM packages created in build/packages/"
