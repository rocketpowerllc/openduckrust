#!/usr/bin/env bash
set -euo pipefail
PRODUCT="openduckrust"
PROFILE="openduckrust"
ENV="${2:-dev}"

if [[ -z "${1:-}" ]]; then
  echo "Usage: ./get-logs.sh <request-id> [env]"
  exit 1
fi

REQUEST_ID="$1"

echo "==> Fetching logs for request ${REQUEST_ID} (${ENV})..."
aws logs filter-log-events \
  --log-group-name "/aws/lambda/${PRODUCT}-api" \
  --filter-pattern "{ $.requestId = \"${REQUEST_ID}\" }" \
  --profile "${PRODUCT}-${ENV}" \
  --output json | jq '.events[].message' -r
