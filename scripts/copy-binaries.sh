#!/usr/bin/env bash

CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- "$(pwd)")

# Check for the expected number of arguments.
if [[ $# -lt 2 ]]; then
    echo "Error: Expected at least two arguments."
    exit 1
fi

cd "$CARBONYL_ROOT"

# Ensure the env script exists and is readable before sourcing.
if [[ -f "scripts/env.sh" && -r "scripts/env.sh" ]]; then
    source "scripts/env.sh"
else
    echo "Error: Cannot find or read scripts/env.sh"
    exit 1
fi

target="$1"
cpu="$2"
triple=$(scripts/platform-triple.sh "$cpu")

dest="build/pre-built/$triple"
src="$CHROMIUM_SRC/out/$target"

lib_ext="so"
[ -f "$src/libEGL.dylib" ] && lib_ext="dylib"

rm -rf "$dest"
mkdir -p "$dest"
cd "$dest"

# List of mandatory files
files_to_copy=(
    "$src/headless_shell"
    "$src/icudtl.dat"
    "$src/libEGL.$lib_ext"
    "$src/libGLESv2.$lib_ext"
    "$src/v8_context_snapshot*.bin"
    "$CARBONYL_ROOT/build/$triple/release/libcarbonyl.$lib_ext"
)

# Copy the files if they exist
for file in "${files_to_copy[@]}"; do
    if [[ -f "$file" ]]; then
        cp "$file" .
    else
        echo "Warning: $file not found."
    fi
done

files="carbonyl"

[ "$lib_ext" == "so" ] && {
    swiftshader_files=(
        "$src/libvk_swiftshader.so"
        "$src/libvulkan.so.1"
        "$src/vk_swiftshader_icd.json"
    )

    for file in "${swiftshader_files[@]}"; do
        if [[ -f "$file" ]]; then
            cp "$file" .
            files+=" $(basename $file)"
        else
            echo "Warning: $file not found."
        fi
    done
}

if [[ "$cpu" == "arm64" ]] && command -v aarch64-linux-gnu-strip &> /dev/null; then
    aarch64-linux-gnu-strip $files
else
    strip $files
fi

echo "Binaries copied to $dest"
