use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_route_metadata_route_mounts_folder_backed_guard_body_is_split(
) {
    let parent = read_runtime_src(&format!(
        "tests/runtime_absorption/{REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_FOLDER_BACKED_ROUTE_PATH}"
    ));
    let children =
        read_runtime_absorption_sources(REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_FOLDER_BACKED_CHILDREN);

    assert_contains_all(
        "review-route metadata route-mount folder-backed parent mounts focused child guards",
        &parent,
        &[
            "#[path = \"folder_backed/metadata_guard.rs\"]",
            "mod metadata_guard;",
            "#[path = \"folder_backed/route_mounts_guard.rs\"]",
            "mod route_mounts_guard;",
            "#[path = \"folder_backed/route_owner.rs\"]",
            "mod route_owner;",
            "#[path = \"folder_backed/status_docs.rs\"]",
            "mod status_docs;",
        ],
    );

    for moved_anchor in [
        "#[test]",
        "STRUCTURE_REVIEW_CHILD_ROUTE_PARENT",
        "STRUCTURE_REVIEW_CHILD_ROUTE_CHILDREN",
        "STRUCTURE_REVIEW_CHILD_ROUTE_METADATA_ROUTE",
        "read_status_support_expected_slice_rows",
        REVIEW_ROUTE_METADATA_GUARD,
        REVIEW_ROUTE_METADATA_GUARD_GUARD,
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "mounts/folder_backed.rs should delegate moved guard body {moved_anchor}"
        );
    }

    assert_contains_all(
        "review-route metadata route-mount folder-backed children preserve moved checks",
        &children,
        &[
            "runtime_15_review_guard_expected_slice_route_metadata_route_mounts_are_folder_backed",
            "runtime_15_review_guard_expected_slice_route_metadata_guard_is_folder_backed",
            "runtime_15_review_guard_expected_slice_route_metadata_route_mounts_folder_backed_status_is_mirrored",
            REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_FOLDER_BACKED_GUARD,
        ],
    );
}
