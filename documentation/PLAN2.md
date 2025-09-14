# ocloc Plan 2 — Diff Mode & CI Integration

A concrete plan to introduce a “diff mode” for ocloc so teams can measure lines-of-code deltas by language between two Git refs (or the working tree) and integrate results into CI as tables, JSON, or Markdown summaries.

---

## Objective

- Report per-language and per-file LOC deltas (code, comment, blank, total) between a base and head.
- Make it easy to run in CI and gate builds based on thresholds.
- Keep performance high by only analyzing changed files and reading blobs directly from Git.

## Scope

- New CLI subcommand: `ocloc diff` (plus supporting flags)
- Git integration using `git2` (no shelling out to `git`)
- Analyzer refactor to support in-memory content (`analyze_reader`)
- New diff formatters (table, JSON, CSV, Markdown)
- CI-friendly thresholds and non-zero exit on violations
- Tests: unit + integration with a temporary Git repo

## CLI Design

- Subcommand: `ocloc diff`
- Sources:
  - `--base <rev>`: base commit/tag/ref
  - `--head <rev>`: head commit/tag/ref (default: `HEAD`)
  - `--merge-base <rev>`: compare `merge-base(HEAD, <rev>)` to `HEAD`
  - `--staged`: compare `HEAD` vs index
  - `--working-tree`: compare index vs working tree
- Filters and output:
  - `--ext rs,py,...` filter by extensions
  - `--by-file` include per-file rows
  - `--json`, `--csv`, `--markdown`, default to table
  - `--summary-only` hide per-file details
- CI guardrails:
  - `--max-code-added <N>`
  - `--max-total-changed <N>`
  - `--max-files <N>`
  - `--fail-on-threshold` (non-zero exit if any threshold exceeded)

## What To Measure

- Per-language deltas: files changed, code_added, code_removed, comment_added, blank_added, total_net
- Per-file deltas (optional): status (A/M/D/R), language, signed deltas
- Global totals and change metadata: files_added, files_deleted, files_modified, files_renamed

## Approach

- Use `git2` to:
  - Resolve base/head commits and diffs
  - Enumerate changed files with statuses (A/M/D/R)
  - Fetch file contents as blobs in both versions without writing to disk
- Counting strategy:
  - Analyze both sides and compute deltas:
    - Added: base = 0s; head = analyze(head blob)
    - Deleted: base = analyze(base blob); head = 0s
    - Modified: delta = head − base (per metric)
    - Renamed: treat under head path/language; include rename metadata
  - Aggregate per-language (default: head language; consider option to attribute deletions to base language)
  - Parallelize per file with `rayon`

## Analyzer Refactor

- Add `fn analyze_reader<R: std::io::BufRead>(rdr: R, path_hint: &Path) -> Result<FileCounts>`
  - `path_hint` is used for language detection (extension/special name)
  - Share logic with `analyze_file` (which becomes a thin wrapper)
- Keep behavior identical across `analyze_file` and `analyze_reader` (unit tests to assert parity)

## Output Formats

- Table: aligned columns with signed integers and a “Net” column
- JSON: CI-friendly schema (see sketch)
- CSV: flat rows; suitable for spreadsheets
- Markdown: table suitable for PR comments / job summaries

### JSON Schema (sketch)

```bash
{
  "base": { "ref": "abc123", "short": "abc1234" },
  "head": { "ref": "def456", "short": "def4567" },
  "files": 42,
  "files_added": 5,
  "files_deleted": 3,
  "files_modified": 32,
  "files_renamed": 2,
  "languages": {
    "Rust": { "files": 3, "code_added": 90, "code_removed": 10, "comment_added": 5, "blank_added": 5, "total_net": 80 }
  },
  "by_file": [
    { "path": "src/lib.rs", "status": "M", "language": "Rust", "delta": { "code": 10, "comment": 1, "blank": 0, "total": 11 } }
  ],
  "totals": { "code_added": 120, "code_removed": 25, "comment_added": 7, "blank_added": 4, "total_net": 106 }
}
```

## CI Integration

- GitHub Actions (example):

```bash
- uses: actions/checkout@v4
  with:
    fetch-depth: 0
- name: Build ocloc
  run: cargo build --release --locked
- name: LOC diff (table + JSON)
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

- GitLab/Jenkins: consume JSON/CSV similarly; print Markdown/table in logs or MR comments.

## Testing Plan

- Unit tests
  - `analyze_reader` vs `analyze_file` parity on identical content
- Integration tests
  - Create a temp Git repo and cover: add, modify, delete, rename
  - Assert per-language and per-file deltas and totals
  - Verify JSON keys and signed values
- Edge cases
  - Shebang-only files without extensions
  - Rename across languages (`.txt` → `.md`): attributed under new language, net totals correct
  - Skip binaries (as with traversal heuristic)

## Performance Considerations

- Only analyze changed files
- Read blobs in memory (no temp files)
- Parallelize with rayon; avoid shared mutable state
- Reuse buffers in analyzer to minimize allocations

## Roadmap

- v0.2: Core diff mode
  - `ocloc diff --base <rev> --head <rev>` with table/json/csv
  - `analyze_reader` refactor
  - `git2` integration and initial tests
- v0.3: Working tree / staged support
  - `--staged`, `--working-tree`, `--merge-base`
- v0.4: Markdown formatter + GH Summary helper
  - `--markdown` output + README examples
  - Makefile `make diff BASE=<rev> HEAD=<rev>` (default `HEAD~1...HEAD`)
- v0.5: CI guardrails
  - Threshold flags + `--fail-on-threshold`
  - Optional per-language thresholds
- v0.6+: Enhancements
  - PR comment script using JSON output
  - SARIF or custom annotations if desired

## Docs & DX

- README: add “Diff Mode” and “CI Integration” sections with examples
- Update `documentation/` as features land
- Keep clippy, fmt, tests green

## Makefile Targets (planned)

- `make diff BASE=<rev> HEAD=<rev>` → runs `ocloc diff` with table output
- `make diff-json BASE=<rev> HEAD=<rev>` → writes `loc_diff.json`

## Risks / Known Limitations

- Language detection for renamed files: attribution defaults to head language (documented)
- Diff accuracy may vary with complex generated files or vendored code; consider future include/exclude patterns for diff mode
- Nested block comments remain out of scope (same as current analyzer)

---

This plan complements PLAN1 and focuses on CI workflows and developer insight into per-language changes per commit/PR while keeping performance and determinism as core goals.
