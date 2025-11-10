#!/usr/bin/env bash
set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

# Validate git-cliff is available
if ! command -v git-cliff &> /dev/null; then
    echo "Error: git-cliff is required but not installed" >&2
    echo "Install from: https://github.com/orhun/git-cliff" >&2
    exit 1
fi

git cliff a69e8b609625b67a3e52e18f73ba5d0f49ceb7c3..HEAD "$@" > changelog.md

echo "✓ Changelog generated successfully: changelog.md"
