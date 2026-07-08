use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_sources_children_stay_budgeted(
) {
    for (path, limit) in [(SOURCES_PARENT_PATH, 30usize)] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count <= limit,
            "{path} should stay below the child-owner sources route budget {limit}; got {line_count} lines"
        );
    }

    for (path, limit) in [
        (SOURCES_CHILDREN[0], 45usize),
        (SOURCES_CHILDREN[1], 120),
        (SOURCES_CHILDREN[2], 75),
        (SOURCES_CHILDREN[3], 65),
        (SOURCES_CHILDREN[4], 95),
        (SOURCES_CHILDREN[5], 35),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count <= limit,
            "{path} should stay below the child-owner sources child budget {limit}; got {line_count} lines"
        );
    }
}
