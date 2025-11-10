#!/usr/bin/env bash
set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- $(pwd))

cd "$CARBONYL_ROOT"
source "scripts/env.sh"

target="$1"
cpu="$2"

# Validate target argument
if [ -z "$target" ]; then
    echo "Error: target argument is required" >&2
    echo "Usage: $0 <target> [cpu]" >&2
    exit 1
fi

triple=$(scripts/platform-triple.sh "$cpu")
dest="build/pre-built/$triple"
src="$CHROMIUM_SRC/out/$target"

# Validate source directory exists
if [ ! -d "$src" ]; then
    echo "Error: Source directory $src does not exist" >&2
    echo "Run scripts/build.sh first to build the project" >&2
    exit 1
fi

# Check for required files
if [ ! -f "$src/headless_shell" ]; then
    echo "Error: headless_shell binary not found in $src" >&2
    echo "Build may have failed. Check build output." >&2
    exit 1
fi

lib_ext="so"
if [ -f "$src"/libEGL.dylib ]; then
    lib_ext="dylib"
fi

# Validate carbonyl library exists
if [ ! -f "$CARBONYL_ROOT/build/$triple/release/libcarbonyl.$lib_ext" ]; then
    echo "Error: libcarbonyl.$lib_ext not found in build/$triple/release/" >&2
    echo "Run cargo build first" >&2
    exit 1
fi

# Validate strip command is available
if ! command -v strip &> /dev/null; then
    echo "Error: strip command is required but not installed" >&2
    exit 1
fi

rm -rf "$dest"
mkdir -p "$dest"
cd "$dest"

cp "$src/headless_shell" carbonyl
cp "$src/icudtl.dat" .
cp "$src/libEGL.$lib_ext" .
cp "$src/libGLESv2.$lib_ext" .
cp "$src"/v8_context_snapshot*.bin .
cp "$CARBONYL_ROOT/build/$triple/release/libcarbonyl.$lib_ext" .

files="carbonyl "

if [ "$lib_ext" == "so" ]; then
    cp "$src/libvk_swiftshader.so" .
    cp "$src/libvulkan.so.1" .
    cp "$src/vk_swiftshader_icd.json" .

    files+=$(echo *.so *.so.1)
fi

if [[ "$cpu" == "arm64" ]] && command -v aarch64-linux-gnu-strip &> /dev/null; then
    aarch64-linux-gnu-strip $files
else
    strip $files
fi

echo "✓ Binaries copied successfully to $dest"
