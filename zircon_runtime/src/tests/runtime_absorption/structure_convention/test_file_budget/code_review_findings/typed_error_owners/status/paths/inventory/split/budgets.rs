use super::super::super::super::super::super::super::*;
use super::super::super::review_guard_paths::TYPED_ERROR_CHILD_OWNER_LINE_BUDGET;
use super::super::super::root_paths::{
    TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_CHILD,
    TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_CHILD,
};

pub(super) fn assert_typed_error_status_doc_paths_child_inventory_children_line_budgets(
    parent: &str,
) {
    assert_line_budget(
        TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_CHILD,
        parent,
        "child-inventory",
    );
    for (path, source) in
        super::sources::typed_error_status_doc_paths_child_inventory_child_sources()
    {
        assert_line_budget(path, &source, "child-inventory");
    }
}

pub(super) fn assert_typed_error_status_doc_paths_child_inventory_split_layout_children_line_budgets(
    parent: &str,
) {
    assert_line_budget(
        TYPED_ERROR_STATUS_DOCS_PATHS_CHILD_INVENTORY_SPLIT_LAYOUT_CHILD,
        parent,
        "child-inventory split-layout",
    );
    for (path, source) in
        super::sources::typed_error_status_doc_paths_child_inventory_split_layout_child_sources()
    {
        assert_line_budget(path, &source, "child-inventory split-layout");
    }
}

fn assert_line_budget(path: &str, source: &str, label: &str) {
    let line_count = source.lines().count();
    assert!(
        line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
        "{path} should stay below the Runtime 15 status-doc paths {label} budget; got {line_count} lines"
    );
}
