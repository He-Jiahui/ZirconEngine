use super::*;

#[test]
fn runtime_15_review_guard_status_support_review_rows_guard_is_folder_backed() {
    let parent = read_runtime_src(STATUS_SUPPORT_REVIEW_ROWS_GUARD_PATH);
    let child_blob = status_support_review_rows_guard_child_source_blob();
    assert_contains_all(
        "status-support review rows guard parent mounts child guards",
        &parent,
        &[
            "#[path = \"status_support_review_guard_rows/route_children.rs\"]",
            "#[path = \"status_support_review_guard_rows/export_chain.rs\"]",
            "#[path = \"status_support_review_guard_rows/status_current.rs\"]",
            "#[path = \"status_support_review_guard_rows/split_layout.rs\"]",
            "runtime_15_review_guard_status_support_review_rows_row_data_owner_is_child_backed",
        ],
    );
    assert!(
        !parent.contains("let doc_anchors = ["),
        "status-support review rows guard parent should route checks instead of owning status-current assertions",
    );
    assert_contains_all(
        "status-support review rows guard children own moved assertions",
        &child_blob,
        &[
            "assert_status_support_review_rows_route_children_are_current",
            "assert_status_support_review_rows_exports_are_current",
            "assert_status_support_review_rows_row_data_status_is_current",
        ],
    );
    assert_status_support_review_rows_guard_status_is_current();
}

fn assert_status_support_review_rows_guard_status_is_current() {
    let status_row =
        read_runtime_src(REVIEW_GUARD_STATUS_SUPPORT_REVIEW_ROWS_STATUS_SUPPORT_GUARD_PATH);
    assert_contains_all(
        "status-support review rows guard status row",
        &status_row,
        &[
            STATUS_SUPPORT_REVIEW_ROWS_GUARD_FOLDER_BACKED_STATUS_NAME,
            STATUS_SUPPORT_REVIEW_ROWS_GUARD_FOLDER_BACKED_STATUS_ID,
            "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/status_support_review_guard_rows.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/status_support_review_guard_rows/route_children.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/status_support_review_guard_rows/export_chain.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/status_support_review_guard_rows/status_current.rs",
            STATUS_SUPPORT_REVIEW_ROWS_GUARD_FOLDER_BACKED_GUARD_NAME,
            "Cargo gate deferred",
        ],
    );
    let doc_anchors = [
        STATUS_SUPPORT_REVIEW_ROWS_GUARD_FOLDER_BACKED_STATUS_NAME,
        STATUS_SUPPORT_REVIEW_ROWS_GUARD_FOLDER_BACKED_STATUS_ID,
        STATUS_SUPPORT_REVIEW_ROWS_GUARD_PATH,
        STATUS_SUPPORT_REVIEW_ROWS_GUARD_FOLDER_BACKED_GUARD_NAME,
        "Cargo gate deferred",
    ];
    for path in [
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        "docs/plans/zircon_runtime/runtime/index.md",
        "docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md",
        "docs/plans/engine-code-review-findings-2026-06.md",
        "docs/plans/engine-code-structure-convention.md",
        "docs/zircon_runtime/structure/module-convention.md",
        ".codex/sessions/20260612-0847-runtime-architecture-implementation.md",
    ] {
        assert_contains_all(path, &read_repo(path), &doc_anchors);
    }
    assert_contains_all(
        "review guard status map records status-support review rows guard split",
        &read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH),
        &[
            STATUS_SUPPORT_REVIEW_ROWS_GUARD_FOLDER_BACKED_STATUS_NAME,
            STATUS_SUPPORT_REVIEW_ROWS_GUARD_FOLDER_BACKED_STATUS_ID,
        ],
    );
    assert_contains_all(
        "review guard date map records status-support review rows guard split",
        &read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH),
        &[
            STATUS_SUPPORT_REVIEW_ROWS_GUARD_FOLDER_BACKED_STATUS_NAME,
            "2026-07-07",
        ],
    );
}
