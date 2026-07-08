use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn rust_source_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_source_files(root, &mut files);
    files.sort();
    files
}

pub(super) fn collect_rust_source_files(root: &Path, files: &mut Vec<PathBuf>) {
    if !root.exists() {
        return;
    }
    for entry in fs::read_dir(root).unwrap_or_else(|error| {
        panic!(
            "failed to read source directory {}: {error}",
            root.display()
        )
    }) {
        let entry = entry.expect("source directory entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_source_files(&path, files);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

pub(super) fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .expect("source path should live under manifest root")
        .to_string_lossy()
        .replace('\\', "/")
}

pub(super) fn trimmed_snippet(line: &str) -> String {
    const MAX_SNIPPET_LEN: usize = 220;
    let trimmed = line.trim();
    if trimmed.len() <= MAX_SNIPPET_LEN {
        trimmed.to_string()
    } else {
        format!("{}...", trimmed[..MAX_SNIPPET_LEN].trim_end())
    }
}
