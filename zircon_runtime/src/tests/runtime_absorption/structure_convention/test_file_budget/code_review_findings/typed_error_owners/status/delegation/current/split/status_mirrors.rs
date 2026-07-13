use super::super::super::super::super::super::super::*;
use super::super::super::super::{
    typed_error_date_map_source, typed_error_status_map_source, typed_error_status_row_source,
    TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_CHILD,
    TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_GUARD,
    TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_OWNERSHIP_CHILD,
    TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SOURCES_CHILD,
    TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_DATE,
    TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_ID,
    TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_BUDGETS_CHILD,
    TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_CHILD,
    TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_CHILD_INVENTORY_CHILD,
    TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_DATE,
    TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_GUARD,
    TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_ID,
    TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_NAME,
    TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_PARENT_BACKFLOW_CHILD,
    TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_PARENT_MOUNTS_CHILD,
    TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_SOURCES_CHILD,
    TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_SPLIT_LAYOUT_CHILD,
    TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_STATUS_MIRRORS_CHILD,
    TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_NAME,
    TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_STATUS_SYNC_CHILD,
};

pub(super) fn assert_typed_error_status_doc_delegation_status_current_status_is_current() {
    assert_status_documents_contain(
        &[
            TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_NAME,
            TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_ID,
            TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_CHILD,
            TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_OWNERSHIP_CHILD,
            TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_STATUS_SYNC_CHILD,
            TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SOURCES_CHILD,
            TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_CHILD,
            TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_GUARD,
            "Cargo gate deferred",
        ],
        "M3 review status map records typed-error status-doc delegation status-current split",
        TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_NAME,
        TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_ID,
        "M3 review date map records typed-error status-doc delegation status-current split",
        TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_DATE,
    );
}

pub(super) fn assert_typed_error_status_doc_delegation_status_current_split_layout_guard_status_is_current(
) {
    assert_status_documents_contain(
        &[
            TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_NAME,
            TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_ID,
            TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_CHILD,
            TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_BUDGETS_CHILD,
            TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_CHILD_INVENTORY_CHILD,
            TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_PARENT_BACKFLOW_CHILD,
            TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_PARENT_MOUNTS_CHILD,
            TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_SOURCES_CHILD,
            TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_SPLIT_LAYOUT_CHILD,
            TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_STATUS_MIRRORS_CHILD,
            TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_GUARD,
            TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_GUARD,
            "Cargo gate deferred",
        ],
        "M3 review status map records typed-error status-doc delegation status-current split-layout guard split",
        TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_NAME,
        TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_ID,
        "M3 review date map records typed-error status-doc delegation status-current split-layout guard split",
        TYPED_ERROR_STATUS_DOCS_DELEGATION_STATUS_CURRENT_SPLIT_LAYOUT_DATE,
    );
}

fn assert_status_documents_contain(
    anchors: &[&str],
    status_label: &str,
    status_name: &str,
    status_id: &str,
    date_label: &str,
    date: &str,
) {
    let status_rows = typed_error_status_row_source();
    let status_map = typed_error_status_map_source();
    let date_map = typed_error_date_map_source();
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for (label, source) in [
        ("typed-error row data", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(label, source, anchors);
    }
    assert_contains_all(status_label, &status_map, &[status_name, status_id]);
    assert_contains_all(date_label, &date_map, &[status_name, date]);
}
