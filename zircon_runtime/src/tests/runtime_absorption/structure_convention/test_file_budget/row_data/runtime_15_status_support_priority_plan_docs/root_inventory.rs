use super::*;

#[test]
fn runtime_15_priority_plan_docs_root_inventory_is_child_owned() {
    let parent = read_runtime_src(PRIORITY_GUARD_PATH);
    let status_rows = production_guard_support_priority_rows_source_blob();
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/priority_plan_doc_maps/inventory_sync_maps.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/priority_plan_doc_maps/inventory_sync_maps.rs",
    );

    assert_contains_all(
        "priority-plan-doc route mounts root inventory children",
        &parent,
        &[
            "#[path = \"runtime_15_status_support_priority_plan_docs/root_paths.rs\"]",
            "#[path = \"runtime_15_status_support_priority_plan_docs/root_statuses.rs\"]",
            "#[path = \"runtime_15_status_support_priority_plan_docs/root_child_rows.rs\"]",
            "#[path = \"runtime_15_status_support_priority_plan_docs/root_source_blobs.rs\"]",
            "#[path = \"runtime_15_status_support_priority_plan_docs/root_inventory.rs\"]",
        ],
    );

    let status_anchors = [
        ROOT_INVENTORY_STATUS_NAME,
        ROOT_INVENTORY_STATUS_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs/root_paths.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs/root_statuses.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs/root_child_rows.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs/root_source_blobs.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs/root_inventory.rs",
        ROOT_INVENTORY_GUARD_NAME,
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "production support priority rows record priority-plan-doc root inventory split",
        &status_rows,
        &status_anchors,
    );
    assert_contains_all(
        "Runtime 15 status-support expected status map records priority-plan-doc root inventory split",
        &status_map,
        &[ROOT_INVENTORY_STATUS_NAME, ROOT_INVENTORY_STATUS_ID],
    );
    assert_contains_all(
        "Runtime 15 status-support expected date map records priority-plan-doc root inventory split",
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
