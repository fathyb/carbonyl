#!/usr/bin/env bash
set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

# Validate required commands
if ! command -v openssl &> /dev/null; then
    echo "Error: openssl is required but not installed" >&2
    exit 1
fi

# Collect hashes from relevant files
for file in chromium/.gclient chromium/patches/*/*.patch libmongoose/browser/*.{cc,h,gn,mojom}; do
    # Skip if file doesn't exist (glob expansion failure)
    if [ ! -e "$file" ]; then
        continue
    fi

    file_sha=$(cat "$file" | openssl sha256)
    result=$(echo -n "$sha/${file_sha: -64}" | openssl sha256)
    sha+="${file_sha: -64} ${file}"$'\n'
done

hash=$(echo "$sha" | sort | openssl sha256)

echo -n "${hash: -16}"
