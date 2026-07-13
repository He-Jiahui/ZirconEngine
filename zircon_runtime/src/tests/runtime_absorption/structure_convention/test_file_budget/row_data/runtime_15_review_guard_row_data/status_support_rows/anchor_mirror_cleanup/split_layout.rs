use super::*;

#[test]
fn runtime_15_review_guard_status_support_anchor_mirror_cleanup_guard_is_folder_backed() {
    let parent = read_runtime_src(STATUS_SUPPORT_ROWS_ANCHOR_MIRROR_CLEANUP_CHILD_PATH);
    let row_cleanup =
        read_runtime_src(STATUS_SUPPORT_ROWS_ANCHOR_MIRROR_CLEANUP_ROW_CLEANUP_CHILD_PATH);
    let status_current =
        read_runtime_src(STATUS_SUPPORT_ROWS_ANCHOR_MIRROR_CLEANUP_STATUS_CURRENT_CHILD_PATH);
    let split_layout =
        read_runtime_src(STATUS_SUPPORT_ROWS_ANCHOR_MIRROR_CLEANUP_SPLIT_LAYOUT_CHILD_PATH);
    let status_rows = review_guard_status_support_review_rows_source_blob();
    let status_map = read_runtime_src(REVIEW_GUARD_TYPED_ERROR_STATUS_MAP_PATH);
    let date_map = read_runtime_src(REVIEW_GUARD_TYPED_ERROR_DATE_MAP_PATH);

    assert_contains_all(
        "review-guard status-support anchor-mirror cleanup mounts focused children",
        &parent,
        &[
            "#[path = \"anchor_mirror_cleanup/row_cleanup.rs\"]",
            "mod row_cleanup;",
            "#[path = \"anchor_mirror_cleanup/split_layout.rs\"]",
            "mod split_layout;",
            "#[path = \"anchor_mirror_cleanup/status_current.rs\"]",
            "mod status_current;",
        ],
    );
    for moved_anchor in [
        "fn runtime_15_review_guard_status_support_parent_has_no_anchor_mirror",
        "status-support child rows retain representative historical anchors",
        "review-guard status-support rows record anchor mirror cleanup",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "anchor_mirror_cleanup.rs should stay a route owner and delegate {moved_anchor}"
        );
    }
    assert_contains_all(
        "review-guard status-support anchor-mirror cleanup children retain moved checks",
        &format!("{row_cleanup}\n{status_current}\n{split_layout}"),
        &[
            STATUS_SUPPORT_ANCHOR_MIRROR_CLEANUP_GUARD_NAME,
            "runtime_15_review_guard_status_support_anchor_mirror_cleanup_status_is_current",
            STATUS_SUPPORT_ANCHOR_MIRROR_CLEANUP_GUARD_FOLDER_BACKED_GUARD_NAME,
        ],
    );

    let status_anchors = [
        STATUS_SUPPORT_ANCHOR_MIRROR_CLEANUP_GUARD_FOLDER_BACKED_STATUS_NAME,
        STATUS_SUPPORT_ANCHOR_MIRROR_CLEANUP_GUARD_FOLDER_BACKED_STATUS_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/status_support_rows/anchor_mirror_cleanup.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/status_support_rows/anchor_mirror_cleanup/row_cleanup.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/status_support_rows/anchor_mirror_cleanup/status_current.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/status_support_rows/anchor_mirror_cleanup/split_layout.rs",
        STATUS_SUPPORT_ANCHOR_MIRROR_CLEANUP_GUARD_FOLDER_BACKED_GUARD_NAME,
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "review-guard status-support rows record anchor mirror cleanup guard folder-backed split",
        &status_rows,
        &status_anchors,
    );
    assert_contains_all(
        "review status map records anchor mirror cleanup guard folder-backed split",
        &status_map,
        &[
            STATUS_SUPPORT_ANCHOR_MIRROR_CLEANUP_GUARD_FOLDER_BACKED_STATUS_NAME,
            STATUS_SUPPORT_ANCHOR_MIRROR_CLEANUP_GUARD_FOLDER_BACKED_STATUS_ID,
        ],
    );
    assert_contains_all(
        "review date map records anchor mirror cleanup guard folder-backed split",
        &date_map,
        &[
            STATUS_SUPPORT_ANCHOR_MIRROR_CLEANUP_GUARD_FOLDER_BACKED_STATUS_NAME,
            "2026-07-06",
        ],
    );

    for (label, path) in [
        (
            "Runtime 15 plan",
            "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        ),
        (
            "Runtime index",
            "docs/plans/zircon_runtime/runtime/index.md",
        ),
        (
            "Frameworks 02 plan",
            "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
        ),
        (
            "review findings",
            "docs/plans/engine-code-review-findings-2026-06.md",
        ),
        (
            "structure convention",
            "docs/plans/engine-code-structure-convention.md",
        ),
        (
            "module convention doc",
            "docs/zircon_runtime/structure/module-convention.md",
        ),
    ] {
        let source = read_repo(path);
        assert_contains_all(label, &source, &status_anchors);
    }
}
