use super::*;

pub(super) fn typed_error_children_source() -> String {
    super::super::super::source_inventory::typed_error_children_source()
}

pub(super) fn moved_guard_absence_child_sources() -> Vec<(&'static str, String)> {
    TYPED_ERROR_MOVED_GUARD_ABSENCE_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn moved_guard_absence_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in moved_guard_absence_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}

pub(super) fn review_guard_status_rows_source() -> String {
    super::super::typed_error_structure_assertion_status_row_source()
}

pub(super) fn review_guard_status_map_source() -> String {
    super::super::typed_error_structure_assertion_status_map_source()
}

pub(super) fn review_guard_date_map_source() -> String {
    super::super::typed_error_structure_assertion_date_map_source()
}
