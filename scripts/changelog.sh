#!/usr/bin/env bash

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))
export SKIP_DEPOT_TOOLS="true"

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

git cliff a69e8b609625b67a3e52e18f73ba5d0f49ceb7c3..HEAD "$@" > changelog.md
