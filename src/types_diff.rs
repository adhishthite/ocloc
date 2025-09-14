use indexmap::IndexMap;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct LineDelta {
    pub files: isize,
    pub code_added: isize,
    pub code_removed: isize,
    pub comment_added: isize,
    pub blank_added: isize,
    pub total_net: isize,
}

impl LineDelta {
    pub fn add_file_delta(&mut self, base: (usize, usize, usize), head: (usize, usize, usize)) {
        let (base_code, base_comment, base_blank) = base;
        let (head_code, head_comment, head_blank) = head;
        self.files += 1;
        self.code_added += head_code as isize - base_code as isize;
        self.comment_added += head_comment as isize - base_comment as isize;
        self.blank_added += head_blank as isize - base_blank as isize;
        self.total_net += (head_code + head_comment + head_blank) as isize
            - (base_code + base_comment + base_blank) as isize;
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct DiffPerFile {
    pub path: String,
    pub status: String,
    pub language: String,
    pub code_delta: isize,
    pub comment_delta: isize,
    pub blank_delta: isize,
    pub total_delta: isize,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct DiffSummary {
    pub base_ref: Option<String>,
    pub head_ref: Option<String>,
    pub files: usize,
    pub files_added: usize,
    pub files_deleted: usize,
    pub files_modified: usize,
    pub files_renamed: usize,
    pub languages: IndexMap<String, LineDelta>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub by_file: Vec<DiffPerFile>,
    pub totals: LineDelta,
}
