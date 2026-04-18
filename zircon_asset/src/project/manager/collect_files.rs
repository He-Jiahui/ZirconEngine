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
        } else if path.is_file() && !is_meta_sidecar(&path) {
            files.push(path);
        }
    }
    Ok(())
}
