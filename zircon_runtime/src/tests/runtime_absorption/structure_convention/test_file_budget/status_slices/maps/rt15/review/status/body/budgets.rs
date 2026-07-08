use super::*;

#[test]
fn runtime_15_status_support_expected_slice_guard_body_children_stay_budgeted() {
    let route_path = format!("tests/runtime_absorption/{GUARD_BODY_ROUTE_PATH}");
    let checks = [
        (route_path.as_str(), 25usize),
        (GUARD_BODY_CHILDREN[0], 45usize),
        (GUARD_BODY_CHILDREN[1], 75usize),
        (GUARD_BODY_CHILDREN[2], 85usize),
        (GUARD_BODY_CHILDREN[3], 70usize),
        (GUARD_BODY_CHILDREN[4], 55usize),
        (GUARD_BODY_CHILDREN[5], 25usize),
    ];

    for (path, budget) in checks {
        let source = if path == route_path.as_str() {
            read_runtime_src(path)
        } else {
            read_runtime_src(&format!("tests/runtime_absorption/{path}"))
        };
        let line_count = source.lines().count();
        assert!(
            line_count <= budget,
            "{path} should stay below the status-support guard body budget of {budget} lines; got {line_count}"
        );
    }
}
