use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_root_guard_body_route_mounts_sources_stay_budgeted() {
    let parent_line_count = read_runtime_src(&format!(
        "tests/runtime_absorption/{ROUTE_MOUNTS_ROUTE_PATH}"
    ))
    .lines()
    .count();
    assert!(
        parent_line_count < 25,
        "{ROUTE_MOUNTS_ROUTE_PATH} should stay below the route-mount route budget 25; got {parent_line_count} lines"
    );

    for (path, limit) in ROUTE_MOUNTS_CHILDREN
        .iter()
        .zip([45usize, 75, 80, 70, 35, 95])
    {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count < limit,
            "{path} should stay below the route-mount child budget {limit}; got {line_count} lines"
        );
    }
}
