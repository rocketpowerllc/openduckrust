#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────
# Build: macOS (Apple Silicon M1/M2/M3/M4 + Intel)
# Target: aarch64-apple-darwin (native on Apple Silicon)
#         x86_64-apple-darwin  (Intel or Rosetta 2)
#
# Useful for development and testing without a Pi.
# ──────────────────────────────────────────────────────────────────
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ]; then
    TARGET="aarch64-apple-darwin"
else
    TARGET="x86_64-apple-darwin"
fi

echo "╔══════════════════════════════════════════════════════╗"
echo "║  OpenDuckRust — macOS ($ARCH)                        ║"
echo "║  Target: ${TARGET}                                   ║"
echo "╚══════════════════════════════════════════════════════╝"

cd "$PROJECT_DIR"
cargo build --target "$TARGET" --release

BINARY="target/${TARGET}/release/openduckrust-runtime"
if [ -f "$BINARY" ]; then
    SIZE=$(du -h "$BINARY" | cut -f1)
    echo ""
    echo "Build complete: $BINARY ($SIZE)"
    echo "(Development build — GPIO/I2C/serial features will not work on macOS)"
fi
