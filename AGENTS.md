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
- Lint/format: `cargo fmt -- --check` and `cargo clippy -- -D warnings`.
- Install locally: `cargo install --path .`.

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
- Prefer incremental patches; avoid drive-by refactors; keep file layout stable.

## Adding support for a new language

- Update `src/languages.rs`:
  - Add a `Language` entry with a canonical `name`, `extensions`, `line_markers`, and optional `block_markers`.
  - For HTML/XML-like formats (Markdown, SVG, XML), use `block_markers: Some(("<!--", "-->"))` and empty `line_markers`.
  - For INI-like formats, include both `;` and `#` as `line_markers`.
- Analyzer behavior (`src/analyzer.rs`):
  - The analyzer is line-based: blank lines are trimmed-empty; pure comment lines increment `comment`; mixed code+comment lines count as `code`.
  - Block comments spanning multiple lines count each line inside as `comment`; single-line block comments count as one `comment` line.
- Tests:
  - Add/extend unit tests in `src/languages.rs` to validate detection for new extensions (and special filenames if needed).
  - Add analyzer tests in `src/analyzer.rs` to cover block and line comment behavior for the new language.
- Output formats (Table/JSON/CSV):
  - Aggregation is generic; new languages appear automatically. Optionally add small formatter tests to ensure names render.
- Development hygiene:
  - Always run `cargo fmt` and `cargo clippy -- -D warnings` before pushing.
  - Run `cargo test` locally.
