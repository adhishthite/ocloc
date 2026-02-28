use anyhow::{Context, Result, bail};
use std::collections::HashSet;
use std::io::Cursor;
use std::path::Path;

use crate::analyzer;
use crate::languages::find_language_for_path;
use crate::types::FileCounts;
use crate::types_diff::{DiffPerFile, DiffSummary, GitRefInfo, LineDelta};
use crate::vcs::VcsContext;

use super::DiffArgs;

#[allow(clippy::too_many_lines)]
pub fn run_diff(args: &DiffArgs) -> Result<()> {
    // Validate incompatible flags
    if args.staged && args.working_tree {
        bail!("--staged and --working-tree are mutually exclusive");
    }
    // Determine repo root from CWD
    let vcs = VcsContext::open(Path::new("."))?;

    // diff mode selection
    #[allow(clippy::items_after_statements)]
    enum Mode {
        Range,
        Staged,
        Worktree,
    }
    let mode = if args.staged {
        Mode::Staged
    } else if args.working_tree {
        Mode::Worktree
    } else {
        Mode::Range
    };

    let (changes, base_ref, head_ref, base_info, head_info) = match mode {
        Mode::Staged => {
            let head_oid = vcs.head_oid().ok();
            let base_info = head_oid.map(|o| GitRefInfo {
                reference: Some(o.to_string()),
                short: Some(format!("{o:.7}")),
            });
            let head_info = Some(GitRefInfo {
                reference: Some("INDEX".to_string()),
                short: Some("INDEX".to_string()),
            });
            (
                vcs.diff_head_to_index()?,
                Some("HEAD".to_string()),
                Some("INDEX".to_string()),
                base_info,
                head_info,
            )
        }
        Mode::Worktree => {
            let base_info = Some(GitRefInfo {
                reference: Some("INDEX".to_string()),
                short: Some("INDEX".to_string()),
            });
            let head_info = Some(GitRefInfo {
                reference: Some("WORKDIR".to_string()),
                short: Some("WORKDIR".to_string()),
            });
            (
                vcs.diff_index_to_workdir()?,
                Some("INDEX".to_string()),
                Some("WORKDIR".to_string()),
                base_info,
                head_info,
            )
        }
        Mode::Range => {
            let head_oid = match args.head.as_deref() {
                Some(h) => vcs.resolve_oid(h)?,
                None => vcs.head_oid()?,
            };
            let base_oid = if let Some(mb) = args.merge_base.as_deref() {
                let other = vcs.resolve_oid(mb)?;
                vcs.merge_base(head_oid, other)?
            } else if let Some(b) = args.base.as_deref() {
                vcs.resolve_oid(b)?
            } else {
                vcs.resolve_oid("HEAD~1")
                    .context("resolve default base HEAD~1")?
            };
            let base_info = Some(GitRefInfo {
                reference: Some(base_oid.to_string()),
                short: Some(format!("{base_oid:.7}")),
            });
            let head_info = Some(GitRefInfo {
                reference: Some(head_oid.to_string()),
                short: Some(format!("{head_oid:.7}")),
            });
            (
                vcs.diff_between(base_oid, head_oid)?,
                args.base.clone(),
                args.head.clone().or_else(|| Some("HEAD".to_string())),
                base_info,
                head_info,
            )
        }
    };

    // Optional extension filter
    let allowed_exts: Option<HashSet<String>> = args.extensions.as_ref().map(|s| {
        s.split(',')
            .filter(|t| !t.trim().is_empty())
            .map(|t| t.trim().trim_start_matches('.').to_ascii_lowercase())
            .collect()
    });

    // Process changes in parallel
    let mut per_file: Vec<DiffPerFile> = Vec::new();
    let mut per_lang: indexmap::IndexMap<String, LineDelta> = indexmap::IndexMap::new();

    let items: Vec<_> = changes
        .into_iter()
        .filter_map(|c| {
            let path_for_lang = c.new_path.as_ref().or(c.old_path.as_ref()).cloned();
            let path_hint = path_for_lang?;

            if let Some(ref allowed) = allowed_exts {
                if let Some(ext) = path_hint.extension().and_then(|s| s.to_str()) {
                    if !allowed.contains(&ext.to_ascii_lowercase()) {
                        return None;
                    }
                } else {
                    return None;
                }
            }

            let lang = find_language_for_path(&path_hint).unwrap_or("Unknown");

            // Analyze base and head content with sensible fallbacks
            #[allow(clippy::option_if_let_else)]
            let base_counts = if let Some(bytes) = vcs.read_blob_bytes(c.oids.old) {
                analyze_bytes(&bytes, &path_hint).unwrap_or_default()
            } else if let Some(ref p) = c.old_path {
                if let Some(bytes) = vcs.read_index_blob_bytes(p) {
                    analyze_bytes(&bytes, &path_hint).unwrap_or_default()
                } else {
                    analyzer::analyze_file(p).unwrap_or_default()
                }
            } else {
                FileCounts::default()
            };
            let head_counts = if let Some(bytes) = vcs.read_blob_bytes(c.oids.new) {
                analyze_bytes(&bytes, &path_hint).unwrap_or_default()
            } else if let Some(ref p) = c.new_path {
                analyzer::analyze_file(p).unwrap_or_default()
            } else {
                FileCounts::default()
            };

            #[allow(clippy::cast_possible_wrap)]
            let code_delta = head_counts.code as isize - base_counts.code as isize;
            #[allow(clippy::cast_possible_wrap)]
            let comment_delta = head_counts.comment as isize - base_counts.comment as isize;
            #[allow(clippy::cast_possible_wrap)]
            let blank_delta = head_counts.blank as isize - base_counts.blank as isize;
            #[allow(clippy::cast_possible_wrap)]
            let total_delta = head_counts.total as isize - base_counts.total as isize;

            let status = c.status;
            let lang = lang.to_string();
            Some((
                path_hint,
                status,
                lang,
                base_counts,
                head_counts,
                code_delta,
                comment_delta,
                blank_delta,
                total_delta,
            ))
        })
        .collect();

    for (
        path_hint,
        status,
        lang,
        base_counts,
        head_counts,
        code_delta,
        comment_delta,
        blank_delta,
        total_delta,
    ) in items
    {
        per_file.push(DiffPerFile {
            path: path_hint.display().to_string(),
            status,
            language: lang.clone(),
            code_delta,
            comment_delta,
            blank_delta,
            total_delta,
        });

        let entry = per_lang.entry(lang.clone()).or_default();
        entry.add_file_delta(
            (base_counts.code, base_counts.comment, base_counts.blank),
            (head_counts.code, head_counts.comment, head_counts.blank),
        );
    }

    // Totals
    let mut totals = LineDelta::default();
    for (_lang, d) in &per_lang {
        totals.files += d.files;
        totals.code_added += d.code_added;
        totals.code_removed += d.code_removed;
        totals.comment_added += d.comment_added;
        totals.blank_added += d.blank_added;
        totals.total_net += d.total_net;
    }

    let summary = DiffSummary {
        base_ref,
        head_ref,
        base: base_info,
        head: head_info,
        files: per_file.len(),
        files_added: per_file.iter().filter(|f| f.status == "A").count(),
        files_deleted: per_file.iter().filter(|f| f.status == "D").count(),
        files_modified: per_file.iter().filter(|f| f.status == "M").count(),
        files_renamed: per_file.iter().filter(|f| f.status == "R").count(),
        languages: per_lang,
        by_file: if args.by_file && !args.summary_only {
            per_file
        } else {
            Vec::new()
        },
        totals,
    };

    // Threshold gating (global + per-language) with output emission
    // Threshold checks: only fail with non-zero exit if explicitly requested
    let mut threshold_errors: Vec<String> = Vec::new();
    if let Some(max) = args.max_code_added {
        #[allow(clippy::cast_possible_wrap)]
        if summary.totals.code_added > max as isize {
            threshold_errors.push(format!(
                "code delta {} exceeds threshold {}",
                summary.totals.code_added, max
            ));
        }
    }
    if let Some(max) = args.max_total_changed {
        if summary.totals.total_net.unsigned_abs() > max {
            threshold_errors.push(format!(
                "total net delta {} exceeds threshold {}",
                summary.totals.total_net, max
            ));
        }
    }
    if let Some(max) = args.max_files {
        if summary.files > max {
            threshold_errors.push(format!(
                "files changed {} exceeds threshold {}",
                summary.files, max
            ));
        }
    }
    if !args.max_code_added_lang.is_empty() {
        let mut limits = std::collections::HashMap::new();
        for spec in &args.max_code_added_lang {
            if let Some((k, v)) = spec.split_once(':') {
                if let Ok(n) = v.parse::<isize>() {
                    limits.insert(k.trim().to_string(), n);
                }
            }
        }
        let mut violations = Vec::new();
        for (lang, d) in &summary.languages {
            if let Some(limit) = limits.get(lang) {
                if d.code_added > *limit {
                    violations.push(format!("{lang}>{limit}"));
                }
            }
        }
        if !violations.is_empty() {
            threshold_errors.push(format!(
                "per-language thresholds exceeded: {}",
                violations.join(", ")
            ));
        }
    }

    if !threshold_errors.is_empty() {
        emit_output(args, &summary);
        if args.fail_on_threshold {
            bail!(threshold_errors.join("; "));
        }
        eprintln!("Warning: {}", threshold_errors.join("; "));
        return Ok(());
    }
    emit_output(args, &summary);
    Ok(())
}

fn analyze_bytes(bytes: &[u8], path_hint: &Path) -> Result<FileCounts> {
    let cursor = Cursor::new(bytes);
    let reader = std::io::BufReader::new(cursor);
    analyzer::analyze_reader_owned(reader, path_hint)
}

fn print_table(s: &DiffSummary) {
    // Simple table: Language, files, codeΔ, commentΔ, blankΔ, totalΔ
    println!(
        "{:<20} {:>7} {:>10} {:>10} {:>10} {:>10}",
        "Language", "files", "code", "comment", "blank", "net"
    );
    println!(
        "{}",
        "-".repeat(20 + 1 + 7 + 1 + 10 + 1 + 10 + 1 + 10 + 1 + 10)
    );
    for (lang, d) in &s.languages {
        println!(
            "{:<20} {:>7} {:>+10} {:>+10} {:>+10} {:>+10}",
            lang, d.files, d.code_added, d.comment_added, d.blank_added, d.total_net
        );
    }
    println!(
        "{}",
        "-".repeat(20 + 1 + 7 + 1 + 10 + 1 + 10 + 1 + 10 + 1 + 10)
    );
    println!(
        "{:<20} {:>7} {:>+10} {:>+10} {:>+10} {:>+10}",
        "Total",
        s.totals.files,
        s.totals.code_added,
        s.totals.comment_added,
        s.totals.blank_added,
        s.totals.total_net
    );
}

fn print_csv(s: &DiffSummary) {
    println!("language,files,code_added,code_removed,comment_added,blank_added,net_delta");
    for (lang, d) in &s.languages {
        println!(
            "{},{},{},{},{},{},{}",
            lang,
            d.files,
            d.code_added,
            d.code_removed,
            d.comment_added,
            d.blank_added,
            d.total_net
        );
    }
    println!(
        "Total,{},{},{},{},{},{}",
        s.totals.files,
        s.totals.code_added,
        s.totals.code_removed,
        s.totals.comment_added,
        s.totals.blank_added,
        s.totals.total_net
    );

    if !s.by_file.is_empty() {
        println!();
        println!("path,status,language,code_delta,comment_delta,blank_delta,net_delta");
        for f in &s.by_file {
            println!(
                "{},{},{},{},{},{},{}",
                f.path,
                f.status,
                f.language,
                f.code_delta,
                f.comment_delta,
                f.blank_delta,
                f.total_delta
            );
        }
    }
}

fn print_markdown(s: &DiffSummary) {
    let base = s.base_ref.as_deref().unwrap_or("<base>");
    let head = s.head_ref.as_deref().unwrap_or("<head>");
    println!("### LOC Diff Summary ({base} → {head})");
    println!(
        "- Files: {} (A:{} · M:{} · D:{} · R:{})",
        s.files, s.files_added, s.files_modified, s.files_deleted, s.files_renamed
    );
    println!(
        "- Code Δ: {} · Comment Δ: {} · Blank Δ: {} · Net Δ: {}\n",
        s.totals.code_added, s.totals.comment_added, s.totals.blank_added, s.totals.total_net
    );

    println!("#### Top Languages by Net Δ");
    println!("| Language | files | code Δ | comment Δ | blank Δ | net Δ |");
    println!("|---------:|-----:|-------:|----------:|--------:|-----:|");
    let mut langs: Vec<_> = s.languages.iter().collect();
    langs.sort_by(|a, b| {
        b.1.total_net
            .abs()
            .cmp(&a.1.total_net.abs())
            .then(b.0.cmp(a.0))
    });
    for (lang, d) in langs.into_iter().take(10) {
        println!(
            "| {} | {} | {} | {} | {} | {} |",
            lang, d.files, d.code_added, d.comment_added, d.blank_added, d.total_net
        );
    }
    println!(
        "| Total | {} | {} | {} | {} | {} |",
        s.totals.files,
        s.totals.code_added,
        s.totals.comment_added,
        s.totals.blank_added,
        s.totals.total_net
    );

    if !s.by_file.is_empty() {
        println!("\n<details><summary>Top Changed Files</summary>\n");
        println!("| File | status | language | code Δ | comment Δ | blank Δ | net Δ |");
        println!("|------|:------:|:--------:|------:|----------:|--------:|-----:|");
        let mut files = s.by_file.clone();
        files.sort_by_key(|b| std::cmp::Reverse(b.total_delta.abs()));
        for f in files.into_iter().take(10) {
            println!(
                "| {} | {} | {} | {} | {} | {} | {} |",
                f.path,
                f.status,
                f.language,
                f.code_delta,
                f.comment_delta,
                f.blank_delta,
                f.total_delta
            );
        }
        println!("\n</details>");
    }
}

fn emit_output(args: &DiffArgs, summary: &DiffSummary) {
    if args.json {
        if let Ok(s) = serde_json::to_string_pretty(summary) {
            println!("{s}");
        }
        return;
    }
    if args.csv {
        print_csv(summary);
        return;
    }
    if args.markdown {
        print_markdown(summary);
        return;
    }
    print_table(summary);
}
