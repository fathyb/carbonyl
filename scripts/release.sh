#!/usr/bin/env bash

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))
export SKIP_DEPOT_TOOLS="true"

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

npm version "$1" --no-git-tag-version
"$CARBONYL_ROOT/scripts/changelog.sh" --tag "$1"
git add -A .
git commit -m "chore(release): $1"
git tag -a "v$1" -m "chore(release): $1"
