#![allow(clippy::must_use_candidate)]

use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct LanguageSpec {
    pub name: String,
    pub extensions: Vec<String>,
    pub line_markers: Vec<String>,
    pub block_markers: Option<(String, String)>,
    #[serde(default)]
    pub special_filenames: Vec<String>,
}

pub struct LanguageRegistry {
    specs: Vec<LanguageSpec>,
    by_ext: HashMap<String, usize>,
    by_special: HashMap<String, usize>,
    // Extensions with multiple claimants (for content-based detection)
    conflicting_exts: HashMap<String, Vec<usize>>,
    // Precomputed bytes per language index for fast access
    line_markers_bytes: Vec<Vec<Vec<u8>>>,
    block_markers_bytes: Vec<Option<(Vec<u8>, Vec<u8>)>>,
}

impl LanguageRegistry {
    fn from_specs(specs: Vec<LanguageSpec>) -> Self {
        let mut by_ext = HashMap::new();
        let mut by_special = HashMap::new();
        let mut ext_claimants: HashMap<String, Vec<usize>> = HashMap::new();
        let mut line_markers_bytes = Vec::with_capacity(specs.len());
        let mut block_markers_bytes = Vec::with_capacity(specs.len());

        for (i, spec) in specs.iter().enumerate() {
            for ext in &spec.extensions {
                let ext_lower = ext.to_ascii_lowercase();
                ext_claimants.entry(ext_lower.clone()).or_default().push(i);
            }
            for name in &spec.special_filenames {
                by_special.insert(name.to_ascii_lowercase(), i);
            }
            line_markers_bytes.push(
                spec.line_markers
                    .iter()
                    .map(|s| s.as_bytes().to_vec())
                    .collect(),
            );
            block_markers_bytes.push(
                spec.block_markers
                    .as_ref()
                    .map(|(a, b)| (a.as_bytes().to_vec(), b.as_bytes().to_vec())),
            );
        }

        // Separate conflicting extensions from unique ones
        let mut conflicting_exts = HashMap::new();
        for (ext, claimants) in ext_claimants {
            if claimants.len() == 1 {
                by_ext.insert(ext, claimants[0]);
            } else {
                conflicting_exts.insert(ext, claimants);
            }
        }

        Self {
            specs,
            by_ext,
            by_special,
            conflicting_exts,
            line_markers_bytes,
            block_markers_bytes,
        }
    }
}

static EMBEDDED_LANG_JSON: &str = include_str!("../assets/languages.json");

pub static REGISTRY: Lazy<LanguageRegistry> = Lazy::new(|| {
    let specs: Vec<LanguageSpec> =
        serde_json::from_str(EMBEDDED_LANG_JSON).expect("invalid embedded languages.json");
    LanguageRegistry::from_specs(specs)
});

pub fn language_registry() -> &'static [LanguageSpec] {
    &REGISTRY.specs
}

pub fn find_language_for_path(path: &Path) -> Option<&'static str> {
    // 1) Special filenames (take precedence over extension)
    if let Some(fname) = path.file_name().and_then(|s| s.to_str()) {
        let lower = fname.to_ascii_lowercase();
        if let Some(&idx) = REGISTRY.by_special.get(&lower) {
            return Some(&language_registry()[idx].name);
        }
        match lower.as_str() {
            "makefile" => return Some("Make"),
            "dockerfile" => return Some("Dockerfile"),
            "cmakelists.txt" => return Some("CMake"),
            _ => {}
        }
    }

    // 2) By extension
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        let ext = ext.to_ascii_lowercase();

        // Check for non-conflicting extension
        if let Some(&idx) = REGISTRY.by_ext.get(&ext) {
            return Some(&language_registry()[idx].name);
        }

        // Check for conflicting extension - use content-based detection
        if let Some(candidates) = REGISTRY.conflicting_exts.get(&ext) {
            if let Some(detected) = detect_language_by_content(path, candidates) {
                return Some(&language_registry()[detected].name);
            }
            // Fallback to first candidate if detection fails
            return Some(&language_registry()[candidates[0]].name);
        }
    }

    // 3) Shebang detection for scripts without extension
    if path.extension().is_none()
        && let Ok(f) = File::open(path)
    {
        let mut rdr = BufReader::new(f);
        let mut first = String::new();
        if rdr.read_line(&mut first).is_ok()
            && let Some(lang) = parse_shebang(&first)
        {
            return Some(lang);
        }
    }
    None
}

pub fn find_language_index_for_path(path: &Path) -> Option<usize> {
    if let Some(fname) = path.file_name().and_then(|s| s.to_str()) {
        let lower = fname.to_ascii_lowercase();
        if let Some(&idx) = REGISTRY.by_special.get(&lower) {
            return Some(idx);
        }
        match lower.as_str() {
            "makefile" => return language_registry().iter().position(|l| l.name == "Make"),
            "dockerfile" => {
                return language_registry()
                    .iter()
                    .position(|l| l.name == "Dockerfile");
            }
            "cmakelists.txt" => return language_registry().iter().position(|l| l.name == "CMake"),
            _ => {}
        }
    }
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        let ext = ext.to_ascii_lowercase();

        // Check for non-conflicting extension
        if let Some(&idx) = REGISTRY.by_ext.get(&ext) {
            return Some(idx);
        }

        // Check for conflicting extension - use content-based detection
        if let Some(candidates) = REGISTRY.conflicting_exts.get(&ext) {
            if let Some(detected) = detect_language_by_content(path, candidates) {
                return Some(detected);
            }
            // Fallback to first candidate
            return Some(candidates[0]);
        }
    }
    if path.extension().is_none()
        && let Ok(f) = File::open(path)
    {
        let mut rdr = BufReader::new(f);
        let mut first = String::new();
        if rdr.read_line(&mut first).is_ok()
            && let Some(lang) = parse_shebang(&first)
        {
            return language_registry().iter().position(|l| l.name == lang);
        }
    }
    None
}

pub type LanguageMarkersBytes = (&'static [Vec<u8>], Option<(&'static [u8], &'static [u8])>);

pub fn language_markers_bytes(idx: usize) -> LanguageMarkersBytes {
    let lines: &'static [Vec<u8>] = &REGISTRY.line_markers_bytes[idx];
    let blocks = REGISTRY.block_markers_bytes[idx]
        .as_ref()
        .map(|(a, b)| (a.as_slice(), b.as_slice()));
    (lines, blocks)
}

/// Content-based language detection for ambiguous extensions
fn detect_language_by_content(path: &Path, candidates: &[usize]) -> Option<usize> {
    // Read first few lines of the file
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .take(50)  // Sample first 50 lines
        .filter_map(Result::ok)
        .collect();

    if lines.is_empty() {
        return None;
    }

    let content = lines.join("\n");
    let content_lower = content.to_lowercase();

    let ext = path.extension()?.to_str()?.to_ascii_lowercase();

    // Specific heuristics based on extension
    match ext.as_str() {
        "m" => detect_m_language(&content, &content_lower, candidates),
        "v" => detect_v_language(&content, &content_lower, candidates),
        "cl" => detect_cl_language(&content, &content_lower, candidates),
        "pp" => detect_pp_language(&content, &content_lower, candidates),
        "il" => detect_il_language(&content, &content_lower, candidates),
        "cj" => detect_cj_language(&content, &content_lower, candidates),
        _ => None,
    }
}

/// Detect .m files: Objective-C, MATLAB, Octave, Mercury
fn detect_m_language(content: &str, content_lower: &str, candidates: &[usize]) -> Option<usize> {
    let specs = &REGISTRY.specs;

    // Objective-C indicators (strongest signals first)
    if content.contains("@interface")
        || content.contains("@implementation")
        || content.contains("@protocol")
        || content.contains("@property")
        || content.contains("#import")
        || content.contains("NSObject")
        || content.contains("NS_ASSUME")
    {
        return candidates
            .iter()
            .find(|&&idx| specs[idx].name == "Objective-C")
            .copied();
    }

    // Mercury indicators
    if content.contains(":- module")
        || content.contains(":- interface")
        || content.contains(":- implementation")
        || content.contains(":- pred ")
        || content.contains(":- func ")
    {
        return candidates
            .iter()
            .find(|&&idx| specs[idx].name == "Mercury")
            .copied();
    }

    // MATLAB/Octave indicators
    if content.contains("function ")
        && (content.contains("end\n") || content.contains("end\r") || content_lower.contains("end;"))
        || content.contains("% ")
        || content_lower.contains("fprintf")
        || content_lower.contains("disp(")
        || content_lower.contains("plot(")
    {
        // Prefer MATLAB over Octave as it's more common
        if let Some(&idx) = candidates.iter().find(|&&idx| specs[idx].name == "MATLAB") {
            return Some(idx);
        }
        return candidates
            .iter()
            .find(|&&idx| specs[idx].name == "Octave")
            .copied();
    }

    // Default to Objective-C if uncertain (most common in modern codebases)
    candidates
        .iter()
        .find(|&&idx| specs[idx].name == "Objective-C")
        .copied()
}

/// Detect .v files: Verilog vs Coq
fn detect_v_language(content: &str, _content_lower: &str, candidates: &[usize]) -> Option<usize> {
    let specs = &REGISTRY.specs;

    // Coq indicators
    if content.contains("Theorem ")
        || content.contains("Proof.")
        || content.contains("Qed.")
        || content.contains("Lemma ")
        || content.contains("Definition ")
        || content.contains("Require ")
    {
        return candidates
            .iter()
            .find(|&&idx| specs[idx].name == "Coq")
            .copied();
    }

    // Verilog indicators
    if content.contains("module ")
        || content.contains("endmodule")
        || content.contains("wire ")
        || content.contains("reg ")
        || content.contains("always @")
    {
        return candidates
            .iter()
            .find(|&&idx| specs[idx].name.contains("Verilog"))
            .copied();
    }

    // Default to Verilog (more common)
    candidates
        .iter()
        .find(|&&idx| specs[idx].name.contains("Verilog"))
        .copied()
}

#[allow(clippy::doc_markdown)]
/// Detect .cl files: OpenCL vs Lisp
fn detect_cl_language(content: &str, _content_lower: &str, candidates: &[usize]) -> Option<usize> {
    let specs = &REGISTRY.specs;

    // Lisp indicators
    if content.contains("(defun ")
        || content.contains("(defmacro ")
        || content.contains("(setq ")
        || content.trim_start().starts_with('(')
    {
        return candidates
            .iter()
            .find(|&&idx| specs[idx].name == "Lisp")
            .copied();
    }

    // OpenCL indicators
    if content.contains("__kernel")
        || content.contains("__global")
        || content.contains("get_global_id")
        || content.contains("cl_")
    {
        return candidates
            .iter()
            .find(|&&idx| specs[idx].name == "OpenCL")
            .copied();
    }

    // Default to OpenCL
    candidates
        .iter()
        .find(|&&idx| specs[idx].name == "OpenCL")
        .copied()
}

/// Detect .pp files: Puppet vs Pascal
fn detect_pp_language(content: &str, _content_lower: &str, candidates: &[usize]) -> Option<usize> {
    let specs = &REGISTRY.specs;

    // Puppet indicators
    if content.contains("class ")
        && (content.contains("=>") || content.contains("node ") || content.contains("define "))
        || content.contains("include ")
        || content.contains("$::")
    {
        return candidates
            .iter()
            .find(|&&idx| specs[idx].name == "Puppet")
            .copied();
    }

    // Pascal indicators
    if content.contains("program ")
        || content.contains("procedure ")
        || content.contains("function ")
        && (content.contains("begin") || content.contains("Begin"))
        || content.contains("uses ")
    {
        return candidates
            .iter()
            .find(|&&idx| specs[idx].name == "Pascal")
            .copied();
    }

    // Default to Puppet (more common in modern dev)
    candidates
        .iter()
        .find(|&&idx| specs[idx].name == "Puppet")
        .copied()
}

/// Detect .il files: SKILL vs .NET IL
fn detect_il_language(content: &str, _content_lower: &str, candidates: &[usize]) -> Option<usize> {
    let specs = &REGISTRY.specs;

    // .NET IL indicators
    if content.contains(".assembly")
        || content.contains(".class")
        || content.contains(".method")
        || content.contains("IL_")
    {
        return candidates
            .iter()
            .find(|&&idx| specs[idx].name == ".NET IL")
            .copied();
    }

    // SKILL indicators (Cadence)
    if content.contains("procedure(")
        || content.contains("defun(")
        || content.trim_start().starts_with(';')
    {
        return candidates
            .iter()
            .find(|&&idx| specs[idx].name == "SKILL")
            .copied();
    }

    // Default to .NET IL
    candidates
        .iter()
        .find(|&&idx| specs[idx].name == ".NET IL")
        .copied()
}

/// Detect .cj files: Cangjie vs Clojure
fn detect_cj_language(content: &str, _content_lower: &str, candidates: &[usize]) -> Option<usize> {
    let specs = &REGISTRY.specs;

    // Clojure indicators
    if content.contains("(ns ")
        || content.contains("(def ")
        || content.contains("(defn ")
        || content.trim_start().starts_with('(')
    {
        return candidates
            .iter()
            .find(|&&idx| specs[idx].name == "Clojure")
            .copied();
    }

    // Cangjie indicators (assume C-like syntax)
    if content.contains("import ")
        || content.contains("package ")
        || content.contains("class ")
        || content.contains("func ")
    {
        return candidates
            .iter()
            .find(|&&idx| specs[idx].name == "Cangjie")
            .copied();
    }

    // Default to Cangjie
    candidates
        .iter()
        .find(|&&idx| specs[idx].name == "Cangjie")
        .copied()
}

fn parse_shebang(line: &str) -> Option<&'static str> {
    let s = line.trim_start();
    if !s.starts_with("#!") {
        return None;
    }
    let s = s[2..].trim();
    // handle /usr/bin/env pattern
    let tokens: Vec<&str> = s.split_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }
    let cmd = if tokens[0].ends_with("env") && tokens.len() > 1 {
        tokens[1]
    } else {
        tokens[0]
    };
    let cmd_lower = cmd.to_ascii_lowercase();
    if cmd_lower.contains("python") {
        return Some("Python");
    }
    if cmd_lower.contains("bash")
        || cmd_lower == "sh"
        || cmd_lower.contains("zsh")
        || cmd_lower.contains("ksh")
        || cmd_lower.contains("fish")
    {
        return Some("Shell");
    }
    if cmd_lower.contains("node") || cmd_lower.contains("deno") {
        return Some("JavaScript");
    }
    if cmd_lower.contains("perl") {
        return Some("Perl");
    }
    if cmd_lower.contains("ruby") {
        return Some("Ruby");
    }
    if cmd_lower.contains("php") {
        return Some("PHP");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn detects_jsx_tsx_by_extension() {
        let dir = tempdir().unwrap();
        let jsx = dir.path().join("a.jsx");
        let tsx = dir.path().join("b.tsx");
        File::create(&jsx).unwrap();
        File::create(&tsx).unwrap();
        assert_eq!(find_language_for_path(&jsx), Some("JavaScript"));
        assert_eq!(find_language_for_path(&tsx), Some("TypeScript"));
    }

    #[test]
    fn detects_special_filenames() {
        let dir = tempdir().unwrap();
        let mk = dir.path().join("Makefile");
        let dk = dir.path().join("Dockerfile");
        File::create(&mk).unwrap();
        File::create(&dk).unwrap();
        assert_eq!(find_language_for_path(&mk), Some("Make"));
        assert_eq!(find_language_for_path(&dk), Some("Dockerfile"));
    }

    #[test]
    fn detects_shebang_python_and_shell() {
        let dir = tempdir().unwrap();
        let py = dir.path().join("script");
        let sh = dir.path().join("run");
        {
            let mut f = File::create(&py).unwrap();
            writeln!(f, "#!/usr/bin/env python3\nprint(123)").unwrap();
        }
        {
            let mut f = File::create(&sh).unwrap();
            writeln!(f, "#!/bin/bash\necho hi").unwrap();
        }
        assert_eq!(find_language_for_path(&py), Some("Python"));
        assert_eq!(find_language_for_path(&sh), Some("Shell"));
    }

    #[test]
    fn detects_doc_and_config_types() {
        let dir = tempdir().unwrap();
        let md = dir.path().join("README.md");
        let mdx = dir.path().join("page.mdx");
        let svg = dir.path().join("icon.svg");
        let ini = dir.path().join("settings.ini");
        let txt = dir.path().join("notes.txt");
        let rst = dir.path().join("guide.rst");
        let adoc = dir.path().join("handbook.adoc");
        let xml = dir.path().join("data.xml");
        File::create(&md).unwrap();
        File::create(&mdx).unwrap();
        File::create(&svg).unwrap();
        File::create(&ini).unwrap();
        File::create(&txt).unwrap();
        File::create(&rst).unwrap();
        File::create(&adoc).unwrap();
        File::create(&xml).unwrap();
        assert_eq!(find_language_for_path(&md), Some("Markdown"));
        assert_eq!(find_language_for_path(&mdx), Some("Markdown"));
        assert_eq!(find_language_for_path(&svg), Some("SVG"));
        assert_eq!(find_language_for_path(&ini), Some("INI"));
        assert_eq!(find_language_for_path(&txt), Some("Text"));
        assert_eq!(find_language_for_path(&rst), Some("reStructuredText"));
        assert_eq!(find_language_for_path(&adoc), Some("AsciiDoc"));
        assert_eq!(find_language_for_path(&xml), Some("XML"));
    }

    #[test]
    fn additional_special_filenames_detection() {
        let dir = tempdir().unwrap();
        let make = dir.path().join("Makefile");
        let dk = dir.path().join("Dockerfile");
        let cm = dir.path().join("CMakeLists.txt");
        let build = dir.path().join("BUILD");
        let ws = dir.path().join("WORKSPACE.bazel");
        let gem = dir.path().join("Gemfile");
        let just = dir.path().join("justfile");
        let readme = dir.path().join("README");
        File::create(&make).unwrap();
        File::create(&dk).unwrap();
        File::create(&cm).unwrap();
        File::create(&build).unwrap();
        File::create(&ws).unwrap();
        File::create(&gem).unwrap();
        File::create(&just).unwrap();
        File::create(&readme).unwrap();
        assert_eq!(find_language_for_path(&make), Some("Make"));
        assert_eq!(find_language_for_path(&dk), Some("Dockerfile"));
        assert_eq!(find_language_for_path(&cm), Some("CMake"));
        assert_eq!(find_language_for_path(&build), Some("Starlark"));
        assert_eq!(find_language_for_path(&ws), Some("Starlark"));
        assert_eq!(find_language_for_path(&gem), Some("Ruby"));
        assert_eq!(find_language_for_path(&just), Some("Just"));
        assert_eq!(find_language_for_path(&readme), Some("Text"));
    }

    #[test]
    fn languages_json_is_consistent() {
        use std::collections::{HashMap, HashSet};
        let specs = language_registry();
        let mut names = HashSet::new();
        let mut ext_counts: HashMap<String, Vec<&str>> = HashMap::new();
        let mut specials = HashSet::new();

        // Known acceptable conflicts (handled by content-based detection or are related variants)
        let acceptable_conflicts = ["m", "v", "cl", "pp", "il", "ils", "cj"];

        for s in specs {
            assert!(!s.name.trim().is_empty(), "language name must be non-empty");
            assert!(names.insert(&s.name), "duplicate language name: {}", s.name);

            for e in &s.extensions {
                let norm = e.to_ascii_lowercase();
                ext_counts.entry(norm).or_default().push(&s.name);
            }

            for f in &s.special_filenames {
                let norm = f.to_ascii_lowercase();
                assert!(
                    specials.insert(norm.clone()),
                    "duplicate special filename across languages: {norm}"
                );
            }

            if let Some((ref a, ref b)) = s.block_markers {
                assert!(
                    !a.is_empty() && !b.is_empty(),
                    "block markers must be non-empty for {}",
                    s.name
                );
            }
        }

        // Check for unexpected conflicts
        for (ext, langs) in ext_counts {
            assert!(langs.len() <= 1 || acceptable_conflicts.contains(&ext.as_str()),
                "Unexpected extension conflict: .{ext} claimed by: {langs:?}"
            );
        }
    }
}
