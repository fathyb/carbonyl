#!/usr/bin/env bash

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

for file in chromium/.gclient src/*.patch src/browser/*.{cc,h,gn,mojom}; do
    file_sha=$(cat "$file" | openssl sha256)
    sha=$(echo -n "$sha/$file_sha" | openssl sha256)
done

echo -n "${sha:0:16}"
