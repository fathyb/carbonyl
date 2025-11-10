#!/usr/bin/env bash

set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

# Hash relevant files for runtime cache key
sha=""
for file in chromium/.gclient chromium/patches/*/*.patch libmongoose/browser/*.{cc,h,gn,mojom}; do
    # Skip if file doesn't exist (glob didn't match)
    [ -f "$file" ] || continue

    file_sha=$(cat "$file" | openssl sha256)
    result=$(echo -n "$sha/${file_sha: -64}" | openssl sha256)
    sha+="${file_sha: -64} ${file}"$'\n'
done

hash=$(echo "$sha" | sort | openssl sha256)

echo -n "${hash: -16}"
