#!/usr/bin/env bash

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

for file in chromium/.gclient chromium/patches/*/*.patch src/browser/*.{cc,h,gn,mojom}; do
    file_sha=$(cat "$file" | openssl sha256)
    result=$(echo -n "$sha/${file_sha: -64}" | openssl sha256)
    sha+="${file_sha: -64} ${file}"$'\n'
done

hash=$(echo "$sha" | sort | openssl sha256)

echo -n "${hash: -16}"
