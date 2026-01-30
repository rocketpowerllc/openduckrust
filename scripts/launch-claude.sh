#!/usr/bin/env bash
set -euo pipefail
PRODUCT="openduckrust"

echo "==> Launching Claude Code session for ${PRODUCT}..."
cd "$(dirname "$0")/.."
claude --dangerously-skip-permissions
