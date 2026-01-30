#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────
# Build: Raspberry Pi Zero 2W (ARM Cortex-A53, AArch64)
# Target: aarch64-unknown-linux-gnu
# Also works on: Pi 3B/3B+, Pi 4B, Pi 5 (64-bit OS)
# ──────────────────────────────────────────────────────────────────
set -euo pipefail

TARGET="aarch64-unknown-linux-gnu"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "╔══════════════════════════════════════════════════════╗"
echo "║  OpenDuckRust — Raspberry Pi Zero 2W / Pi 3/4/5     ║"
echo "║  Target: ${TARGET}                                   ║"
echo "╚══════════════════════════════════════════════════════╝"

# Ensure target is installed
rustup target add "$TARGET" 2>/dev/null || true

# Check for cross-compiler
if ! command -v aarch64-linux-gnu-gcc &>/dev/null; then
    echo ""
    echo "⚠ Cross-compiler not found. Install it:"
    echo "  Ubuntu/Debian: sudo apt install gcc-aarch64-linux-gnu g++-aarch64-linux-gnu"
    echo "  macOS (brew):  brew install aarch64-elf-gcc"
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
    echo ""
    echo "Deploy to Pi:"
    echo "  scp $BINARY pi@<PI_IP>:~/"
    echo "  ssh pi@<PI_IP> './openduckrust-runtime --onnx-model-path policy.onnx'"
fi
