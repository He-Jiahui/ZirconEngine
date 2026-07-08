use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_route_metadata_is_child_owned() {
    let parent = read_runtime_src(STRUCTURE_REVIEW_CHILD_ROUTE_PARENT);
    let children = read_runtime_sources(STRUCTURE_REVIEW_CHILD_ROUTE_CHILDREN);
    let source_children = read_runtime_absorption_sources(REVIEW_ROUTE_CHILD_SOURCE_CHILDREN);
    let route_metadata_source_children =
        read_runtime_absorption_sources(REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_CHILDREN);
    let guard_body_children = read_runtime_absorption_sources(REVIEW_ROUTE_GUARD_BODY_CHILDREN);

    assert_contains_all(
        "review-route parent mounts child-owned route metadata",
        &parent,
        &[
            "#[path = \"review_route_children/sources.rs\"]",
            "mod sources;",
            "#[path = \"review_route_children/guard_body.rs\"]",
            "mod guard_body;",
            "#[path = \"review_route_children/route_metadata.rs\"]",
            "mod route_metadata;",
            "use sources::*;",
        ],
    );

    for moved_anchor in [
        "STATUS_SUPPORT_EXPECTED_SLICE_ROWS",
        "fn read_status_support_expected_slice_rows",
        "#[test]",
        "runtime_15_review_guard_expected_slice_maps_are_folder_backed",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "review_route_children.rs should delegate moved route metadata {moved_anchor}"
        );
    }

    assert_contains_all(
        "review-route children own moved declarations",
        &format!("{children}\n{source_children}\n{route_metadata_source_children}\n{guard_body_children}"),
        &[
            "const STATUS_SUPPORT_EXPECTED_SLICE_ROWS",
            "fn read_status_support_expected_slice_rows",
            "runtime_15_review_guard_expected_slice_maps_are_folder_backed",
            REVIEW_ROUTE_METADATA_GUARD,
            REVIEW_ROUTE_GUARD_BODY_GUARD,
        ],
    );
}
