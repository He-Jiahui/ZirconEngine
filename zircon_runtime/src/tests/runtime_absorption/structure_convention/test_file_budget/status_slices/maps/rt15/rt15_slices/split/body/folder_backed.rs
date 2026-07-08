use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_maps_guard_body_is_folder_backed() {
    let parent = read_runtime_15_map("split/guard_body.rs");
    let children = GUARD_BODY_CHILDREN
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n");
    let budget_children = GUARD_BODY_BUDGET_CHILDREN
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n");
    let route_mount_children = GUARD_BODY_ROUTE_MOUNTS_CHILDREN
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n");

    assert_contains_all(
        "Runtime 15 expected-slice maps guard body route",
        &parent,
        &[
            "#[path = \"body/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"body/doc_mirrors.rs\"]",
            "mod doc_mirrors;",
            "#[path = \"body/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"body/paths.rs\"]",
            "mod paths;",
            "#[path = \"body/route_mounts.rs\"]",
            "mod route_mounts;",
            "#[path = \"body/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "use paths::*;",
        ],
    );
    for moved_anchor in [
        "#[test]",
        "read_runtime_15_map_parent",
        "status_mirrors::assert_status_rows_and_docs_are_synced",
        GUARD,
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "rt15_slices/split/guard_body.rs should delegate moved body anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "Runtime 15 expected-slice maps guard body children",
        &format!("{children}\n{budget_children}\n{route_mount_children}"),
        &[
            GUARD,
            GUARD_BODY_GUARD,
            BUDGETS_GUARD,
            ROUTE_MOUNTS_GUARD,
            "runtime_15_status_output_runtime_15_expected_slice_maps_guard_body_children_stay_budgeted",
            "runtime_15_status_output_runtime_15_expected_slice_maps_guard_body_docs_are_synced",
            "runtime_15_status_output_runtime_15_expected_slice_maps_guard_body_status_mirrors_are_synced",
            "runtime_15_status_output_runtime_15_expected_slice_maps_guard_body_route_mounts_children_stay_budgeted",
        ],
    );
}
