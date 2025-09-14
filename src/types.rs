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
