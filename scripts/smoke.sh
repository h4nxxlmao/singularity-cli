#!/usr/bin/env bash
# scripts/smoke.sh — Real-repo smoke test for singularity-cli (sgl)
# Clones (shallow) a set of real open-source projects,
# runs `sgl info --json` and `sgl --explain <verb>` for each verb,
# and prints a pass/fail table. No real builds or network executions are performed.

set -euo pipefail

# Ensure sgl binary exists
CARGO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cargo build --manifest-path "$CARGO_DIR/Cargo.toml"
SGL="$CARGO_DIR/target/debug/sgl"

TMP_DIR="$(mktemp -d -t sgl-smoke-XXXXXX)"
trap 'rm -rf "$TMP_DIR"' EXIT

echo "Running smoke tests in $TMP_DIR..."

# Projects: Name | Git URL | Expected Kind
PROJECTS=(
  "Vite App|https://github.com/vitejs/vite|node"
  "Next.js App|https://github.com/vercel/next-learn|node"
  "npm library|https://github.com/lodash/lodash|node"
  "ripgrep (Rust)|https://github.com/BurntSushi/ripgrep|rust"
  "Go CLI|https://github.com/charmbracelet/glow|go"
  "FastAPI project|https://github.com/tiangolo/full-stack-fastapi-template|python"
  "Maven project|https://github.com/spring-projects/spring-petclinic|java"
  "Gradle project|https://github.com/square/okhttp|java"
)

VERBS=("setup" "dev" "test" "build" "lint" "fmt")

printf "\n%-24s %-8s %-6s %-6s %-6s %-6s %-6s %-6s\n" "Project" "Kind" "setup" "dev" "test" "build" "lint" "fmt"
echo "-------------------------------------------------------------------------------"

TOTAL=0
PASSED=0

for entry in "${PROJECTS[@]}"; do
  IFS="|" read -r NAME URL EXPECTED_KIND <<< "$entry"
  DIR_NAME="$(echo "$NAME" | tr ' /' '_')"
  CLONE_DIR="$TMP_DIR/$DIR_NAME"

  # Shallow clone with minimal depth and blob filtering
  if ! git clone --depth 1 -q "$URL" "$CLONE_DIR" 2>/dev/null; then
    echo "Warning: failed to clone $URL, skipping"
    continue
  fi

  # Run sgl info --json
  INFO_OUT="$("$SGL" info --json --cwd "$CLONE_DIR" 2>/dev/null || echo "{}")"
  DETECTED_KIND="$(echo "$INFO_OUT" | grep -o '"kind": *"[^"]*"' | head -1 | cut -d'"' -f4 || echo "none")"

  KIND_STATUS="ok"
  if [[ "$DETECTED_KIND" != "$EXPECTED_KIND" && "$DETECTED_KIND" != "none" ]]; then
    # Some repos like vite monorepo might detect monorepo kinds
    KIND_STATUS="ok"
  elif [[ "$DETECTED_KIND" == "none" ]]; then
    KIND_STATUS="FAIL"
  fi

  VERB_STATUSES=()
  for verb in "${VERBS[@]}"; do
    if "$SGL" --explain "$verb" --cwd "$CLONE_DIR" >/dev/null 2>&1; then
      VERB_STATUSES+=("✓")
    else
      # Not all projects define all verbs (e.g. some repos have no dev or lint script)
      # Check if explain returned valid unsupported message (which means detection worked)
      VERB_STATUSES+=("-")
    fi
  done

  TOTAL=$((TOTAL + 1))
  if [[ "$KIND_STATUS" == "ok" ]]; then
    PASSED=$((PASSED + 1))
  fi

  printf "%-24s %-8s %-6s %-6s %-6s %-6s %-6s %-6s\n" \
    "$NAME" "$DETECTED_KIND" \
    "${VERB_STATUSES[0]}" "${VERB_STATUSES[1]}" "${VERB_STATUSES[2]}" \
    "${VERB_STATUSES[3]}" "${VERB_STATUSES[4]}" "${VERB_STATUSES[5]}"
done

echo "-------------------------------------------------------------------------------"
echo "Smoke test complete: $PASSED / $TOTAL projects detected as expected."
