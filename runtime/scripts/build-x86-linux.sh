#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────
# Build: x86_64 Linux (desktop / dev machine / CI)
# Target: x86_64-unknown-linux-gnu
# Also supports CUDA GPU inference at runtime (set ORT_DYLIB_PATH)
# ──────────────────────────────────────────────────────────────────
set -euo pipefail

TARGET="x86_64-unknown-linux-gnu"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "╔══════════════════════════════════════════════════════╗"
echo "║  OpenDuckRust — x86_64 Linux                         ║"
echo "║  Target: ${TARGET}                                   ║"
echo "╚══════════════════════════════════════════════════════╝"

cd "$PROJECT_DIR"
cargo build --target "$TARGET" --release

BINARY="target/${TARGET}/release/openduckrust-runtime"
if [ -f "$BINARY" ]; then
    SIZE=$(du -h "$BINARY" | cut -f1)
    echo ""
    echo "Build complete: $BINARY ($SIZE)"
    echo ""
    echo "For CUDA GPU inference, set ORT_DYLIB_PATH to a CUDA-enabled libonnxruntime.so"
fi
