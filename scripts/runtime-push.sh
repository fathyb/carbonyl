#!/usr/bin/env bash
set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

# Validate required commands
if ! command -v tar &> /dev/null; then
    echo "Error: tar is required but not installed" >&2
    exit 1
fi

if ! command -v aws &> /dev/null; then
    echo "Error: aws CLI is required but not installed" >&2
    echo "Install from: https://aws.amazon.com/cli/" >&2
    exit 1
fi

# Validate AWS credentials are set
if [ -z "$CDN_ACCESS_KEY_ID" ]; then
    echo "Error: CDN_ACCESS_KEY_ID environment variable is not set" >&2
    exit 1
fi

if [ -z "$CDN_SECRET_ACCESS_KEY" ]; then
    echo "Error: CDN_SECRET_ACCESS_KEY environment variable is not set" >&2
    exit 1
fi

echo "Computing Chromium patches hash.."

hash=$(scripts/runtime-hash.sh)
triple=$(scripts/platform-triple.sh "$1")

# Validate pre-built directory exists
if [ ! -d "build/pre-built/$triple" ]; then
    echo "Error: Pre-built directory build/pre-built/$triple does not exist" >&2
    echo "Run scripts/copy-binaries.sh first" >&2
    exit 1
fi

echo "Archiving binaries.."

cd build/pre-built
tar czf "$triple.tgz" "$triple"

# Validate archive was created
if [ ! -f "$triple.tgz" ]; then
    echo "Error: Failed to create archive $triple.tgz" >&2
    exit 1
fi

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

echo "✓ Binaries pushed successfully to runtime/$hash/$triple.tgz"
