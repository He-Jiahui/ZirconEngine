use super::*;

#[test]
fn runtime_15_module_layout_status_mirror_children_are_child_owned() {
    let status_rows = read_runtime_src(PRODUCTION_GUARD_SUPPORT_ROWS_PATH);
    let status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);

    let status_anchors = [
        STATUS_MIRROR_CHILD_SPLIT_NAME,
        STATUS_MIRROR_CHILD_SPLIT_ID,
        "structure_convention/test_file_budget/status_output_row_data/module_layout/status_mirrors.rs",
        "structure_convention/test_file_budget/status_output_row_data/module_layout/status_mirrors/child_split_status.rs",
        "structure_convention/test_file_budget/status_output_row_data/module_layout/status_mirrors/layout_folder_status.rs",
        "structure_convention/test_file_budget/status_output_row_data/module_layout/status_mirrors/historical_status.rs",
        "structure_convention/test_file_budget/status_output_row_data/module_layout/status_mirrors/folder_backed_status.rs",
        "runtime_15_module_layout_status_mirror_children_are_child_owned",
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "production support row data records module-layout status-mirror child split",
        &status_rows,
        &status_anchors,
    );
    assert_contains_all(
        "Runtime 15 status-support expected status map records module-layout status-mirror child split",
        &status_map,
        &[STATUS_MIRROR_CHILD_SPLIT_NAME, STATUS_MIRROR_CHILD_SPLIT_ID],
    );
    assert_contains_all(
        "Runtime 15 status-support expected date map records module-layout status-mirror child split",
        &date_map,
        &[STATUS_MIRROR_CHILD_SPLIT_NAME, "2026-07-04"],
    );

    let doc_anchors = [
        STATUS_MIRROR_CHILD_SPLIT_NAME,
        STATUS_MIRROR_CHILD_SPLIT_ID,
        "runtime_15_module_layout_status_mirror_children_are_child_owned",
    ];
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
            "structure convention",
            "docs/plans/engine-code-structure-convention.md",
        ),
        (
            "review findings",
            "docs/plans/engine-code-review-findings-2026-06.md",
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
        assert_contains_all(label, &source, &doc_anchors);
    }
}
