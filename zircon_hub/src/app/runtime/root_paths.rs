use std::path::PathBuf;

use crate::projects::project_filesystem_path_key;

pub(super) fn push_unique_root(roots: &mut Vec<PathBuf>, path: PathBuf) {
    if path.as_os_str().is_empty() {
        return;
    }
    let candidate_key = project_filesystem_path_key(&path);
    if roots
        .iter()
        .any(|root| project_filesystem_path_key(root) == candidate_key)
    {
        return;
    }
    roots.push(path);
}

pub(super) fn push_development_roots(roots: &mut Vec<PathBuf>, source_dir: PathBuf) {
    push_unique_root(roots, source_dir);
    if let Ok(current_dir) = std::env::current_dir() {
        push_unique_root(roots, current_dir);
    }
    if let Some(compiled_repo_root) = compiled_repo_root() {
        push_unique_root(roots, compiled_repo_root);
    }
}

fn compiled_repo_root() -> Option<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|path| path.to_path_buf())
}
