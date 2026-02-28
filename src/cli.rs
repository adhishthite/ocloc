use std::path::PathBuf;

use anyhow::Result;
use clap::{ArgAction, Parser, ValueHint};

mod run_impl;
mod sub_diff;

#[allow(clippy::struct_excessive_bools)]
#[derive(Parser, Debug, Clone)]
#[command(name = "ocloc", version, about = "Fast, reliable lines-of-code counter", long_about = None)]
pub struct Args {
    /// Subcommand (use without subcommand for regular analysis)
    #[command(subcommand)]
    pub cmd: Option<Subcommand>,

    /// Path to scan (directory or file)
    #[arg(value_name = "PATH", default_value = ".", value_hint = ValueHint::AnyPath)]
    pub path: PathBuf,

    /// Limit by comma-separated extensions (no dots), e.g. rs,py,js
    #[arg(long = "ext", value_name = "LIST")]
    pub extensions: Option<String>,

    /// Use a custom ignore file (defaults to .gitignore handling)
    #[arg(long = "ignore-file", value_name = "PATH", value_hint = ValueHint::FilePath)]
    pub ignore_file: Option<PathBuf>,

    /// Output JSON instead of table
    #[arg(long = "json", action = ArgAction::SetTrue, conflicts_with = "csv")]
    pub json: bool,

    /// Output CSV instead of table
    #[arg(long = "csv", action = ArgAction::SetTrue, conflicts_with = "json")]
    pub csv: bool,

    /// Follow symlinks
    #[arg(long = "follow-symlinks", action = ArgAction::SetTrue)]
    pub follow_symlinks: bool,

    /// Minimum file size in bytes
    #[arg(long = "min-size", value_name = "BYTES")]
    pub min_size: Option<u64>,

    /// Maximum file size in bytes
    #[arg(long = "max-size", value_name = "BYTES")]
    pub max_size: Option<u64>,

    /// Set rayon thread pool size (0 = default)
    #[arg(long = "threads", value_name = "N", default_value_t = 0)]
    pub threads: usize,

    /// Verbose logging
    #[arg(long = "verbose", short = 'v', action = ArgAction::Count)]
    pub verbose: u8,

    /// Show a progress bar
    #[arg(long = "progress", action = ArgAction::SetTrue)]
    pub progress: bool,

    /// Skip empty files (files with 0 bytes)
    #[arg(long = "skip-empty", action = ArgAction::SetTrue)]
    pub skip_empty: bool,

    /// Enable memory-mapping for files larger than this size in bytes (default: 4 MiB)
    #[arg(long = "mmap-large", value_name = "BYTES")]
    pub mmap_large: Option<u64>,

    /// Disable memory-mapping optimization entirely
    #[arg(long = "no-mmap", action = ArgAction::SetTrue)]
    pub no_mmap: bool,

    /// Ultra-fast mode: prioritize speed over details
    /// - Disables progress and per-language aggregation
    /// - Minimizes metadata calls
    /// - Lowers mmap threshold aggressively (unless --no-mmap)
    #[arg(long = "ultra", action = ArgAction::SetTrue)]
    pub ultra: bool,
}

/// Runs the CLI application.
///
/// # Errors
/// Returns an error if command execution fails.
pub fn run() -> Result<()> {
    let args = Args::parse();
    if let Some(cmd) = &args.cmd {
        return match cmd {
            Subcommand::Diff(diff_args) => sub_diff::run_diff(diff_args),
        }
    }
    run_impl::run_with_args(&args)
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum Subcommand {
    /// Show LOC deltas between two git refs or working tree
    Diff(DiffArgs),
}

#[allow(clippy::struct_excessive_bools)]
#[derive(clap::Args, Debug, Clone)]
pub struct DiffArgs {
    /// Base git rev (commit, tag, or ref)
    #[arg(long)]
    pub base: Option<String>,

    /// Head git rev (defaults to HEAD)
    #[arg(long)]
    pub head: Option<String>,

    /// Use merge-base between HEAD and this ref as base
    #[arg(long = "merge-base")]
    pub merge_base: Option<String>,

    /// Compare HEAD vs index (staged changes)
    #[arg(long = "staged", action = ArgAction::SetTrue)]
    pub staged: bool,

    /// Compare index vs working tree (unstaged changes)
    #[arg(long = "working-tree", action = ArgAction::SetTrue)]
    pub working_tree: bool,

    /// Output JSON
    #[arg(long = "json", action = ArgAction::SetTrue, conflicts_with = "csv")]
    pub json: bool,

    /// Output CSV
    #[arg(long = "csv", action = ArgAction::SetTrue, conflicts_with = "json")]
    pub csv: bool,

    /// Output Markdown
    #[arg(long = "markdown", action = ArgAction::SetTrue)]
    pub markdown: bool,

    /// Include per-file detail
    #[arg(long = "by-file", action = ArgAction::SetTrue)]
    pub by_file: bool,

    /// Summary only: hide per-file details in outputs
    #[arg(long = "summary-only", action = ArgAction::SetTrue)]
    pub summary_only: bool,

    /// Fail if code added exceeds this threshold
    #[arg(long = "max-code-added")]
    pub max_code_added: Option<usize>,

    /// Per-language max code thresholds, e.g. --max-code-added-lang Rust:500,Python:100
    #[arg(long = "max-code-added-lang")]
    pub max_code_added_lang: Vec<String>,

    /// Fail if absolute net total changed exceeds this threshold
    #[arg(long = "max-total-changed")]
    pub max_total_changed: Option<usize>,

    /// Fail if number of changed files exceeds this threshold
    #[arg(long = "max-files")]
    pub max_files: Option<usize>,

    /// Explicitly fail (non-zero exit) when any threshold is exceeded (thresholds otherwise also fail)
    #[arg(long = "fail-on-threshold", action = ArgAction::SetTrue)]
    pub fail_on_threshold: bool,

    /// Limit by comma-separated extensions (no dots)
    #[arg(long = "ext", value_name = "LIST")]
    pub extensions: Option<String>,
}
