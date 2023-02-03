#!/usr/bin/env bash

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

echo "Computing Chromium patches hash.."

hash=$(scripts/runtime-hash.sh)
triple=$(scripts/platform-triple.sh "$1")

echo "Archiving binaries.."

cd build/pre-built
tar cvzf "$triple.tgz" "$triple"

echo "Pushing $triple.tgz to object storage.."

AWS_PAGER="" \
AWS_ACCESS_KEY_ID="$CDN_ACCESS_KEY_ID" \
AWS_SECRET_ACCESS_KEY="$CDN_SECRET_ACCESS_KEY" \
    aws s3api put-object \
        --endpoint-url "https://7985f304d3a79d71fb63aeb17a31fe30.r2.cloudflarestorage.com" \
        --bucket "carbonyl-runtime" \
        --key "runtime/$hash/$triple.tgz" \
        --body "$triple.tgz"
