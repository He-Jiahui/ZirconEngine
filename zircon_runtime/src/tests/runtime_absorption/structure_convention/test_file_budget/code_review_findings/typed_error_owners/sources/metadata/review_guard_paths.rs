use super::super::super::super::super::*;

pub(in super::super) const TYPED_ERROR_CHILD_OWNER_LINE_BUDGET: usize = 800;

pub(in super::super) const REVIEW_GUARD_STATUS_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows.rs";
pub(in super::super) const REVIEW_GUARD_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs";
pub(in super::super) const REVIEW_GUARD_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs";
pub(in super::super) const REVIEW_GUARD_TYPED_ERROR_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_maps.rs";
pub(in super::super) const REVIEW_GUARD_TYPED_ERROR_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/typed_error_maps.rs";

pub(in super::super) fn typed_error_source_inventory_status_rows_source() -> String {
    let mut source = String::new();
    for path in [
        REVIEW_GUARD_STATUS_ROWS_PATH,
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows/source_inventory_foundation_rows.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows/source_inventory_delegation_rows.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows/source_inventory_inventory_metadata_rows.rs",
    ] {
        source.push_str(&read_runtime_src(path));
        source.push('\n');
    }
    source
}

pub(in super::super) fn typed_error_source_inventory_status_map_source() -> String {
    format!(
        "{}\n{}\n{}",
        read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH),
        read_runtime_src(REVIEW_GUARD_TYPED_ERROR_STATUS_MAP_PATH),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_maps/source_inventory_rows.rs",
        )
    )
}

pub(in super::super) fn typed_error_source_inventory_date_map_source() -> String {
    format!(
        "{}\n{}\n{}",
        read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH),
        read_runtime_src(REVIEW_GUARD_TYPED_ERROR_DATE_MAP_PATH),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/typed_error_maps/source_inventory_rows.rs",
        )
    )
}
