#!/usr/bin/env bash
set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

version="$1"

# Validate version argument
if [ -z "$version" ]; then
    echo "Error: version argument is required" >&2
    echo "Usage: $0 <version>" >&2
    echo "Example: $0 1.0.0" >&2
    exit 1
fi

# Validate version format (basic semver check)
if ! [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
    echo "Error: Invalid version format: $version" >&2
    echo "Expected format: X.Y.Z or X.Y.Z-prerelease" >&2
    exit 1
fi

# Validate required commands
if ! command -v npm &> /dev/null; then
    echo "Error: npm is required but not installed" >&2
    exit 1
fi

if ! command -v git &> /dev/null; then
    echo "Error: git is required but not installed" >&2
    exit 1
fi

# Check for uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
    echo "Error: There are uncommitted changes in the repository" >&2
    echo "Commit or stash changes before creating a release" >&2
    exit 1
fi

npm version "$version" --no-git-tag-version
"$CARBONYL_ROOT/scripts/changelog.sh" --tag "$version"
git add -A .
git commit -m "chore(release): $version"
git tag -a "v$version" -m "chore(release): $version"

echo "✓ Release $version created successfully"
echo "Run 'git push && git push --tags' to publish"
