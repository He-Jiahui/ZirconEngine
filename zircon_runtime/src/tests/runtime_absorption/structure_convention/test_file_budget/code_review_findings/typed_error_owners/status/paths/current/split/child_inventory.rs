use super::super::super::super::super::super::super::*;
use super::super::super::child_inventory::TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_CHILDREN;
use super::super::super::root_paths::{
    typed_error_status_doc_root_paths_source_blob,
    TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_CHILD,
};

pub(super) fn assert_typed_error_status_doc_paths_status_current_direct_child_inventory() {
    let child_tree = format!(
        "{}\n{}\n{}",
        super::super::sources::typed_error_status_doc_paths_status_current_child_source_blob(),
        super::sources::typed_error_status_doc_paths_status_current_split_layout_child_source_blob(
        ),
        typed_error_status_doc_root_paths_source_blob()
    );

    for (_, child_path, anchor) in TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_CHILDREN {
        assert!(
            child_tree.contains(child_path),
            "typed-error status-doc paths status-current child tree should inventory {child_path}"
        );
        assert!(
            child_tree.contains(anchor),
            "typed-error status-doc paths status-current child {child_path} should own anchor {anchor}"
        );
    }
}

pub(super) fn assert_typed_error_status_doc_paths_status_current_split_layout_child_inventory() {
    let child_tree = format!(
        "{}\n{}",
        super::sources::typed_error_status_doc_paths_status_current_split_layout_child_source_blob(
        ),
        typed_error_status_doc_root_paths_source_blob()
    );

    for (module_name, child_path, anchor) in
        super::sources::TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_CHILDREN
    {
        let path_attr = format!("#[path = \"split/{module_name}.rs\"]");
        assert!(
            child_tree.contains(child_path),
            "typed-error status-doc paths status-current split-layout tree should include {child_path}"
        );
        assert!(
            child_tree.contains(anchor),
            "typed-error status-doc paths status-current split-layout child {child_path} should own {anchor}"
        );
        assert!(
            read_runtime_src(TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_SPLIT_LAYOUT_CHILD)
                .contains(&path_attr),
            "typed-error status-doc paths status-current split-layout parent should mount {module_name}"
        );
    }
}
