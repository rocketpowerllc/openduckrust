#!/usr/bin/env bash
set -euo pipefail
PRODUCT="openduckrust"
PROFILE="openduckrust"
REGION="us-east-1"

echo "==> Building Rust backend (release, Lambda target)..."
cd "$(dirname "$0")/../backend"
cargo build --release --target x86_64-unknown-linux-gnu

echo "==> Exporting Swagger spec..."
cargo run --bin export-swagger
cp openapi.json ../web-app/public/openapi.json

echo "==> Pruning old Lambda versions (keeping last 5)..."
FUNCTION_NAME="${PRODUCT}-api"
VERSIONS=$(aws lambda list-versions-by-function \
  --function-name "${FUNCTION_NAME}" \
  --profile "${PROFILE}" \
  --region "${REGION}" \
  --query "Versions[?Version!='\$LATEST'].Version" \
  --output text | tr '\t' '\n' | sort -n)

TOTAL=$(echo "${VERSIONS}" | wc -l | tr -d ' ')
if [[ "${TOTAL}" -gt 5 ]]; then
  TO_DELETE=$(echo "${VERSIONS}" | head -n $(( TOTAL - 5 )))
  for V in ${TO_DELETE}; do
    echo "   Deleting version ${V}..."
    aws lambda delete-function \
      --function-name "${FUNCTION_NAME}" \
      --qualifier "${V}" \
      --profile "${PROFILE}" \
      --region "${REGION}"
  done
fi

echo "==> Invalidating CloudFront (API)..."
API_DIST_ID=$(aws cloudfront list-distributions \
  --profile "${PROFILE}" \
  --query "DistributionList.Items[?Comment=='${PRODUCT}-api'].Id" \
  --output text)

if [[ -n "${API_DIST_ID}" ]]; then
  aws cloudfront create-invalidation \
    --distribution-id "${API_DIST_ID}" \
    --paths "/*" \
    --profile "${PROFILE}"
fi

echo "==> Backend deployment complete."
