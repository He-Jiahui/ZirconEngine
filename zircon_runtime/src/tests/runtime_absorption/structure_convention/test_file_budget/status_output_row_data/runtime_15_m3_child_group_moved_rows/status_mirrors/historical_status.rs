use super::*;

#[test]
fn runtime_15_m3_child_group_moved_row_historical_status_is_current() {
    let status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);

    assert_contains_all(
        "Runtime 15 status-support map owns M3 child-group moved-row child-owner split",
        &status_map,
        &[CHILD_OWNER_STATUS_NAME, CHILD_OWNER_STATUS_ID],
    );
    assert_contains_all(
        "Runtime 15 status-support date map owns M3 child-group moved-row child-owner split",
        &date_map,
        &[CHILD_OWNER_STATUS_NAME, "2026-06-30"],
    );
}
