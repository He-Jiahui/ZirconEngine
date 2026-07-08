use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_root_guard_body_child_ownership_sources_stay_budgeted() {
    let parent_line_count = read_runtime_src(&format!(
        "tests/runtime_absorption/{CHILD_OWNERSHIP_ROUTE_PATH}"
    ))
    .lines()
    .count();
    assert!(
        parent_line_count < 25,
        "{CHILD_OWNERSHIP_ROUTE_PATH} should stay below the child-ownership route budget 25; got {parent_line_count} lines"
    );

    for (path, limit) in CHILD_OWNERSHIP_CHILDREN
        .iter()
        .zip([45usize, 75, 80, 35, 80, 95])
    {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count < limit,
            "{path} should stay below the child-ownership child budget {limit}; got {line_count} lines"
        );
    }
}
