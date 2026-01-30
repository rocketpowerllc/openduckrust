#!/usr/bin/env bash
set -euo pipefail
PRODUCT="openduckrust"
PROFILE="openduckrust"

echo "==> Building web app..."
cd "$(dirname "$0")/../web-app"
npm run build

echo "==> Syncing to S3..."
aws s3 sync ./dist "s3://${PRODUCT}-web-app" \
  --delete \
  --profile "${PROFILE}"

echo "==> Invalidating CloudFront (web-app)..."
WEB_DIST_ID=$(aws cloudfront list-distributions \
  --profile "${PROFILE}" \
  --query "DistributionList.Items[?Comment=='${PRODUCT}-web-app'].Id" \
  --output text)

if [[ -n "${WEB_DIST_ID}" ]]; then
  aws cloudfront create-invalidation \
    --distribution-id "${WEB_DIST_ID}" \
    --paths "/*" \
    --profile "${PROFILE}"
fi

echo "==> Web app deployed."
