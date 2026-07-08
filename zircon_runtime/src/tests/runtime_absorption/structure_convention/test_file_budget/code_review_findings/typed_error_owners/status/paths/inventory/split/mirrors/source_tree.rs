use super::super::super::super::super::super::super::super::*;
use super::super::super::super::root_paths::TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_STATUS_MIRRORS_CHILD;
use super::status_current::typed_error_status_doc_paths_child_inventory_split_layout_status_mirrors_status_current_child_source_blob;

const TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_STATUS_MIRRORS_STATUS_CURRENT_CHILD:
    &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/inventory/split/mirrors/status_current.rs";

const TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_STATUS_MIRROR_CHILDREN:
    &[&str] = &[
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/inventory/split/mirrors/source_tree.rs",
    TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_STATUS_MIRRORS_STATUS_CURRENT_CHILD,
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/inventory/split/mirrors/status_documents.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/inventory/split/mirrors/status_maps.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/paths/inventory/split/mirrors/folder_backed.rs",
];

pub(in super::super) fn typed_error_status_doc_paths_child_inventory_split_layout_status_mirrors_child_source_blob(
) -> String {
    let mut blob = read_runtime_src(
        TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_STATUS_MIRRORS_CHILD,
    );
    blob.push('\n');
    for &child_path in
        typed_error_status_doc_paths_child_inventory_split_layout_status_mirrors_child_paths()
    {
        if child_path
            == TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_STATUS_MIRRORS_STATUS_CURRENT_CHILD
        {
            blob.push_str(
                &typed_error_status_doc_paths_child_inventory_split_layout_status_mirrors_status_current_child_source_blob(),
            );
        } else {
            blob.push_str(&read_runtime_src(child_path));
        }
        blob.push('\n');
    }
    blob
}

pub(super) fn typed_error_status_doc_paths_child_inventory_split_layout_status_mirrors_child_paths(
) -> &'static [&'static str] {
    TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_STATUS_MIRROR_CHILDREN
}
