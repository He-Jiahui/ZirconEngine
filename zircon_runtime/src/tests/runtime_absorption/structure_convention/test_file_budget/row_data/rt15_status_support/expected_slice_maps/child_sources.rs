use super::*;

pub(super) const EXPECTED_SLICE_ROW_DATA_OWNER_STATUS_ANCHORS: &[&str] = &[
    EXPECTED_SLICE_ROW_DATA_OWNER_STATUS_NAME,
    EXPECTED_SLICE_ROW_DATA_OWNER_STATUS_ID,
    "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps.rs",
    "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/base_maps.rs",
    "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/structure_support.rs",
    "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/status_support_maps.rs",
    EXPECTED_SLICE_ROW_DATA_OWNER_GUARD_NAME,
    "Cargo gate deferred",
];

pub(super) const EXPECTED_SLICE_ROW_DATA_GUARD_STATUS_ANCHORS: &[&str] = &[
    EXPECTED_SLICE_ROW_DATA_GUARD_STATUS_NAME,
    EXPECTED_SLICE_ROW_DATA_GUARD_STATUS_ID,
    "structure_convention/test_file_budget/row_data/rt15_status_support/expected_slice_maps.rs",
    "structure_convention/test_file_budget/row_data/rt15_status_support/expected_slice_maps/route_mounts.rs",
    "structure_convention/test_file_budget/row_data/rt15_status_support/expected_slice_maps/aggregation.rs",
    "structure_convention/test_file_budget/row_data/rt15_status_support/expected_slice_maps/status_mirrors.rs",
    "structure_convention/test_file_budget/row_data/rt15_status_support/expected_slice_maps/folder_backed.rs",
    EXPECTED_SLICE_ROW_DATA_GUARD_NAME,
    "Cargo gate deferred",
];

pub(super) fn expected_slice_child_source(module_name: &str, path: &str) -> String {
    match module_name {
        "top_level_support" => {
            source_with_children(path, EXPECTED_SLICE_TOP_LEVEL_SUPPORT_CHILDREN)
        }
        "route_metadata" => source_with_children(path, EXPECTED_SLICE_ROUTE_METADATA_CHILDREN),
        "status_support_maps" => {
            source_with_children(path, EXPECTED_SLICE_STATUS_SUPPORT_MAPS_CHILDREN)
        }
        "review_guard_structure" => source_with_two_child_sets(
            path,
            EXPECTED_SLICE_REVIEW_GUARD_STRUCTURE_CHILDREN,
            EXPECTED_SLICE_REVIEW_GUARD_STRUCTURE_NESTED_CHILDREN,
        ),
        "structure_support" => source_with_two_child_sets(
            path,
            EXPECTED_SLICE_STRUCTURE_SUPPORT_CHILDREN,
            EXPECTED_SLICE_STRUCTURE_SUPPORT_NESTED_CHILDREN,
        ),
        _ => read_runtime_src(path),
    }
}

fn source_with_children(path: &str, children: &[&str]) -> String {
    std::iter::once(path)
        .chain(children.iter().copied())
        .map(read_runtime_src)
        .collect::<Vec<_>>()
        .join("\n")
}

fn source_with_two_child_sets(path: &str, first: &[&str], second: &[&str]) -> String {
    std::iter::once(path)
        .chain(first.iter().copied())
        .chain(second.iter().copied())
        .map(read_runtime_src)
        .collect::<Vec<_>>()
        .join("\n")
}
