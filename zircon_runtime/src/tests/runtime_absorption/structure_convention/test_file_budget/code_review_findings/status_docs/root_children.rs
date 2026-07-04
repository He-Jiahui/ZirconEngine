use super::*;

pub(super) const STATUS_DOC_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "sync",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/sync.rs",
        "runtime_15_code_review_findings_status_docs_are_child_owner",
    ),
    (
        "source_anchor_guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/source_anchor_guard.rs",
        "runtime_15_code_review_findings_status_docs_source_anchors_are_child_owner",
    ),
    (
        "status_anchor_guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/status_anchor_guard.rs",
        "runtime_15_code_review_findings_status_docs_status_anchors_are_child_owner",
    ),
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/delegation.rs",
        "runtime_15_code_review_findings_status_docs_folder_backed_status_is_current",
    ),
    (
        "status_mirrors",
        STATUS_DOC_STATUS_MIRRORS_OWNER,
        STATUS_DOC_STATUS_MIRROR_CHILD_OWNER_GUARD,
    ),
];

pub(super) const STATUS_DOC_ROOT_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "root_paths",
        STATUS_DOC_ROOT_PATHS_CHILD,
        "STATUS_DOC_ROOT_PATHS_CHILD",
    ),
    (
        "root_statuses",
        STATUS_DOC_ROOT_STATUSES_CHILD,
        STATUS_DOC_ROOT_INVENTORY_STATUS,
    ),
    (
        "root_row_sources",
        STATUS_DOC_ROOT_ROW_SOURCES_CHILD,
        "review_guard_status_rows_source",
    ),
    (
        "root_children",
        STATUS_DOC_ROOT_CHILDREN_CHILD,
        "STATUS_DOC_ROOT_CHILDREN",
    ),
    (
        "root_inventory",
        STATUS_DOC_ROOT_INVENTORY_CHILD,
        STATUS_DOC_ROOT_INVENTORY_GUARD,
    ),
];

pub(super) fn status_doc_child_sources() -> Vec<(&'static str, String)> {
    STATUS_DOC_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn status_doc_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in status_doc_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob.push_str(&status_anchor_guard::status_anchor_guard_child_source_blob());
    blob
}
