# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog and adheres to Semantic Versioning.

## [Unreleased]

## [0.4.0] - 2025-09-15

### Added

- Memory-mapping support for large files (>64MB) with automatic fallback
- Ultra-fast mode (`--ultra`/`-u`) that skips comment analysis for maximum speed
- Skip empty files option (`--skip-empty`) to exclude zero-byte files from analysis
- Custom benchmark script with detailed performance metrics
- Enhanced benchmark scripts with configurable runs and comparison tools

### Improved

- Significant performance improvements through memory-mapped I/O for large files
- Optimized analyzer with better buffer reuse and reduced allocations
- More efficient traversal with parallel processing optimizations
- Fixed clippy warnings for better code quality (type complexity, unnecessary clones)

### Changed

- Default behavior now uses memory mapping for files larger than 64MB
- Improved error handling and fallback mechanisms for file reading

## [0.3.1] - 2025-09-15

### Fixed

- Include updated Cargo.lock in the repository to support `--locked` builds in CI.

### Added

- Diff CSV per-file output when `--by-file` is set.
- New flags: `--summary-only`, `--max-total-changed`, `--max-files`, `--fail-on-threshold`.
- JSON now includes `base`/`head` refs with short SHAs; totals include `code_removed`.

### Docs

- README and AGENTS updated for new flags, CSV format, and macOS release notes.

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
