#!/bin/bash

set -e

export DISPLAY=:99

Xvfb $DISPLAY -screen 0 1920x1080x8 &
/runtime/electron --no-sandbox --headless --disable-dev-shm-usage /app/build/html2svg.js "$@"

