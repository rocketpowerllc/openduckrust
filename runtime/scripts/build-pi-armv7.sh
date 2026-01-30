#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────
# Build: Raspberry Pi (32-bit OS), BeagleBone, older ARM SBCs
# Target: armv7-unknown-linux-gnueabihf
# Also works on: BeagleBone AI, Orange Pi (ARMv7), 32-bit Pi OS
# ──────────────────────────────────────────────────────────────────
set -euo pipefail

TARGET="armv7-unknown-linux-gnueabihf"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "╔══════════════════════════════════════════════════════╗"
echo "║  OpenDuckRust — ARMv7 (32-bit Pi OS / BeagleBone)   ║"
echo "║  Target: ${TARGET}                                   ║"
echo "╚══════════════════════════════════════════════════════╝"

rustup target add "$TARGET" 2>/dev/null || true

if ! command -v arm-linux-gnueabihf-gcc &>/dev/null; then
    echo ""
    echo "⚠ Cross-compiler not found. Install it:"
    echo "  Ubuntu/Debian: sudo apt install gcc-arm-linux-gnueabihf g++-arm-linux-gnueabihf"
    echo "  Or use cross:  cargo install cross && cross build --target $TARGET --release"
    echo ""
    exit 1
fi

cd "$PROJECT_DIR"
cargo build --target "$TARGET" --release

BINARY="target/${TARGET}/release/openduckrust-runtime"
if [ -f "$BINARY" ]; then
    SIZE=$(du -h "$BINARY" | cut -f1)
    echo ""
    echo "Build complete: $BINARY ($SIZE)"
fi
