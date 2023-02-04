#!/usr/bin/env bash

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

target="$1"
cpu="$2"

triple=$(scripts/platform-triple.sh "$cpu")
dest="build/pre-built/$triple"
src="chromium/src/out/$target"

lib_ext="so"
if [ -f "$src"/libEGL.dylib ]; then
    lib_ext="dylib"
fi

rm -rf "$dest"
mkdir -p "$dest"

cp "$src/headless_shell" "$dest/carbonyl"
cp "$src/icudtl.dat" "$dest"
cp "$src/libEGL.$lib_ext" "$dest"
cp "$src/libGLESv2.$lib_ext" "$dest"
cp "$src"/v8_context_snapshot*.bin "$dest"

strip "$dest/carbonyl"

echo "Binaries copied to $dest, archiving.."

cd build/pre-built
tar cvzf "$triple.tgz" "$triple"

echo "Binaries archived to $dest.tgz"
