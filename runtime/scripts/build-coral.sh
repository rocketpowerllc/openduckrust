#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────
# Build: Google Coral Dev Board / Dev Board Mini
# Target: aarch64-unknown-linux-gnu
#
# Edge TPU acceleration requires libedgetpu C++ bindings (no official
# Rust SDK). This build uses CPU-only ONNX inference.
# For Edge TPU, use the TFLite C API via FFI or run inference
# through the Coral Python API in a sidecar process.
# ──────────────────────────────────────────────────────────────────
set -euo pipefail

TARGET="aarch64-unknown-linux-gnu"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "╔══════════════════════════════════════════════════════╗"
echo "║  OpenDuckRust — Google Coral Dev Board                ║"
echo "║  Target: ${TARGET}                                   ║"
echo "║  Note: Edge TPU not accessible from Rust (CPU only)   ║"
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
