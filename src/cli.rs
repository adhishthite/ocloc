use std::path::PathBuf;

use anyhow::Result;
use clap::{ArgAction, Parser, ValueHint};

mod run_impl;

#[derive(Parser, Debug, Clone)]
#[command(name = "ocloc", version, about = "Fast, reliable lines-of-code counter", long_about = None)]
pub struct Args {
    /// Path to scan (directory or file)
    #[arg(value_name = "PATH", default_value = ".", value_hint = ValueHint::DirPath)]
    pub path: PathBuf,

    /// Limit by comma-separated extensions (no dots), e.g. rs,py,js
    #[arg(long = "ext", value_name = "LIST")]
    pub extensions: Option<String>,

    /// Use a custom ignore file (defaults to .gitignore handling)
    #[arg(long = "ignore-file", value_name = "PATH", value_hint = ValueHint::FilePath)]
    pub ignore_file: Option<PathBuf>,

    /// Output JSON instead of table
    #[arg(long = "json", action = ArgAction::SetTrue)]
    pub json: bool,

    /// Output CSV instead of table
    #[arg(long = "csv", action = ArgAction::SetTrue)]
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
}

pub fn run() -> Result<()> {
    let args = Args::parse();
    run_impl::run_with_args(args)
}
