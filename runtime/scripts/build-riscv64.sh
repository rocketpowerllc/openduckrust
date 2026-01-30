#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────
# Build: RISC-V 64-bit SBCs
# Target: riscv64gc-unknown-linux-gnu
# Works on: BeagleV-Ahead, Milk-V Mars/Jupiter/Megrez,
#           VisionFive 2, Banana Pi BPI-F3, Sipeed LicheeRV
# ──────────────────────────────────────────────────────────────────
set -euo pipefail

TARGET="riscv64gc-unknown-linux-gnu"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "╔══════════════════════════════════════════════════════╗"
echo "║  OpenDuckRust — RISC-V 64-bit                        ║"
echo "║  Target: ${TARGET}                                   ║"
echo "║  Boards: BeagleV, Milk-V, VisionFive, Banana Pi      ║"
echo "╚══════════════════════════════════════════════════════╝"

rustup target add "$TARGET" 2>/dev/null || true

if ! command -v riscv64-linux-gnu-gcc &>/dev/null; then
    echo ""
    echo "⚠ Cross-compiler not found. Install it:"
    echo "  Ubuntu/Debian: sudo apt install gcc-riscv64-linux-gnu g++-riscv64-linux-gnu"
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
