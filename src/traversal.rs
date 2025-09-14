use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use ignore::{WalkBuilder, overrides::OverrideBuilder};

pub struct TraversalOptions {
    pub follow_symlinks: bool,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub ignore_file: Option<PathBuf>,
    pub allowed_exts: Option<HashSet<String>>, // lowercase, no dot
}

pub fn collect_files(root: &Path, opts: TraversalOptions) -> Result<Vec<PathBuf>> {
    let mut builder = WalkBuilder::new(root);
    builder.follow_links(opts.follow_symlinks);
    builder.hidden(false);
    builder.git_ignore(true);
    builder.git_exclude(true);
    builder.git_global(true);

    if let Some(ref custom) = opts.ignore_file {
        // The ignore crate doesn't directly take a custom path easily for patterns; naive load
        if let Ok(patterns) = fs::read_to_string(custom) {
            let mut ob = OverrideBuilder::new(root);
            for line in patterns.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                let _ = ob.add(line);
            }
            if let Ok(ov) = ob.build() {
                builder.add_custom_ignore_filename(custom.file_name().unwrap_or_default());
                builder.overrides(ov);
            }
        }
    }

    let mut out = Vec::new();
    for dent in builder.build() {
        let dent = match dent {
            Ok(d) => d,
            Err(_) => continue,
        };
        let path = dent.path();
        if !path.is_file() {
            continue;
        }

        if let Some(ref allowed) = opts.allowed_exts {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if !allowed.contains(&ext.to_ascii_lowercase()) {
                    continue;
                }
            } else {
                continue;
            }
        }

        if let Ok(md) = fs::metadata(path) {
            if let Some(min) = opts.min_size
                && md.len() < min
            {
                continue;
            }
            if let Some(max) = opts.max_size
                && md.len() > max
            {
                continue;
            }
        }

        // skip obvious binaries by a quick UTF-8 check of the first chunk
        if let Ok(mut f) = fs::File::open(path) {
            use std::io::Read;
            let mut buf = [0u8; 512];
            if let Ok(n) = f.read(&mut buf) {
                let slice = &buf[..n];
                if std::str::from_utf8(slice).is_err() {
                    continue;
                }
            }
        }

        out.push(path.to_path_buf());
    }

    Ok(out)
}
