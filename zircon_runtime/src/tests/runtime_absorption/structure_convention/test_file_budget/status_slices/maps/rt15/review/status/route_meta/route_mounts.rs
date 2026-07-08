use super::*;

#[test]
fn runtime_15_status_support_expected_slice_guard_route_metadata_is_child_owned() {
    let parent = read_runtime_src(STRUCTURE_REVIEW_STATUS_SUPPORT_EXPECTED_SLICE_GUARD);
    let children =
        read_runtime_sources(STRUCTURE_REVIEW_STATUS_SUPPORT_EXPECTED_SLICE_GUARD_CHILDREN);
    let path_route =
        read_runtime_src(STRUCTURE_REVIEW_STATUS_SUPPORT_EXPECTED_SLICE_GUARD_CHILDREN[0]);
    let path_children = read_runtime_sources(STATUS_SUPPORT_EXPECTED_SLICE_PATH_CHILDREN);
    let route_children = read_runtime_absorption_sources(ROUTE_METADATA_CHILDREN);

    assert_contains_all(
        "status-support expected-slice parent mounts child-owned route metadata",
        &parent,
        &[
            "#[path = \"status/paths.rs\"]",
            "mod paths;",
            "#[path = \"status/sources.rs\"]",
            "mod sources;",
            "#[path = \"status/guard_body.rs\"]",
            "mod guard_body;",
            "#[path = \"status/route_metadata.rs\"]",
            "mod route_metadata;",
            "use paths::*;",
            "use sources::*;",
        ],
    );
    assert_contains_all(
        "status-support expected-slice paths route owner",
        &path_route,
        &[
            "#[path = \"paths/child_group_routes.rs\"]",
            "#[path = \"paths/expected_slice_rows.rs\"]",
            "#[path = \"paths/plan_doc_routes.rs\"]",
            "#[path = \"paths/priority_plan_doc_routes.rs\"]",
            "#[path = \"paths/review_guard_routes.rs\"]",
            "#[path = \"paths/row_data_routes.rs\"]",
            "#[path = \"paths/runtime_index_anchor_routes.rs\"]",
            "STATUS_SUPPORT_EXPECTED_SLICE_PATH_CHILDREN",
        ],
    );

    for moved_anchor in [
        "const STATUS_SUPPORT_ROW_DATA_ROUTE_CHILDREN",
        "const STATUS_SUPPORT_EXPECTED_SLICE_ROWS",
        "fn read_runtime_sources",
        "fn read_status_support_expected_slice_rows",
        "#[test]",
        "runtime_15_status_support_expected_slice_maps_are_child_owned",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "status_support_expected_slice.rs should delegate moved route metadata {moved_anchor}"
        );
    }

    assert_contains_all(
        "status-support expected-slice route metadata children own moved declarations",
        &format!("{children}\n{path_children}\n{route_children}"),
        &[
            "pub(super) const STATUS_SUPPORT_ROW_DATA_ROUTE_CHILDREN",
            "pub(super) const STATUS_SUPPORT_EXPECTED_SLICE_ROWS",
            "pub(super) fn read_runtime_sources",
            "pub(super) fn read_status_support_expected_slice_rows",
            "runtime_15_status_support_expected_slice_maps_are_child_owned",
            GUARD_ROUTE_METADATA_GUARD,
            ROUTE_METADATA_GUARD,
        ],
    );
}
