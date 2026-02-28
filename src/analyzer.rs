use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use anyhow::{Context, Result};

use crate::languages::{find_language_index_for_path, language_markers_bytes};
use crate::types::FileCounts;
use once_cell::sync::OnceCell;

struct AnalyzerConfig {
    no_mmap: bool,
    mmap_threshold: u64,
}

static ANALYZER_CONFIG: OnceCell<AnalyzerConfig> = OnceCell::new();

pub fn set_analyzer_config(no_mmap: bool, mmap_threshold: Option<u64>) {
    let _ = ANALYZER_CONFIG.set(AnalyzerConfig {
        no_mmap,
        mmap_threshold: mmap_threshold.unwrap_or(4 * 1024 * 1024),
    });
}

/// Analyzes a file and returns line count statistics.
///
/// # Errors
/// Returns an error if the file cannot be opened or read.
pub fn analyze_file(path: &Path) -> Result<FileCounts> {
    let file = File::open(path).with_context(|| format!("open file: {}", path.display()))?;
    // Use mmap for large files to reduce syscall overhead (configurable)
    if let Some(cfg) = ANALYZER_CONFIG.get() {
        if !cfg.no_mmap {
            if let Ok(meta) = file.metadata() {
                if meta.len() >= cfg.mmap_threshold {
                    // Safety: file is not mutated while mapping; read-only map
                    if let Ok(mmap) = unsafe { memmap2::Mmap::map(&file) } {
                        let mut rdr = io::Cursor::new(&mmap[..]);
                        return analyze_reader(&mut rdr, path);
                    }
                }
            }
        }
    }
    let mut reader = BufReader::new(file);
    analyze_reader(&mut reader, path)
}

/// Analyzes a buffered reader and returns line count statistics.
///
/// # Errors
/// Returns an error if reading from the reader fails.
pub fn analyze_reader<R: BufRead + ?Sized>(reader: &mut R, path_hint: &Path) -> Result<FileCounts> {
    // Locate language by extension; unknown -> skip counts but still produce 0s
    let lang_idx = find_language_index_for_path(path_hint);

    let mut counts = FileCounts::one_file();
    let mut buf = Vec::with_capacity(8192);
    let mut in_block: Option<(Vec<u8>, Vec<u8>)> = None;

    // Obtain markers
    #[allow(clippy::items_after_statements)]
    type MarkersTuple = (&'static [Vec<u8>], Option<(&'static [u8], &'static [u8])>);
    let (line_markers_vec, block_markers_bytes): MarkersTuple = lang_idx.map_or((&[], None), language_markers_bytes);

    // Fast zero-byte file handling if possible
    if let Ok(slice) = reader.fill_buf() {
        if slice.is_empty() {
            return Ok(counts);
        }
    }

    // Process by chunks and split on newlines using memchr for speed
    let mut pending: Vec<u8> = Vec::new();
    loop {
        buf.resize(8192, 0);
        let n = match io::Read::read(reader, &mut buf) {
            Ok(0) => 0,
            Ok(n) => n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e).with_context(|| format!("read: {}", path_hint.display())),
        };
        if n == 0 {
            if !pending.is_empty() {
                process_line(
                    &mut counts,
                    line_markers_vec,
                    block_markers_bytes.as_ref(),
                    &mut in_block,
                    trim_cr(&pending),
                );
                pending.clear();
            }
            break;
        }
        let chunk = &buf[..n];
        let mut start = 0;
        for i in memchr::memchr_iter(b'\n', chunk) {
            if pending.is_empty() {
                process_line(
                    &mut counts,
                    line_markers_vec,
                    block_markers_bytes.as_ref(),
                    &mut in_block,
                    trim_cr(&chunk[start..i]),
                );
            } else {
                pending.extend_from_slice(&chunk[start..i]);
                let line = trim_cr(&pending);
                process_line(
                    &mut counts,
                    line_markers_vec,
                    block_markers_bytes.as_ref(),
                    &mut in_block,
                    line,
                );
                pending.clear();
            }
            start = i + 1;
        }
        if start < chunk.len() {
            pending.extend_from_slice(&chunk[start..]);
        }
    }

    Ok(counts)
}

/// Backward-compatible wrapper for callers that pass an owned reader.
///
/// # Errors
/// Returns an error if reading from the reader fails.
pub fn analyze_reader_owned<R: BufRead>(mut reader: R, path_hint: &Path) -> Result<FileCounts> {
    analyze_reader(&mut reader, path_hint)
}

const fn trim_ascii_start(mut s: &[u8]) -> &[u8] {
    while let Some((&b, rest)) = s.split_first() {
        if b.is_ascii_whitespace() {
            s = rest;
        } else {
            break;
        }
    }
    s
}

const fn trim_cr(s: &[u8]) -> &[u8] {
    if let Some((&last, body)) = s.split_last() {
        if last == b'\r' {
            return body;
        }
    }
    s
}

fn find_bytes(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    memchr::memmem::find(hay, needle)
}

fn process_line(
    counts: &mut FileCounts,
    line_markers: &[Vec<u8>],
    block_markers: Option<&(&[u8], &[u8])>,
    in_block: &mut Option<(Vec<u8>, Vec<u8>)>,
    raw: &[u8],
) {
    counts.total += 1;
    let trimmed = trim_ascii_start(raw);
    if trimmed.is_empty() {
        counts.blank += 1;
        return;
    }

    // If already in a block, search for end
    if let Some((_, ref end)) = *in_block {
        if let Some(idx) = find_bytes(trimmed, end.as_slice()) {
            let after = &trimmed[idx + end.len()..];
            *in_block = None;
            if trim_ascii_start(after).is_empty() {
                counts.comment += 1;
            } else {
                counts.code += 1;
            }
            return;
        }
        counts.comment += 1;
        return;
    }

    if let Some(&(start, end)) = block_markers {
        if let Some(start_idx) = find_bytes(trimmed, start) {
            if let Some(end_rel) = find_bytes(&trimmed[start_idx + start.len()..], end) {
                let before = &trimmed[..start_idx];
                let after = &trimmed[start_idx + start.len() + end_rel + end.len()..];
                if trim_ascii_start(before).is_empty() && trim_ascii_start(after).is_empty() {
                    counts.comment += 1;
                } else {
                    counts.code += 1;
                }
                return;
            }
            // starts block; remains open
            *in_block = Some((start.to_vec(), end.to_vec()));
            let before = &trimmed[..start_idx];
            if trim_ascii_start(before).is_empty() {
                counts.comment += 1;
            } else {
                counts.code += 1;
            }
            return;
        }
    }

    // Line comments
    let leading = trim_ascii_start(trimmed);
    for bytes in line_markers {
        let bytes = bytes.as_slice();
        if leading.len() >= bytes.len() && &leading[..bytes.len()] == bytes {
            counts.comment += 1;
            return;
        }
    }
    counts.code += 1;
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
        let mut f = File::create(&path).unwrap();
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
        let mut f = File::create(&path).unwrap();
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
        let mut f = File::create(&path).unwrap();
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
        let mut f = File::create(&path).unwrap();
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
        let mut f = File::create(&path).unwrap();
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
        let mut f = File::create(&path).unwrap();
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
