use super::*;

pub(super) fn structure_guard_child_sources() -> Vec<(&'static str, String)> {
    STRUCTURE_GUARD_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn structure_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in structure_guard_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}

pub(super) fn structure_guard_status_row_source() -> String {
    format!(
        "{}\n{}",
        read_runtime_src(STRUCTURE_GUARD_ROW_PARENT),
        read_runtime_src(STRUCTURE_GUARD_ROWS),
    )
}
