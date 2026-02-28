use anyhow::Result;
// use rayon::prelude::*; // not used after switching to WalkParallel
use std::collections::HashSet;
use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use crate::languages::find_language_for_path;
use crate::traversal::{TraversalOptions, build_walk_builder};
use crate::types::{AnalyzeResult, FileCounts, FileStats};
use crate::{analyzer, formatters};

use super::Args;

#[allow(clippy::too_many_lines)]
pub fn run_with_args(args: &Args) -> Result<()> {
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
            eprintln!("Extensions filter: {list}");
        }
    }
    // Configure analyzer global settings (no-mmap and threshold)
    analyzer::set_analyzer_config(args.no_mmap, args.mmap_large);
    // Build a parallel walker; we'll analyze as we traverse
    let walker = build_walk_builder(&args.path, &opts).build_parallel();

    // Track statistics (legacy counters not used after parallel refactor)

    // Progress setup (unknown length with parallel walk)
    let pb = if args.progress && !args.ultra {
        let pb = indicatif::ProgressBar::new_spinner();
        #[allow(clippy::literal_string_with_formatting_args)]
        if let Ok(style) =
            indicatif::ProgressStyle::with_template("{spinner} {pos} files {elapsed}")
        {
            pb.set_style(style.tick_chars("⠁⠃⠇⠋⠙⠸⢰⣠⣄⡆"));
        }
        Some(pb)
    } else {
        None
    };

    // Switch to direct parallel analysis over files, with batched progress updates
    let ignored_counter = Arc::new(AtomicUsize::new(0));
    let empty_counter = Arc::new(AtomicUsize::new(0));
    let progress_counter = Arc::new(AtomicUsize::new(0));
    let global_map: Arc<std::sync::Mutex<indexmap::IndexMap<String, FileCounts>>> =
        Arc::new(std::sync::Mutex::new(indexmap::IndexMap::new()));

    #[allow(clippy::items_after_statements)]
    struct ThreadAgg {
        local: indexmap::IndexMap<String, FileCounts>,
        global: Arc<std::sync::Mutex<indexmap::IndexMap<String, FileCounts>>>,
    }
    #[allow(clippy::items_after_statements)]
    impl ThreadAgg {
        fn new(global: Arc<std::sync::Mutex<indexmap::IndexMap<String, FileCounts>>>) -> Self {
            Self {
                local: indexmap::IndexMap::new(),
                global,
            }
        }
        fn add(&mut self, lang: String, counts: FileCounts) {
            let entry = self.local.entry(lang).or_default();
            entry.merge(&counts);
        }
    }
    #[allow(clippy::items_after_statements)]
    impl Drop for ThreadAgg {
        fn drop(&mut self) {
            if let Ok(mut g) = self.global.lock() {
                for (lang, counts) in self.local.drain(..) {
                    let e = g.entry(lang).or_default();
                    e.merge(&counts);
                }
            }
        }
    }

    walker.run(|| {
        let ignored_counter = ignored_counter.clone();
        let empty_counter = empty_counter.clone();
        let progress_counter = progress_counter.clone();
        let mut agg = ThreadAgg::new(global_map.clone());
        let pb_inner = pb.clone();
        Box::new(move |entry: Result<ignore::DirEntry, ignore::Error>| {
            let dent: ignore::DirEntry = match entry {
                Ok(d) => d,
                Err(_) => return ignore::WalkState::Continue,
            };
            let path = dent.path();
            if !path.is_file() {
                return ignore::WalkState::Continue;
            }

            // Count every visited file for stats/progress
            let n = progress_counter.fetch_add(1, Ordering::Relaxed) + 1;
            if let Some(pb) = &pb_inner {
                if n % 128 == 0 {
                    pb.set_position(n as u64);
                }
            }

            // Detect language (may read shebang)
            let lang_opt = find_language_for_path(path);
            if lang_opt.is_none() {
                ignored_counter.fetch_add(1, Ordering::Relaxed);
                return ignore::WalkState::Continue;
            }
            let lang = lang_opt.unwrap().to_string();

            // Guard metadata calls: only when filters require it
            if opts.min_size.is_some() || opts.max_size.is_some() || args.skip_empty {
                if let Ok(md) = fs::metadata(path) {
                    // Apply min/max size filters if provided
                    if let Some(min) = opts.min_size {
                        if md.len() < min {
                            return ignore::WalkState::Continue;
                        }
                    }
                    if let Some(max) = opts.max_size {
                        if md.len() > max {
                            return ignore::WalkState::Continue;
                        }
                    }
                    if args.skip_empty && md.len() == 0 {
                        empty_counter.fetch_add(1, Ordering::Relaxed);
                        return ignore::WalkState::Continue;
                    }
                }
            }

            let counts = analyzer::analyze_file(path).unwrap_or_else(|_| FileCounts::default());
            if counts.files > 0 {
                if args.ultra {
                    // In ultra mode, avoid per-language aggregation; accumulate totals only
                    let total_only = agg.local.entry("__TOTAL__".to_string()).or_default();
                    total_only.merge(&counts);
                } else {
                    agg.add(lang, counts);
                }
            }
            ignore::WalkState::Continue
        })
    });

    let per_lang_map = Arc::try_unwrap(global_map).unwrap().into_inner()?;

    let mut per_lang: indexmap::IndexMap<String, FileCounts> = indexmap::IndexMap::new();
    let mut totals = FileCounts::default();
    if args.ultra {
        if let Some(total_counts) = per_lang_map.get("__TOTAL__") {
            totals = *total_counts;
        }
    } else {
        for (lang, counts) in &per_lang_map {
            per_lang.insert(lang.clone(), *counts);
            totals.merge(counts);
        }
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
        total_files: progress_counter.load(Ordering::Relaxed),
        unique_files: if args.ultra {
            totals.files
        } else {
            per_lang_map.values().map(|c| c.files).sum::<usize>()
        },
        ignored_files: ignored_counter.load(Ordering::Relaxed),
        empty_files: if args.skip_empty {
            0
        } else {
            empty_counter.load(Ordering::Relaxed)
        },
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
                .unwrap_or_else(|_| args.path.clone())
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
        println!("{s}");
        return Ok(());
    }
    if args.csv {
        let s = formatters::csv::format(&analyze);
        println!("{s}");
        return Ok(());
    }

    // default pretty table (ultra still prints table, but with only totals)
    let s = formatters::table::format(&analyze);
    println!("{s}");
    Ok(())
}
