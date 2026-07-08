use std::ffi::OsStr;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime manifest should live under the repository root")
        .to_path_buf()
}

fn repo_path(relative: &str) -> PathBuf {
    repo_root().join(relative)
}

pub(super) fn read_repo_file(relative: &str) -> String {
    let path = repo_path(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

pub(super) fn top_level_entry_names(relative: &str, include_root_mod: bool) -> Vec<String> {
    let dir = repo_path(relative);
    let mut entries = std::fs::read_dir(&dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| {
            let entry = entry.unwrap_or_else(|error| panic!("failed to read dir entry: {error}"));
            entry.file_name().into_string().unwrap_or_else(|name| {
                panic!("non-utf8 filename under {}: {name:?}", dir.display())
            })
        })
        .filter(|name| include_root_mod || name != "mod.rs")
        .collect::<Vec<_>>();
    entries.sort();
    entries
}

pub(super) fn rust_files_under(relative: &str) -> Vec<PathBuf> {
    let mut pending = vec![repo_path(relative)];
    let mut files = Vec::new();

    while let Some(path) = pending.pop() {
        for entry in std::fs::read_dir(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
        {
            let entry = entry.unwrap_or_else(|error| panic!("failed to read dir entry: {error}"));
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }

    files.sort();
    files
}

fn has_component(path: &Path, component: &str) -> bool {
    let component = OsStr::new(component);
    path.components()
        .any(|path_component| path_component.as_os_str() == component)
}

pub(super) fn production_ui_file(path: &Path) -> bool {
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    !has_component(path, "tests")
        && !has_component(path, "test_fixtures")
        && filename != "tests.rs"
        && !filename.ends_with("_tests.rs")
}

pub(super) fn matching_line_count(files: &[PathBuf], needle: &str) -> usize {
    files
        .iter()
        .map(|path| {
            std::fs::read_to_string(path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .lines()
                .filter(|line| line.contains(needle))
                .count()
        })
        .sum()
}

pub(super) fn files_with_matching_line(files: &[PathBuf], needle: &str) -> Vec<PathBuf> {
    files
        .iter()
        .filter(|path| {
            std::fs::read_to_string(path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .lines()
                .any(|line| line.contains(needle))
        })
        .cloned()
        .collect()
}
