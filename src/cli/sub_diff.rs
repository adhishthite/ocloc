use anyhow::{Context, Result, bail};
use std::collections::HashSet;
use std::io::Cursor;
use std::path::Path;

use crate::analyzer;
use crate::languages::find_language_for_path;
use crate::types::FileCounts;
use crate::types_diff::{DiffPerFile, DiffSummary, LineDelta};
use crate::vcs::VcsContext;

use super::DiffArgs;

pub fn run_diff(args: &DiffArgs) -> Result<()> {
    // Validate incompatible flags
    if args.staged && args.working_tree {
        bail!("--staged and --working-tree are mutually exclusive");
    }
    // Determine repo root from CWD
    let vcs = VcsContext::open(Path::new("."))?;

    // diff mode selection
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

    let (changes, base_ref, head_ref) = match mode {
        Mode::Staged => (
            vcs.diff_head_to_index()?,
            Some("HEAD".to_string()),
            Some("INDEX".to_string()),
        ),
        Mode::Worktree => (
            vcs.diff_index_to_workdir()?,
            Some("INDEX".to_string()),
            Some("WORKDIR".to_string()),
        ),
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
            (
                vcs.diff_between(base_oid, head_oid)?,
                args.base.clone(),
                args.head.clone().or_else(|| Some("HEAD".to_string())),
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

            let code_delta = head_counts.code as isize - base_counts.code as isize;
            let comment_delta = head_counts.comment as isize - base_counts.comment as isize;
            let blank_delta = head_counts.blank as isize - base_counts.blank as isize;
            let total_delta = head_counts.total as isize - base_counts.total as isize;

            let status = c.status.clone();
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
    for (_lang, d) in per_lang.iter() {
        totals.files += d.files;
        totals.code_added += d.code_added;
        totals.comment_added += d.comment_added;
        totals.blank_added += d.blank_added;
        totals.total_net += d.total_net;
    }

    let summary = DiffSummary {
        base_ref,
        head_ref,
        files: per_file.len(),
        files_added: per_file.iter().filter(|f| f.status == "A").count(),
        files_deleted: per_file.iter().filter(|f| f.status == "D").count(),
        files_modified: per_file.iter().filter(|f| f.status == "M").count(),
        files_renamed: per_file.iter().filter(|f| f.status == "R").count(),
        languages: per_lang,
        by_file: if args.by_file { per_file } else { Vec::new() },
        totals,
    };

    // Threshold gating (global + per-language) with output emission
    if let Some(max) = args.max_code_added {
        if summary.totals.code_added > max as isize {
            emit_output(args, &summary);
            bail!(
                "code delta {} exceeds threshold {}",
                summary.totals.code_added,
                max
            );
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
                    violations.push(format!("{}>{}", lang, limit));
                }
            }
        }
        if !violations.is_empty() {
            emit_output(args, &summary);
            bail!(
                "per-language thresholds exceeded: {}",
                violations.join(", ")
            );
        }
    }

    emit_output(args, &summary);
    Ok(())
}

fn analyze_bytes(bytes: &[u8], path_hint: &Path) -> Result<FileCounts> {
    let cursor = Cursor::new(bytes);
    let reader = std::io::BufReader::new(cursor);
    analyzer::analyze_reader(reader, path_hint)
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
    println!("language,files,code_delta,comment_delta,blank_delta,net_delta");
    for (lang, d) in &s.languages {
        println!(
            "{},{},{},{},{},{}",
            lang, d.files, d.code_added, d.comment_added, d.blank_added, d.total_net
        );
    }
    println!(
        "Total,{},{},{},{},{}",
        s.totals.files,
        s.totals.code_added,
        s.totals.comment_added,
        s.totals.blank_added,
        s.totals.total_net
    );
}

fn print_markdown(s: &DiffSummary) {
    let base = s.base_ref.as_deref().unwrap_or("<base>");
    let head = s.head_ref.as_deref().unwrap_or("<head>");
    println!("### LOC Diff Summary ({} → {})", base, head);
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
        files.sort_by(|a, b| b.total_delta.abs().cmp(&a.total_delta.abs()));
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

fn emit_output(args: &super::DiffArgs, summary: &DiffSummary) {
    if args.json {
        if let Ok(s) = serde_json::to_string_pretty(summary) {
            println!("{}", s);
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
