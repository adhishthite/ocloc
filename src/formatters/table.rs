use crate::types::{AnalyzeResult, FileCounts};

pub fn format(a: &AnalyzeResult) -> String {
    // Compute dynamic column widths with more generous minimums
    let mut lang_w: usize = 12; // increased minimum for Language column
    let mut files_w: usize = 8; // increased for "files" header
    let mut code_w: usize = 10; // increased for larger numbers
    let mut comm_w: usize = 10; // increased for "comment" header
    let mut blank_w: usize = 10; // increased for consistency
    let mut total_w: usize = 10; // increased for consistency

    let update_w = |w: &mut usize, val: usize| {
        let l = format_num(val).len();
        if l > *w {
            *w = l;
        }
    };

    lang_w = a
        .per_lang
        .keys()
        .map(|s| s.len())
        .chain(std::iter::once("Total".len()))
        .max()
        .unwrap_or(lang_w)
        .max(12); // increased from 8 to match new minimum

    for c in a.per_lang.values() {
        update_w(&mut files_w, c.files);
        update_w(&mut code_w, c.code);
        update_w(&mut comm_w, c.comment);
        update_w(&mut blank_w, c.blank);
        update_w(&mut total_w, c.total);
    }
    update_w(&mut files_w, a.totals.files);
    update_w(&mut code_w, a.totals.code);
    update_w(&mut comm_w, a.totals.comment);
    update_w(&mut blank_w, a.totals.blank);
    update_w(&mut total_w, a.totals.total);

    // Spacing between columns - increased for more spacious look
    let gutter: usize = 8; // increased from 5 to 8 for wider spacing
    let sep = " ".repeat(gutter);

    // Bundle widths to avoid passing too many args downstream
    let widths = ColWidths {
        lang: lang_w,
        files: files_w,
        blank: blank_w,
        comm: comm_w,
        code: code_w,
        total: total_w,
    };

    // Header (cells aligned, then joined with gutter spacing) - matching cloc's order
    let h_lang = format!("{:<w$}", "Language", w = widths.lang);
    let h_files = format!("{:>w$}", "files", w = widths.files);
    let h_blank = format!("{:>w$}", "blank", w = widths.blank);
    let h_comm = format!("{:>w$}", "comment", w = widths.comm);
    let h_code = format!("{:>w$}", "code", w = widths.code);
    let h_total = format!("{:>w$}", "Total", w = widths.total);
    let header = [h_lang, h_files, h_blank, h_comm, h_code, h_total].join(&sep);

    // Create a separator line that matches the total width of the table
    let sep_len = widths.lang
        + widths.files
        + widths.blank
        + widths.comm
        + widths.code
        + widths.total
        + gutter * 5;
    let separator = "-".repeat(sep_len);

    let mut lines = Vec::new();
    lines.push(header);
    lines.push(separator.clone());

    // Rows (already sorted by caller)
    for (lang, counts) in &a.per_lang {
        lines.push(format_row(lang, counts, &widths, &sep));
    }

    // Add separator before totals like cloc does
    lines.push(separator);
    // Emphasize totals
    let total_line = format_row("Total", &a.totals, &widths, &sep);
    lines.push(total_line);

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    #[test]
    fn table_includes_new_languages() {
        let mut per: IndexMap<String, FileCounts> = IndexMap::new();
        per.insert(
            "INI".to_string(),
            FileCounts {
                files: 3,
                total: 9,
                code: 6,
                comment: 2,
                blank: 1,
            },
        );
        per.insert(
            "Text".to_string(),
            FileCounts {
                files: 1,
                total: 4,
                code: 4,
                comment: 0,
                blank: 0,
            },
        );
        let mut totals = FileCounts::default();
        for v in per.values() {
            totals.merge(v);
        }
        let a = AnalyzeResult {
            per_lang: per,
            totals,
            files_analyzed: totals.files,
        };
        let out = format(&a);
        assert!(out.contains("INI"));
        assert!(out.contains("Text"));
        assert!(out.contains("Total"));
    }
}

struct ColWidths {
    lang: usize,
    files: usize,
    blank: usize,
    comm: usize,
    code: usize,
    total: usize,
}

fn format_row(lang: &str, c: &FileCounts, w: &ColWidths, sep: &str) -> String {
    // Prepare plain cells with alignment first
    let name_plain = format!("{:<w$}", lang, w = w.lang);
    let files_plain = format!("{:>w$}", format_num(c.files), w = w.files);
    let blank_plain = format!("{:>w$}", format_num(c.blank), w = w.blank);
    let comm_plain = format!("{:>w$}", format_num(c.comment), w = w.comm);
    let code_plain = format!("{:>w$}", format_num(c.code), w = w.code);
    let total_plain = format!("{:>w$}", format_num(c.total), w = w.total);

    [
        name_plain,
        files_plain,
        blank_plain,
        comm_plain,
        code_plain,
        total_plain,
    ]
    .join(sep)
}

fn format_num(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    let bytes = s.as_bytes();
    let mut i = bytes.len() as isize - 1;
    let mut count = 0;
    while i >= 0 {
        out.insert(0, bytes[i as usize] as char);
        count += 1;
        if count == 3 && i > 0 {
            out.insert(0, ',');
            count = 0;
        }
        i -= 1;
    }
    out
}

// Colors removed: produce plain, deterministic table output
