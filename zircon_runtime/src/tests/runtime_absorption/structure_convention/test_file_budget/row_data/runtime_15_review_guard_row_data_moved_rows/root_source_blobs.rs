use super::*;

pub(super) fn moved_row_child_sources() -> Vec<(&'static str, String)> {
    MOVED_ROWS_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn moved_row_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in moved_row_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob.push_str(&read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/child_ownership.rs",
    ));
    blob.push('\n');
    blob.push_str(&super::code_review_rows::code_review_rows_child_source_blob());
    blob.push('\n');
    for path in [
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/mirrors/child_split_status.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/mirrors/moved_row_status.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/mirrors/folder_backed_status.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/mirrors/scope_budgets.rs",
    ] {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
