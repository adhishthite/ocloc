#!/usr/bin/env bash
set -euo pipefail

# Benchmark on a large public repo (elasticsearch)

# Colors
RED="\033[0;31m"
GREEN="\033[0;32m"
YELLOW="\033[1;33m"
BLUE="\033[0;34m"
BOLD="\033[1m"
NC="\033[0m"

REPO_URL="https://github.com/elastic/elasticsearch"
REPO_NAME="elasticsearch"

# Resolve repository root from this script's location
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}        ocloc vs cloc — Large Repo Benchmark (elasticsearch)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

WORKDIR="$(mktemp -d -t ocloc-bench-large-XXXXXX)"
cleanup() {
  echo -e "${YELLOW}Cleaning up temporary files...${NC}"
  rm -rf "$WORKDIR"
}
trap cleanup EXIT

echo -e "${YELLOW}Working directory:${NC} $WORKDIR"
cd "$WORKDIR"

echo -e "${YELLOW}Cloning repo:${NC} $REPO_URL"
export GIT_TERMINAL_PROMPT=0
git clone --depth 1 --filter=blob:none "$REPO_URL" "$REPO_NAME" >/dev/null 2>&1 || {
  echo -e "${RED}Failed to clone $REPO_URL. Check network access and try again.${NC}"
  exit 1
}

cd "$TARGET_DIR"

# Ensure ocloc binary is built (release)
if [[ ! -x "$TARGET_DIR/target/release/ocloc" ]]; then
  echo -e "${YELLOW}Building ocloc (release)...${NC}"
  cargo build --release >/dev/null
fi
OCLOC_BIN="$TARGET_DIR/target/release/ocloc"

cd "$WORKDIR/$REPO_NAME"
echo -e "${YELLOW}Analyzing repository:${NC} $(pwd)"

measure_time() {
  local __out_var=$1; shift
  local __time_var=$1; shift
  local tmp_out
  tmp_out=$(mktemp)
  local t
  TIMEFORMAT='%3R'
  t=$( { time "$@" >"$tmp_out" 2>&1; } 2>&1 ) || true
  printf -v "$__out_var" '%s' "$(cat "$tmp_out")"
  printf -v "$__time_var" '%s' "$t"
  rm -f "$tmp_out"
}

parse_ocloc_totals() {
  local s="$1"
  local line
  line=$(printf '%s\n' "$s" | grep -E '^Total[[:space:]]' | head -n 1 || true)
  if [[ -z "$line" ]]; then
    echo "0 0 0 0 0"
    return
  fi
  local nums
  nums=($(echo "$line" | grep -Eo '[0-9][0-9,]*'))
  local files blank comment code total
  files=${nums[0]//,/}
  blank=${nums[1]//,/}
  comment=${nums[2]//,/}
  code=${nums[3]//,/}
  total=${nums[4]//,/}
  echo "$files $blank $comment $code $total"
}

parse_cloc_sum() {
  local s="$1"
  local line
  line=$(printf '%s\n' "$s" | awk '/^SUM:/{print; exit}')
  if [[ -z "$line" ]]; then
    echo "0 0 0 0 0"
    return
  fi
  nums=($(echo "$line" | grep -Eo '[0-9]+'))
  local files blank comment code total
  files=${nums[0]}
  blank=${nums[1]}
  comment=${nums[2]}
  code=${nums[3]}
  total=$((blank + comment + code))
  echo "$files $blank $comment $code $total"
}

echo -e "${BLUE}${BOLD}Running ocloc...${NC}"
OCLOC_OUT=""
OCLOC_TIME=""
measure_time OCLOC_OUT OCLOC_TIME "$OCLOC_BIN" .
read -r ofiles oblank ocomment ocode ototal <<<"$(parse_ocloc_totals "$OCLOC_OUT")"

CLOC_PRESENT=true
if ! command -v cloc >/dev/null 2>&1; then
  CLOC_PRESENT=false
  echo -e "${YELLOW}cloc is not installed; skipping cloc run. Install with: brew install cloc${NC}"
fi

if $CLOC_PRESENT; then
  echo -e "${BLUE}${BOLD}Running cloc...${NC}"
  CLOC_OUT=""
  CLOC_TIME=""
  measure_time CLOC_OUT CLOC_TIME cloc . --quiet
  read -r cfiles cblank ccomment ccode ctotal <<<"$(parse_cloc_sum "$CLOC_OUT")"
fi

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BOLD}Results (elasticsearch)${NC}"
printf "%-10s  %10s  %10s  %10s  %10s  %10s  %10s\n" "Tool" "Time(s)" "Files" "Blank" "Comment" "Code" "Total"
printf "%-10s  %10s  %10d  %10d  %10d  %10d  %10d\n" "ocloc" "$OCLOC_TIME" "$ofiles" "$oblank" "$ocomment" "$ocode" "$ototal"
if $CLOC_PRESENT; then
  printf "%-10s  %10s  %10d  %10d  %10d  %10d  %10d\n" "cloc" "$CLOC_TIME" "$cfiles" "$cblank" "$ccomment" "$ccode" "$ctotal"
  SPEEDUP=$(awk -v a="$CLOC_TIME" -v b="$OCLOC_TIME" 'BEGIN{ if (b>0) printf "%.2fx", a/b; else print "N/A" }')
  echo -e "${GREEN}Speedup (cloc/ocloc):${NC} $SPEEDUP"
fi
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

echo -e "${GREEN}Benchmark complete.${NC}"
