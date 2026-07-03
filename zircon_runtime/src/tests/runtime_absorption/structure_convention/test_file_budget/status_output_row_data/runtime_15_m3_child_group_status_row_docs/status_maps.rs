use super::*;

#[test]
fn runtime_15_m3_child_group_status_row_doc_maps_are_current() {
    let expected_status_map = read_runtime_src(M3_STRUCTURE_SUPPORT_STATUS_MAP_PATH);
    let expected_date_map = read_runtime_src(M3_STRUCTURE_SUPPORT_DATE_MAP_PATH);
    let review_expected_status_map = read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH);
    let review_expected_date_map = read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH);
    let status_support_expected_status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let status_support_expected_date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);

    assert_contains_all(
        "Runtime 15 parent status map keeps M3 foundation child-owner splits",
        &expected_status_map,
        &[
            "Runtime 15 M3 lock-poison status row-data child-owner split",
            "runtime_15_lock_poison_status_row_data_child_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 module-convention status row-data child-owner split",
            "runtime_15_module_convention_status_row_data_child_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 status-support map owns M3 child-group row-doc split",
        &status_support_expected_status_map,
        &[CHILD_OWNER_STATUS_NAME, CHILD_OWNER_STATUS_ID],
    );
    assert_contains_all(
        "Runtime 15 review map owns M3 top-row status child-owner split",
        &review_expected_status_map,
        &[
            "Runtime 15 M3 review top-row status row-data child-owner split",
            "runtime_15_review_top_row_status_row_data_child_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 parent date map keeps M3 foundation child-owner split dates",
        &expected_date_map,
        &[
            "Runtime 15 M3 lock-poison status row-data child-owner split",
            "Runtime 15 M3 module-convention status row-data child-owner split",
            "2026-06-28",
        ],
    );
    assert_contains_all(
        "Runtime 15 status-support date map owns M3 child-group row-doc split date",
        &status_support_expected_date_map,
        &[CHILD_OWNER_STATUS_NAME, "2026-06-30"],
    );
    assert_contains_all(
        "Runtime 15 review date map owns M3 top-row status child-owner split date",
        &review_expected_date_map,
        &[
            "Runtime 15 M3 review top-row status row-data child-owner split",
            "2026-06-28",
        ],
    );
    assert_contains_all(
        "Runtime 15 status-support map owns M3 child-group row-doc folder-backed split",
        &status_support_expected_status_map,
        &[FOLDER_BACKED_STATUS_NAME, FOLDER_BACKED_STATUS_ID],
    );
    assert_contains_all(
        "Runtime 15 status-support date map owns M3 child-group row-doc folder-backed split date",
        &status_support_expected_date_map,
        &[FOLDER_BACKED_STATUS_NAME, "2026-07-02"],
    );
}
