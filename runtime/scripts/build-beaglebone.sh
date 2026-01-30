#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────
# Build: BeagleBone Black / AI (ARMv7 Cortex-A8/A15)
# Target: armv7-unknown-linux-gnueabihf
# For BeagleBone AI-64 (AArch64), use build-pi-zero2w.sh instead.
# ──────────────────────────────────────────────────────────────────
set -euo pipefail

TARGET="armv7-unknown-linux-gnueabihf"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "╔══════════════════════════════════════════════════════╗"
echo "║  OpenDuckRust — BeagleBone Black / AI                ║"
echo "║  Target: ${TARGET}                                   ║"
echo "╚══════════════════════════════════════════════════════╝"

rustup target add "$TARGET" 2>/dev/null || true

if ! command -v arm-linux-gnueabihf-gcc &>/dev/null; then
    echo "⚠ Cross-compiler not found. Install: sudo apt install gcc-arm-linux-gnueabihf"
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
