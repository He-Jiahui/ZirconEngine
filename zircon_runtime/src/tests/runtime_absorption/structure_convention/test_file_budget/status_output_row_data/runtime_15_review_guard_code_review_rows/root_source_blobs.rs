use super::*;

pub(super) fn code_review_rows_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in CODE_REVIEW_ROWS_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
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
