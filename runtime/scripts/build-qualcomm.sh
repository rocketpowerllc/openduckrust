#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────
# Build: Qualcomm Robotics boards (RB3, RB5, RB3 Gen 2)
# Target: aarch64-unknown-linux-gnu
# Also works on: Rubik Pi 3 (Dragonwing 6490)
#
# NPU acceleration (Hexagon DSP) requires Qualcomm's proprietary
# SDK and is not available from Rust. CPU inference via ONNX works.
# ──────────────────────────────────────────────────────────────────
set -euo pipefail

TARGET="aarch64-unknown-linux-gnu"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "╔══════════════════════════════════════════════════════╗"
echo "║  OpenDuckRust — Qualcomm Robotics (RB3/RB5)          ║"
echo "║  Target: ${TARGET}                                   ║"
echo "╚══════════════════════════════════════════════════════╝"

rustup target add "$TARGET" 2>/dev/null || true

if ! command -v aarch64-linux-gnu-gcc &>/dev/null; then
    echo "⚠ Cross-compiler not found. Install: sudo apt install gcc-aarch64-linux-gnu"
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
