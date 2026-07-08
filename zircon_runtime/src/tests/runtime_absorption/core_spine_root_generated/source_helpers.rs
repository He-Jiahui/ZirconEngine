use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn core_root_entries(runtime_root: &Path) -> BTreeSet<String> {
    fs::read_dir(runtime_root.join("src").join("core"))
        .unwrap_or_else(|error| panic!("failed to read core root: {error}"))
        .map(|entry| {
            entry
                .unwrap_or_else(|error| panic!("failed to read core root entry: {error}"))
                .file_name()
                .into_string()
                .unwrap_or_else(|name| panic!("non-utf8 core root entry: {name:?}"))
        })
        .collect()
}

pub(super) fn public_modules(path: &Path) -> Vec<String> {
    let source = read_source(path);
    source
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub mod "))
        .map(|decl| decl.trim_end_matches(';'))
        .map(String::from)
        .collect()
}

pub(super) fn public_use_count(path: &Path) -> usize {
    let source = read_source(path);
    source
        .lines()
        .filter(|line| line.trim_start().starts_with("pub use "))
        .count()
}

pub(super) fn crate_visible_graphics_reexport_count(path: &Path) -> usize {
    let source = read_source(path);
    let start_marker = "pub(crate) use graphics::{";
    let Some(start) = source.find(start_marker) else {
        return 0;
    };
    let body_start = start + start_marker.len();
    let body_end = source[body_start..]
        .find("};")
        .map(|offset| body_start + offset)
        .expect("crate-visible graphics alias block should terminate");
    source[body_start..body_end]
        .replace('\n', " ")
        .split(',')
        .map(str::trim)
        .filter(|symbol| !symbol.is_empty())
        .count()
}

pub(super) fn export_template_files(export_root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_source_files(export_root, &mut files);
    files.retain(|path| is_export_template_file(export_root, path));
    files.sort();
    files
}

fn is_export_template_file(export_root: &Path, path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    let relative = path
        .strip_prefix(export_root)
        .expect("template path should live below export_build_plan")
        .to_string_lossy()
        .replace('\\', "/");

    file_name.contains("template")
        || file_name == "generated_files.rs"
        || file_name == "platform_host_files.rs"
        || relative.starts_with("platform_host_files/")
}

pub(super) fn rust_test_count(path: &Path) -> usize {
    let source = read_source(path);
    source.matches("#[test]").count()
}

pub(super) fn rust_test_count_in_files(runtime_root: &Path, relatives: &[&str]) -> usize {
    relatives
        .iter()
        .map(|relative| rust_test_count(&runtime_root.join(relative)))
        .sum()
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

pub(super) fn read_source(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

pub(super) fn string_set(values: &[&str]) -> BTreeSet<String> {
    values.iter().copied().map(String::from).collect()
}

pub(super) fn string_vec(values: &[&str]) -> Vec<String> {
    values.iter().copied().map(String::from).collect()
}
