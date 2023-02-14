#!/usr/bin/env bash

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

VERSION_ID="$(git rev-parse --short HEAD)" \
    node "$CARBONYL_ROOT/scripts/npm-package.mjs"
