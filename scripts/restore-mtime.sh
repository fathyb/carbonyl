#!/usr/bin/env bash

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

source "$CARBONYL_ROOT/scripts/env.sh"

for file in $(git diff --name-only HEAD "$1"); do
    mtime="$(git log --pretty=format:%cd -n 1 --date=format:%Y%m%d%H%M.%S "$file")"

    touch -t "$mtime" "$file"
done
