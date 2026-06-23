mod asset_fixture_projection;
mod binding_event_roots;
mod layout_tree_surface;
mod template_namespace;

use std::fs;
use std::path::{Path, PathBuf};

fn collect_ui_toml_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_ui_toml_files_inner(root, &mut files);
    files.sort();
    files
}

fn collect_ui_toml_files_inner(root: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            collect_ui_toml_files_inner(&path, files);
            continue;
        }

        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".ui.toml"))
        {
            files.push(path);
        }
    }
}

fn rel_paths(paths: &[PathBuf], base: &Path) -> Vec<String> {
    paths
        .iter()
        .map(|path| {
            relative_path(path, base)
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect()
}

fn format_paths(paths: &[PathBuf], base: &Path) -> String {
    rel_paths(paths, base)
        .into_iter()
        .map(|path| path.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn relative_path(path: &Path, base: &Path) -> PathBuf {
    path.strip_prefix(base)
        .expect("path should stay under the manifest dir")
        .to_path_buf()
}
