# ocloc Plan

_A concise, actionable plan for the agent to build a `cloc`-like CLI in Rust._

---

## Project Goal

Build `ocloc`, a reliable, fast, and testable CLI tool that counts lines of code with per-language breakdowns, supports common ignore rules, is parallelized for performance, and can output human-friendly and machine-readable formats (table, CSV, JSON).

## Guiding Principles

- Keep the tool simple and correct before optimizing.
- Make parsing deterministic and well-tested.
- Favor explicit ownership and small functions to keep the borrow checker manageable.
- Incrementally add features behind CLI flags.

## High-level Milestones

1. Minimal working prototype (already scaffolded)
2. Correct comment/blank/code classification with unit tests
3. File-system filters: extensions, `.gitignore`, file size, explicit include/exclude
4. Parallel processing with rayon and a configurable thread pool
5. Output formats: pretty table, JSON, CSV
6. CLI polish: progress bar, verbose logging, dry-run, config file
7. CI, tests, linting, and release process

## Minimum Viable Product (MVP)

- Walk a directory recursively and list files to analyze.
- For each file count: total, blank, comment, code lines.
- Support common extensions: rs, py, js, ts, c, cpp, java, go, sh, pl, html, css.
- Single-threaded or rayon-based processing (either OK for MVP).
- Print a human-readable summary and per-extension breakdown.
- Include unit tests for the parser.

## Feature Breakdown (agent action items)

### 1. Core file traversal

- Use `walkdir` to recurse directories.
- Respect symbolic links only when `--follow-symlinks` is provided.
- Skip binary files by size or simple heuristic (non-UTF8 first chunk).
- Produce a stream/Vec of `PathBuf` to analyze.

### 2. Comment & blank detection engine

- Create a `Language` struct:

  ```rust
  struct Language {
    name: &'static str,
    extensions: &'static [&'static str],
    line_markers: &'static [&'static str],
    block_markers: Option<(&'static str, &'static str)>,
  }
  ```

- Provide a language registry loaded from JSON (`assets/languages.json`) and a helper to find language by extension or special filename. Use `include_str!` + `once_cell::sync::Lazy` to parse once at startup and build fast lookup maps.
- The analyzer should be line-based and maintain a minimal `State` for block comments and string-literal heuristics when needed.
- Edge cases to test explicitly:

  - Block comment start and end on same line.
  - Nested block comments when language allows them (or document that nested are unsupported).
  - Triple-quoted strings in Python that are not comments.
  - Shebang lines for scripts that imply language.

### 3. Per-file analyzer API

- Signature: `fn analyze_file(path: &Path) -> Result<FileCounts>`
- `FileCounts`:

  ```rust
  struct FileCounts { files: usize, total: usize, code: usize, comment: usize, blank: usize }
  ```

- Analyzer must be well-covered by unit tests using temporary files and string fixtures.

### 4. Aggregation and output

- Aggregate results per-language (by canonical language name) and global totals.
- Output modes:

  - `--summary` (default): pretty table grouped by language
  - `--json`: machine-readable
  - `--csv`: flat rows

- Implement output behind a trait `Formatter` with implementations for Table, JSON, CSV. This makes testing easier.

### 5. CLI and UX

- Use `clap` v4 with a `Args` struct.
- Flags to include:

  - `--path <PATH>` default `.`
  - `--ext rs,py,js` to limit by extensions
  - `--ignore-file <PATH>` support `.gitignore` and custom ignore
  - `--threads <N>` or `--jobs` to control rayon threadpool
  - `--json` / `--csv` / `--pretty`
  - `--follow-symlinks`
  - `--min-size` and `--max-size` for files
  - `--progress` enable progress bar
  - `--verbose` for debug logging

### 6. Parallelism and performance

- Use `rayon::par_iter` on the collected file list.
- Beware of shared mutable state; return per-file `FileCounts` and reduce with `.reduce`.
- Add benchmarks for typical repo sizes (1000 files, 100k lines) and tune thread pool.

### 7. Ignores and heuristics

- Implement `.gitignore` parsing using `ignore` crate or simple parsing that supports patterns and negations.
- Use file size and first-chunk UTF-8 check to skip binaries.

### 8. Testing

- Unit tests for analyzer logic using inline fixtures.
- Integration tests using a `tests/fixtures` folder with small example repos.
- CI: GitHub Actions matrix for stable toolchain; run `cargo test`, `cargo clippy`, `cargo fmt -- --check`.

### 9. Packaging and release

- Add `cargo-release` or manual release notes.
- Provide `install` instructions: `cargo install --path .`.
- Optionally publish to crates.io once stable.

## Data model and types (reference)

- `Language` (described above)
- `FileCounts` (per-file or aggregated)
- `AnalyzeResult { per_lang: HashMap<String, FileCounts>, totals: FileCounts, files_analyzed: usize }`

## Acceptance criteria

- `cargo test` passes and includes tests for comment parsing edge cases.
- Command `./target/release/ocloc .` returns plausible totals on a small repo.
- `--json` produces valid JSON schema: `{ totals: {...}, languages: { "Rust": {...} } }`.
- Respect `.gitignore` by default when present.
- Reasonable performance on medium repos (e.g., tens of thousands of lines) courtesy of parallelism.

## Developer tasks (first sprint, itemized)

1. Implement `Language` registry and lookup by extension and filename.
2. Implement `analyze_file` with line and block comment handling. Add unit tests.
3. Implement file traversal using `walkdir` and basic filtering. Add integration test that runs analyzer on `tests/fixtures/simple_repo`.
4. Wire up CLI with `clap` and a `run` function that aggregates results.
5. Implement Table and JSON formatters and add tests for their outputs.
6. Add GitHub Actions workflow: run tests, clippy, fmt check on push and PR.

## Example commands the agent can run locally

- `cargo test --lib`
- `cargo run --release -- . --json > sample.json`
- `cargo clippy -- -D warnings`
- `cargo fmt -- --check`

## Notes and risks

- Python triple-quoted strings are hard to perfectly classify without AST parsing; document known limitations and aim for heuristic correctness.
- Nested block comments are language-specific; initially document as unsupported or implement per-language rules.
- `.gitignore` pattern support can get complex; leverage the `ignore` crate to avoid reimplementation.

## Extensions and future work

- Per-file language detection using content heuristics or `enry`/`linguist` style detection.
- Add a language-agnostic token-based complexity estimate.
- Add a server mode to stream results over HTTP for large-scale analysis.
- Add a `--watch` mode to update counts incrementally.

---
