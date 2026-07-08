use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(super) struct RayonReference {
    pub(super) path: String,
    pub(super) line: usize,
    pub(super) snippet: String,
}

pub(super) fn rust_source_files(root: &Path) -> Vec<PathBuf> {
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

pub(super) fn collect_rayon_references(
    manifest_root: &Path,
    files: &[PathBuf],
) -> Vec<RayonReference> {
    let mut references = Vec::new();
    for path in files {
        let relative = relative_path(manifest_root, path);
        if is_test_path(&relative) {
            continue;
        }

        let source = fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"));
        for (line_index, line) in source.lines().enumerate() {
            if line_mentions_rayon(line) {
                references.push(RayonReference {
                    path: relative.clone(),
                    line: line_index + 1,
                    snippet: line.trim().to_string(),
                });
            }
        }
    }
    references
}

fn line_mentions_rayon(line: &str) -> bool {
    line.contains("use rayon")
        || line.contains("rayon::")
        || line.contains(".par_iter(")
        || line.contains(".par_chunks")
        || line.contains(".into_par_iter(")
}

pub(super) fn classify_rayon_reference(relative_path: &str) -> Option<&'static str> {
    match relative_path {
        "src/core/runtime/tasks/pool.rs" => Some("core-task-pool-rayon-owner"),
        "src/core/runtime/tasks/parallel_for.rs" => Some("core-task-parallel-for-owner"),
        _ => None,
    }
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
