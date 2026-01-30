#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────
# Build OpenDuckRust runtime for ALL supported targets.
#
# Requires either:
#   - All cross-compilers installed (see list below), or
#   - The `cross` tool: cargo install cross
#
# Usage:
#   ./build-all.sh              # Build all targets
#   ./build-all.sh --use-cross  # Use cross (Docker) for all targets
#   ./build-all.sh --list       # List all targets without building
# ──────────────────────────────────────────────────────────────────
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

USE_CROSS=false
LIST_ONLY=false

for arg in "$@"; do
    case $arg in
        --use-cross) USE_CROSS=true ;;
        --list)      LIST_ONLY=true ;;
    esac
done

# ── Target definitions ──

declare -A TARGETS
TARGETS=(
    # ── ARM64 (AArch64) ──
    ["aarch64-unknown-linux-gnu"]="Pi Zero 2W, Pi 3/4/5, Jetson (all), DGX Spark, Orange Pi 5, Rock 5B, Qualcomm RB, Coral, BeagleBone AI-64"
    ["aarch64-unknown-linux-musl"]="Any ARM64 Linux (fully static binary)"

    # ── ARM32 ──
    ["armv7-unknown-linux-gnueabihf"]="Pi (32-bit OS), BeagleBone Black/AI, older ARM SBCs"
    ["arm-unknown-linux-gnueabihf"]="Pi Zero W (original, ARMv6)"

    # ── RISC-V ──
    ["riscv64gc-unknown-linux-gnu"]="BeagleV, Milk-V Mars/Jupiter/Megrez, VisionFive 2, Banana Pi BPI-F3"

    # ── x86_64 ──
    ["x86_64-unknown-linux-gnu"]="Desktop Linux, dev machines, CI, CUDA GPU workstations"
    ["x86_64-unknown-linux-musl"]="x86_64 Linux (fully static binary)"

    # ── macOS ──
    ["aarch64-apple-darwin"]="macOS Apple Silicon (M1/M2/M3/M4) — development only"
    ["x86_64-apple-darwin"]="macOS Intel — development only"
)

# Sorted target list
SORTED_TARGETS=(
    "aarch64-unknown-linux-gnu"
    "aarch64-unknown-linux-musl"
    "armv7-unknown-linux-gnueabihf"
    "arm-unknown-linux-gnueabihf"
    "riscv64gc-unknown-linux-gnu"
    "x86_64-unknown-linux-gnu"
    "x86_64-unknown-linux-musl"
    "aarch64-apple-darwin"
    "x86_64-apple-darwin"
)

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  OpenDuckRust — Multi-Target Build                          ║"
echo "║  Targets: ${#SORTED_TARGETS[@]} platforms                   ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

if [ "$LIST_ONLY" = true ]; then
    printf "%-42s  %s\n" "TARGET TRIPLE" "HARDWARE"
    printf "%-42s  %s\n" "──────────────────────────────────────────" "────────────────────────────────────────────────────"
    for target in "${SORTED_TARGETS[@]}"; do
        printf "%-42s  %s\n" "$target" "${TARGETS[$target]}"
    done
    echo ""
    echo "Accelerator support (runtime-linked, not compile targets):"
    echo "  CUDA        — NVIDIA Jetson (Xavier+), DGX Spark, x86_64 GPU workstations"
    echo "  TensorRT    — NVIDIA Jetson (JetPack 5+), DGX Spark"
    echo "  Edge TPU    — Google Coral (requires C FFI, no native Rust SDK)"
    echo "  Hexagon DSP — Qualcomm RB boards (proprietary SDK only)"
    echo ""
    exit 0
fi

cd "$PROJECT_DIR"

SUCCEEDED=()
FAILED=()
SKIPPED=()

build_target() {
    local target=$1
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Building: $target"
    echo "Hardware: ${TARGETS[$target]}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    # Skip macOS targets when not on macOS
    if [[ "$target" == *"apple-darwin"* ]] && [[ "$(uname)" != "Darwin" ]]; then
        echo "  ⏭ Skipped (not on macOS)"
        SKIPPED+=("$target")
        return
    fi

    # Skip non-macOS targets when on macOS and target is linux (unless using cross)
    if [[ "$(uname)" == "Darwin" ]] && [[ "$target" == *"linux"* ]] && [ "$USE_CROSS" = false ]; then
        # Check if cross-compiler exists
        local linker=""
        case $target in
            aarch64-unknown-linux-gnu)         linker="aarch64-linux-gnu-gcc" ;;
            aarch64-unknown-linux-musl)        linker="aarch64-linux-musl-gcc" ;;
            armv7-unknown-linux-gnueabihf)     linker="arm-linux-gnueabihf-gcc" ;;
            arm-unknown-linux-gnueabihf)       linker="arm-linux-gnueabihf-gcc" ;;
            riscv64gc-unknown-linux-gnu)       linker="riscv64-linux-gnu-gcc" ;;
            x86_64-unknown-linux-musl)         linker="musl-gcc" ;;
            x86_64-unknown-linux-gnu)          linker="gcc" ;;  # native
        esac

        if [ -n "$linker" ] && ! command -v "$linker" &>/dev/null; then
            echo "  ⏭ Skipped (missing $linker — use --use-cross)"
            SKIPPED+=("$target")
            return
        fi
    fi

    rustup target add "$target" 2>/dev/null || true

    if [ "$USE_CROSS" = true ] && [[ "$target" != *"apple-darwin"* ]]; then
        if cross build --target "$target" --release 2>&1; then
            SUCCEEDED+=("$target")
        else
            FAILED+=("$target")
        fi
    else
        if cargo build --target "$target" --release 2>&1; then
            SUCCEEDED+=("$target")
        else
            FAILED+=("$target")
        fi
    fi
}

# Build all targets
for target in "${SORTED_TARGETS[@]}"; do
    build_target "$target"
done

# ── Summary ──

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  Build Summary                                              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

if [ ${#SUCCEEDED[@]} -gt 0 ]; then
    echo "Succeeded (${#SUCCEEDED[@]}):"
    for t in "${SUCCEEDED[@]}"; do
        BINARY="target/${t}/release/openduckrust-runtime"
        if [ -f "$BINARY" ]; then
            SIZE=$(du -h "$BINARY" | cut -f1)
            printf "  ✓ %-42s  %s\n" "$t" "$SIZE"
        else
            printf "  ✓ %-42s\n" "$t"
        fi
    done
    echo ""
fi

if [ ${#FAILED[@]} -gt 0 ]; then
    echo "Failed (${#FAILED[@]}):"
    for t in "${FAILED[@]}"; do
        printf "  ✗ %s\n" "$t"
    done
    echo ""
fi

if [ ${#SKIPPED[@]} -gt 0 ]; then
    echo "Skipped (${#SKIPPED[@]}):"
    for t in "${SKIPPED[@]}"; do
        printf "  ⏭ %-42s  %s\n" "$t" "${TARGETS[$t]}"
    done
    echo ""
fi

echo "Binaries at: $PROJECT_DIR/target/<target>/release/openduckrust-runtime"
