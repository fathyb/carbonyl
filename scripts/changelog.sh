#!/usr/bin/env bash

set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

# Check if git-cliff is available
if ! command -v git-cliff >/dev/null 2>&1; then
    echo "Error: git-cliff command not found"
    echo "Install it from: https://github.com/orhun/git-cliff"
    exit 1
fi

git cliff a69e8b609625b67a3e52e18f73ba5d0f49ceb7c3..HEAD "$@" > changelog.md

echo "Changelog generated: changelog.md"
