use super::*;

pub(super) fn priority_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in PRIORITY_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}

pub(super) fn production_guard_support_priority_rows_source_blob() -> String {
    let mut blob = read_runtime_src(PRODUCTION_GUARD_SUPPORT_ROWS_PATH);
    for path in [
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/runtime_row_data/status_support_priority_rows/expected_slice_rows.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/runtime_row_data/status_support_priority_rows/priority_plan_docs_rows.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/runtime_row_data/status_support_priority_rows/row_data_guard_rows.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/runtime_row_data/status_support_priority_rows/row_data_owner.rs",
    ] {
        blob.push('\n');
        blob.push_str(&read_runtime_src(path));
    }
    blob
}
