use super::*;

#[test]
fn runtime_15_review_guard_status_support_rows_guard_is_folder_backed() {
    let parent = read_runtime_src(STATUS_SUPPORT_ROWS_GUARD_PATH);
    let child_sources = status_support_rows_guard_child_source_blob();

    assert_contains_all(
        "review-guard status-support rows guard mounts focused children",
        &parent,
        &[
            "#[path = \"status_support_rows/anchor_mirror_cleanup.rs\"]",
            "mod anchor_mirror_cleanup;",
            "#[path = \"status_support_rows/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"status_support_rows/split_layout.rs\"]",
            "mod split_layout;",
        ],
    );
    for moved_anchor in [
        "fn runtime_15_review_guard_status_support_rows_are_folder_backed",
        "fn runtime_15_review_guard_status_support_parent_has_no_anchor_mirror",
        "review-guard status-support row-data parent mounts focused children",
        "status-support child rows retain representative historical anchors",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "status_support_rows.rs should stay a route owner and delegate {moved_anchor}"
        );
    }
    assert_contains_all(
        "review-guard status-support rows guard children retain moved tests",
        &child_sources,
        &[
            STATUS_SUPPORT_ROWS_FOLDER_BACKED_GUARD_NAME,
            STATUS_SUPPORT_ANCHOR_MIRROR_CLEANUP_GUARD_NAME,
            STATUS_SUPPORT_ROWS_GUARD_FOLDER_BACKED_GUARD_NAME,
        ],
    );
}
