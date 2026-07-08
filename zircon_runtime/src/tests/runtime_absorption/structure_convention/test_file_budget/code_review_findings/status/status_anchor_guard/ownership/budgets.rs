use super::*;

pub(super) fn assert_status_anchor_line_budgets(parent: &str, child: &str) {
    for (path, source) in [
        (STATUS_DOC_PARENT_PATH, parent),
        (STATUS_DOC_STATUS_ANCHORS_OWNER, child),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 400,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
    for (path, source) in status_anchors::status_doc_status_anchor_child_sources() {
        let line_count = source.lines().count();
        assert!(
            line_count < 400,
            "{path} should stay below the focused status-anchor child budget; got {line_count} lines"
        );
    }
}
