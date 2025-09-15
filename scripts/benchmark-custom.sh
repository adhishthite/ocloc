#!/usr/bin/env bash
set -euo pipefail

# Benchmark on a user-provided directory

# Colors
RED="\033[0;31m"
GREEN="\033[0;32m"
YELLOW="\033[1;33m"
BLUE="\033[0;34m"
BOLD="\033[1m"
NC="\033[0m"

# Default options
SKIP_CLOC=false
SKIP_TOKEI=false

# Function to display usage
usage() {
  echo "Usage: $0 [OPTIONS] <directory>"
  echo ""
  echo "Options:"
  echo "  --skip-cloc     Skip cloc benchmark (only compare ocloc vs tokei)"
  echo "  --skip-tokei    Skip tokei benchmark (only compare ocloc vs cloc)"
  echo "  -h, --help      Show this help message"
  echo ""
  echo "Examples:"
  echo "  $0 /path/to/project                    # Compare all three tools"
  echo "  $0 --skip-cloc /path/to/project        # Compare only ocloc vs tokei"
  echo "  $0 --skip-tokei /path/to/project       # Compare only ocloc vs cloc"
  exit 0
}

# Parse arguments
TARGET_PATH=""
while [[ $# -gt 0 ]]; do
  case $1 in
    --skip-cloc)
      SKIP_CLOC=true
      shift
      ;;
    --skip-tokei)
      SKIP_TOKEI=true
      shift
      ;;
    -h|--help)
      usage
      ;;
    -*)
      echo -e "${RED}Error: Unknown option: $1${NC}"
      usage
      ;;
    *)
      if [[ -z "$TARGET_PATH" ]]; then
        TARGET_PATH="$1"
      else
        echo -e "${RED}Error: Multiple directories provided${NC}"
        usage
      fi
      shift
      ;;
  esac
done

# Check for argument
if [[ -z "$TARGET_PATH" ]]; then
  echo -e "${RED}Error: No directory provided${NC}"
  usage
fi

# Validate directory exists
if [[ ! -d "$TARGET_PATH" ]]; then
  echo -e "${RED}Error: '$TARGET_PATH' is not a valid directory${NC}"
  exit 1
fi

# Get absolute path
TARGET_PATH="$(cd "$TARGET_PATH" && pwd)"
REPO_NAME="$(basename "$TARGET_PATH")"

# Resolve repository root from this script's location
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
if $SKIP_CLOC && $SKIP_TOKEI; then
  echo -e "${BLUE}                 ocloc — Custom Directory Benchmark${NC}"
elif $SKIP_CLOC; then
  echo -e "${BLUE}          ocloc vs tokei — Custom Directory Benchmark${NC}"
elif $SKIP_TOKEI; then
  echo -e "${BLUE}           ocloc vs cloc — Custom Directory Benchmark${NC}"
else
  echo -e "${BLUE}        ocloc vs cloc vs tokei — Custom Directory Benchmark${NC}"
fi
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

echo -e "${YELLOW}Target directory:${NC} $TARGET_PATH"
echo -e "${YELLOW}Directory name:${NC} $REPO_NAME"

# Ensure ocloc binary is built (release)
if [[ ! -x "$TARGET_DIR/target/release/ocloc" ]]; then
  echo -e "${YELLOW}Building ocloc (release)...${NC}"
  cargo build --release >/dev/null
fi
OCLOC_BIN="$TARGET_DIR/target/release/ocloc"

cd "$TARGET_PATH"
echo -e "${YELLOW}Analyzing directory:${NC} $(pwd)"

# Count files for size estimation
FILE_COUNT=$(find . -type f 2>/dev/null | wc -l | xargs)
echo -e "${YELLOW}Total files found:${NC} $FILE_COUNT"

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

parse_tokei_totals() {
  local s="$1"
  local line
  line=$(printf '%s\n' "$s" | grep -E '^[[:space:]]*Total[[:space:]]' | tail -n 1 || true)
  if [[ -z "$line" ]]; then
    echo "0 0 0 0 0"
    return
  fi
  local nums
  nums=($(echo "$line" | grep -Eo '[0-9]+'))
  local files blank comment code total
  files=${nums[0]}
  total=${nums[1]}
  blank=${nums[2]}
  comment=${nums[3]}
  code=${nums[4]}
  echo "$files $blank $comment $code $total"
}

echo -e "${BLUE}${BOLD}Running ocloc...${NC}"
OCLOC_OUT=""
OCLOC_TIME=""
measure_time OCLOC_OUT OCLOC_TIME "$OCLOC_BIN" .
read -r ofiles oblank ocomment ocode ototal <<<"$(parse_ocloc_totals "$OCLOC_OUT")"

CLOC_PRESENT=true
if $SKIP_CLOC; then
  CLOC_PRESENT=false
  echo -e "${YELLOW}Skipping cloc benchmark (--skip-cloc option)${NC}"
elif ! command -v cloc >/dev/null 2>&1; then
  CLOC_PRESENT=false
  echo -e "${YELLOW}cloc is not installed; skipping cloc run. Install with: brew install cloc${NC}"
fi

if $CLOC_PRESENT; then
  echo -e "${BLUE}${BOLD}Running cloc...${NC}"
  CLOC_OUT=""
  CLOC_TIME=""
  # Add timeout for very large directories
  if [[ $FILE_COUNT -gt 50000 ]]; then
    echo -e "${YELLOW}Large directory detected. cloc may take a while...${NC}"
  fi
  measure_time CLOC_OUT CLOC_TIME cloc . --quiet
  read -r cfiles cblank ccomment ccode ctotal <<<"$(parse_cloc_sum "$CLOC_OUT")"
fi

TOKEI_PRESENT=true
if $SKIP_TOKEI; then
  TOKEI_PRESENT=false
  echo -e "${YELLOW}Skipping tokei benchmark (--skip-tokei option)${NC}"
elif ! command -v tokei >/dev/null 2>&1; then
  TOKEI_PRESENT=false
  echo -e "${YELLOW}tokei is not installed; skipping tokei run. Install with: cargo install tokei${NC}"
fi

if $TOKEI_PRESENT; then
  echo -e "${BLUE}${BOLD}Running tokei...${NC}"
  TOKEI_OUT=""
  TOKEI_TIME=""
  measure_time TOKEI_OUT TOKEI_TIME tokei .
  read -r tfiles tblank tcomment tcode ttotal <<<"$(parse_tokei_totals "$TOKEI_OUT")"
fi

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BOLD}Results ($REPO_NAME)${NC}"
printf "%-10s  %10s  %10s  %10s  %10s  %10s  %10s\n" "Tool" "Time(s)" "Files" "Blank" "Comment" "Code" "Total"
printf "%-10s  %10s  %10d  %10d  %10d  %10d  %10d\n" "ocloc" "$OCLOC_TIME" "$ofiles" "$oblank" "$ocomment" "$ocode" "$ototal"
if $CLOC_PRESENT; then
  printf "%-10s  %10s  %10d  %10d  %10d  %10d  %10d\n" "cloc" "$CLOC_TIME" "$cfiles" "$cblank" "$ccomment" "$ccode" "$ctotal"
fi
if $TOKEI_PRESENT; then
  printf "%-10s  %10s  %10d  %10d  %10d  %10d  %10d\n" "tokei" "$TOKEI_TIME" "$tfiles" "$tblank" "$tcomment" "$tcode" "$ttotal"
fi

if $CLOC_PRESENT || $TOKEI_PRESENT; then
  echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo -e "${GREEN}Speedup comparisons:${NC}"
  if $CLOC_PRESENT; then
    SPEEDUP_CLOC=$(awk -v a="$CLOC_TIME" -v b="$OCLOC_TIME" 'BEGIN{ if (b>0) printf "%.2fx", a/b; else print "N/A" }')
    echo -e "  ocloc vs cloc:  ${BOLD}$SPEEDUP_CLOC${NC} faster"
  fi
  if $TOKEI_PRESENT; then
    SPEEDUP_TOKEI=$(awk -v a="$TOKEI_TIME" -v b="$OCLOC_TIME" 'BEGIN{ if (b>0) printf "%.2fx", a/b; else print "N/A" }')
    echo -e "  ocloc vs tokei: ${BOLD}$SPEEDUP_TOKEI${NC} faster"
  fi
fi
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

echo -e "${GREEN}Benchmark complete.${NC}"