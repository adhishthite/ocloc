use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;

use crate::languages::find_language_for_path;
use crate::traversal::{TraversalOptions, collect_files};
use crate::types::{AnalyzeResult, FileCounts};
use crate::{analyzer, formatters};

use super::Args;

pub fn run_with_args(args: Args) -> Result<()> {
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
    if args.verbose > 0 {
        eprintln!("Found {} files to analyze", files.len());
    }

    // Progress setup
    let pb = if args.progress {
        let pb = indicatif::ProgressBar::new(files.len() as u64);
        pb.set_style(
            indicatif::ProgressStyle::with_template("{spinner} {pos}/{len} files {wide_bar} {eta}")
                .unwrap()
                .tick_chars("⠁⠃⠇⠋⠙⠸⢰⣠⣄⡆"),
        );
        Some(pb)
    } else {
        None
    };

    let results: Vec<(String, FileCounts)> = files
        .par_iter()
        .filter_map(|path| {
            let lang = find_language_for_path(path)?;
            Some((lang.to_string(), path))
        })
        .filter(|(lang, path)| {
            if lang.is_empty() {
                return false;
            }
            // file exists & readable basic check
            fs::metadata(path).is_ok()
        })
        .map(|(lang, path)| {
            let counts = analyzer::analyze_file(path).unwrap_or_else(|_| FileCounts::default());
            if let Some(ref pb) = pb {
                pb.inc(1);
            }
            (lang, counts)
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

    let analyze = AnalyzeResult {
        per_lang,
        totals,
        files_analyzed: totals.files,
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
