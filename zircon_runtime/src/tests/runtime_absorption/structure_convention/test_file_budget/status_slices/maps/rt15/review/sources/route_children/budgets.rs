use super::*;

#[test]
fn runtime_15_review_guard_root_source_route_children_children_stay_budgeted() {
    for (path, limit) in SOURCE_ROUTE_CHILDREN_CHILDREN
        .iter()
        .zip([45usize, 105, 80, 20, 25, 70, 60])
    {
        let source_path = format!("tests/runtime_absorption/{path}");
        let line_count = read_runtime_src(&source_path).lines().count();
        assert!(
            line_count < limit,
            "{path} should stay below the route-children child budget {limit}; got {line_count} lines"
        );
    }
}
