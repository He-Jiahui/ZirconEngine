use std::fs;
use std::path::{Path, PathBuf};

use super::is_meta_sidecar::is_meta_sidecar;

pub(super) fn collect_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, files)?;
        } else if path.is_file() && !is_meta_sidecar(&path) && !is_auxiliary_source_file(&path) {
            files.push(path);
        }
    }
    Ok(())
}

fn is_auxiliary_source_file(path: &Path) -> bool {
    // External glTF buffers and raw font binaries are source auxiliaries, not standalone assets.
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("bin")
                || extension.eq_ignore_ascii_case("ttf")
                || extension.eq_ignore_ascii_case("otf")
                || extension.eq_ignore_ascii_case("woff")
                || extension.eq_ignore_ascii_case("woff2")
        })
}
