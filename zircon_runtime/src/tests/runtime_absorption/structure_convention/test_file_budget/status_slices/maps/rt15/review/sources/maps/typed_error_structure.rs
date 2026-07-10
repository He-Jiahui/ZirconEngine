use super::*;

pub(in super::super::super) const TYPED_ERROR_STRUCTURE_STATUS_MAP_SOURCE: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/sources/maps/typed_error_structure.rs";
pub(in super::super::super) const STATUS_REVIEW_TYPED_ERROR_STRUCTURE_CHILD: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps/typed_error_structure_maps.rs";
pub(in super::super::super) const STATUS_REVIEW_TYPED_ERROR_STRUCTURE_CHILDREN: &[&str] = &[
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps/typed_error_structure_maps/expected_slice_map_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps/typed_error_structure_maps/moved_guard_absence_maps.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps/typed_error_structure_maps/native_plugin_loader_maps.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps/typed_error_structure_maps/structure_assertion_maps.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps/typed_error_structure_maps/structure_guard_maps.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps/typed_error_structure_maps/top_level_maps.rs",
];
pub(in super::super::super) const DATE_REVIEW_TYPED_ERROR_STRUCTURE_CHILD: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps/typed_error_structure_maps.rs";
pub(in super::super::super) const DATE_REVIEW_TYPED_ERROR_STRUCTURE_CHILDREN: &[&str] = &[
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps/typed_error_structure_maps/expected_slice_map_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps/typed_error_structure_maps/moved_guard_absence_maps.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps/typed_error_structure_maps/native_plugin_loader_maps.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps/typed_error_structure_maps/structure_assertion_maps.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps/typed_error_structure_maps/structure_guard_maps.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps/typed_error_structure_maps/top_level_maps.rs",
];

pub(in super::super::super) fn read_status_review_typed_error_structure_sources() -> String {
    read_review_route_sources(
        STATUS_REVIEW_TYPED_ERROR_STRUCTURE_CHILD,
        STATUS_REVIEW_TYPED_ERROR_STRUCTURE_CHILDREN,
    )
}

pub(in super::super::super) fn read_date_review_typed_error_structure_sources() -> String {
    read_review_route_sources(
        DATE_REVIEW_TYPED_ERROR_STRUCTURE_CHILD,
        DATE_REVIEW_TYPED_ERROR_STRUCTURE_CHILDREN,
    )
}

fn read_review_route_sources(parent: &str, children: &[&str]) -> String {
    std::iter::once(parent)
        .chain(children.iter().copied())
        .map(read_runtime_src)
        .collect::<Vec<_>>()
        .join("\n")
}
