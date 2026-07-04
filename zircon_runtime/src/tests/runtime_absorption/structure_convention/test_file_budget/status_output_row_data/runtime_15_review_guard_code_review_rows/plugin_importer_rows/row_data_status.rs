use super::*;

pub(super) fn assert_plugin_importer_row_data_owner_status_row_is_current() {
    let row_data_owner = read_runtime_src(PLUGIN_IMPORTER_ROW_DATA_OWNER_PATH);

    assert_contains_all(
        "plugin-importer row-data owner child split row is current",
        &row_data_owner,
        &[
            PLUGIN_IMPORTER_ROWS_ROW_DATA_STATUS_ID,
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows/review_guards.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows/status_docs.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows/source_inventory.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows/structure_assertions.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows/row_data_owner.rs",
            PLUGIN_IMPORTER_ROWS_ROW_DATA_GUARD_NAME,
            "Cargo gate deferred",
        ],
    );
}
