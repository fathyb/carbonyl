#!/usr/bin/env bash

set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

source "$CARBONYL_ROOT/scripts/env.sh"

if [ -z "$1" ]; then
    echo "Error: commit hash argument is required"
    echo "Usage: $0 <commit-hash>"
    exit 1
fi

for file in $(git diff --name-only HEAD "$1"); do
    if [ -f "$file" ]; then
        mtime="$(git log --pretty=format:%cd -n 1 --date=format:%Y%m%d%H%M.%S "$file")"

        if [ -n "$mtime" ]; then
            touch -t "$mtime" "$file"
        fi
    fi
done
