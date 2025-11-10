#!/usr/bin/env bash

set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

if [ -z "$1" ]; then
    echo "Error: Version argument is required"
    echo "Usage: $0 <version>"
    echo "Example: $0 1.0.0"
    exit 1
fi

version="$1"

# Validate version format (basic semver check)
if ! [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
    echo "Error: Invalid version format: $version"
    echo "Expected format: X.Y.Z or X.Y.Z-prerelease"
    exit 1
fi

# Check if npm is available
if ! command -v npm >/dev/null 2>&1; then
    echo "Error: npm command not found"
    exit 1
fi

# Check if git cliff is available for changelog
if ! command -v git-cliff >/dev/null 2>&1; then
    echo "Warning: git-cliff not found, changelog generation may fail"
fi

# Check for uncommitted changes
if ! git diff-index --quiet HEAD -- 2>/dev/null; then
    echo "Error: You have uncommitted changes. Please commit or stash them first."
    exit 1
fi

npm version "$version" --no-git-tag-version
"$CARBONYL_ROOT/scripts/changelog.sh" --tag "$version"
git add -A .
git commit -m "chore(release): $version"
git tag -a "v$version" -m "chore(release): $version"

echo "Release $version created successfully!"
echo "To push: git push && git push --tags"
