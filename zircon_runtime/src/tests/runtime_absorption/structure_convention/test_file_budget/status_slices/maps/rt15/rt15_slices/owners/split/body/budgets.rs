use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_body_children_stay_budgeted(
) {
    for (path, source, max_lines) in [
        (PARENT_PATH, read_child_owner_parent(), 20usize),
        (ROUTE_MOUNTS_PATH, read_child_owner("route_mounts.rs"), 80),
        (
            LITERAL_OWNERSHIP_PATH,
            read_child_owner("literal_ownership.rs"),
            130,
        ),
        (BUDGETS_PATH, read_child_owner("budgets.rs"), 25),
        (
            STATUS_MIRRORS_PATH,
            read_child_owner("status_mirrors.rs"),
            70,
        ),
        (SPLIT_LAYOUT_PATH, read_child_owner("split_layout.rs"), 20),
        (
            SPLIT_LAYOUT_SOURCES_PATH,
            read_child_owner("split/sources.rs"),
            110,
        ),
        (
            SPLIT_LAYOUT_GUARD_BODY_PATH,
            read_child_owner("split/guard_body.rs"),
            25,
        ),
        (
            SPLIT_LAYOUT_ROUTE_METADATA_PATH,
            read_child_owner("split/route_metadata.rs"),
            150,
        ),
        (
            SPLIT_LAYOUT_STATUS_MIRRORS_PATH,
            read_child_owner("split/status_mirrors.rs"),
            100,
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= max_lines,
            "{path} has {line_count} lines, expected <= {max_lines}"
        );
    }

    for (path, limit) in [
        (GUARD_BODY_CHILDREN[0], 70usize),
        (GUARD_BODY_CHILDREN[1], 90),
        (GUARD_BODY_CHILDREN[2], 65),
        (GUARD_BODY_CHILDREN[3], 40),
        (GUARD_BODY_CHILDREN[4], 25),
        (GUARD_BODY_CHILDREN[5], 80),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count <= limit,
            "{path} should stay below the child-owner guard-body budget {limit}; got {line_count} lines"
        );
    }
}
