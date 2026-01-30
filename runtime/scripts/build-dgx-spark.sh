#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────
# Build: NVIDIA DGX Spark (GB10 Grace Blackwell)
# Target: aarch64-unknown-linux-gnu
#
# The DGX Spark runs Ubuntu 24.04 on AArch64 with Blackwell GPU.
# Same Rust target as all ARM64 Linux. CUDA 13.0+ required for
# Blackwell (sm_121) GPU acceleration.
#
# On-device native build is recommended for CUDA 13.0 support.
# ──────────────────────────────────────────────────────────────────
set -euo pipefail

TARGET="aarch64-unknown-linux-gnu"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "╔══════════════════════════════════════════════════════╗"
echo "║  OpenDuckRust — NVIDIA DGX Spark (GB10 Blackwell)    ║"
echo "║  Target: ${TARGET}                                   ║"
echo "║  GPU: Blackwell (sm_121), CUDA 13.0+                 ║"
echo "╚══════════════════════════════════════════════════════╝"
echo ""
echo "NOTE: The DGX Spark has 20 ARM Neoverse V2 cores and can"
echo "build natively. For GPU inference, build on-device with"
echo "ONNX Runtime compiled against CUDA 13.0+."
echo ""

rustup target add "$TARGET" 2>/dev/null || true

if ! command -v aarch64-linux-gnu-gcc &>/dev/null; then
    echo "⚠ Cross-compiler not found. Install it:"
    echo "  Ubuntu/Debian: sudo apt install gcc-aarch64-linux-gnu g++-aarch64-linux-gnu"
    echo "  Or build natively on the DGX Spark itself."
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
    echo "Deploy:"
    echo "  scp $BINARY user@<DGX_SPARK_IP>:~/"
fi
