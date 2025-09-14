# ocloc

Fast, reliable lines-of-code counter in Rust with per-language breakdowns and multiple output formats (table, JSON, CSV).

Requirements

- Rust toolchain (stable) and Cargo installed

Build & Run

- Build debug: `cargo build`
- Build release: `cargo build --release`
- Run on current directory (pretty table): `cargo run -- .`
- JSON output: `cargo run -- . --json`
- CSV output: `cargo run -- . --csv`
- Limit by extensions: `cargo run -- . --ext rs,py,js`
- Control threads: `cargo run -- . --threads 8`
- Follow symlinks: `cargo run -- . --follow-symlinks`
- Size filters: `cargo run -- . --min-size 1 --max-size 100000`
- Use custom ignore file: `cargo run -- . --ignore-file tests/fixtures/ignore_repo/.customignore`
- Skip empty files: `cargo run -- . --skip-empty`

Install Locally

- `cargo install --path .`

What It Does

- Recursively scans a path while respecting `.gitignore` by default.
- Detects language by extension, special file names (e.g., `Makefile`), or shebang for scripts.
- Counts total, blank, comment, and code lines accurately for common languages.
- Outputs a per-language summary with grand totals.

Supported Languages (initial)

- Rust (`.rs`)
- Python (`.py` + shebang)
- JavaScript (`.js`, `.jsx` + shebang via `node`)
- TypeScript (`.ts`, `.tsx` + shebang via `deno`)
- C/C++ (`.c`, `.h`, `.cpp`, `.cc`, `.hpp`, `.hh`)
- Java (`.java`)
- Go (`.go`)
- Shell (`.sh` + shebang for `bash`, `sh`, `zsh`, `ksh`, `fish`)
- Perl (`.pl` + shebang)
- Ruby (`.rb` + shebang)
- PHP (`.php` + shebang)
- HTML (`.html`, `.htm`)
- CSS (`.css`)
- Makefile (`Makefile`)
- Dockerfile (`Dockerfile`)
- YAML (`.yml`, `.yaml`)
- TOML (`.toml`)
- Markdown (`.md`, `.mdx`, `.markdown`, `.mdown`, `.mkd`, `.mkdn`)
- SVG (`.svg`)
- XML (`.xml`)
- INI/Config (`.ini`, `.cfg`, `.conf`, `.properties`)
- Text (`.txt`, `.text`)
- reStructuredText (`.rst`)
- AsciiDoc (`.adoc`, `.asciidoc`)

Special filenames recognized

- Build/config: `Dockerfile`, `Makefile`, `CMakeLists.txt`, `BUILD`, `BUILD.bazel`, `WORKSPACE`, `WORKSPACE.bazel`, `MODULE.bazel`, `justfile`
- Ruby ecosystem: `Gemfile`, `Rakefile`, `Podfile`, `Capfile`, `Vagrantfile`, `Brewfile`
- Env/config: `.editorconfig`, `.env`, `.envrc`
- Docs/legal: `README`, `LICENSE`, `COPYING`, `CHANGELOG`, `CHANGES`, `NEWS`

Notes

- Python triple-quoted strings are not parsed as comments; this is a known limitation.
- Nested block comments are not supported.
- For best performance on large repos, use `--release` and adjust `--threads`.

Development

- Format: `cargo fmt` (auto-fixes). Pre-commit hook runs this automatically.
- Lint: `cargo clippy -- -D warnings`
- Test: `cargo test`
- Always run: `cargo fmt` and `cargo clippy -D warnings` before pushing

Git Hooks

- A pre-commit hook script is provided at `scripts/pre-commit`.
- Install it (symlink) so commits auto-format, lint, and test:
  - `bash scripts/install-git-hooks.sh`
  - or manually:
    - `mkdir -p .git/hooks`
    - `ln -sf ../../scripts/pre-commit .git/hooks/pre-commit`
    - `chmod +x scripts/pre-commit`
    - Optionally: `chmod +x .git/hooks/pre-commit`

JSON Schema

- Top-level object:
  - `totals`: `{ files, total, code, comment, blank }`
  - `languages`: object keyed by language name with the same `{ files, total, code, comment, blank }` structure
  - `files_analyzed`: total number of files counted

Example:

```bash
{
  "totals": { "files": 12, "total": 345, "code": 280, "comment": 25, "blank": 40 },
  "languages": {
    "Rust": { "files": 6, "total": 200, "code": 170, "comment": 10, "blank": 20 },
    "Python": { "files": 3, "total": 80, "code": 60, "comment": 10, "blank": 10 }
  },
  "files_analyzed": 12
}
```

CSV Schema

- Header: `language,files,code,comment,blank,total`
- Rows include one per language plus a final `Total` row.

Progress & Verbose

- Enable progress bar: `--progress` (shows a spinner and counts)
- Verbose logging: `-v` (repeat up to `-vv` for more detail)

How to add a new language

- Edit `assets/languages.json` and add an object with fields: `name`, `extensions`, `line_markers`, and optional `block_markers` (array of two strings) and `special_filenames`.
- For HTML/XML-like formats (Markdown, SVG, XML), use `"block_markers": ["<!--", "-->"]` and leave `line_markers` empty.
- For INI-like formats, include both `;` and `#` in `line_markers`.
- Add or update tests:
  - Detection: extend `src/languages.rs` tests to cover new extensions or filenames.
  - Analysis: add a unit test in `src/analyzer.rs` for comment/blank/code classification.
- Run `cargo test`, `cargo fmt`, and `cargo clippy -- -D warnings`.
- Update the Supported Languages list above if user-facing.
