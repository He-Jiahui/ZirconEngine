use super::*;

pub(super) fn review_guard_row_data_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in REVIEW_GUARD_ROW_DATA_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}

pub(super) fn moved_rows_child_source_blob() -> String {
    [
        read_runtime_src(MOVED_ROWS_DELEGATION_PATH),
        read_runtime_src(MOVED_ROWS_CODE_REVIEW_PATH),
        read_runtime_src(MOVED_ROWS_TYPED_ERROR_PATH),
        read_runtime_src(MOVED_ROWS_STATUS_MIRRORS_PATH),
    ]
    .join("\n")
}
