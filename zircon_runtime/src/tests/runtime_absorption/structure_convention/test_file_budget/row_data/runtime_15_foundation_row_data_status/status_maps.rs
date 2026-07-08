use super::*;

#[test]
fn runtime_15_foundation_row_data_status_doc_maps_are_child_owned() {
    let expected_status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let expected_date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);
    let production_guard_status_rows = format!(
        "{}\n{}",
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/runtime_row_data/foundation_rows.rs",
        ),
        read_runtime_src(PRODUCTION_GUARD_SUPPORT_STATUS_DOCS_PATH),
    );

    assert_contains_all(
        "Runtime 15 expected status map records foundation row-data status-doc splits",
        &expected_status_map,
        &[
            FOUNDATION_ROW_DATA_SPLIT_NAME,
            FOUNDATION_ROW_DATA_SPLIT_ID,
            FOUNDATION_TOPIC_SPLIT_NAME,
            FOUNDATION_TOPIC_SPLIT_ID,
            FOUNDATION_GUARD_SPLIT_NAME,
            FOUNDATION_GUARD_SPLIT_ID,
            STATUS_DOC_SPLIT_NAME,
            STATUS_DOC_SPLIT_ID,
            ROW_COUNT_SYNC_NAME,
            ROW_COUNT_SYNC_ID,
            FOLDER_BACKED_STATUS_NAME,
            FOLDER_BACKED_STATUS_ID,
        ],
    );
    assert_contains_all(
        "Runtime 15 expected date map records foundation row-data status-doc splits",
        &expected_date_map,
        &[
            FOUNDATION_ROW_DATA_SPLIT_NAME,
            FOUNDATION_TOPIC_SPLIT_NAME,
            FOUNDATION_GUARD_SPLIT_NAME,
            STATUS_DOC_SPLIT_NAME,
            ROW_COUNT_SYNC_NAME,
            FOLDER_BACKED_STATUS_NAME,
            "2026-06-30",
            "2026-07-01",
            "2026-07-02",
        ],
    );
    assert_contains_all(
        "Runtime 15 production-support row data records foundation status-doc split",
        production_guard_status_rows.as_str(),
        &[
            FOUNDATION_GUARD_SPLIT_NAME,
            FOUNDATION_GUARD_SPLIT_ID,
            STATUS_DOC_SPLIT_NAME,
            STATUS_DOC_SPLIT_ID,
            FOLDER_BACKED_STATUS_NAME,
            FOLDER_BACKED_STATUS_ID,
            "structure_convention/test_file_budget/row_data/runtime_15_foundation_row_data.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_foundation_row_data_status_docs.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_foundation_row_data_status/delegation.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_foundation_row_data_status/status_maps.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_foundation_row_data_status/doc_mirrors.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_foundation_row_data_status/row_count.rs",
            "runtime_15_status_output_foundation_row_data_status_docs_are_child_owner",
            "Cargo gate deferred",
        ],
    );
}
