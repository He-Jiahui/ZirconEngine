use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn line_count(path: &Path) -> usize {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
        .lines()
        .count()
}

pub(super) fn collect_direct_rayon_paths(source_root: &Path) -> BTreeSet<String> {
    let runtime_root = source_root
        .parent()
        .expect("runtime source root should have manifest parent");
    rust_source_files(source_root)
        .into_iter()
        .filter(|path| !is_test_path(&relative_path(runtime_root, path)))
        .filter(|path| {
            fs::read_to_string(path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .lines()
                .any(line_mentions_rayon)
        })
        .map(|path| relative_path(runtime_root, &path))
        .collect()
}

fn rust_source_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_source_files(root, &mut files);
    files.sort();
    files
}

fn collect_rust_source_files(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).expect("runtime source directory should be readable") {
        let entry = entry.expect("runtime source entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_source_files(&path, files);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

fn line_mentions_rayon(line: &str) -> bool {
    line.contains("use rayon")
        || line.contains("rayon::")
        || line.contains(".par_iter(")
        || line.contains(".par_chunks")
        || line.contains(".into_par_iter(")
}

fn is_test_path(relative_path: &str) -> bool {
    let file_name = relative_path.rsplit('/').next().unwrap_or(relative_path);
    relative_path.split('/').any(|part| part == "tests")
        || file_name == "tests.rs"
        || file_name.ends_with("_tests.rs")
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
