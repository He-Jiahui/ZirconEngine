use std::path::Path;

pub(super) fn assert_files_exist(runtime_root: &Path, files: &[&str], label: &str) {
    for file in files {
        assert!(
            runtime_root.join(file).exists(),
            "{label} file `{file}` is missing; update ecs_kernel_data_boundary before changing the Runtime 08 owner set"
        );
    }
}

pub(super) fn assert_source_anchors(label: &str, sources: &[&str], anchors: &[&str]) {
    for anchor in anchors {
        assert!(
            sources.iter().any(|source| source.contains(anchor)),
            "{label} should retain `{anchor}`"
        );
    }
}
