#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────
# Build: NVIDIA Jetson (all models) — CPU-only ONNX inference
# Target: aarch64-unknown-linux-gnu
# Works on: Jetson Nano, TX2, Xavier NX/AGX, Orin Nano/NX/AGX, Thor
#
# This builds with CPU-only ONNX inference. For CUDA/TensorRT
# acceleration, use build-jetson-cuda.sh instead.
# ──────────────────────────────────────────────────────────────────
set -euo pipefail

TARGET="aarch64-unknown-linux-gnu"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "╔══════════════════════════════════════════════════════╗"
echo "║  OpenDuckRust — NVIDIA Jetson (CPU inference)        ║"
echo "║  Target: ${TARGET}                                   ║"
echo "║  Boards: Nano, TX2, Xavier, Orin, Thor               ║"
echo "╚══════════════════════════════════════════════════════╝"

rustup target add "$TARGET" 2>/dev/null || true

if ! command -v aarch64-linux-gnu-gcc &>/dev/null; then
    echo ""
    echo "⚠ Cross-compiler not found. Install it:"
    echo "  Ubuntu/Debian: sudo apt install gcc-aarch64-linux-gnu g++-aarch64-linux-gnu"
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
    echo "Deploy to Jetson:"
    echo "  scp $BINARY jetson@<JETSON_IP>:~/"
    echo "  ssh jetson@<JETSON_IP> './openduckrust-runtime --onnx-model-path policy.onnx'"
fi
