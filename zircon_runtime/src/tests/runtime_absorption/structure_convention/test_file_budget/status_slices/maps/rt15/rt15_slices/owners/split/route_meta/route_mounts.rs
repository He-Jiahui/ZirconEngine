use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_route_metadata_is_child_owned(
) {
    let parent = read_child_owner("split_layout.rs");
    let route_metadata_children = read_route_metadata_children();
    let children = format!(
        "{}\n{}\n{}\n{}\n{}",
        read_child_owner("split/sources.rs"),
        read_child_owner("split/guard_body.rs"),
        read_child_owner("split/route_metadata.rs"),
        read_child_owner("split/status_mirrors.rs"),
        route_metadata_children
    );

    assert_contains_all(
        "child-owner split-layout parent mounts child-owned route metadata",
        &parent,
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
    for moved_anchor in [
        "const SLICE",
        "fn assert_status_docs_for_child_owner_split",
        "#[test]",
        "runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_is_folder_backed",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "owners/split_layout.rs should delegate moved route metadata {moved_anchor}"
        );
    }
    assert_contains_all(
        "child-owner split-layout children own moved declarations",
        &children,
        &[
            "const SLICE",
            "fn assert_status_docs_for_child_owner_split",
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_is_folder_backed",
            ROUTE_GUARD,
        ],
    );
}
