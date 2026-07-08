use super::*;

#[test]
fn runtime_15_status_support_review_guard_row_data_route_guard_stays_within_budget() {
    for (path, source) in [
        (
            STATUS_SUPPORT_REVIEW_GUARD_ROW_DATA_CHILD,
            read_runtime_src(STATUS_SUPPORT_REVIEW_GUARD_ROW_DATA_CHILD),
        ),
        (
            DATE_SUPPORT_REVIEW_GUARD_ROW_DATA_CHILD,
            read_runtime_src(DATE_SUPPORT_REVIEW_GUARD_ROW_DATA_CHILD),
        ),
        (
            STRUCTURE_REVIEW_STATUS_SUPPORT_REVIEW_GUARD_ROW_DATA_ROUTE_CHILD,
            include_str!("../review_guard_row_data_route_children.rs").to_string(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 190,
            "{path} should stay below the Runtime 15 review-guard row-data route budget; got {line_count} lines"
        );
    }

    let status_paths = status_support_review_guard_row_data_child_paths();
    let date_paths = date_support_review_guard_row_data_child_paths();
    for path in status_paths.iter().chain(date_paths.iter()) {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < 80,
            "{path} should stay below the Runtime 15 review-guard row-data child budget; got {line_count} lines"
        );
    }

    for path in STRUCTURE_REVIEW_STATUS_SUPPORT_REVIEW_GUARD_ROW_DATA_ROUTE_GUARD_CHILDREN {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < 140,
            "{path} should stay below the Runtime 15 review-guard row-data route guard child budget; got {line_count} lines"
        );
    }
}
