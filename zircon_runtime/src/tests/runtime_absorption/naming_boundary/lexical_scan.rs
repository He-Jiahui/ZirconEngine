use std::fs;
use std::path::{Path, PathBuf};

#[path = "lexical_scan/production_lines.rs"]
mod production_lines;

use production_lines::production_source_lines;

#[derive(Debug)]
pub(super) struct NamingReference {
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

pub(super) fn collect_naming_references(
    manifest_root: &Path,
    files: &[PathBuf],
    term: &str,
) -> Vec<NamingReference> {
    let mut references = Vec::new();
    for path in files {
        let relative = relative_path(manifest_root, path);
        let source = fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"));
        for (line_index, line) in production_source_lines(&source) {
            if line_has_term(line, term) {
                references.push(NamingReference {
                    path: relative.clone(),
                    line: line_index,
                    snippet: line.trim().to_string(),
                });
            }
        }
    }
    references
}

pub(super) fn collect_server_references(
    manifest_root: &Path,
    files: &[PathBuf],
) -> Vec<NamingReference> {
    let mut references = Vec::new();
    for path in files {
        let relative = relative_path(manifest_root, path);
        let source = fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"));
        for (line_index, line) in production_source_lines(&source) {
            if server_tokens(line).next().is_some() {
                references.push(NamingReference {
                    path: relative.clone(),
                    line: line_index,
                    snippet: line.trim().to_string(),
                });
            }
        }
    }
    references
}

fn line_has_term(line: &str, term: &str) -> bool {
    line.split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .any(|token| token.to_ascii_lowercase().contains(term))
}

fn server_tokens(line: &str) -> impl Iterator<Item = &str> {
    line.split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .filter(|token| token_has_server_component(token))
}

fn token_has_server_component(token: &str) -> bool {
    let lower = token.to_ascii_lowercase();
    let mut search_start = 0;
    while let Some(relative_index) = lower[search_start..].find("server") {
        let start = search_start + relative_index;
        let end = start + "server".len();
        if start >= 2 && &lower[start - 2..end] == "observer" {
            search_start = end;
            continue;
        }
        if start >= 2 && end < lower.len() && &lower[start - 2..end + 1] == "observers" {
            search_start = end;
            continue;
        }
        return true;
    }
    false
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .expect("runtime source path should live under manifest root")
        .to_string_lossy()
        .replace('\\', "/")
}
