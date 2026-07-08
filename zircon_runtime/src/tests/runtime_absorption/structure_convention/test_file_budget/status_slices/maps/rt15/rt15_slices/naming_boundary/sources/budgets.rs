use super::*;

#[test]
fn runtime_15_naming_boundary_expected_slice_sources_children_stay_budgeted() {
    for (path, limit) in [(SOURCES_PARENT_PATH, 40usize)] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count <= limit,
            "{path} should stay below the naming-boundary sources route budget {limit}; got {line_count} lines"
        );
    }

    for (path, limit) in [
        (SOURCES_CHILDREN[0], 35usize),
        (SOURCES_CHILDREN[1], 100),
        (SOURCES_CHILDREN[2], 85),
        (SOURCES_CHILDREN[3], 35),
        (SOURCES_CHILDREN[4], 45),
        (SOURCES_CHILDREN[5], 50),
        (SOURCES_CHILDREN[6], 95),
        (SOURCES_CHILDREN[7], 45),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count <= limit,
            "{path} should stay below the naming-boundary sources child budget {limit}; got {line_count} lines"
        );
    }
}
