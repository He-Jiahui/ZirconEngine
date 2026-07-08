use super::*;

pub(super) fn read_runtime_sources(paths: &[&str]) -> String {
    paths
        .iter()
        .map(|path| read_runtime_src(path))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn read_status_support_expected_slice_rows() -> String {
    read_review_guard_structure_rows()
}
