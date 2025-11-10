#!/usr/bin/env bash

set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

target="$1"
cpu="$2"

if [ -z "$target" ]; then
    echo "Error: target argument is required"
    echo "Usage: $0 <target> [cpu]"
    exit 1
fi

triple=$(scripts/platform-triple.sh "$cpu")

if [ -z "$triple" ]; then
    echo "Error: Failed to determine platform triple"
    exit 1
fi

dest="build/pre-built/$triple"
src="$CHROMIUM_SRC/out/$target"

if [ ! -d "$src" ]; then
    echo "Error: Source directory does not exist: $src"
    exit 1
fi

lib_ext="so"
if [ -f "$src"/libEGL.dylib ]; then
    lib_ext="dylib"
fi

rm -rf "$dest"
mkdir -p "$dest"
cd "$dest"

# Copy required files with validation
if [ ! -f "$src/headless_shell" ]; then
    echo "Error: headless_shell not found at $src/headless_shell"
    exit 1
fi
cp "$src/headless_shell" carbonyl

for file in icudtl.dat "libEGL.$lib_ext" "libGLESv2.$lib_ext"; do
    if [ ! -f "$src/$file" ]; then
        echo "Error: Required file not found: $src/$file"
        exit 1
    fi
    cp "$src/$file" .
done

# Copy v8 snapshot (using glob)
if ! cp "$src"/v8_context_snapshot*.bin . 2>/dev/null; then
    echo "Error: v8_context_snapshot*.bin not found in $src"
    exit 1
fi

# Copy carbonyl library
if [ ! -f "$CARBONYL_ROOT/build/$triple/release/libcarbonyl.$lib_ext" ]; then
    echo "Error: libcarbonyl.$lib_ext not found at $CARBONYL_ROOT/build/$triple/release/"
    exit 1
fi
cp "$CARBONYL_ROOT/build/$triple/release/libcarbonyl.$lib_ext" .

files="carbonyl "

# Copy Linux-specific files
if [ "$lib_ext" == "so" ]; then
    for file in libvk_swiftshader.so libvulkan.so.1 vk_swiftshader_icd.json; do
        if [ ! -f "$src/$file" ]; then
            echo "Error: Required file not found: $src/$file"
            exit 1
        fi
        cp "$src/$file" .
    done

    files+=$(echo *.so *.so.1)
fi

# Strip binaries
if [[ "$cpu" == "arm64" ]] && command -v aarch64-linux-gnu-strip >/dev/null 2>&1; then
    aarch64-linux-gnu-strip $files
elif command -v strip >/dev/null 2>&1; then
    strip $files
else
    echo "Warning: strip command not found, binaries will not be stripped"
fi

echo "Binaries copied to $dest"
