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
