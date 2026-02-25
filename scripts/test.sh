#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "==> Rebuilding quadlet with fresh database"
"${SCRIPT_DIR}/quadlet-dev.sh" test

echo ""
echo "==> Running frontend tests"
cd "${PROJECT_ROOT}/web"
npm test

echo ""
echo "==> Running backend tests"
cd "${PROJECT_ROOT}/api"
cargo test

echo ""
echo "==> Running E2E tests"
cd "${PROJECT_ROOT}/web"
E2E_BASE_URL=http://127.0.0.1:8080 npx playwright test

echo ""
echo "==> All tests passed"
