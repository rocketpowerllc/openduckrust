#!/usr/bin/env bash
set -euo pipefail
PRODUCT="openduckrust"
PROFILE="openduckrust"

echo "==> Syncing marketing site to S3..."
aws s3 sync "$(dirname "$0")/../marketing-site" \
  "s3://${PRODUCT}-marketing" \
  --delete \
  --profile "${PROFILE}"

echo "==> Invalidating CloudFront (marketing)..."
MKT_DIST_ID=$(aws cloudfront list-distributions \
  --profile "${PROFILE}" \
  --query "DistributionList.Items[?Comment=='${PRODUCT}-marketing'].Id" \
  --output text)

if [[ -n "${MKT_DIST_ID}" ]]; then
  aws cloudfront create-invalidation \
    --distribution-id "${MKT_DIST_ID}" \
    --paths "/*" \
    --profile "${PROFILE}"
fi

echo "==> Marketing site deployed."
