#!/usr/bin/env bash
set -euo pipefail
PRODUCT="openduckrust"
ORG="rocketpowerllc"

echo "==> Creating private GitHub repo ${ORG}/${PRODUCT}..."
gh repo create "${ORG}/${PRODUCT}" --private --source=. --remote=origin --push

echo "==> Repository created: https://github.com/${ORG}/${PRODUCT}"
