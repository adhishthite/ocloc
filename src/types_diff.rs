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
    #[allow(clippy::cast_possible_wrap)]
    pub const fn add_file_delta(&mut self, base: (usize, usize, usize), head: (usize, usize, usize)) {
        let (base_code, base_comment, base_blank) = base;
        let (head_code, head_comment, head_blank) = head;
        self.files += 1;
        // Track added vs removed for code
        if head_code >= base_code {
            self.code_added += (head_code - base_code) as isize;
        } else {
            self.code_removed += (base_code - head_code) as isize;
        }
        // Only track additions for comment/blank per plan
        if head_comment > base_comment {
            self.comment_added += (head_comment - base_comment) as isize;
        }
        if head_blank > base_blank {
            self.blank_added += (head_blank - base_blank) as isize;
        }
        // Net total change across all categories
        self.total_net += (head_code + head_comment + head_blank) as isize
            - (base_code + base_comment + base_blank) as isize;
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct GitRefInfo {
    #[serde(rename = "ref")]
    pub reference: Option<String>,
    pub short: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base: Option<GitRefInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<GitRefInfo>,
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
