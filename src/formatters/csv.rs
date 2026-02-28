use crate::types::{AnalyzeResult, FileCounts};

#[must_use]
pub fn format(a: &AnalyzeResult) -> String {
    let mut out = String::new();
    out.push_str("language,files,code,comment,blank,total\n");
    for (lang, c) in &a.per_lang {
        push_row(&mut out, lang, c);
    }
    push_row(&mut out, "Total", &a.totals);
    out
}

fn push_row(out: &mut String, lang: &str, c: &FileCounts) {
    use std::fmt::Write as _;
    let _ = writeln!(
        out,
        "{},{},{},{},{},{}",
        lang, c.files, c.code, c.comment, c.blank, c.total
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    #[test]
    fn csv_includes_new_languages() {
        let mut per: IndexMap<String, FileCounts> = IndexMap::new();
        per.insert(
            "Markdown".to_string(),
            FileCounts {
                files: 2,
                total: 10,
                code: 8,
                comment: 1,
                blank: 1,
            },
        );
        per.insert(
            "SVG".to_string(),
            FileCounts {
                files: 1,
                total: 5,
                code: 3,
                comment: 2,
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
            stats: None,
            analyzed_path: None,
        };
        let out = format(&a);
        assert!(out.contains("language,files,code,comment,blank,total"));
        assert!(out.contains("Markdown"));
        assert!(out.contains("SVG"));
        assert!(out.contains("\nTotal,"));
    }
}
