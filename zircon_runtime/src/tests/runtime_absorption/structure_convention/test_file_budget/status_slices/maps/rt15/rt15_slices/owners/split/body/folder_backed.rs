use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_body_is_folder_backed() {
    let parent = read_child_owner("split/guard_body.rs");
    let guard_body_children = GUARD_BODY_CHILDREN
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n");
    let route_mount_children = GUARD_BODY_ROUTE_MOUNTS_CHILDREN
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n");
    let children = format!("{guard_body_children}\n{route_mount_children}");

    assert_contains_all(
        "Runtime 15 expected-slice child-owner guard body route",
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
        "read_child_owner_parent",
        "TOP_LEVEL_SUPPORT_ROWS_PATH",
        GUARD,
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "owners/split/guard_body.rs should delegate moved body anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "Runtime 15 expected-slice child-owner guard body children",
        &children,
        &[
            GUARD,
            GUARD_BODY_GUARD,
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_body_children_stay_budgeted",
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_body_docs_are_synced",
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_body_route_mounts_is_folder_backed",
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_body_status_mirrors_are_synced",
        ],
    );
}
