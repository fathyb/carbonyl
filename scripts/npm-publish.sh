#!/usr/bin/env bash

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))
export SKIP_DEPOT_TOOLS="true"

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

VERSION_ID="$(git rev-parse --short HEAD)" \
    node "$CARBONYL_ROOT/scripts/npm-publish.mjs"

cd "$CARBONYL_ROOT/build/packages"

cd carbonyl-linux-amd64
yarn publish --non-interactive --access public "$@"

cd ../carbonyl-linux-arm64
yarn publish --non-interactive --access public "$@"

cd ../carbonyl-macos-amd64
yarn publish --non-interactive --access public "$@"

cd ../carbonyl-macos-arm64
yarn publish --non-interactive --access public "$@"

cd ../carbonyl
yarn publish --non-interactive --access public "$@"
