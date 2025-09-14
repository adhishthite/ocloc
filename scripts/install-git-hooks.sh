#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"
HOOKS_DIR="$ROOT_DIR/.git/hooks"

mkdir -p "$HOOKS_DIR"
ln -sf ../../scripts/pre-commit "$HOOKS_DIR/pre-commit"
chmod +x "$ROOT_DIR/scripts/pre-commit"
echo "Installed pre-commit hook to $HOOKS_DIR/pre-commit"

# Install pre-push hook
ln -sf ../../scripts/pre-push "$HOOKS_DIR/pre-push"
chmod +x "$ROOT_DIR/scripts/pre-push"
echo "Installed pre-push hook to $HOOKS_DIR/pre-push"
