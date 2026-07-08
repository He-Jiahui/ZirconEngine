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
    blob
}
