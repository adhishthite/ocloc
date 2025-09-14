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
}

impl LanguageRegistry {
    fn from_specs(specs: Vec<LanguageSpec>) -> Self {
        let mut by_ext = HashMap::new();
        let mut by_special = HashMap::new();
        for (i, spec) in specs.iter().enumerate() {
            for ext in &spec.extensions {
                by_ext.insert(ext.to_ascii_lowercase(), i);
            }
            for name in &spec.special_filenames {
                by_special.insert(name.to_ascii_lowercase(), i);
            }
        }
        Self {
            specs,
            by_ext,
            by_special,
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
        if let Some(&idx) = REGISTRY.by_ext.get(&ext) {
            return Some(&language_registry()[idx].name);
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
        std::fs::File::create(&jsx).unwrap();
        std::fs::File::create(&tsx).unwrap();
        assert_eq!(find_language_for_path(&jsx), Some("JavaScript"));
        assert_eq!(find_language_for_path(&tsx), Some("TypeScript"));
    }

    #[test]
    fn detects_special_filenames() {
        let dir = tempdir().unwrap();
        let mk = dir.path().join("Makefile");
        let dk = dir.path().join("Dockerfile");
        std::fs::File::create(&mk).unwrap();
        std::fs::File::create(&dk).unwrap();
        assert_eq!(find_language_for_path(&mk), Some("Make"));
        assert_eq!(find_language_for_path(&dk), Some("Dockerfile"));
    }

    #[test]
    fn detects_shebang_python_and_shell() {
        let dir = tempdir().unwrap();
        let py = dir.path().join("script");
        let sh = dir.path().join("run");
        {
            let mut f = std::fs::File::create(&py).unwrap();
            writeln!(f, "#!/usr/bin/env python3\nprint(123)").unwrap();
        }
        {
            let mut f = std::fs::File::create(&sh).unwrap();
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
        std::fs::File::create(&md).unwrap();
        std::fs::File::create(&mdx).unwrap();
        std::fs::File::create(&svg).unwrap();
        std::fs::File::create(&ini).unwrap();
        std::fs::File::create(&txt).unwrap();
        std::fs::File::create(&rst).unwrap();
        std::fs::File::create(&adoc).unwrap();
        std::fs::File::create(&xml).unwrap();
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
        std::fs::File::create(&make).unwrap();
        std::fs::File::create(&dk).unwrap();
        std::fs::File::create(&cm).unwrap();
        std::fs::File::create(&build).unwrap();
        std::fs::File::create(&ws).unwrap();
        std::fs::File::create(&gem).unwrap();
        std::fs::File::create(&just).unwrap();
        std::fs::File::create(&readme).unwrap();
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
        use std::collections::HashSet;
        let specs = language_registry();
        let mut names = HashSet::new();
        let mut exts = HashSet::new();
        let mut specials = HashSet::new();
        for s in specs {
            assert!(!s.name.trim().is_empty(), "language name must be non-empty");
            assert!(names.insert(&s.name), "duplicate language name: {}", s.name);
            for e in &s.extensions {
                let norm = e.to_ascii_lowercase();
                assert!(
                    exts.insert(norm.clone()),
                    "duplicate extension across languages: {}",
                    norm
                );
            }
            for f in &s.special_filenames {
                let norm = f.to_ascii_lowercase();
                assert!(
                    specials.insert(norm.clone()),
                    "duplicate special filename across languages: {}",
                    norm
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
    }
}
