use super::*;

#[test]
fn runtime_15_status_output_runtime_15_m3_row_data_is_child_owner() {
    let parent = read_runtime_src(TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15 = read_runtime_src(RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15_m3 = read_runtime_src(RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15_m3_status_support = read_runtime_src(RUNTIME_15_M3_STATUS_SUPPORT_ROW_DATA_PATH);
    let expected_status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let expected_date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);

    assert_contains_all(
        "top-level status row data aggregation keeps Runtime 15 M3 group",
        &parent,
        &[
            "runtime_15::RUNTIME_15_FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M4_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_F12_RESOURCE_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 status row parent delegates M3 rows",
        &runtime_15,
        &[
            "#[path = \"runtime_15/m3.rs\"]",
            "mod m3;",
            "pub(super) const RUNTIME_15_M3_FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "#[path = \"runtime_15/m4.rs\"]",
            "mod m4;",
        ],
    );
    for moved_m3_row in [
        "Runtime 15 M3 graphics dead-code guard module split",
        "Runtime 15 M3 status output Runtime 15 row data split",
        "Runtime 15 M3 status output expected-slice maps split",
        ROW_DATA_SPLIT_STATUS_ID,
    ] {
        assert!(
            !runtime_15.contains(moved_m3_row),
            "expected_status_row_data/runtime_15.rs should delegate M3 row literals instead of keeping {moved_m3_row}"
        );
    }
    assert_contains_all(
        "Runtime 15 M3 status row parent mounts M3 child groups",
        &runtime_15_m3,
        &[
            "#[path = \"m3/foundation_guards.rs\"]",
            "mod foundation_guards;",
            "#[path = \"m3/status_support.rs\"]",
            "mod status_support;",
            "pub(super) const STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 M3 status support child owns M3 row split literals",
        &runtime_15_m3_status_support,
        &[
            "pub(super) const EXPECTED_STATUS_OUTPUT_SLICES",
            "Runtime 15 M3 status output Runtime 15 row data split",
            "Runtime 15 M3 status output Runtime 15 M4 row data split",
            "Runtime 15 M3 status output expected-slice maps split",
            ROW_DATA_SPLIT_STATUS_NAME,
            ROW_DATA_SPLIT_STATUS_ID,
            ROW_DATA_SPLIT_GUARD_NAME,
        ],
    );

    assert_contains_all(
        "Runtime 15 expected status map records M3 row split",
        &expected_status_map,
        &[ROW_DATA_SPLIT_STATUS_NAME, ROW_DATA_SPLIT_STATUS_ID],
    );
    assert_contains_all(
        "Runtime 15 expected date map records M3 row split",
        &expected_date_map,
        &[ROW_DATA_SPLIT_STATUS_NAME, "2026-06-23"],
    );
}
