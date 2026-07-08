use super::*;

pub(super) fn code_review_rows_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in CODE_REVIEW_ROWS_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}

pub(super) fn status_support_review_guard_source_blob() -> String {
    [
        read_runtime_src(STATUS_SUPPORT_ROWS_PATH),
        read_runtime_src(STATUS_SUPPORT_REVIEW_GUARD_ROWS_PATH),
        read_runtime_src(STATUS_SUPPORT_REVIEW_GUARD_CODE_REVIEW_ROWS_PATH),
        read_runtime_src(STATUS_SUPPORT_REVIEW_GUARD_ROW_DATA_ROWS_PATH),
    ]
    .join("\n")
}

pub(super) fn status_support_status_map_source_blob() -> String {
    [
        read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH),
        read_runtime_src("tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/row_data_maps/review_guard_row_data_maps/code_review_maps.rs"),
        read_runtime_src("tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/row_data_maps/review_guard_row_data_maps/row_data_guard_maps.rs"),
    ]
    .join("\n")
}

pub(super) fn status_support_date_map_source_blob() -> String {
    [
        read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH),
        read_runtime_src("tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/row_data_maps/review_guard_row_data_maps/code_review_maps.rs"),
        read_runtime_src("tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/row_data_maps/review_guard_row_data_maps/row_data_guard_maps.rs"),
    ]
    .join("\n")
}

pub(super) fn review_guard_status_map_source_blob() -> String {
    [
        read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH),
        read_runtime_src("tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_maps/row_data_rows.rs"),
    ]
    .join("\n")
}

pub(super) fn review_guard_date_map_source_blob() -> String {
    [
        read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH),
        read_runtime_src("tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/typed_error_maps/row_data_rows.rs"),
    ]
    .join("\n")
}

pub(super) fn structure_guard_rows_source_blob() -> String {
    [
        read_runtime_src(STRUCTURE_GUARD_ROWS_PATH),
        structure_guard_root_and_children_source_blob(),
        read_runtime_src(STRUCTURE_GUARD_STATUS_DOCS_PATH),
        read_runtime_src(STRUCTURE_GUARD_FOLDER_BACKED_SUMMARY_PATH),
        read_runtime_src(STRUCTURE_GUARD_TYPED_ERROR_PATH),
        read_runtime_src(STRUCTURE_GUARD_ROW_DATA_OWNER_PATH),
    ]
    .concat()
}

pub(super) fn review_guard_rows_source_blob() -> String {
    [
        read_runtime_src(REVIEW_GUARD_ROWS_PATH),
        read_runtime_src("tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/review_guard_rows/core_rows.rs"),
        read_runtime_src("tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/review_guard_rows/p0_rows.rs"),
        read_runtime_src("tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/review_guard_rows/f8_rows.rs"),
        read_runtime_src("tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/review_guard_rows/late_api_rows.rs"),
        read_runtime_src(REVIEW_GUARD_ROW_DATA_OWNER_PATH),
    ]
    .concat()
}

pub(super) fn structure_guard_root_and_children_source_blob() -> String {
    [
        read_runtime_src(STRUCTURE_GUARD_ROOT_AND_CHILDREN_PATH),
        read_runtime_src("tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/code_review_findings.rs"),
        read_runtime_src("tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/p0_robustness.rs"),
        read_runtime_src("tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/plugin_importer_dx.rs"),
        read_runtime_src("tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/p0_native_fixture.rs"),
        read_runtime_src("tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/f8_child_owner.rs"),
        read_runtime_src("tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/late_api_cleanup.rs"),
    ]
    .concat()
}
