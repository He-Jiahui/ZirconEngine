use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_route_metadata_status_mirrors_are_folder_backed() {
    let parent = read_runtime_src(&format!(
        "tests/runtime_absorption/{REVIEW_ROUTE_METADATA_STATUS_MIRRORS_ROUTE_PATH}"
    ));
    let children = read_runtime_absorption_sources(REVIEW_ROUTE_METADATA_STATUS_MIRRORS_CHILDREN);

    assert_contains_all(
        "review-route metadata status mirror parent",
        &parent,
        &[
            "#[path = \"mirrors/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"mirrors/row_maps.rs\"]",
            "mod row_maps;",
            "#[path = \"mirrors/status_docs.rs\"]",
            "mod status_docs;",
        ],
    );
    for moved_anchor in [
        "#[test]",
        "read_status_support_expected_slice_rows",
        "read_status_review_foundation_sources",
        "read_date_review_foundation_sources",
        "read_repo(\"docs/",
        "runtime_15_review_guard_expected_slice_route_metadata_status_is_mirrored",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "review-route metadata status_mirrors.rs should delegate moved status mirror {moved_anchor}"
        );
    }
    assert_contains_all(
        "review-route metadata status mirror children",
        &children,
        &[
            "runtime_15_review_guard_expected_slice_route_metadata_row_maps_are_mirrored",
            "runtime_15_review_guard_expected_slice_route_metadata_status_is_mirrored",
            REVIEW_ROUTE_METADATA_STATUS_MIRRORS_GUARD,
            "REVIEW_ROUTE_METADATA_STATUS_MIRRORS_STATUS",
            "REVIEW_ROUTE_METADATA_STATUS_MIRRORS_FRAMEWORKS_STATUS",
        ],
    );
}
