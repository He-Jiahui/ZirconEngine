use super::*;

pub(super) fn status_docs_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in STATUS_DOCS_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}

pub(super) fn status_row_docs_guard_source() -> String {
    format!(
        "{}\n{}",
        read_runtime_src(STATUS_ROW_DOCS_GUARD_PATH),
        read_runtime_src(STATUS_ROW_DOCS_ROW_SOURCES_PATH)
    )
}
