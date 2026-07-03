use super::*;

#[test]
fn runtime_15_status_output_runtime_15_m2_row_data_is_child_owner() {
    let runtime_15_row_data_guard = read_runtime_src(RUNTIME_15_ROW_DATA_GUARD_PATH);
    let parent = read_runtime_src(TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15 = read_runtime_src(RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15_foundation =
        read_runtime_src(RUNTIME_15_FOUNDATION_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15_m2 = read_runtime_src(RUNTIME_15_M2_EXPECTED_STATUS_ROW_DATA_PATH);

    assert!(
        !runtime_15_row_data_guard.contains(CHILD_OWNER_GUARD_NAME),
        "runtime_15_row_data.rs should delegate the M2 row-data guard to runtime_15_m2_row_data.rs"
    );
    assert_contains_all(
        "M2 row-data guard records the historical child-owner split",
        &read_runtime_src(M2_ROW_DATA_GUARD_PATH),
        &[
            CHILD_OWNER_STATUS_NAME,
            CHILD_OWNER_STATUS_ID,
            CHILD_OWNER_GUARD_NAME,
        ],
    );

    assert_contains_all(
        "top-level status rows include Runtime 15 M2 row-data group",
        &parent,
        &["runtime_15::RUNTIME_15_M2_EXPECTED_STATUS_OUTPUT_SLICES"],
    );
    assert_contains_all(
        "Runtime 15 root delegates M2 rows",
        &runtime_15,
        &[
            "#[path = \"runtime_15/m2.rs\"]",
            "mod m2;",
            "pub(super) const RUNTIME_15_M2_EXPECTED_STATUS_OUTPUT_SLICES",
            "m2::EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    for moved_row in [
        "Runtime 15 M2 core runtime state module naming hard cutover",
        "Runtime 15 M2 editor Workbench archived fixture naming hard cutover",
    ] {
        assert!(
            !runtime_15.contains(moved_row),
            "expected_status_row_data/runtime_15.rs should delegate M2 row literal {moved_row}"
        );
        assert!(
            !runtime_15_foundation.contains(moved_row),
            "expected_status_row_data/runtime_15/foundation.rs should not keep M2 row literal {moved_row}"
        );
    }
    assert_contains_all(
        "Runtime 15 M2 child owns M2 rows",
        &runtime_15_m2,
        &[
            "pub(super) const EXPECTED_STATUS_OUTPUT_SLICES",
            ROW_DATA_SPLIT_STATUS_NAME,
            "Runtime 15 M2 core runtime state module naming hard cutover",
            "Runtime 15 M2 editor Workbench archived fixture naming hard cutover",
        ],
    );
}
