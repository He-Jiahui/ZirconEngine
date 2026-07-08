use super::*;

#[test]
fn runtime_15_review_guard_row_data_delegation_guard_status_is_current() {
    let status_rows = read_runtime_src(STATUS_SUPPORT_REVIEW_GUARD_ROWS_PATH);
    let status_map = read_runtime_src(STATUS_SUPPORT_BASE_CHILD_OWNER_STATUS_MAP_PATH);
    let date_map = read_runtime_src(STATUS_SUPPORT_BASE_CHILD_OWNER_DATE_MAP_PATH);
    let status_anchors = [
        DELEGATION_GUARD_FOLDER_BACKED_STATUS_NAME,
        DELEGATION_GUARD_FOLDER_BACKED_STATUS_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/delegation.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/delegation/route_mounts.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/delegation/status_inventory.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/delegation/child_inventory.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/delegation/split_layout.rs",
        DELEGATION_GUARD_FOLDER_BACKED_GUARD_NAME,
        "Cargo gate deferred",
    ];

    assert_contains_all(
        "production guard review rows record delegation guard folder-backed split",
        &status_rows,
        &status_anchors,
    );
    assert_contains_all(
        "M3 status-support status map records delegation guard folder-backed split",
        &status_map,
        &[
            DELEGATION_GUARD_FOLDER_BACKED_STATUS_NAME,
            DELEGATION_GUARD_FOLDER_BACKED_STATUS_ID,
        ],
    );
    assert_contains_all(
        "M3 status-support date map records delegation guard folder-backed split",
        &date_map,
        &[DELEGATION_GUARD_FOLDER_BACKED_STATUS_NAME, "2026-07-06"],
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
            "docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md",
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
