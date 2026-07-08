use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_maps_guard_body_budgets_are_folder_backed() {
    let parent = read_runtime_src(&format!("tests/runtime_absorption/{BUDGETS_ROUTE_PATH}"));
    let children = GUARD_BODY_BUDGET_CHILDREN
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n");

    assert_contains_all(
        "Runtime 15 expected-slice maps guard-body budget route",
        &parent,
        &[
            "#[path = \"budgets/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"budgets/guard_body_children.rs\"]",
            "mod guard_body_children;",
            "#[path = \"budgets/paths.rs\"]",
            "mod paths;",
            "#[path = \"budgets/route_mount_children.rs\"]",
            "mod route_mount_children;",
            "#[path = \"budgets/status_docs.rs\"]",
            "mod status_docs;",
            "#[path = \"budgets/top_level_maps.rs\"]",
            "mod top_level_maps;",
            "use paths::*;",
        ],
    );
    for moved_anchor in [
        "#[test]",
        "read_runtime_15_map_parent",
        "GUARD_BODY_ROUTE_MOUNTS_CHILDREN",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "split/body/budgets.rs should delegate moved body anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "Runtime 15 expected-slice maps guard-body budget children",
        &children,
        &[
            BUDGETS_GUARD,
            "runtime_15_status_output_runtime_15_expected_slice_maps_guard_body_children_stay_budgeted",
            "runtime_15_status_output_runtime_15_expected_slice_maps_guard_body_route_mounts_budgeted",
            "runtime_15_status_output_runtime_15_expected_slice_maps_guard_body_top_level_maps_stay_budgeted",
            "runtime_15_status_output_runtime_15_expected_slice_maps_guard_body_budgets_status_is_synced",
        ],
    );
}
