#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────
# Build: AArch64 Linux (fully static binary via musl)
# Target: aarch64-unknown-linux-musl
# Produces a portable static binary for any ARM64 Linux device.
# Works on: All Pi models (64-bit), Jetsons, Orange Pi, Rock Pi,
#           Qualcomm RB boards, Google Coral, BeagleBone AI-64
# ──────────────────────────────────────────────────────────────────
set -euo pipefail

TARGET="aarch64-unknown-linux-musl"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "╔══════════════════════════════════════════════════════╗"
echo "║  OpenDuckRust — AArch64 Linux (static/musl)          ║"
echo "║  Target: ${TARGET}                                   ║"
echo "╚══════════════════════════════════════════════════════╝"

rustup target add "$TARGET" 2>/dev/null || true

cd "$PROJECT_DIR"

# Prefer cross for musl targets (handles the musl toolchain in Docker)
if command -v cross &>/dev/null; then
    echo "Using cross for musl build..."
    cross build --target "$TARGET" --release
else
    echo "Using cargo (requires aarch64-linux-musl-gcc)..."
    cargo build --target "$TARGET" --release
fi

BINARY="target/${TARGET}/release/openduckrust-runtime"
if [ -f "$BINARY" ]; then
    SIZE=$(du -h "$BINARY" | cut -f1)
    echo ""
    echo "Build complete: $BINARY ($SIZE)"
    echo "Fully static binary — runs on any ARM64 Linux."
fi
