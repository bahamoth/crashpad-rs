#!/bin/bash
# Parse nextest JSON output and create unified test-results.txt

set -e

PLATFORM="${1:-Linux}"
ARCH="${2:-$(uname -m)}"
JSON_FILE="${3:-test-output.json}"

# Extract test statistics
TOTAL=$(grep -c '"type":"test","event":"started"' "$JSON_FILE" 2>/dev/null || echo "0")
PASSED=$(grep -c '"type":"test","event":"ok"' "$JSON_FILE" 2>/dev/null || echo "0")
FAILED=$(grep -c '"type":"test","event":"failed"' "$JSON_FILE" 2>/dev/null || echo "0")
SKIPPED=$(grep -c '"type":"test","event":"ignored"' "$JSON_FILE" 2>/dev/null || echo "0")

# Calculate total duration (sum of all suite times)
DURATION=$(grep '"type":"suite","event":"ok"' "$JSON_FILE" 2>/dev/null | \
           jq -r '.exec_time' | \
           awk '{sum+=$1} END {printf "%.3f", sum}' || echo "0.000")

# Create test-results.txt
cat > test-results.txt << EOF
platform: $PLATFORM
arch: $ARCH
test_count: $TOTAL
test_passed: $PASSED
test_failed: $FAILED
test_skipped: $SKIPPED
total_duration: ${DURATION}s

EOF

# Extract slowest tests (top 5)
if [ "$PASSED" -gt "0" ]; then
  echo "slowest_tests:" >> test-results.txt
  grep '"type":"test","event":"ok"' "$JSON_FILE" 2>/dev/null | \
    jq -r '"\(.name | split("$")[1]): \(.exec_time)s"' | \
    sort -t: -k2 -rn | \
    head -5 | \
    while read line; do
      echo "  - $line" >> test-results.txt
    done
fi

# Extract failed tests if any
if [ "$FAILED" -gt "0" ]; then
  echo "" >> test-results.txt
  echo "failed_tests:" >> test-results.txt
  grep '"type":"test","event":"failed"' "$JSON_FILE" 2>/dev/null | \
    jq -r '.name | split("$")[1]' | \
    while read test; do
      echo "  - $test" >> test-results.txt
    done
fi

echo "Test results saved to test-results.txt"
cat test-results.txt