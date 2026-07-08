use super::*;

#[test]
fn runtime_15_status_support_parent_route_guard_stays_within_budget() {
    for (path, source) in [
        (STATUS_SUPPORT_CHILD, read_runtime_src(STATUS_SUPPORT_CHILD)),
        (DATE_SUPPORT_CHILD, read_runtime_src(DATE_SUPPORT_CHILD)),
        (
            STRUCTURE_REVIEW_STATUS_SUPPORT_PARENT_ROUTE_CHILD,
            include_str!("../parent_route_children.rs").to_string(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 220,
            "{path} should stay below the Runtime 15 parent route budget; got {line_count} lines"
        );
    }

    for path in STATUS_SUPPORT_PARENT_ROUTE_CHILDREN
        .iter()
        .chain(DATE_SUPPORT_PARENT_ROUTE_CHILDREN.iter())
    {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < 120,
            "{path} should stay below the Runtime 15 parent-map child budget; got {line_count} lines"
        );
    }

    for path in STRUCTURE_REVIEW_STATUS_SUPPORT_PARENT_ROUTE_GUARD_CHILDREN {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < 140,
            "{path} should stay below the Runtime 15 parent-route guard child budget; got {line_count} lines"
        );
    }
}
