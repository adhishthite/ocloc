use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct Language {
    pub name: &'static str,
    pub extensions: &'static [&'static str],
    pub line_markers: &'static [&'static str],
    pub block_markers: Option<(&'static str, &'static str)>,
}

pub fn language_registry() -> &'static [Language] {
    // Note: Keep minimal to start; expand with tests.
    static REPO: &[Language] = &[
        Language {
            name: "Rust",
            extensions: &["rs"],
            line_markers: &["//"],
            block_markers: Some(("/*", "*/")),
        },
        Language {
            name: "Python",
            extensions: &["py"],
            line_markers: &["#"],
            block_markers: None,
        },
        Language {
            name: "JavaScript",
            extensions: &["js", "jsx"],
            line_markers: &["//"],
            block_markers: Some(("/*", "*/")),
        },
        Language {
            name: "TypeScript",
            extensions: &["ts", "tsx"],
            line_markers: &["//"],
            block_markers: Some(("/*", "*/")),
        },
        Language {
            name: "C",
            extensions: &["c", "h"],
            line_markers: &["//"],
            block_markers: Some(("/*", "*/")),
        },
        Language {
            name: "C++",
            extensions: &["cpp", "cc", "hpp", "hh"],
            line_markers: &["//"],
            block_markers: Some(("/*", "*/")),
        },
        Language {
            name: "Java",
            extensions: &["java"],
            line_markers: &["//"],
            block_markers: Some(("/*", "*/")),
        },
        Language {
            name: "Go",
            extensions: &["go"],
            line_markers: &["//"],
            block_markers: Some(("/*", "*/")),
        },
        Language {
            name: "Shell",
            extensions: &["sh"],
            line_markers: &["#"],
            block_markers: None,
        },
        Language {
            name: "Perl",
            extensions: &["pl"],
            line_markers: &["#"],
            block_markers: None,
        },
        Language {
            name: "Ruby",
            extensions: &["rb"],
            line_markers: &["#"],
            block_markers: None,
        },
        Language {
            name: "PHP",
            extensions: &["php"],
            line_markers: &["//", "#"],
            block_markers: Some(("/*", "*/")),
        },
        Language {
            name: "HTML",
            extensions: &["html", "htm"],
            line_markers: &[],
            block_markers: Some(("<!--", "-->")),
        },
        Language {
            name: "CSS",
            extensions: &["css"],
            line_markers: &[],
            block_markers: Some(("/*", "*/")),
        },
        Language {
            name: "YAML",
            extensions: &["yml", "yaml"],
            line_markers: &["#"],
            block_markers: None,
        },
        Language {
            name: "TOML",
            extensions: &["toml"],
            line_markers: &["#"],
            block_markers: None,
        },
        Language {
            name: "Dockerfile",
            extensions: &[],
            line_markers: &["#"],
            block_markers: None,
        },
        Language {
            name: "Make",
            extensions: &[],
            line_markers: &["#"],
            block_markers: None,
        },
        Language {
            name: "CMake",
            extensions: &["cmake"],
            line_markers: &["#"],
            block_markers: None,
        },
    ];
    REPO
}

pub fn find_language_for_path(path: &Path) -> Option<&'static str> {
    // 1) By extension
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        let ext = ext.to_ascii_lowercase();
        for lang in language_registry() {
            if lang.extensions.iter().any(|e| *e == ext) {
                return Some(lang.name);
            }
        }
    }

    // 2) Special filenames
    if let Some(fname) = path.file_name().and_then(|s| s.to_str()) {
        let lower = fname.to_ascii_lowercase();
        match lower.as_str() {
            "makefile" => return Some("Make"),
            "dockerfile" => return Some("Dockerfile"),
            "cmakelists.txt" => return Some("CMake"),
            _ => {}
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
}
