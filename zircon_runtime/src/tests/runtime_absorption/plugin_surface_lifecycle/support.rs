use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn native_plugin_namespace_reexport_symbols() -> Vec<String> {
    let source = include_str!("../../../plugin/native.rs");
    let start_marker = "pub use super::native_plugin_loader::{";
    let start = source
        .find(start_marker)
        .expect("plugin::native should expose the native loader public namespace");
    let body_start = start + start_marker.len();
    let body_end = source[body_start..]
        .find("};")
        .map(|offset| body_start + offset)
        .expect("native namespace re-export block should terminate");

    source[body_start..body_end]
        .replace('\n', " ")
        .split(',')
        .map(str::trim)
        .filter(|symbol| !symbol.is_empty())
        .map(String::from)
        .collect()
}

pub(super) fn native_root_import_leak_files(root: &Path) -> BTreeSet<String> {
    let workspace_root = workspace_root_from(root);
    rust_source_files(root)
        .into_iter()
        .filter(|path| {
            let source = fs::read_to_string(path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            has_native_root_import_leak(&source)
        })
        .map(|path| relative_path(&workspace_root, &path))
        .collect()
}

fn has_native_root_import_leak(source: &str) -> bool {
    if source.contains("crate::plugin::NativePlugin")
        || source.contains("crate::plugin::ZIRCON_NATIVE_PLUGIN")
        || source.contains("zircon_runtime::plugin::NativePlugin")
        || source.contains("zircon_runtime::plugin::ZIRCON_NATIVE_PLUGIN")
    {
        return true;
    }

    for marker in ["use crate::plugin::", "use zircon_runtime::plugin::"] {
        let mut search_start = 0;
        while let Some(relative_start) = source[search_start..].find(marker) {
            let statement_start = search_start + relative_start;
            let statement_tail = &source[statement_start..];
            if statement_tail.starts_with("use crate::plugin::native::")
                || statement_tail.starts_with("use zircon_runtime::plugin::native::")
            {
                search_start = statement_start + marker.len();
                continue;
            }

            let statement_end = statement_tail.find(';').unwrap_or(statement_tail.len());
            let statement = &statement_tail[..statement_end];
            if statement.contains("NativePlugin") || statement.contains("ZIRCON_NATIVE_PLUGIN") {
                return true;
            }
            search_start = statement_start + statement_end + 1;
        }
    }

    false
}

pub(super) fn files_containing(root: &Path, patterns: &[&str]) -> BTreeSet<String> {
    let workspace_root = workspace_root_from(root);
    rust_source_files(root)
        .into_iter()
        .filter(|path| {
            let source = fs::read_to_string(path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            patterns.iter().any(|pattern| source.contains(pattern))
        })
        .map(|path| relative_path(&workspace_root, &path))
        .collect()
}

pub(super) fn location_count(root: &Path, patterns: &[&str]) -> usize {
    rust_source_files(root)
        .into_iter()
        .map(|path| {
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            patterns
                .iter()
                .map(|pattern| source.matches(pattern).count())
                .sum::<usize>()
        })
        .sum()
}

fn rust_source_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_source_files(root, &mut files);
    files.sort();
    files
}

fn collect_rust_source_files(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
    {
        let entry = entry.unwrap_or_else(|error| panic!("failed to read entry: {error}"));
        let path = entry.path();
        if path.is_dir() {
            collect_rust_source_files(&path, files);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

fn workspace_root_from(path: &Path) -> PathBuf {
    path.ancestors()
        .find(|ancestor| ancestor.join("zircon_runtime").is_dir())
        .unwrap_or(path)
        .to_path_buf()
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
