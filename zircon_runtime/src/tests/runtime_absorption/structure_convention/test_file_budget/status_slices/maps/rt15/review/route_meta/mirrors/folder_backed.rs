use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_root_route_metadata_status_mirrors_are_folder_backed() {
    let parent = read_runtime_src(STRUCTURE_REVIEW_ROUTE_METADATA_CHILDREN[2]);
    let children = read_review_root_sources(STRUCTURE_REVIEW_ROUTE_METADATA_STATUS_MIRROR_CHILDREN);

    assert_contains_all(
        "review guard root route-metadata status-mirror parent",
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
        "let status_rows =",
        "read_repo(",
        "ROOT_ROUTE_METADATA_ROUTE_MOUNTS_FRAMEWORKS_STATUS",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "review/route_meta/status_mirrors.rs should delegate moved status mirror {moved_anchor}"
        );
    }
    assert_contains_all(
        "review guard root route-metadata status-mirror children",
        &children,
        &[
            ROOT_ROUTE_METADATA_STATUS_MIRROR_GUARD_GUARD,
            "runtime_15_review_guard_expected_slice_root_route_metadata_status_is_mirrored",
            "runtime_15_review_guard_expected_slice_root_route_metadata_docs_are_mirrored",
        ],
    );
}
