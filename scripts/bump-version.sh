#!/usr/bin/env bash
set -euo pipefail

if [ $# -ne 1 ]; then
  echo "Usage: $0 X.Y.Z" >&2
  exit 1
fi

NEW_VERSION="$1"

# Update Cargo.toml version
tmpfile=$(mktemp)
awk -v ver="$NEW_VERSION" '
  BEGIN { done=0 }
  /^version = ".*"$/ && done==0 { print "version = \"" ver "\""; done=1; next }
  { print }
' Cargo.toml > "$tmpfile"
mv "$tmpfile" Cargo.toml

# Update CHANGELOG.md Unreleased -> version header with today date
today=$(date +%Y-%m-%d)
tmpchg=$(mktemp)
awk -v ver="$NEW_VERSION" -v today="$today" '
  BEGIN { done=0 }
  /^## \[Unreleased\]/ && done==0 { print; print ""; print "## [" ver "] - " today; done=1; next }
  { print }
' CHANGELOG.md > "$tmpchg" || true
mv "$tmpchg" CHANGELOG.md

echo "Bumped to $NEW_VERSION"
