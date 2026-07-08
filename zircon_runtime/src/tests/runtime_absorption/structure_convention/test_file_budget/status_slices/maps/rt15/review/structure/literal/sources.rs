use super::*;

pub(super) fn read_sources(paths: &[&str]) -> String {
    paths
        .iter()
        .map(|path| read_runtime_src(path))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn read_literal_owner_source(relative_path: &str) -> String {
    read_runtime_src(&format!(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/{relative_path}"
    ))
}
