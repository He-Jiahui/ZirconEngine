use super::*;

#[test]
fn runtime_15_m3_child_group_status_doc_maps_are_current() {
    let status_support_expected_status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let status_support_expected_date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);

    assert_contains_all(
        "Runtime 15 status-support map owns M3 child-group status-doc split",
        &status_support_expected_status_map,
        &[
            ROW_DATA_STATUS_NAME,
            ROW_DATA_STATUS_ID,
            HISTORICAL_STATUS_NAME,
            HISTORICAL_STATUS_ID,
            FOLDER_BACKED_STATUS_NAME,
            FOLDER_BACKED_STATUS_ID,
        ],
    );
    assert_contains_all(
        "Runtime 15 status-support date map owns M3 child-group status-doc split date",
        &status_support_expected_date_map,
        &[
            ROW_DATA_STATUS_NAME,
            HISTORICAL_STATUS_NAME,
            FOLDER_BACKED_STATUS_NAME,
            "2026-06-30",
            "2026-07-03",
        ],
    );
}
