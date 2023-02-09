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
cp "build/$triple/release/libcarbonyl.$lib_ext" "$dest"

if [ "$lib_ext" == "so" ]; then
    cp "$src/libVkICD_mock_icd.so" "$dest"
    cp "$src/libVkLayer_khronos_validation.so" "$dest"
    cp "$src/libvk_swiftshader.so" "$dest"
    cp "$src/libvulkan.so.1" "$dest"
    cp "$src/vk_swiftshader_icd.json" "$dest"
fi

cd "$dest"

if [[ "$cpu" == "arm64" ]] && command -v aarch64-linux-gnu-strip; then
    aarch64-linux-gnu-strip carbonyl *.so *.so.1
else
    strip carbonyl *.so *.so.1
fi

echo "Binaries copied to $dest"

