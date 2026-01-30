#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────
# Build: Raspberry Pi Zero W (original) (ARM1176JZF-S, ARMv6)
# Target: arm-unknown-linux-gnueabihf
# ──────────────────────────────────────────────────────────────────
set -euo pipefail

TARGET="arm-unknown-linux-gnueabihf"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "╔══════════════════════════════════════════════════════╗"
echo "║  OpenDuckRust — Raspberry Pi Zero W (original)       ║"
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
