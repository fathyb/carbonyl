#!/usr/bin/env bash

set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

# Check required environment variables
if [ -z "$CDN_ACCESS_KEY_ID" ]; then
    echo "Error: CDN_ACCESS_KEY_ID environment variable is required"
    exit 1
fi

if [ -z "$CDN_SECRET_ACCESS_KEY" ]; then
    echo "Error: CDN_SECRET_ACCESS_KEY environment variable is required"
    exit 1
fi

# Check if aws CLI is available
if ! command -v aws >/dev/null 2>&1; then
    echo "Error: aws command not found"
    exit 1
fi

echo "Computing Chromium patches hash.."

hash=$(scripts/runtime-hash.sh)

if [ -z "$hash" ]; then
    echo "Error: Failed to compute runtime hash"
    exit 1
fi

triple=$(scripts/platform-triple.sh "$1")

if [ -z "$triple" ]; then
    echo "Error: Failed to determine platform triple"
    exit 1
fi

if [ ! -d "build/pre-built/$triple" ]; then
    echo "Error: Pre-built directory not found: build/pre-built/$triple"
    echo "Run scripts/copy-binaries.sh first"
    exit 1
fi

echo "Archiving binaries.."

cd build/pre-built
tar cvzf "$triple.tgz" "$triple"

echo "Binaries archived to build/pre-built/$triple.tgz"

echo "Pushing $triple.tgz to object storage.."

AWS_PAGER="" \
AWS_ACCESS_KEY_ID="$CDN_ACCESS_KEY_ID" \
AWS_SECRET_ACCESS_KEY="$CDN_SECRET_ACCESS_KEY" \
    aws s3api put-object \
        --endpoint-url "https://7985f304d3a79d71fb63aeb17a31fe30.r2.cloudflarestorage.com" \
        --bucket "carbonyl-runtime" \
        --key "runtime/$hash/$triple.tgz" \
        --body "$triple.tgz"

echo "Successfully pushed to: runtime/$hash/$triple.tgz"
