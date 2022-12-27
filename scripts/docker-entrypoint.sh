#!/bin/bash

set -e

export DISPLAY=:99

Xvfb $DISPLAY -screen 0 1920x1080x24 &
node /app/build/carbonyl.cli.js "$@"
