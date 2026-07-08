use super::*;

#[test]
fn runtime_15_row_data_status_support_rows_are_child_owned() {
    let runtime_15_m3_status_support = read_runtime_src(RUNTIME_15_ROW_DATA_STATUS_ROWS_PATH);

    assert_contains_all(
        "Runtime 15 M3 status support rows keep historical Runtime 15 row-data split",
        &runtime_15_m3_status_support,
        &[
            ROW_DATA_SPLIT_STATUS_NAME,
            ROW_DATA_SPLIT_STATUS_ID,
            "plan_status/status_output_tables/expected_status_row_data.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
            ROW_DATA_SPLIT_GUARD_NAME,
        ],
    );
}
