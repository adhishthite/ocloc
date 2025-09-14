#!/usr/bin/env bash
set -euo pipefail

if command -v sha256sum >/dev/null 2>&1; then
  sha256sum "$@"
else
  shasum -a 256 "$@"
fi

