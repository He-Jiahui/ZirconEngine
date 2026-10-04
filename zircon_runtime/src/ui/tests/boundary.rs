//! UI 边界结构测试共用文件扫描辅助函数；扫描结果用于资源位置与模块导出限制的断言。
mod asset_fixture_projection;
mod binding_event_roots;
mod layout_tree_surface;
mod template_namespace;

use std::fs;
use std::path::{Path, PathBuf};

fn collect_ui_toml_files(root: &Path) -> Vec<PathBuf> {
    collect_files_with_suffixes(root, &[".ui.toml"])
}

fn collect_zui_files(root: &Path) -> Vec<PathBuf> {
    collect_files_with_suffixes(root, &[".zui"])
}

fn collect_ui_document_files(root: &Path) -> Vec<PathBuf> {
    collect_files_with_suffixes(root, &[".ui.toml", ".zui"])
}

fn collect_files_with_suffixes(root: &Path, suffixes: &[&str]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_files_with_suffixes_inner(root, suffixes, &mut files);
    files.sort();
    files
}

fn collect_files_with_suffixes_inner(root: &Path, suffixes: &[&str], files: &mut Vec<PathBuf>) {
    // BUG: [CR-UI-TEST-0801] 目录不可读取时扫描结果被当作空集合，资源目录的否定断言会假通过；证据：boundary/asset_fixture_projection.rs:4-13 的否定断言。
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            collect_files_with_suffixes_inner(&path, suffixes, files);
            continue;
        }

        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| suffixes.iter().any(|suffix| name.ends_with(suffix)))
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
