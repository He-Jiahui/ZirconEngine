use super::*;

#[test]
fn runtime_15_review_guard_status_support_anchor_mirror_cleanup_status_is_current() {
    let review_guard_rows = review_guard_status_support_review_rows_source_blob();
    let status_map = read_runtime_src(REVIEW_GUARD_TYPED_ERROR_STATUS_MAP_PATH);
    let date_map = read_runtime_src(REVIEW_GUARD_TYPED_ERROR_DATE_MAP_PATH);

    let status_anchors = [
        STATUS_SUPPORT_ANCHOR_MIRROR_CLEANUP_STATUS_NAME,
        STATUS_SUPPORT_ANCHOR_MIRROR_CLEANUP_STATUS_ID,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows/review_guard_rows.rs",
        STATUS_SUPPORT_ANCHOR_MIRROR_CLEANUP_GUARD_NAME,
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "review-guard status-support rows record anchor mirror cleanup",
        &review_guard_rows,
        &status_anchors,
    );
    assert_contains_all(
        "review status map records status-support anchor mirror cleanup",
        &status_map,
        &[
            STATUS_SUPPORT_ANCHOR_MIRROR_CLEANUP_STATUS_NAME,
            STATUS_SUPPORT_ANCHOR_MIRROR_CLEANUP_STATUS_ID,
        ],
    );
    assert_contains_all(
        "review date map records status-support anchor mirror cleanup",
        &date_map,
        &[
            STATUS_SUPPORT_ANCHOR_MIRROR_CLEANUP_STATUS_NAME,
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
        (
            "runtime implementation session",
            ".codex/sessions/20260612-0847-runtime-architecture-implementation.md",
        ),
    ] {
        let source = read_repo(path);
        assert_contains_all(label, &source, &status_anchors);
    }
}
