use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_split_layout_routes_are_child_owned(
) {
    let split_layout = read_child_owner("split_layout.rs");
    let split_layout_children = [
        read_child_owner("split/sources.rs"),
        read_child_owner("split/guard_body.rs"),
        read_child_owner("split/route_metadata.rs"),
        read_child_owner("split/status_mirrors.rs"),
    ]
    .join("\n");
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
    let route_metadata_children = [
        read_child_owner("split/route_meta/budgets.rs"),
        read_child_owner("split/route_meta/doc_mirrors.rs"),
        read_child_owner("split/route_meta/folder_backed.rs"),
        read_child_owner("split/route_meta/paths.rs"),
        read_child_owner("split/route_meta/route_mounts.rs"),
        read_child_owner("split/route_meta/status_mirrors.rs"),
    ]
    .join("\n");

    assert_contains_all(
        "Runtime 15 expected-slice child-owner split-layout route",
        &split_layout,
        &[
            "#[path = \"split/sources.rs\"]",
            "mod sources;",
            "#[path = \"split/guard_body.rs\"]",
            "mod guard_body;",
            "#[path = \"split/route_metadata.rs\"]",
            "mod route_metadata;",
            "use sources::*;",
        ],
    );
    assert_contains_all(
        "Runtime 15 expected-slice child-owner split-layout children",
        &format!(
            "{split_layout_children}\n{guard_body_children}\n{route_mount_children}\n{route_metadata_children}"
        ),
        &[
            "fn runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_is_folder_backed",
            "fn runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_route_metadata_is_child_owned",
            "fn assert_status_docs_for_child_owner_split",
            GUARD_BODY_GUARD,
        ],
    );
}
