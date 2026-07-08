use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_route_metadata_route_mounts_are_folder_backed() {
    let parent = read_runtime_src(&format!(
        "tests/runtime_absorption/{REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_ROUTE_PATH}"
    ));
    let children = format!(
        "{}\n{}",
        read_runtime_absorption_sources(REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_CHILDREN),
        read_runtime_absorption_sources(REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_FOLDER_BACKED_CHILDREN)
    );

    assert_contains_all(
        "review-route metadata route-mount parent mounts focused child owners",
        &parent,
        &[
            "#[path = \"mounts/child_owned.rs\"]",
            "mod child_owned;",
            "#[path = \"mounts/folder_backed.rs\"]",
            "mod folder_backed;",
        ],
    );

    for moved_anchor in [
        "#[test]",
        "STRUCTURE_REVIEW_CHILD_ROUTE_PARENT",
        "STRUCTURE_REVIEW_CHILD_ROUTE_CHILDREN",
        "STRUCTURE_REVIEW_CHILD_ROUTE_METADATA_ROUTE",
        REVIEW_ROUTE_METADATA_GUARD,
        REVIEW_ROUTE_METADATA_GUARD_GUARD,
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "review_route_children/route_meta/route_mounts.rs should delegate moved route-mount guard {moved_anchor}"
        );
    }

    assert_contains_all(
        "review-route metadata route-mount children preserve moved guards",
        &children,
        &[
            REVIEW_ROUTE_METADATA_GUARD,
            REVIEW_ROUTE_METADATA_GUARD_GUARD,
            "runtime_15_review_guard_expected_slice_route_metadata_is_child_owned",
            "runtime_15_review_guard_expected_slice_route_metadata_guard_is_folder_backed",
        ],
    );
}
