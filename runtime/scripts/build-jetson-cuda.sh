#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────
# Build: NVIDIA Jetson with CUDA/TensorRT GPU inference
# Target: aarch64-unknown-linux-gnu (CUDA is linked at runtime)
#
# Works on: Jetson Xavier NX/AGX, Orin Nano/NX/AGX, Thor
# Requires: JetPack 5.x+ with CUDA and TensorRT installed on-device
#
# This script builds with the ONNX Runtime dynamic loading feature.
# On the Jetson, you must have libonnxruntime.so built with CUDA
# and TensorRT execution providers. Set ORT_DYLIB_PATH at runtime.
#
# On-device build (recommended for CUDA):
#   1. SSH into the Jetson
#   2. Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
#   3. cargo build --release
#   4. ORT_DYLIB_PATH=/usr/lib/libonnxruntime.so ./target/release/openduckrust-runtime ...
# ──────────────────────────────────────────────────────────────────
set -euo pipefail

TARGET="aarch64-unknown-linux-gnu"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "╔══════════════════════════════════════════════════════╗"
echo "║  OpenDuckRust — NVIDIA Jetson (CUDA/TensorRT)        ║"
echo "║  Target: ${TARGET}                                   ║"
echo "║  Boards: Xavier NX/AGX, Orin, Thor                   ║"
echo "╚══════════════════════════════════════════════════════╝"
echo ""
echo "NOTE: For GPU-accelerated inference, building on-device is"
echo "recommended so that ONNX Runtime links against the Jetson's"
echo "native CUDA and TensorRT libraries."
echo ""
echo "Cross-compilation produces a CPU-only binary. To enable CUDA:"
echo "  1. SSH into the Jetson"
echo "  2. Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
echo "  3. cargo build --release"
echo "  4. Set ORT_DYLIB_PATH to the CUDA-enabled libonnxruntime.so"
echo ""

rustup target add "$TARGET" 2>/dev/null || true

if ! command -v aarch64-linux-gnu-gcc &>/dev/null; then
    echo "⚠ Cross-compiler not found. Install it:"
    echo "  Ubuntu/Debian: sudo apt install gcc-aarch64-linux-gnu g++-aarch64-linux-gnu"
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
    echo "Deploy to Jetson and run with CUDA:"
    echo "  scp $BINARY jetson@<JETSON_IP>:~/"
    echo "  ssh jetson@<JETSON_IP>"
    echo '  export ORT_DYLIB_PATH=/usr/lib/libonnxruntime.so'
    echo "  ./openduckrust-runtime --onnx-model-path policy.onnx"
    echo ""
    echo "Build ONNX Runtime with CUDA/TensorRT on Jetson:"
    echo "  git clone --recursive https://github.com/microsoft/onnxruntime"
    echo "  cd onnxruntime"
    echo '  ./build.sh --config Release --build_shared_lib --parallel \'
    echo '    --use_cuda --cuda_home /usr/local/cuda \'
    echo '    --use_tensorrt --tensorrt_home /usr/lib/aarch64-linux-gnu'
fi
