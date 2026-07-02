#!/usr/bin/env bash
set -euo pipefail

INSTALL_DIR="${CARBONYL_INSTALL_DIR:-$HOME/Downloads/carbonyl-0.0.3}"

if [ ! -f "$INSTALL_DIR/libcarbonyl.dylib" ]; then
    echo "libcarbonyl.dylib not found at $INSTALL_DIR"
    echo "Set CARBONYL_INSTALL_DIR=<path> to point elsewhere."
    exit 1
fi

ARCH="$(file "$INSTALL_DIR/libcarbonyl.dylib" | grep -oE 'x86_64|arm64')"
case "$ARCH" in
    x86_64) TARGET="x86_64-apple-darwin" ;;
    arm64)  TARGET="aarch64-apple-darwin" ;;
    *) echo "Unknown arch: $ARCH"; exit 1 ;;
esac

export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-10.13}"

cd "$(dirname -- "$0")/.."

rustup target add "$TARGET" >/dev/null 2>&1 || true

cargo build --target "$TARGET" --release

OUT="build/$TARGET/release/libcarbonyl.dylib"
install_name_tool -id @executable_path/libcarbonyl.dylib "$OUT"
cp "$OUT" "$INSTALL_DIR/libcarbonyl.dylib"

echo "Installed $OUT -> $INSTALL_DIR/libcarbonyl.dylib"
echo "Run: $INSTALL_DIR/carbonyl <url>"
