use super::*;

pub(in super::super::super) const STRUCTURE_SUPPORT_EXPECTED_SLICE_ROW_CHILDREN: &[&str] = &[
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/structure_support/foundation_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/structure_support/guard_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/structure_support/map_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/structure_support/parent_route_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/structure_support/review_route_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/structure_support/row_data_owner_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/structure_support/typed_error_rows.rs",
];

pub(in super::super::super) fn read_review_guard_structure_rows() -> String {
    std::iter::once(REVIEW_GUARD_STRUCTURE_ROWS)
        .chain(REVIEW_GUARD_STRUCTURE_ROW_CHILDREN.iter().copied())
        .chain(REVIEW_GUARD_STRUCTURE_ROW_GRANDCHILDREN.iter().copied())
        .chain(
            STRUCTURE_SUPPORT_EXPECTED_SLICE_ROW_CHILDREN
                .iter()
                .copied(),
        )
        .chain(
            STRUCTURE_SUPPORT_ROW_DATA_OWNER_ROW_CHILDREN
                .iter()
                .copied(),
        )
        .map(read_runtime_src)
        .collect::<Vec<_>>()
        .join("\n")
}

pub(in super::super::super) fn read_structure_support_expected_slice_rows() -> String {
    std::iter::once(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/structure_support.rs",
    )
    .chain(STRUCTURE_SUPPORT_EXPECTED_SLICE_ROW_CHILDREN.iter().copied())
    .chain(STRUCTURE_SUPPORT_ROW_DATA_OWNER_ROW_CHILDREN.iter().copied())
    .map(read_runtime_src)
    .collect::<Vec<_>>()
    .join("\n")
}
