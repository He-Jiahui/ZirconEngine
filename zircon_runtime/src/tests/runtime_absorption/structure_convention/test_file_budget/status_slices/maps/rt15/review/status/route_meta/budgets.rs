use super::*;

fn assert_runtime_line_budget(path: &str, limit: usize, label: &str) {
    let line_count = read_runtime_src(path).lines().count();
    assert!(
        line_count < limit,
        "{path} should stay below the {label} budget {limit}; got {line_count} lines"
    );
}

fn assert_absorption_line_budget(path: &str, limit: usize, label: &str) {
    assert_runtime_line_budget(&format!("tests/runtime_absorption/{path}"), limit, label);
}

#[test]
fn runtime_15_status_support_expected_slice_route_metadata_children_stay_budgeted() {
    let route = STRUCTURE_REVIEW_STATUS_SUPPORT_EXPECTED_SLICE_GUARD;
    let route_children = STRUCTURE_REVIEW_STATUS_SUPPORT_EXPECTED_SLICE_GUARD_CHILDREN;
    for (path, limit) in [
        (route, 35usize),
        (route_children[0], 80),
        (route_children[1], 35),
        (route_children[2], 130),
        (route_children[3], 20),
    ] {
        assert_runtime_line_budget(path, limit, "status-support route");
    }
    for path in STATUS_SUPPORT_EXPECTED_SLICE_PATH_CHILDREN {
        assert_runtime_line_budget(path, 35, "status-support paths child");
    }
    for (path, limit) in [
        (ROUTE_METADATA_CHILDREN[0], 55usize),
        (ROUTE_METADATA_CHILDREN[1], 65),
        (ROUTE_METADATA_CHILDREN[2], 55),
        (ROUTE_METADATA_CHILDREN[3], 75),
        (ROUTE_METADATA_CHILDREN[4], 20),
    ] {
        assert_absorption_line_budget(path, limit, "status-support route metadata child");
    }
    for (path, limit) in [
        (ROUTE_METADATA_STATUS_MIRRORS_CHILDREN[0], 70usize),
        (ROUTE_METADATA_STATUS_MIRRORS_CHILDREN[1], 75),
        (ROUTE_METADATA_STATUS_MIRRORS_CHILDREN[2], 100),
    ] {
        assert_absorption_line_budget(path, limit, "status-support route metadata status mirror");
    }
}
