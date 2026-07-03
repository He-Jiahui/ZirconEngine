use super::*;

#[test]
fn runtime_15_m3_child_groups_representative_rows_are_child_owned() {
    let runtime_15_m3 = read_runtime_src(RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH);
    let row_owner_blob = [
        FOUNDATION_GUARDS_ROW_DATA_PATH,
        LOCK_POISON_STATUS_ROW_DATA_PATH,
        MODULE_CONVENTION_STATUS_ROW_DATA_PATH,
        REVIEW_STATUS_SYNC_ROW_DATA_PATH,
        STATUS_SUPPORT_ROW_DATA_PATH,
        UI_TESTS_SECOND_ROW_DATA_PATH,
        PRODUCTION_GUARD_SUPPORT_ROWS_PATH,
    ]
    .iter()
    .map(|path| read_runtime_src(path))
    .collect::<Vec<_>>()
    .join("\n");

    for moved_row in [
        "Runtime 15 M3 graphics dead-code guard module split",
        "Runtime 15 M3 UI runtime input ownership test folder split",
        "Runtime 15 M3 status output Runtime 15 M3 row data split",
        "Runtime 15 M3 production file budget guard child-owner split",
    ] {
        assert!(
            !runtime_15_m3.contains(moved_row),
            "expected_status_row_data/runtime_15/m3.rs should delegate row literals instead of keeping {moved_row}"
        );
    }
    assert_contains_all(
        "Runtime 15 M3 child row owners keep representative row literals",
        &row_owner_blob,
        &[
            "Runtime 15 M3 graphics dead-code guard module split",
            "Runtime 15 M3 production direct lock unwrap global gate",
            "Runtime 15 M3 lock-poison status row-data child-owner split",
            "Runtime 15 M3 module convention gate output contract",
            "Runtime 15 M3 module-convention status row-data child-owner split",
            "Runtime 15 M3 status output Runtime 15 M3 row data split",
            "Runtime 15 M3 UI runtime input ownership test folder split",
            "Runtime 15 M3 F19 scene renderer construction top-row closed status sync",
            HISTORICAL_CHILD_OWNER_STATUS_NAME,
            "Runtime 15 M3 review top-row status row-data child-owner split",
            "runtime_15_lock_poison_status_row_data_child_owner_split_static_passed_cargo_deferred",
            "runtime_15_module_convention_status_row_data_child_owner_split_static_passed_cargo_deferred",
            HISTORICAL_CHILD_OWNER_STATUS_ID,
            "runtime_15_review_top_row_status_row_data_child_owner_split_static_passed_cargo_deferred",
        ],
    );
}
