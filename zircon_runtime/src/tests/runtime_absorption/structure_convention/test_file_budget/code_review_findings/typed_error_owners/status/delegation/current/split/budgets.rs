use super::super::super::super::super::super::super::*;
use super::super::super::super::{
    TYPED_ERROR_CHILD_OWNER_LINE_BUDGET, TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_CHILD,
    TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_CHILD,
};

pub(super) fn assert_typed_error_status_doc_delegation_status_current_children_line_budgets(
    parent: &str,
) {
    assert_line_budget(
        TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_CHILD,
        parent,
        "delegation status-current",
    );
    for (path, source) in
        super::sources::typed_error_status_doc_delegation_status_current_child_sources()
    {
        assert_line_budget(path, &source, "delegation status-current");
    }
}

pub(super) fn assert_typed_error_status_doc_delegation_status_current_split_layout_children_line_budgets(
    parent: &str,
) {
    assert_line_budget(
        TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_CHILD,
        parent,
        "delegation status-current split-layout",
    );
    for (path, source) in
        super::sources::typed_error_status_doc_delegation_status_current_split_layout_child_sources(
        )
    {
        assert_line_budget(path, &source, "delegation status-current split-layout");
    }
}

fn assert_line_budget(path: &str, source: &str, label: &str) {
    let line_count = source.lines().count();
    assert!(
        line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
        "{path} should stay below the Runtime 15 status-doc {label} budget; got {line_count} lines"
    );
}
