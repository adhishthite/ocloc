use anyhow::{Context, Result, anyhow};
use git2::{Delta, DiffOptions, Oid, Repository};
use std::path::{Path, PathBuf};

//

#[derive(Debug, Clone, Copy)]
pub struct FileChangeOids {
    pub old: Option<Oid>,
    pub new: Option<Oid>,
}

#[derive(Debug, Clone)]
pub struct FileChange {
    pub status: String, // "A","M","D","R"
    pub old_path: Option<PathBuf>,
    pub new_path: Option<PathBuf>,
    pub oids: FileChangeOids,
}

pub struct VcsContext {
    pub repo: Repository,
}

impl VcsContext {
    pub fn open(path: &Path) -> Result<Self> {
        let repo = Repository::discover(path).context("open git repo")?;
        Ok(Self { repo })
    }

    pub fn resolve_oid(&self, rev: &str) -> Result<Oid> {
        let obj = self
            .repo
            .revparse_single(rev)
            .with_context(|| format!("resolve rev {rev}"))?;
        Ok(obj.id())
    }

    pub fn merge_base(&self, a: Oid, b: Oid) -> Result<Oid> {
        let base = self.repo.merge_base(a, b).context("merge-base")?;
        Ok(base)
    }

    pub fn head_oid(&self) -> Result<Oid> {
        let head = self.repo.head()?;
        head.target()
            .ok_or_else(|| anyhow!("detached HEAD not supported"))
    }

    pub fn diff_between(&self, base: Oid, head: Oid) -> Result<Vec<FileChange>> {
        let base_commit = self.repo.find_commit(base)?;
        let head_commit = self.repo.find_commit(head)?;
        let base_tree = base_commit.tree()?;
        let head_tree = head_commit.tree()?;
        let mut opts = DiffOptions::new();
        opts.recurse_ignored_dirs(true)
            .ignore_submodules(true)
            .include_typechange(true)
            .show_binary(false);
        let diff =
            self.repo
                .diff_tree_to_tree(Some(&base_tree), Some(&head_tree), Some(&mut opts))?;
        self.collect_changes_from_diff(diff)
    }

    pub fn diff_head_to_index(&self) -> Result<Vec<FileChange>> {
        let head_commit = self.repo.find_commit(self.head_oid()?)?;
        let head_tree = head_commit.tree()?;
        let mut index = self.repo.index()?;
        let index_tree = index
            .write_tree_to(&self.repo)
            .and_then(|oid| self.repo.find_tree(oid))?;
        let mut opts = DiffOptions::new();
        opts.recurse_ignored_dirs(true)
            .ignore_submodules(true)
            .include_typechange(true)
            .show_binary(false);
        let diff =
            self.repo
                .diff_tree_to_tree(Some(&head_tree), Some(&index_tree), Some(&mut opts))?;
        self.collect_changes_from_diff(diff)
    }

    pub fn diff_index_to_workdir(&self) -> Result<Vec<FileChange>> {
        let mut opts = DiffOptions::new();
        opts.recurse_ignored_dirs(true)
            .ignore_submodules(true)
            .include_typechange(true)
            .show_binary(false);
        let diff = self.repo.diff_index_to_workdir(None, Some(&mut opts))?;
        self.collect_changes_from_diff(diff)
    }

    fn collect_changes_from_diff(&self, mut diff: git2::Diff) -> Result<Vec<FileChange>> {
        let mut out = Vec::new();
        // Enable rename detection so we can report 'R' statuses
        let mut find_opts = git2::DiffFindOptions::new();
        find_opts.renames(true).renames_from_rewrites(true);
        let _ = diff.find_similar(Some(&mut find_opts));
        diff.print(git2::DiffFormat::NameStatus, |_delta, _hunk, _line| true)?;

        diff.foreach(
            &mut |d, _| {
                let status = match d.status() {
                    Delta::Added => "A",
                    Delta::Modified => "M",
                    Delta::Deleted => "D",
                    Delta::Renamed => "R",
                    _ => "M",
                }
                .to_string();
                let old_path = d.old_file().path().map(|p| p.to_path_buf());
                let new_path = d.new_file().path().map(|p| p.to_path_buf());
                let old_id = d.old_file().id();
                let new_id = d.new_file().id();
                let old_oid = if old_id.is_zero() { None } else { Some(old_id) };
                let new_oid = if new_id.is_zero() { None } else { Some(new_id) };
                out.push(FileChange {
                    status,
                    old_path,
                    new_path,
                    oids: FileChangeOids {
                        old: old_oid,
                        new: new_oid,
                    },
                });
                true
            },
            None,
            None,
            None,
        )?;
        Ok(out)
    }

    pub fn read_blob_bytes(&self, oid: Option<Oid>) -> Option<Vec<u8>> {
        let oid = oid?;
        let blob = self.repo.find_blob(oid).ok()?;
        Some(blob.content().to_vec())
    }

    pub fn read_index_blob_bytes(&self, path: &Path) -> Option<Vec<u8>> {
        let index = self.repo.index().ok()?;
        // Paths from diff are repo-relative; ensure we look up by that
        let rel = path;
        let entry = index.get_path(rel, 0)?;
        let blob = self.repo.find_blob(entry.id).ok()?;
        Some(blob.content().to_vec())
    }
}
