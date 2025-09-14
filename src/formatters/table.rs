use crate::types::{AnalyzeResult, FileCounts};

pub fn format(a: &AnalyzeResult) -> String {
    let mut lines = Vec::new();
    lines.push("Language           Files   Code    Comm    Blank   Total".to_string());
    lines.push("---------------------------------------------------------".to_string());

    // Already sorted by caller; maintain iteration order
    for (lang, counts) in &a.per_lang {
        lines.push(format_row(lang, counts));
    }

    lines.push("".into());
    lines.push(format_row("Total", &a.totals));
    lines.join("\n")
}

fn format_row(lang: &str, c: &FileCounts) -> String {
    format!(
        "{:<18} {:>5} {:>7} {:>7} {:>7} {:>7}",
        lang, c.files, c.code, c.comment, c.blank, c.total
    )
}
