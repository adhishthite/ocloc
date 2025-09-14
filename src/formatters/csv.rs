use crate::types::{AnalyzeResult, FileCounts};

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
