# Repository Guidelines

## Project Structure & Module Organization

- `src/` — Rust sources; `main.rs` is the entry point. Future modules may include `cli/`, `analyzer/`, and `formatters/` within `src/`.
- `tests/` — Integration tests and fixtures (create as features land). Use `tests/fixtures/` for small sample repos.
- `documentation/` — Design notes and roadmap; start with `documentation/PLAN1.md`.
- `target/` — Build artifacts (ignored by Git).

## Build, Test, and Development Commands

- Build: `cargo build` (debug) or `cargo build --release`.
- Run: `cargo run -- .` to analyze the current directory. Example JSON: `cargo run --release -- . --json > out.json`.
- Test: `cargo test` (all), `cargo test --lib` (unit only).
- Diff tests: `cargo test --tests diff_` to run diff-mode tests only.
- Lint/format: `cargo fmt -- --check` and `cargo clippy -- -D warnings`.
- Install locally: `cargo install --path .`.
- Toolchain: Rust stable ≥ 1.85 (see `rust-version` in Cargo.toml).

## Coding Style & Naming Conventions

- Edition: Rust 2024; indent 4 spaces; rely on `rustfmt` defaults.
- Naming: `snake_case` for functions/vars, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for constants.
- Structure small, testable modules; prefer pure functions and explicit ownership.
- Required: run `cargo fmt` and fix all Clippy warnings before pushing.

## Testing Guidelines

- Unit tests live next to code with `#[cfg(test)]` modules.
- Integration tests go in `tests/`; keep fixtures minimal and deterministic.
- Focus coverage on parsing edge cases, language detection, ignores, and aggregation logic.
- Name tests clearly (e.g., `handles_block_comments`, `detects_python_shebang`).

## Commit & Pull Request Guidelines

- Use Conventional Commits: `feat:`, `fix:`, `refactor:`, `test:`, `docs:` (e.g., `feat(cli): add --json output`).
- One focused change per PR; link issues; include sample CLI output or `--help` diffs when relevant.
- Ensure CI passes: build, tests, `fmt`, and `clippy`.

## Security & Configuration Tips

- Do not commit secrets or large binaries; keep fixtures small.
- Respect `.gitignore`; avoid following symlinks by default; document any deviation.

## Agent-Specific Notes

- Align changes with `documentation/PLAN1.md` milestones.
- Diff mode plan/spec is in `documentation/PLAN2.md`; follow it for changes under `src/cli/sub_diff.rs`, `src/vcs.rs`, and `src/types_diff.rs`.

## Diff Mode & CI

- Subcommand `ocloc diff` supports:
  - Range: `--base <rev>`, `--head <rev>`, or `--merge-base <rev>`
  - Index/working: `--staged` (HEAD vs index), `--working-tree` (index vs worktree)
  - Output: `--json`, `--csv`, `--markdown`, `--by-file`
  - Thresholds: `--max-code-added <N>`, `--max-total-changed <N>`, `--max-files <N>`
  - Use `--fail-on-threshold` to return non-zero on any violation
  - Use `--summary-only` to hide per-file details
  - Per-language thresholds: repeatable `--max-code-added-lang LANG:N`
- Rename detection is enabled in diff mode; renamed files are reported as status `R`.
- Make targets: `make diff`, `make diff-json`, `make diff-md` with `BASE`/`HEAD` overrides.
- Tests live in `tests/diff_*.rs` and create ephemeral git repos; keep them deterministic.
- Prefer incremental patches; avoid drive-by refactors; keep file layout stable.

### CI/Release Notes

- CI builds release binaries and runs fmt/clippy/tests.
- Release workflow publishes macOS (`aarch64-apple-darwin`, `x86_64-apple-darwin`), Linux (`x86_64-unknown-linux-gnu`), and Windows (`x86_64-pc-windows-msvc`) artifacts, and updates a Homebrew tap for macOS when configured.

## Adding support for a new language

- Edit `assets/languages.json`:
  - Add an object with `name`, `extensions`, `line_markers`, optional `block_markers` (two strings), and optional `special_filenames`.
  - For HTML/XML-like formats (Markdown, SVG, XML), set `block_markers` to `["<!--", "-->"]` and keep `line_markers` empty.
  - For INI-like formats, include both `;` and `#` in `line_markers`.
- Detection and analyzer behavior:
  - The analyzer is line-based: trimmed-empty lines are `blank`; pure comment lines increment `comment`; mixed code+comment lines count as `code`.
  - Block comments across multiple lines count each line as `comment`; single-line blocks count as one `comment` line.
- Tests:
  - Extend unit tests in `src/languages.rs` for detection via new extensions or filenames.
  - Add analyzer tests in `src/analyzer.rs` for new comment/blank/code rules.
- Output formats (Table/JSON/CSV):
  - Aggregation is generic; new languages appear automatically. Formatter tests can assert names appear.
- Development hygiene:
  - Always run `cargo fmt` and `cargo clippy -- -D warnings` before pushing, and `cargo test` locally.
