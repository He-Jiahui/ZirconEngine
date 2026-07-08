use super::*;

#[test]
fn runtime_15_status_support_expected_slice_guard_body_status_mirrors_children_stay_budgeted() {
    let route_path = format!("tests/runtime_absorption/{GUARD_BODY_STATUS_MIRRORS_ROUTE_PATH}");
    let checks = [
        (route_path.as_str(), 25usize),
        (GUARD_BODY_STATUS_MIRROR_CHILDREN[0], 45usize),
        (GUARD_BODY_STATUS_MIRROR_CHILDREN[1], 60usize),
        (GUARD_BODY_STATUS_MIRROR_CHILDREN[2], 85usize),
        (GUARD_BODY_STATUS_MIRROR_CHILDREN[3], 55usize),
        (GUARD_BODY_STATUS_MIRROR_CHILDREN[4], 95usize),
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
            "{path} should stay below the status-support guard body status-mirror budget of {budget} lines; got {line_count}"
        );
    }
}
