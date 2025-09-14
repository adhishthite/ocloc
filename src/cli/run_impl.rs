use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::time::Instant;

use crate::languages::find_language_for_path;
use crate::traversal::{TraversalOptions, collect_files};
use crate::types::{AnalyzeResult, FileCounts, FileStats};
use crate::{analyzer, formatters};

use super::Args;

pub fn run_with_args(args: Args) -> Result<()> {
    let start_time = Instant::now();
    if args.threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(args.threads)
            .build_global()
            .ok();
    }

    let allowed_exts: Option<HashSet<String>> = args.extensions.as_ref().map(|s| {
        s.split(',')
            .filter(|t| !t.trim().is_empty())
            .map(|t| t.trim().trim_start_matches('.').to_ascii_lowercase())
            .collect()
    });

    let opts = TraversalOptions {
        follow_symlinks: args.follow_symlinks,
        min_size: args.min_size,
        max_size: args.max_size,
        ignore_file: args.ignore_file.clone(),
        allowed_exts,
    };

    if args.verbose > 0 {
        eprintln!("Scanning path: {}", args.path.display());
        if let Some(ref list) = args.extensions {
            eprintln!("Extensions filter: {}", list);
        }
    }
    let files = collect_files(&args.path, opts)?;
    let total_files = files.len();
    if args.verbose > 0 {
        eprintln!("Found {} files to analyze", total_files);
    }

    // Track statistics
    let mut empty_files = 0usize;
    let mut ignored_files = 0usize;

    // Progress setup
    let pb = if args.progress {
        let pb = indicatif::ProgressBar::new(files.len() as u64);
        if let Ok(style) =
            indicatif::ProgressStyle::with_template("{spinner} {pos}/{len} files {wide_bar} {eta}")
        {
            pb.set_style(style.tick_chars("⠁⠃⠇⠋⠙⠸⢰⣠⣄⡆"));
        }
        Some(pb)
    } else {
        None
    };

    // First pass: categorize files
    let categorized: Vec<(Option<String>, &std::path::PathBuf, bool)> = files
        .iter()
        .map(|path| {
            let lang = find_language_for_path(path);
            let is_empty = fs::metadata(path).map(|m| m.len() == 0).unwrap_or(false);
            (lang.map(|l| l.to_string()), path, is_empty)
        })
        .collect();

    // Count statistics
    for (lang, _path, is_empty) in &categorized {
        if lang.is_none() || lang.as_ref().map(|l| l.is_empty()).unwrap_or(true) {
            ignored_files += 1;
        } else if *is_empty {
            empty_files += 1;
        }
    }

    let results: Vec<(String, FileCounts)> = categorized
        .into_par_iter()
        .filter_map(|(lang, path, is_empty)| {
            if let Some(l) = lang
                && !l.is_empty()
            {
                // Skip empty files if the flag is set
                if args.skip_empty && is_empty {
                    if args.verbose > 1 {
                        eprintln!("Skipping empty file: {}", path.display());
                    }
                    return None;
                }
                return Some((l, path));
            }
            None
        })
        .filter_map(|(lang, path)| {
            // Check if file still exists and is readable
            if fs::metadata(path).is_ok() {
                let counts = analyzer::analyze_file(path).unwrap_or_else(|_| FileCounts::default());
                if let Some(ref pb) = pb {
                    pb.inc(1);
                }
                Some((lang, counts))
            } else {
                None
            }
        })
        .collect();

    let mut per_lang: indexmap::IndexMap<String, FileCounts> = indexmap::IndexMap::new();
    let mut totals = FileCounts::default();

    for (lang, counts) in results.into_iter() {
        let entry = per_lang.entry(lang).or_default();
        entry.merge(&counts);
        totals.merge(&counts);
    }

    // Sort per_lang by descending code (then total) before serializing/printing
    let mut per_lang: Vec<(String, FileCounts)> = per_lang.into_iter().collect();
    per_lang.sort_by(|a, b| {
        b.1.code
            .cmp(&a.1.code)
            .then_with(|| b.1.total.cmp(&a.1.total))
            .then_with(|| a.0.cmp(&b.0))
    });
    let per_lang: indexmap::IndexMap<String, FileCounts> = per_lang.into_iter().collect();

    let elapsed = start_time.elapsed().as_secs_f64();

    let stats = FileStats {
        total_files,
        unique_files: total_files - ignored_files, // Files that have a recognized language
        ignored_files,
        empty_files: if args.skip_empty { 0 } else { empty_files },
        elapsed_seconds: elapsed,
    };

    let analyze = AnalyzeResult {
        per_lang,
        totals,
        files_analyzed: totals.files,
        stats: Some(stats),
        analyzed_path: Some(
            args.path
                .canonicalize()
                .unwrap_or(args.path.clone())
                .display()
                .to_string(),
        ),
    };

    if let Some(pb) = pb {
        pb.finish_and_clear();
    }
    if args.verbose > 1 {
        eprintln!(
            "Totals: files={}, code={}, comment={}, blank={}, total={}",
            analyze.totals.files,
            analyze.totals.code,
            analyze.totals.comment,
            analyze.totals.blank,
            analyze.totals.total
        );
    }

    if args.json {
        let s = serde_json::to_string_pretty(&analyze)?;
        println!("{}", s);
        return Ok(());
    }
    if args.csv {
        let s = formatters::csv::format(&analyze);
        println!("{}", s);
        return Ok(());
    }

    // default pretty table
    let s = formatters::table::format(&analyze);
    println!("{}", s);
    Ok(())
}
