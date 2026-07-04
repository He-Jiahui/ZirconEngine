use super::*;

#[test]
fn runtime_15_module_layout_child_summary_root_inventory_is_child_owned() {
    let parent = read_runtime_src(CHILD_SUMMARY_PARENT_PATH);
    let status_rows = read_runtime_src(PRODUCTION_GUARD_SUPPORT_MODULE_LAYOUT_ROWS_PATH);
    let status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);

    assert_contains_all(
        "module-layout child-summary root mounts inventory children",
        &parent,
        &[
            "#[path = \"module_layout_child_summaries/root_paths.rs\"]",
            "#[path = \"module_layout_child_summaries/root_statuses.rs\"]",
            "#[path = \"module_layout_child_summaries/root_child_rows.rs\"]",
            "#[path = \"module_layout_child_summaries/root_source_blobs.rs\"]",
            "#[path = \"module_layout_child_summaries/root_inventory.rs\"]",
        ],
    );

    let status_anchors = [
        ROOT_INVENTORY_STATUS_NAME,
        ROOT_INVENTORY_STATUS_ID,
        "structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries.rs",
        "structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries/root_paths.rs",
        "structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries/root_statuses.rs",
        "structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries/root_child_rows.rs",
        "structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries/root_source_blobs.rs",
        "structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries/root_inventory.rs",
        ROOT_INVENTORY_GUARD_NAME,
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "production guard module-layout rows record child-summary root inventory split",
        &status_rows,
        &status_anchors,
    );
    assert_contains_all(
        "M3 status-support status map records child-summary root inventory split",
        &status_map,
        &[ROOT_INVENTORY_STATUS_NAME, ROOT_INVENTORY_STATUS_ID],
    );
    assert_contains_all(
        "M3 status-support date map records child-summary root inventory split",
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
        (
            "runtime implementation session",
            ".codex/sessions/20260612-0847-runtime-architecture-implementation.md",
        ),
    ] {
        let source = read_repo(path);
        assert_contains_all(label, &source, &status_anchors);
    }
}
