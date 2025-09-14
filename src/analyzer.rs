use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::{Context, Result};

use crate::languages::language_registry;
use crate::types::FileCounts;

pub fn analyze_file(path: &Path) -> Result<FileCounts> {
    let file = File::open(path).with_context(|| format!("open file: {}", path.display()))?;
    let mut reader = BufReader::new(file);

    // Locate language by extension; unknown -> skip counts but still produce 0s
    let lang = super::languages::find_language_for_path(path);

    let mut counts = FileCounts::one_file();
    let mut buf = String::new();
    let mut in_block: Option<(&'static str, &'static str)> = None;

    // Obtain markers
    let (line_markers, block_markers) = if let Some(name) = lang {
        if let Some(lang) = language_registry().iter().find(|l| l.name == name) {
            (lang.line_markers, lang.block_markers)
        } else {
            (&[][..], None)
        }
    } else {
        (&[][..], None)
    };

    loop {
        buf.clear();
        let bytes = reader.read_line(&mut buf)?;
        if bytes == 0 {
            break;
        }
        counts.total += 1;

        let line = buf.trim_end_matches(['\n', '\r']);
        let trimmed = line.trim();
        if trimmed.is_empty() {
            counts.blank += 1;
            continue;
        }

        // HTML-like block comments, C-style, etc.
        let mut handled_comment = false;
        let cur = trimmed;

        // If already in a block, search for end
        if let Some((_start, end)) = in_block {
            if let Some(idx) = cur.find(end) {
                // block ends on this line; may have code before/after comment
                let after = &cur[idx + end.len()..];
                in_block = None;
                // If there is non-whitespace after end, treat as code
                if after.trim().is_empty() {
                    counts.comment += 1; // treat entire line as comment
                } else {
                    counts.code += 1;
                }
                handled_comment = true;
            } else {
                counts.comment += 1;
                handled_comment = true;
            }
        } else if let Some((start, end)) = block_markers
            && let Some(start_idx) = cur.find(start)
        {
            if let Some(end_idx) = cur[start_idx + start.len()..].find(end) {
                // start and end on same line
                let before = &cur[..start_idx];
                let after = &cur[start_idx + start.len() + end_idx + end.len()..];
                if before.trim().is_empty() && after.trim().is_empty() {
                    counts.comment += 1;
                } else {
                    counts.code += 1; // mixed line counts as code
                }
                handled_comment = true;
            } else {
                // starts block; remains open
                in_block = Some((start, end));
                let before = &cur[..start_idx];
                if before.trim().is_empty() {
                    counts.comment += 1;
                } else {
                    counts.code += 1; // code before comment start
                }
                handled_comment = true;
            }
        }

        if handled_comment {
            continue;
        }

        // Line comments
        let mut is_line_comment = false;
        for m in line_markers {
            if cur.trim_start().starts_with(m) {
                is_line_comment = true;
                break;
            }
        }
        if is_line_comment {
            counts.comment += 1;
        } else {
            counts.code += 1;
        }
    }

    Ok(counts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn rust_line_and_block_comments() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("sample.rs");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            "// line\ncode\n/* block */\ncode /* mid */ more\n/* start\ncontinued\nend */\n"
        )
        .unwrap();
        let counts = analyze_file(&path).unwrap();
        assert_eq!(counts.total, 7);
        assert_eq!(counts.comment, 5);
        assert_eq!(counts.code, 2);
        assert_eq!(counts.blank, 0);
    }

    #[test]
    fn python_triple_quoted_strings_treated_as_code() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("doc.py");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            "\n\n\"\"\"Module docstring\nspans lines\n\"\"\"\n\n# comment line\nprint(1)\n"
        )
        .unwrap();
        let counts = analyze_file(&path).unwrap();
        // total lines: 8
        assert_eq!(counts.total, 8);
        // blanks: three explicit blank lines (2 leading, 1 after docstring)
        assert_eq!(counts.blank, 3);
        // comment: the single '#' line
        assert_eq!(counts.comment, 1);
        // remaining are code (triple-quote lines + inner text + print)
        assert_eq!(counts.code, 4);
    }

    #[test]
    fn html_block_comments() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("page.html");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            "<!-- head -->\n<div>content</div>\n<!-- start\ncontinued\nend -->\n<div><!-- mid --></div>\n"
        )
        .unwrap();
        let counts = analyze_file(&path).unwrap();
        assert_eq!(counts.total, 6);
        // comment lines: 1 (single-line), 3 (multiline block: start, middle, end).
        // Mixed inline comment counts as code.
        assert_eq!(counts.comment, 4);
        assert_eq!(counts.code, 2);
        assert_eq!(counts.blank, 0);
    }

    #[test]
    fn markdown_html_comments() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("README.md");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            "# Title\n\n<!-- intro -->\nSome text paragraph.\n<!-- start\nmultiline\nend -->\n"
        )
        .unwrap();
        let counts = analyze_file(&path).unwrap();
        // Lines: 7 total
        assert_eq!(counts.total, 7);
        // Blank: 1 (second line)
        assert_eq!(counts.blank, 1);
        // Comment: 1 (single line block), 3 (multiline start/middle/end)
        assert_eq!(counts.comment, 4);
        // Code: title + text
        assert_eq!(counts.code, 2);
    }

    #[test]
    fn ini_line_comments() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("settings.ini");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            "; leading comment\n# another comment\n\n[section]\nkey=value\nkey2 = value2  # trailing\n"
        )
        .unwrap();
        let counts = analyze_file(&path).unwrap();
        // total lines: 6
        assert_eq!(counts.total, 6);
        // blanks: 1
        assert_eq!(counts.blank, 1);
        // comment lines: 2 (leading two). trailing comment is on a code line and not detected as comment-only
        assert_eq!(counts.comment, 2);
        // code lines: 3
        assert_eq!(counts.code, 3);
    }

    #[test]
    fn svg_xml_comments() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("icon.svg");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            "<?xml version=\"1.0\"?>\n<!-- single -->\n<svg>\n  <!-- start\n  mid\n  end -->\n</svg>\n"
        )
        .unwrap();
        let counts = analyze_file(&path).unwrap();
        // total lines: 7 (trailing newline after </svg>)
        assert_eq!(counts.total, 7);
        // comments: 1 (single), 3 (multiline)
        assert_eq!(counts.comment, 4);
        // blanks: 0
        assert_eq!(counts.blank, 0);
        // code: xml decl + <svg> and </svg>
        assert_eq!(counts.code, 3);
    }
}
