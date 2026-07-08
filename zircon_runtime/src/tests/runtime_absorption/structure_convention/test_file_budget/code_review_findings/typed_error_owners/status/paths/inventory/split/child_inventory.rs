use super::super::super::super::super::super::super::*;
use super::super::super::root_paths::{
    typed_error_status_doc_root_paths_source_blob,
    TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_CHILD,
    TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_CHILD,
};

pub(super) fn assert_typed_error_status_doc_paths_child_inventory_direct_child_inventory() {
    let parent = read_runtime_src(TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_CHILD);
    let child_tree = format!(
        "{}\n{}",
        super::sources::typed_error_status_doc_paths_child_inventory_child_source_blob(),
        typed_error_status_doc_root_paths_source_blob()
    );

    for (module_name, child_path, anchor) in
        super::sources::TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_CHILDREN
    {
        let path_attr = format!("#[path = \"inventory/{module_name}.rs\"]");
        assert!(
            parent.contains(&path_attr),
            "typed-error status-doc paths child-inventory parent should mount {module_name}"
        );
        assert!(
            child_tree.contains(child_path),
            "typed-error status-doc paths child-inventory tree should include {child_path}"
        );
        assert!(
            child_tree.contains(anchor),
            "typed-error status-doc paths child-inventory child {child_path} should own {anchor}"
        );
    }
}

pub(super) fn assert_typed_error_status_doc_paths_child_inventory_split_layout_child_inventory() {
    let child_tree = format!(
        "{}\n{}",
        super::sources::typed_error_status_doc_paths_child_inventory_split_layout_child_source_blob(
        ),
        typed_error_status_doc_root_paths_source_blob()
    );

    for (module_name, child_path, anchor) in
        super::sources::TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_CHILDREN
    {
        let path_attr = format!("#[path = \"split/{module_name}.rs\"]");
        assert!(
            child_tree.contains(child_path),
            "typed-error status-doc paths child-inventory split-layout tree should include {child_path}"
        );
        assert!(
            child_tree.contains(anchor),
            "typed-error status-doc paths child-inventory split-layout child {child_path} should own {anchor}"
        );
        assert!(
            read_runtime_src(TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_CHILD)
                .contains(&path_attr),
            "typed-error status-doc paths child-inventory split-layout parent should mount {module_name}"
        );
    }
}
