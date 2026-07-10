use super::*;

const CHILD_OWNER_READ_ROOT: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/owners";

pub(in super::super) fn read_child_owner(relative_path: &str) -> String {
    read_runtime_src(&format!("{CHILD_OWNER_READ_ROOT}/{relative_path}"))
}

pub(in super::super) fn read_child_owner_parent() -> String {
    read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/child_owners.rs",
    )
}

pub(in super::super) fn read_runtime_absorption_sources(paths: &[&str]) -> String {
    paths
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(in super::super) fn read_status_support_expected_slice_rows() -> String {
    format!(
        "{}\n{}",
        read_top_level_support_row_sources(),
        read_route_metadata_row_sources()
    )
}

pub(in super::super) fn read_top_level_support_row_sources() -> String {
    std::iter::once(TOP_LEVEL_SUPPORT_ROWS_PATH)
        .chain(TOP_LEVEL_SUPPORT_ROW_CHILDREN.iter().copied())
        .map(read_runtime_src)
        .collect::<Vec<_>>()
        .join("\n")
}

pub(in super::super) fn read_route_metadata_row_sources() -> String {
    std::iter::once(ROUTE_METADATA_ROWS_PATH)
        .chain(ROUTE_METADATA_ROW_CHILDREN.iter().copied())
        .map(read_runtime_src)
        .collect::<Vec<_>>()
        .join("\n")
}
