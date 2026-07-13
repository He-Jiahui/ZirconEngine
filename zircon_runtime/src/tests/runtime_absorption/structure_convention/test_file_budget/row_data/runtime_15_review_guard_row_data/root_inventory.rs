use super::*;

#[test]
fn runtime_15_review_guard_row_data_root_inventory_is_child_owned() {
    let parent = read_runtime_src(REVIEW_GUARD_ROW_DATA_GUARD_PATH);
    let status_rows = read_runtime_src(STATUS_SUPPORT_REVIEW_GUARD_ROWS_PATH);
    let status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);

    assert_contains_all(
        "review-guard row-data root mounts inventory children",
        &parent,
        &[
            "#[path = \"runtime_15_review_guard_row_data/root_paths.rs\"]",
            "#[path = \"runtime_15_review_guard_row_data/root_statuses.rs\"]",
            "#[path = \"runtime_15_review_guard_row_data/root_child_rows.rs\"]",
            "#[path = \"runtime_15_review_guard_row_data/root_source_blobs.rs\"]",
            "#[path = \"runtime_15_review_guard_row_data/root_inventory.rs\"]",
        ],
    );

    let status_anchors = [
        ROOT_INVENTORY_STATUS_NAME,
        ROOT_INVENTORY_STATUS_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/root_paths.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/root_statuses.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/root_child_rows.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/root_source_blobs.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/root_inventory.rs",
        ROOT_INVENTORY_GUARD_NAME,
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "production guard review rows record review-guard row-data root inventory split",
        &status_rows,
        &status_anchors,
    );
    assert_contains_all(
        "M3 status-support status map records review-guard row-data root inventory split",
        &status_map,
        &[ROOT_INVENTORY_STATUS_NAME, ROOT_INVENTORY_STATUS_ID],
    );
    assert_contains_all(
        "M3 status-support date map records review-guard row-data root inventory split",
        &date_map,
        &[ROOT_INVENTORY_STATUS_NAME, "2026-07-04"],
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
