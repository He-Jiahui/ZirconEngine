use std::path::Path;

pub(super) fn assert_files_exist(runtime_root: &Path, files: &[&str], label: &str, boundary: &str) {
    for file in files {
        let path = runtime_root.join(file);
        assert!(
            path.exists(),
            "{label} `{}` is missing; update {boundary}",
            path.display()
        );
    }
}

pub(super) fn assert_contains_all(label: &str, source: &str, required: &[&str]) {
    for anchor in required {
        assert!(
            source.contains(anchor),
            "{label} should contain Runtime 04 asset-pipeline audit anchor `{anchor}`"
        );
    }
}
