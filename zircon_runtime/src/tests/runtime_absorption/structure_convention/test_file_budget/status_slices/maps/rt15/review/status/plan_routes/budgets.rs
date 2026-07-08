use super::*;

#[test]
fn runtime_15_status_support_plan_doc_route_guard_stays_within_budget() {
    for (path, source) in [
        (
            STATUS_SUPPORT_PLAN_DOC_CHILD,
            read_runtime_src(STATUS_SUPPORT_PLAN_DOC_CHILD),
        ),
        (
            DATE_SUPPORT_PLAN_DOC_CHILD,
            read_runtime_src(DATE_SUPPORT_PLAN_DOC_CHILD),
        ),
        (
            STRUCTURE_REVIEW_STATUS_SUPPORT_PLAN_DOC_ROUTE_CHILD,
            include_str!("../plan_doc_route_children.rs").to_string(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 160,
            "{path} should stay below the Runtime 15 plan-doc route budget; got {line_count} lines"
        );
    }

    let status_paths = status_support_plan_doc_child_paths();
    let date_paths = date_support_plan_doc_child_paths();
    for path in status_paths.iter().chain(date_paths.iter()) {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < 120,
            "{path} should stay below the Runtime 15 plan-doc child budget; got {line_count} lines"
        );
    }

    for path in STRUCTURE_REVIEW_STATUS_SUPPORT_PLAN_DOC_ROUTE_GUARD_CHILDREN {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < 140,
            "{path} should stay below the Runtime 15 plan-doc route guard child budget; got {line_count} lines"
        );
    }
}
