# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog and adheres to Semantic Versioning.

## [Unreleased]
- Add richer Markdown diff summary (top languages and files)
- Add per-language thresholds for diff mode (`--max-code-added-lang LANG:N`)
- Implement `--staged` (HEAD vs index) and `--working-tree` (index vs worktree)
- Add rename detection in diff mode
- Add tests for renames, staged, and worktree modes
- Improve README and AGENTS docs for CI/diff mode

## [0.1.0] - 2025-09-14
- Initial release
- Core analyzer and language detection
- JSON/CSV/table output
- Parallel traversal and progress bar
- Benchmark scripts and Make targets

