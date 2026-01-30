#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────
# Build: x86_64 Linux (fully static binary via musl)
# Target: x86_64-unknown-linux-musl
# Produces a single portable binary with no dynamic library deps.
# ──────────────────────────────────────────────────────────────────
set -euo pipefail

TARGET="x86_64-unknown-linux-musl"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "╔══════════════════════════════════════════════════════╗"
echo "║  OpenDuckRust — x86_64 Linux (static/musl)           ║"
echo "║  Target: ${TARGET}                                   ║"
echo "╚══════════════════════════════════════════════════════╝"

rustup target add "$TARGET" 2>/dev/null || true

if ! command -v musl-gcc &>/dev/null; then
    echo ""
    echo "⚠ musl toolchain not found. Install it:"
    echo "  Ubuntu/Debian: sudo apt install musl-tools"
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
    echo "This is a fully static binary — no dynamic library dependencies."
fi
