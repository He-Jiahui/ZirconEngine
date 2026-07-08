use super::*;

#[test]
fn runtime_15_review_guard_foundation_status_mirrors_are_folder_backed() {
    let parent = read_runtime_src(STRUCTURE_REVIEW_FOUNDATION_MAPS_CHILDREN[1]);
    let children = read_review_root_sources(STRUCTURE_REVIEW_FOUNDATION_STATUS_MIRROR_CHILDREN);

    assert_contains_all(
        "review foundation status-mirror parent",
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
        "read_repo(",
        "let docs_required",
        "let status_rows",
        "REVIEW_FOUNDATION_ROUTE_MOUNT_GUARD_FRAMEWORKS_STATUS",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "foundation_review_maps/status_mirrors.rs should delegate moved status mirror {moved_anchor}"
        );
    }
    assert_contains_all(
        "review foundation status-mirror children",
        &children,
        &[
            REVIEW_FOUNDATION_STATUS_MIRROR_GUARD_GUARD,
            "runtime_15_review_guard_foundation_status_date_maps_status_is_mirrored",
            "runtime_15_review_guard_foundation_status_date_maps_docs_are_mirrored",
        ],
    );
}
