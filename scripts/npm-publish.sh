#!/usr/bin/env bash

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))
export SKIP_DEPOT_TOOLS="true"

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

"$CARBONYL_ROOT/scripts/runtime-pull.sh" aarch64-unknown-linux-gnu
"$CARBONYL_ROOT/scripts/runtime-pull.sh" x86_64-unknown-linux-gnu
"$CARBONYL_ROOT/scripts/runtime-pull.sh" aarch64-apple-darwin
"$CARBONYL_ROOT/scripts/runtime-pull.sh" x86_64-apple-darwin

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
