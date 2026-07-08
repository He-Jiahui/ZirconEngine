use super::super::super::super::super::*;
use super::super::*;

pub(super) fn assert_typed_error_status_mirror_child_budgets_are_current() {
    let parent = read_runtime_src(TYPED_ERROR_STATUS_DOCS_CHILD);
    for (path, source) in [(TYPED_ERROR_STATUS_DOCS_CHILD, parent)]
        .into_iter()
        .chain(typed_error_status_docs_child_sources())
    {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
