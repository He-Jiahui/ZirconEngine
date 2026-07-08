use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_sources_stay_budgeted() {
    for (path, source) in [
        (STATUS_REVIEW_CHILD, read_runtime_src(STATUS_REVIEW_CHILD)),
        (DATE_REVIEW_CHILD, read_runtime_src(DATE_REVIEW_CHILD)),
        (
            STATUS_REVIEW_FOUNDATION_CHILD,
            read_runtime_src(STATUS_REVIEW_FOUNDATION_CHILD),
        ),
        (
            STATUS_REVIEW_CODE_REVIEW_CHILD,
            read_runtime_src(STATUS_REVIEW_CODE_REVIEW_CHILD),
        ),
        (
            STATUS_REVIEW_TYPED_ERROR_STRUCTURE_CHILD,
            read_runtime_src(STATUS_REVIEW_TYPED_ERROR_STRUCTURE_CHILD),
        ),
        (
            STATUS_REVIEW_PLUGIN_IMPORTER_CHILD,
            read_runtime_src(STATUS_REVIEW_PLUGIN_IMPORTER_CHILD),
        ),
        (
            STATUS_REVIEW_TOP_ROW_CHILD,
            read_runtime_src(STATUS_REVIEW_TOP_ROW_CHILD),
        ),
        (
            DATE_REVIEW_FOUNDATION_CHILD,
            read_runtime_src(DATE_REVIEW_FOUNDATION_CHILD),
        ),
        (
            DATE_REVIEW_CODE_REVIEW_CHILD,
            read_runtime_src(DATE_REVIEW_CODE_REVIEW_CHILD),
        ),
        (
            DATE_REVIEW_TYPED_ERROR_STRUCTURE_CHILD,
            read_runtime_src(DATE_REVIEW_TYPED_ERROR_STRUCTURE_CHILD),
        ),
        (
            DATE_REVIEW_PLUGIN_IMPORTER_CHILD,
            read_runtime_src(DATE_REVIEW_PLUGIN_IMPORTER_CHILD),
        ),
        (
            DATE_REVIEW_TOP_ROW_CHILD,
            read_runtime_src(DATE_REVIEW_TOP_ROW_CHILD),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 180,
            "{path} should stay below the Runtime 15 review expected-slice map budget; got {line_count} lines"
        );
    }

    for path in STATUS_REVIEW_FOUNDATION_CHILDREN
        .iter()
        .chain(DATE_REVIEW_FOUNDATION_CHILDREN.iter())
    {
        let line_count = read_runtime_src(path).lines().count();
        assert!(
            line_count < 90,
            "{path} should stay below the Runtime 15 review foundation child map budget; got {line_count} lines"
        );
    }
}
