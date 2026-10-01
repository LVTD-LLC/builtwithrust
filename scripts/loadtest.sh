#!/usr/bin/env bash
# Load test the three hottest page types. Requires oha (brew install oha).
# Usage: scripts/loadtest.sh [base_url]   (default http://127.0.0.1:3000)
set -euo pipefail
BASE="${1:-http://127.0.0.1:3000}"
command -v oha >/dev/null || { echo "install oha: brew install oha / cargo install oha"; exit 1; }
for path in / /projects/zed /categories/developer-tools; do
  echo "== $path"
  oha -z 10s -c 64 --no-tui -H 'Accept-Encoding: br' "$BASE$path" \
    | grep -E "Requests/sec|Slowest|Fastest|Average|99.00%|\[200\]"
done
