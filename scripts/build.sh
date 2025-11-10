#!/usr/bin/env bash
set -eo pipefail

export CARBONYL_ROOT=$(cd $(dirname -- "$0") && dirname -- "$(pwd)")
export INSTALL_DEPOT_TOOLS="true"

cd "$CARBONYL_ROOT"
source scripts/env.sh

target="$1"
cpu="$2"

# Validate target argument
if [ -z "$target" ]; then
    echo "Error: target argument is required" >&2
    echo "Usage: $0 <target> [cpu] [additional-ninja-args]" >&2
    exit 1
fi

if [ ! -z "$target" ]; then
    shift
fi
if [ ! -z "$cpu" ]; then
    shift
fi

# Validate required commands
if [ -z "$CARBONYL_SKIP_CARGO_BUILD" ]; then
    if ! command -v cargo &> /dev/null; then
        echo "Error: cargo is required but not installed" >&2
        echo "Install Rust from https://rustup.rs/" >&2
        exit 1
    fi
fi

if ! command -v ninja &> /dev/null; then
    echo "Error: ninja is required but not installed" >&2
    exit 1
fi

triple=$(scripts/platform-triple.sh "$cpu")

if [ -z "$CARBONYL_SKIP_CARGO_BUILD" ]; then
    if [ -z "$MACOSX_DEPLOYMENT_TARGET" ]; then
        export MACOSX_DEPLOYMENT_TARGET=10.13
    fi

    cargo build --target "$triple" --release
fi

# Validate build output directory exists
if [ ! -d "$CHROMIUM_SRC/out/$target" ]; then
    echo "Error: Target directory $CHROMIUM_SRC/out/$target does not exist" >&2
    echo "Run gn gen commands first to create the build directory" >&2
    exit 1
fi

if [ -f "build/$triple/release/libcarbonyl.dylib" ]; then
    cp "build/$triple/release/libcarbonyl.dylib" "$CHROMIUM_SRC/out/$target"
    if command -v install_name_tool &> /dev/null; then
        install_name_tool \
            -id @executable_path/libcarbonyl.dylib \
            "build/$triple/release/libcarbonyl.dylib"
    fi
else
    if [ ! -f "build/$triple/release/libcarbonyl.so" ]; then
        echo "Error: Neither libcarbonyl.dylib nor libcarbonyl.so found in build/$triple/release/" >&2
        exit 1
    fi
    cp "build/$triple/release/libcarbonyl.so" "$CHROMIUM_SRC/out/$target"
fi

cd "$CHROMIUM_SRC/out/$target"

ninja headless:headless_shell "$@"

echo "✓ Build completed successfully for target: $target"
