use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_sources_stay_within_budget() {
    for (path, source) in [
        (STATUS_PARENT, read_runtime_src(STATUS_PARENT)),
        (DATE_PARENT, read_runtime_src(DATE_PARENT)),
        (
            STATUS_STRUCTURE_ROUTE_MAP,
            read_runtime_src(STATUS_STRUCTURE_ROUTE_MAP),
        ),
        (
            DATE_STRUCTURE_ROUTE_MAP,
            read_runtime_src(DATE_STRUCTURE_ROUTE_MAP),
        ),
        (STATUS_REVIEW_CHILD, read_runtime_src(STATUS_REVIEW_CHILD)),
        (
            STATUS_REVIEW_FOUNDATION_CHILD,
            read_runtime_src(STATUS_REVIEW_FOUNDATION_CHILD),
        ),
        (
            STATUS_REVIEW_TYPED_ERROR_CHILD,
            read_runtime_src(STATUS_REVIEW_TYPED_ERROR_CHILD),
        ),
        (
            STATUS_REVIEW_TOP_ROW_CHILD,
            read_runtime_src(STATUS_REVIEW_TOP_ROW_CHILD),
        ),
        (STATUS_NAMING_CHILD, read_runtime_src(STATUS_NAMING_CHILD)),
        (STATUS_SUPPORT_CHILD, read_runtime_src(STATUS_SUPPORT_CHILD)),
        (
            STATUS_SUPPORT_ROW_DATA_CHILD,
            read_runtime_src(STATUS_SUPPORT_ROW_DATA_CHILD),
        ),
        (
            STATUS_SUPPORT_PLAN_DOC_CHILD,
            read_runtime_src(STATUS_SUPPORT_PLAN_DOC_CHILD),
        ),
        (DATE_REVIEW_CHILD, read_runtime_src(DATE_REVIEW_CHILD)),
        (
            DATE_REVIEW_FOUNDATION_CHILD,
            read_runtime_src(DATE_REVIEW_FOUNDATION_CHILD),
        ),
        (
            DATE_REVIEW_TYPED_ERROR_CHILD,
            read_runtime_src(DATE_REVIEW_TYPED_ERROR_CHILD),
        ),
        (
            DATE_REVIEW_TOP_ROW_CHILD,
            read_runtime_src(DATE_REVIEW_TOP_ROW_CHILD),
        ),
        (DATE_NAMING_CHILD, read_runtime_src(DATE_NAMING_CHILD)),
        (DATE_SUPPORT_CHILD, read_runtime_src(DATE_SUPPORT_CHILD)),
        (
            DATE_SUPPORT_ROW_DATA_CHILD,
            read_runtime_src(DATE_SUPPORT_ROW_DATA_CHILD),
        ),
        (
            DATE_SUPPORT_PLAN_DOC_CHILD,
            read_runtime_src(DATE_SUPPORT_PLAN_DOC_CHILD),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 400,
            "{path} should stay below the Runtime 15 focused expected-slice budget; got {line_count} lines"
        );
    }

    for path in STATUS_STRUCTURE_ROUTE_MAP_CHILDREN
        .iter()
        .chain(DATE_STRUCTURE_ROUTE_MAP_CHILDREN.iter())
    {
        let line_count = read_runtime_src(path).lines().count();
        assert!(
            line_count < 90,
            "{path} should stay below the Runtime 15 structure route map child budget; got {line_count} lines"
        );
    }

    for path in STATUS_REVIEW_FOUNDATION_CHILDREN
        .iter()
        .chain(DATE_REVIEW_FOUNDATION_CHILDREN.iter())
        .chain(STATUS_REVIEW_TYPED_ERROR_CHILDREN.iter())
        .chain(DATE_REVIEW_TYPED_ERROR_CHILDREN.iter())
    {
        let line_count = read_runtime_src(path).lines().count();
        assert!(
            line_count < 110,
            "{path} should stay below the Runtime 15 focused review map child budget; got {line_count} lines"
        );
    }
}
