#!/usr/bin/env bash

set -eo pipefail

mode="$1"
shift

cd electron/src
CHROMIUM_BUILDTOOLS_PATH=`pwd`/buildtools \
    gn gen "out/${mode}" --args="import(\"//electron/build/args/$mode.gn\") ${GN_ARGS}" "$@"
