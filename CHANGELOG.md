# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog and adheres to Semantic Versioning.

## [Unreleased]

## [0.3.0] - 2025-09-15

- Nothing yet.

## [0.2.1] - 2025-09-15

### Changed

- Release workflow now publishes macOS-only artifacts (aarch64-apple-darwin, x86_64-apple-darwin).
- CI uses macos-13 for x86_64 builds to avoid cross-compilation issues.

### Docs

- README: add Homebrew install instructions, macOS-only artifact note, CSV diff examples, and rename detection note.
- AGENTS.md: add toolchain requirement (Rust ≥ 1.85) and diff-mode details (per-language thresholds, rename detection, CSV/Markdown outputs).

## [0.2.0] - 2025-09-14

### Added

- Diff mode (`ocloc diff`) with:
  - Range: `--base <rev>`, `--head <rev>`, `--merge-base <rev>`.
  - Index/working: `--staged` (HEAD vs index), `--working-tree` (index vs worktree).
  - Output: `--json`, `--csv`, `--markdown`, with per-file details via `--by-file`.
  - Thresholds: global `--max-code-added <N>` and per-language `--max-code-added-lang LANG:N`.
  - Rename detection (status `R`).
- Makefile helpers: `make diff`, `make diff-json`, `make diff-md` with `BASE`/`HEAD` overrides.
- Enhanced CLI stats and benchmarking.

### Changed

- Language support structure and analyzer improvements; refined table/CSV formatting.

### Tests

- Deterministic integration tests for diff mode, staged/worktree, and renames under `tests/diff_*.rs`.

## [0.1.0] - 2025-09-14

- Initial release
- Core analyzer and language detection
- JSON/CSV/table output
- Parallel traversal and progress bar
- Benchmark scripts and Make targets
