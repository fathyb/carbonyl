#!/usr/bin/env bash

set -eo pipefail

git -C electron/src apply --ignore-space-change --ignore-whitespace --reject < src/chromium.patch
git -C electron/src/third_party/skia apply --ignore-space-change --ignore-whitespace --reject < src/skia.patch
