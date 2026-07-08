use super::*;

pub(super) fn direct_assertion_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in DIRECT_ASSERTION_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}

pub(super) fn direct_assertion_row_data_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in DIRECT_ASSERTION_ROW_DATA_CHILD_ROWS {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
