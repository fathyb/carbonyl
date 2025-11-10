#!/usr/bin/env bash
set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

source "$CARBONYL_ROOT/scripts/env.sh"

commit_ref="$1"

# Validate commit reference argument
if [ -z "$commit_ref" ]; then
    echo "Error: commit reference argument is required" >&2
    echo "Usage: $0 <commit-ref>" >&2
    exit 1
fi

# Validate git is available
if ! command -v git &> /dev/null; then
    echo "Error: git is required but not installed" >&2
    exit 1
fi

for file in $(git diff --name-only HEAD "$commit_ref"); do
    # Skip if file doesn't exist in working tree
    if [ ! -e "$file" ]; then
        continue
    fi

    mtime="$(git log --pretty=format:%cd -n 1 --date=format:%Y%m%d%H%M.%S "$file")"

    touch -t "$mtime" "$file"
done

echo "✓ Modified times restored for changed files"
