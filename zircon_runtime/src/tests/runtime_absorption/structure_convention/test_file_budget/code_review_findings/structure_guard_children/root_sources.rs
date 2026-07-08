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
    blob.push_str(&budgets::structure_guard_children_budget_child_source_blob());
    blob.push_str(&folder_backed_summary::folder_backed_summary_structure_child_source_blob());
    blob.push_str(&plugin_importer::plugin_importer_structure_guard_child_source_blob());
    blob.push_str(&status_docs::code_review_status_doc_child_tree_source());
    blob.push_str(&typed_error::typed_error_structure_guard_child_source_blob());
    blob
}

pub(super) fn structure_guard_status_row_source() -> String {
    format!(
        "{}\n{}",
        read_runtime_src(STRUCTURE_GUARD_ROW_PARENT),
        read_runtime_src(STRUCTURE_GUARD_ROWS),
    )
}

pub(super) fn structure_guard_status_map_source() -> String {
    format!(
        "{}\n{}",
        read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH),
        read_runtime_src(REVIEW_GUARD_STRUCTURE_GUARD_STATUS_MAP_PATH),
    )
}

pub(super) fn structure_guard_date_map_source() -> String {
    format!(
        "{}\n{}",
        read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH),
        read_runtime_src(REVIEW_GUARD_STRUCTURE_GUARD_DATE_MAP_PATH),
    )
}
