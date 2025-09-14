use indexmap::IndexMap;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct FileCounts {
    pub files: usize,
    pub total: usize,
    pub code: usize,
    pub comment: usize,
    pub blank: usize,
}

impl FileCounts {
    pub fn one_file() -> Self {
        FileCounts {
            files: 1,
            ..Default::default()
        }
    }

    pub fn merge(&mut self, other: &FileCounts) {
        self.files += other.files;
        self.total += other.total;
        self.code += other.code;
        self.comment += other.comment;
        self.blank += other.blank;
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalyzeResult {
    #[serde(rename = "languages")]
    pub per_lang: IndexMap<String, FileCounts>,
    pub totals: FileCounts,
    pub files_analyzed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_includes_new_languages() {
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
        };
        let s = serde_json::to_string_pretty(&a).unwrap();
        assert!(s.contains("\"Markdown\""));
        assert!(s.contains("\"SVG\""));
        assert!(s.contains("\"languages\""));
        assert!(s.contains("\"totals\""));
    }
}
