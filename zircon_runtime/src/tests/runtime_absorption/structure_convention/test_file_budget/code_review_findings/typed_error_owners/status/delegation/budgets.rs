use super::super::super::super::super::*;
use super::super::*;

pub(super) fn assert_typed_error_status_doc_delegation_budgets_are_current() {
    let typed_error_parent = read_runtime_src(TYPED_ERROR_STRUCTURE_CHILD);
    let status_docs_parent = read_runtime_src(TYPED_ERROR_STATUS_DOCS_CHILD);

    for (path, source) in [
        (TYPED_ERROR_STRUCTURE_CHILD, typed_error_parent),
        (TYPED_ERROR_STATUS_DOCS_CHILD, status_docs_parent),
    ]
    .into_iter()
    .chain(typed_error_status_docs_child_sources())
    .chain(typed_error_status_doc_row_child_sources())
    {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
