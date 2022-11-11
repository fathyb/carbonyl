#!/usr/bin/env bash

set -eo pipefail

mode="$1"
shift

ninja -C "electron/src/out/$mode" electron "$@"
