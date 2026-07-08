use super::*;

#[test]
fn runtime_15_review_guard_source_status_map_children_stay_budgeted() {
    for (path, limit) in SOURCE_STATUS_MAPS_CHILDREN
        .iter()
        .zip([30usize, 30, 85, 45, 125, 20, 65, 35, 95, 45, 45, 85])
    {
        let source_path = format!("tests/runtime_absorption/{path}");
        let line_count = read_runtime_src(&source_path).lines().count();
        assert!(
            line_count < limit,
            "{path} should stay below the status-map child budget {limit}; got {line_count} lines"
        );
    }
}
