use std::fs;
use std::path::Path;

pub(super) fn assert_files_exist(runtime_root: &Path, files: &[&str], label: &str) {
    for file in files {
        let path = runtime_root.join(file);
        assert!(
            path.exists(),
            "{label} file `{}` is missing; update script_binding_boundary before changing Runtime 13 ownership",
            path.display()
        );
    }
}

pub(super) fn assert_file_line_budget(
    runtime_root: &Path,
    file: &str,
    max_lines: usize,
    label: &str,
) {
    let path = runtime_root.join(file);
    let line_count = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
        .lines()
        .count();
    assert!(
        line_count <= max_lines,
        "{label} `{file}` has {line_count} lines, exceeding the {max_lines}-line owner budget"
    );
}

pub(super) fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.match_indices(needle).count()
}
