use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_typed_error_route_metadata_is_child_owned() {
    let parent = read_runtime_src(STRUCTURE_TYPED_ERROR_EXPECTED_SLICE_GUARD);
    let children = read_runtime_sources(STRUCTURE_TYPED_ERROR_EXPECTED_SLICE_GUARD_CHILDREN);
    let route_metadata_children = read_route_metadata_children();

    assert_contains_all(
        "typed-error expected-slice parent mounts child-owned route metadata",
        &parent,
        &[
            "#[path = \"typed_error/sources.rs\"]",
            "mod sources;",
            "#[path = \"typed_error/guard_body.rs\"]",
            "mod guard_body;",
            "#[path = \"typed_error/map_rows.rs\"]",
            "mod map_rows;",
            "#[path = \"typed_error/route_metadata.rs\"]",
            "mod route_metadata;",
            "use sources::*;",
        ],
    );

    for moved_anchor in [
        "STATUS_SUPPORT_EXPECTED_SLICE_ROWS",
        "fn read_status_support_expected_slice_rows",
        "#[test]",
        "runtime_15_review_guard_expected_slice_typed_error_maps_are_child_owned",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed_error_expected_slice.rs should delegate moved route metadata {moved_anchor}"
        );
    }

    assert_contains_all(
        "typed-error expected-slice children own moved declarations",
        &format!("{children}\n{route_metadata_children}"),
        &[
            "const STATUS_SUPPORT_EXPECTED_SLICE_ROWS",
            "pub(super) fn read_status_support_expected_slice_rows",
            "runtime_15_review_guard_typed_error_expected_slice_map_rows_are_folder_backed",
            "runtime_15_review_guard_expected_slice_typed_error_maps_are_child_owned",
            TYPED_ERROR_ROUTE_GUARD,
        ],
    );
}
