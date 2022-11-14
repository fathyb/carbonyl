#!/bin/bash

set -e

export DISPLAY=:99

Xvfb $DISPLAY -screen 0 1920x1080x24 &
/runtime/electron --no-sandbox --headless --disable-audio-output --mute-audio --force-color-profile=srgb --disable-dev-shm-usage /app/build/html2svg.js "$@"

