#!/usr/bin/env bash

set -exo pipefail

cd electron

gclient sync --with_tags "$@"
