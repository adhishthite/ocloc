# ocloc 🚀

**A blazingly fast lines-of-code counter and analyzer written in Rust** - up to 25x faster than cloc!

`ocloc` (pronounced "oh-clock") is a modern, high-performance alternative to traditional code counting tools. It leverages Rust's safety and parallelism to deliver lightning-fast analysis of your codebase while providing beautiful, informative output.

## ✨ Features

- **⚡ Blazing Fast**: 6-23x faster than cloc on real-world codebases
- **📊 Beautiful Reports**: Professional output with file statistics, performance metrics, and formatted tables
- **🎯 Accurate Detection**: Recognizes 50+ languages by extension, filename, and shebang
- **🔧 Flexible Output**: Table (with colors!), JSON, or CSV formats
- **🚶 Respects .gitignore**: Automatically follows your repository's ignore rules
- **⚙️ Parallel Processing**: Leverages all CPU cores for maximum performance
- **📈 Real-time Progress**: Optional progress bar for large repositories

## 📸 Example Output

```text
═══════════════════════════════════════════════════════════════════════════════════
                         REPORT FOR: ELASTICSEARCH
                         Generated: September 14, 2025 at 07:37 PM
═══════════════════════════════════════════════════════════════════════════════════

File Statistics:
─────────────────────────────────────
  Text Files    :     33,417
  Unique Files  :     31,432
  Ignored Files :      1,985
  Empty Files   :        116
─────────────────────────────────────

Performance:
─────────────────────────────────────
  Elapsed Time  :       2.35 s
  Files/sec     :    13,365.5
  Lines/sec     :   2,331,653
─────────────────────────────────────

Language               files             blank           comment              code             Total
----------------------------------------------------------------------------------------------------
Java                  24,580           557,465           506,620         3,651,048         4,715,133
YAML                   2,144            33,104             5,100           315,863           354,067
Markdown               2,112            42,852               162           120,864           163,878
JSON                   1,319                32                 0           106,530           106,562
Text                     761            17,614                 0            86,349           103,963
----------------------------------------------------------------------------------------------------
Total                 31,432           654,995           515,965         4,312,465         5,483,425
----------------------------------------------------------------------------------------------------
```

## 🏎️ Performance Comparison

Real-world benchmarks on popular repositories:

| Repository                | Files | Lines | cloc Time | ocloc Time | **Speedup**      |
| ------------------------- | ----- | ----- | --------- | ---------- | ---------------- |
| Small (elasticgpt-agents) | 302   | 53K   | 0.45s     | 0.07s      | **6.4x faster**  |
| Large (elasticsearch)     | 31K   | 5.5M  | 56s       | 2.35s      | **23.8x faster** |

### 🚀 ocloc processes over **2.3 million lines per second** on modern hardware!

Think about that for a moment - ocloc can analyze:

- The entire Linux kernel (~30M lines) in **~13 seconds**
- A typical microservice (~50K lines) in **20 milliseconds**
- Your entire monorepo while you blink

Notes:

- Results from M2 MacBook Pro; your hardware may vary
- See the Benchmarking section for reproduction steps

## 🚀 Installation

### From Source (GitHub)

```bash
# Build and install
cargo install --path .

# Or run directly
cargo run --release -- /path/to/analyze
```

### Release & Distribute (maintainers)

To publish a new release to crates.io:

1. Ensure `Cargo.toml` metadata (description, license, repository, keywords, categories) is accurate.
2. Log in locally: `cargo login` and enter your crates.io API token.
3. Tag and push a release: `git tag v0.1.0 && git push origin v0.1.0`.
4. Publish: `cargo publish` (use `cargo publish --dry-run` first).

GitHub Releases are created automatically when pushing a `v*.*.*` tag. The workflow builds and attaches versioned artifacts:

- `ocloc-<version>-<target>.tar.gz` (Linux/macOS)
- `ocloc-<version>-<target>.zip` (Windows)
- `SHA256SUMS.txt` (checksums for all artifacts)

Example targets: `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`.

Homebrew tap updates can be automated if you set secrets `TAP_REPO` and `TAP_TOKEN`. The formula will be updated to the latest tag with the correct tarball and SHA256.

### Version bump helper

Use the Make target to bump versions across Cargo.toml and CHANGELOG.md:

```bash
make bump VERSION=0.1.1
# Review the changes, commit, and run:
make release-all
```

### Prerequisites

- Rust toolchain (stable)
- Cargo

## 📖 Usage

### Basic Usage

```bash
# Analyze current directory
ocloc .

# Analyze specific path
ocloc /path/to/project

# With options
ocloc . --skip-empty --progress
```

### Command Line Options

```bash
ocloc [OPTIONS] [PATH]

Options:
  --json              Output as JSON
  --csv               Output as CSV
  --skip-empty        Skip empty files (0 bytes)
  --progress          Show progress bar
  --ext <LIST>        Filter by extensions (e.g., rs,py,js)
  --threads <N>       Set thread count (0 = auto)
  --follow-symlinks   Follow symbolic links
  --min-size <BYTES>  Minimum file size
  --max-size <BYTES>  Maximum file size
  --ignore-file <PATH> Custom ignore file
  -v, --verbose       Verbose output
  -h, --help          Print help
  -V, --version       Print version
```

### Examples

```bash
# Analyze only Rust and Python files
ocloc . --ext rs,py

# Export as JSON for further processing
ocloc . --json > stats.json

# Skip empty files (like __init__.py)
ocloc . --skip-empty

# Show progress for large repositories
ocloc /large/repo --progress

# Use custom thread count
ocloc . --threads 16
```

## 📋 Supported Languages

ocloc supports 50+ programming languages and file formats:

**Programming Languages**: Rust, Python, JavaScript, TypeScript, Java, C/C++, Go, Ruby, PHP, Perl, Shell/Bash, and more

**Markup & Config**: HTML, XML, JSON, YAML, TOML, Markdown, INI/Config files

**Special Files**: Dockerfile, Makefile, CMakeLists.txt, Gemfile, Rakefile, and various build files

## 📊 Output Formats

### JSON Output

```json
{
  "languages": {
    "Rust": {
      "files": 15,
      "total": 1338,
      "code": 1138,
      "comment": 75,
      "blank": 125
    },
    "Python": { "files": 2, "total": 4, "code": 2, "comment": 2, "blank": 0 }
  },
  "totals": {
    "files": 29,
    "total": 2996,
    "code": 2551,
    "comment": 90,
    "blank": 355
  },
  "files_analyzed": 29
}
```

### CSV Output

```csv
language,files,code,comment,blank,total
Rust,15,1138,75,125,1338
Python,2,2,2,0,4
Total,29,2551,90,355,2996
```

## 🔧 Development

### Using the Makefile

A comprehensive Makefile is provided for common development tasks:

```bash
# Show all available commands
make help

# Quick start commands
make build          # Build debug version
make release        # Build optimized release
make install        # Install to ~/.cargo/bin
make test           # Run all tests
make check          # Run format, lint, and tests
make ci             # Run full CI pipeline

# Development commands
make run            # Run on current directory (debug)
make run-release    # Run on current directory (release)
make fmt            # Format code
make lint           # Run clippy linter
make clean          # Remove build artifacts
make compare        # Compare performance with cloc
make version-show   # Print version from Cargo.toml
make release-all    # Run check, build release, tag and push
make tag-release    # Tag current version and push
make publish-crates-dry  # Dry run crates.io publish
make publish-crates      # Publish to crates.io (requires cargo login)
```

## 🧪 Benchmarking

Two helper scripts compare ocloc with cloc on public repositories. cloc is optional; if not installed, the scripts skip its run and show a warning.

- Medium repo (yt-dlp): `bash scripts/benchmark-small.sh`
- Large repo (elasticsearch): `bash scripts/benchmark-large.sh`

Or via Makefile targets (builds release first):

- `make bench-small`
- `make bench-large`

Notes:

- Scripts clone repos to a temporary directory and delete all files afterward.
- They build `ocloc` in release mode if not already built.
- Output includes timing and totals in a compact table, plus a speedup when cloc is available.

## 🔀 Diff Mode (CI-friendly)

Analyze what changed between two Git refs and aggregate LOC deltas by language. Useful for PRs and CI gates.

Basic usage:

```bash
# Compare HEAD~1..HEAD
ocloc diff --base HEAD~1 --head HEAD

# Use merge-base with a branch
ocloc diff --merge-base origin/main

# Machine-readable output
ocloc diff --base HEAD~1 --head HEAD --json > loc_diff.json
ocloc diff --base HEAD~1 --head HEAD --markdown > loc_diff.md

# Include per-file rows in JSON/CSV/Markdown and richer Markdown summary
ocloc diff --base HEAD~1 --head HEAD --json --by-file
ocloc diff --base HEAD~1 --head HEAD --markdown --by-file > summary.md

# Gate on thresholds (non-zero exit when exceeded)
ocloc diff --base HEAD~1 --head HEAD --max-code-added 2500
# Per-language thresholds (repeatable): LANG:N pairs
ocloc diff --base HEAD~1 --head HEAD --max-code-added-lang Rust:800 --max-code-added-lang Python:200
```

Makefile helpers:

```bash
# Defaults to BASE=HEAD~1 and HEAD=HEAD
make diff
make diff-json    # writes loc_diff.json
make diff-md      # writes loc_diff.md

# Override base/head
make diff BASE=origin/main HEAD=HEAD
```

GitHub Actions snippet:

```yaml
- uses: actions/checkout@v4
  with:
    fetch-depth: 0
- name: Build
  run: cargo build --release --locked
- name: LOC diff
  run: |
    BASE="${{ github.event.pull_request.base.sha || github.event.before }}"
    HEAD="${{ github.sha }}"
    ./target/release/ocloc diff --base "$BASE" --head "$HEAD" --json > loc_diff.json
    ./target/release/ocloc diff --base "$BASE" --head "$HEAD" --markdown > loc_diff.md
    if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
      cat loc_diff.md >> "$GITHUB_STEP_SUMMARY"
    fi
- name: Gate on LOC increase
  run: |
    CODE_ADDED=$(jq '.totals.code_added' loc_diff.json)
    if [ "$CODE_ADDED" -gt 2500 ]; then
      echo "Too many code lines added: $CODE_ADDED" >&2
      exit 1
    fi
```

Local tips:
- `ocloc diff --staged` compares your staged changes to HEAD.
- `ocloc diff --working-tree` compares unstaged working changes to the index.
- Use `--ext` to limit analysis to specific languages (e.g., `--ext rs,py`).

### Manual Build Commands

```bash
# Debug build
cargo build

# Release build (recommended for performance)
cargo build --release

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings
```

### Git Hooks

A pre-commit hook is provided to ensure code quality:

```bash
# Install the pre-commit hook
bash scripts/install-git-hooks.sh
```

## 🎯 Why ocloc?

While [cloc](https://github.com/AlDanial/cloc) has been the gold standard for counting lines of code for years, ocloc brings several advantages:

1. **Speed**: Written in Rust with parallel processing, ocloc is 6-23x faster
2. **Throughput**: Processes over 2.3 million lines per second
3. **Modern Output**: Beautiful, informative reports with performance metrics
4. **Memory Efficient**: Rust's ownership model ensures efficient memory usage
5. **Type Safe**: Catches errors at compile time, reducing runtime issues

## 🙏 Acknowledgments

ocloc is inspired by [cloc](https://github.com/AlDanial/cloc) by Al Danial. We stand on the shoulders of giants and are grateful for the groundwork laid by cloc over the years. If you need advanced features like `--git-diff`, `--by-file-by-lang`, or other specialized options, cloc remains an excellent choice.

ocloc aims to be a modern, performance-focused alternative for the common use case of quickly analyzing codebases.

## 🤝 Contributing

Contributions are welcome! To add support for a new language:

1. Edit `assets/languages.json` with the language definition
2. Add tests in `src/languages.rs` and `src/analyzer.rs`
3. Run tests and ensure all checks pass
4. Submit a PR with your changes

## 📄 License

MIT License - see LICENSE file for details

## ⚡ Performance Tips

- Always use `--release` builds for best performance
- Use `--threads` to control parallelism (default: all cores)
- Use `--skip-empty` to skip empty files for faster analysis
- For very large repos, combine with `--progress` to monitor progress

---

Built with ❤️ and ⚡ in Rust
